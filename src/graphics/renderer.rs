use anyhow::Result;
use windows::Win32::{Foundation::RECT, Graphics::Direct3D12::*};

use super::core::{
    command_queue::{ClosedCommandList, CommandList},
    descriptor::{Descriptor, DescriptorIter, ShaderVisibleSrvHeap},
    device::Device,
    query::{TimestampQueryIter, TimestampQueryPool},
    resource::Resource,
    root_signature::{
        RootSignature, ROOT_PARAM_INDEX_CONSTANTS, ROOT_PARAM_INDEX_DIRECT_SRV,
        ROOT_PARAM_INDEX_SRV, ROOT_PARAM_INDEX_UAV,
    },
    swap_chain::RenderTarget,
};

pub enum ViewportKind {
    Full,
    Adjust,
}

pub struct Renderer {
    device: Device,
    command_list: CommandList,
    render_target: RenderTarget,
    shader_visible_descriptors: DescriptorIter,
    timestamp_querys: TimestampQueryIter,
    viewports: [D3D12_VIEWPORT; 2],
}

impl Renderer {
    pub fn new(
        device: &Device,
        root_signature: &RootSignature,
        command_list: CommandList,
        render_target: RenderTarget,
        shader_visible_descriptor_heap: &ShaderVisibleSrvHeap,
        timestamp_query_pool: &TimestampQueryPool,
        clear_color: &[f32; 4],
    ) -> Result<Self> {
        unsafe {
            command_list.SetGraphicsRootSignature(root_signature.as_ref());
            command_list.SetComputeRootSignature(root_signature.as_ref());

            let descriptor_heap = shader_visible_descriptor_heap.as_ref().clone();
            command_list.SetDescriptorHeaps(&[Some(descriptor_heap)]);

            command_list.ResourceBarrier(&[render_target.buffer.transition_barrier(
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            )]);

            let rtvs = [render_target.rtv.cpu];
            let dsv = render_target.dsv.cpu;

            command_list.OMSetRenderTargets(1, Some(rtvs.as_ptr()), false, Some(&dsv));

            for rtv in rtvs {
                command_list.ClearRenderTargetView(rtv, clear_color, None);
            }
            command_list.ClearDepthStencilView(dsv, D3D12_CLEAR_FLAG_DEPTH, 1.0, 0, &[]);

            let (width, height) = render_target.buffer.size();
            let adjusted = width.max(height) as f32;

            let viewports = [
                D3D12_VIEWPORT {
                    Width: width as _,
                    Height: height as _,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                    ..Default::default()
                },
                D3D12_VIEWPORT {
                    TopLeftX: (width as f32 - adjusted) / 2.0,
                    TopLeftY: (height as f32 - adjusted) / 2.0,
                    Width: adjusted,
                    Height: adjusted,
                    MinDepth: 0.0,
                    MaxDepth: 1.0,
                },
            ];

            command_list.RSSetViewports(&[viewports[0]]);

            command_list.RSSetScissorRects(&[RECT {
                right: width as _,
                bottom: height as _,
                ..Default::default()
            }]);

            Ok(Self {
                device: device.clone(),
                command_list,
                render_target,
                shader_visible_descriptors: shader_visible_descriptor_heap.iter(),
                timestamp_querys: timestamp_query_pool.iter(),
                viewports,
            })
        }
    }

    pub fn set_viewport(&mut self, viewport_kind: ViewportKind) {
        let viewport = match viewport_kind {
            ViewportKind::Full => self.viewports[0],
            ViewportKind::Adjust => self.viewports[1],
        };

        unsafe {
            self.command_list.RSSetViewports(&[viewport]);
        }
    }

    pub fn resolve_query(&mut self, buffer: &Resource) -> Option<Vec<String>> {
        self.command_list.resolve_query(
            self.timestamp_querys.heap(),
            D3D12_QUERY_TYPE_TIMESTAMP,
            self.timestamp_querys.count(),
            buffer,
        );

        self.timestamp_querys.take_labels()
    }

    pub fn close(self) -> Result<ClosedCommandList> {
        self.command_list
            .resource_barrier(&[self.render_target.buffer.transition_barrier(
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);

        self.command_list.close()
    }

    pub fn set_compute_constants<T>(&mut self, params: &T) {
        unsafe {
            self.command_list.SetComputeRoot32BitConstants(
                ROOT_PARAM_INDEX_CONSTANTS,
                std::mem::size_of_val(params) as u32 / 4,
                params as *const _ as _,
                0,
            );
        }
    }

    pub fn set_graphics_constants<T>(&mut self, params: &T) {
        unsafe {
            self.command_list.SetGraphicsRoot32BitConstants(
                ROOT_PARAM_INDEX_CONSTANTS,
                std::mem::size_of_val(params) as u32 / 4,
                params as *const _ as _,
                0,
            );
        }
    }

    pub fn set_compute_srvs(&mut self, descriptors: &[Descriptor]) {
        unsafe {
            let descriptor = self.copy_descriptors(descriptors)[0];
            self.command_list
                .SetComputeRootDescriptorTable(ROOT_PARAM_INDEX_SRV, descriptor.gpu);
        }
    }

    pub fn set_graphics_srvs(&mut self, descriptors: &[Descriptor]) {
        unsafe {
            let descriptor = self.copy_descriptors(descriptors)[0];
            self.SetGraphicsRootDescriptorTable(ROOT_PARAM_INDEX_SRV, descriptor.gpu);
        }
    }

    pub fn set_shared_srv(&mut self, srv: Descriptor) {
        unsafe {
            let descriptor = self.copy_descriptors(&[srv])[0];
            self.SetComputeRootDescriptorTable(ROOT_PARAM_INDEX_DIRECT_SRV, descriptor.gpu);
            self.SetGraphicsRootDescriptorTable(ROOT_PARAM_INDEX_DIRECT_SRV, descriptor.gpu);
        }
    }

    pub fn set_uavs(&mut self, descriptors: &[Descriptor]) {
        unsafe {
            let descriptor = self.copy_descriptors(descriptors)[0];
            self.SetComputeRootDescriptorTable(ROOT_PARAM_INDEX_UAV, descriptor.gpu);
        }
    }

    pub fn clear_uav(&mut self, uav: Descriptor, resource: &Resource) {
        unsafe {
            let uav_shader_visible = self.copy_descriptors(&[uav])[0];
            self.ClearUnorderedAccessViewUint(
                uav_shader_visible.gpu,
                uav.cpu,
                resource.as_ref(),
                &[0; 4],
                &[],
            );
        }
    }

    fn copy_descriptors(&mut self, descriptors: &[Descriptor]) -> Vec<Descriptor> {
        let mut copied_descriptors = vec![];

        for &src in descriptors {
            let dst = self
                .shader_visible_descriptors
                .next()
                .expect("too many descriptors");

            unsafe {
                self.device.CopyDescriptorsSimple(
                    1,
                    dst.cpu,
                    src.cpu,
                    D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                );
            }

            copied_descriptors.push(dst);
        }

        copied_descriptors
    }

    pub fn timestamp(&mut self, label: &str) {
        if let Some(index) = self.timestamp_querys.next(label) {
            unsafe {
                self.command_list.EndQuery(
                    self.timestamp_querys.heap().as_ref(),
                    D3D12_QUERY_TYPE_TIMESTAMP,
                    index,
                )
            };
        }
    }
}

impl AsRef<CommandList> for Renderer {
    fn as_ref(&self) -> &CommandList {
        &self.command_list
    }
}

impl std::ops::Deref for Renderer {
    type Target = CommandList;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
