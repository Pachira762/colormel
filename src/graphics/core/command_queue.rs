use anyhow::Result;
use windows::{
    core::Interface,
    Win32::Graphics::{Direct3D::D3D_PRIMITIVE_TOPOLOGY, Direct3D12::*},
};

use super::{
    device::Device, pso::PipelineState, root_signature::RootSignature, wrap::CommandQueueDesc,
};

pub struct CommandQueue {
    command_queue: ID3D12CommandQueue,
    command_allocator: ID3D12CommandAllocator,
    command_list: ID3D12GraphicsCommandList6,
}

impl CommandQueue {
    pub fn new(device: &Device) -> Result<Self> {
        unsafe {
            let command_queue: ID3D12CommandQueue =
                device.CreateCommandQueue(&CommandQueueDesc::direct())?;

            let command_allocator =
                device.CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

            let command_list: ID3D12GraphicsCommandList6 = device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                None,
            )?;
            command_list.Close()?;

            Ok(Self {
                command_queue,
                command_allocator,
                command_list,
            })
        }
    }

    pub fn command_list(&self) -> Result<CommandList> {
        unsafe {
            self.command_allocator.Reset()?;
            self.command_list.Reset(&self.command_allocator, None)?;

            Ok(CommandList(self.command_list.clone()))
        }
    }

    pub fn execute(&self, command_list: ClosedCommandList) -> Result<()> {
        unsafe {
            self.command_queue
                .ExecuteCommandLists(&[Some(command_list.0)]);

            Ok(())
        }
    }
}

impl AsRef<ID3D12CommandQueue> for CommandQueue {
    fn as_ref(&self) -> &ID3D12CommandQueue {
        &self.command_queue
    }
}

impl std::ops::Deref for CommandQueue {
    type Target = ID3D12CommandQueue;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[derive(Clone)]
pub struct CommandList(ID3D12GraphicsCommandList6);

impl CommandList {
    pub fn resource_barrier(&self, barriers: &[D3D12_RESOURCE_BARRIER]) {
        unsafe {
            self.ResourceBarrier(barriers);
        }
    }

    pub fn set_descriptor_heaps(&self, heaps: &[&ID3D12DescriptorHeap]) {
        unsafe {
            let heaps: Vec<_> = heaps.iter().map(|&heap| Some(heap.clone())).collect();
            self.SetDescriptorHeaps(&heaps);
        }
    }

    pub fn set_root_signature(&self, root_signature: &RootSignature) {
        unsafe {
            self.SetComputeRootSignature(root_signature.as_ref());
            self.SetGraphicsRootSignature(root_signature.as_ref());
        }
    }

    pub fn set_pipeline_state(&self, pso: &PipelineState) {
        unsafe {
            self.SetPipelineState(pso.as_ref());
        }
    }

    pub fn set_primitive_topology(&self, primitive_topology: D3D_PRIMITIVE_TOPOLOGY) {
        unsafe {
            self.IASetPrimitiveTopology(primitive_topology);
        }
    }

    pub fn set_vertex_buffers(&self, views: &[D3D12_VERTEX_BUFFER_VIEW]) {
        unsafe {
            self.IASetVertexBuffers(0, Some(views));
        }
    }

    pub fn draw(&self, vertex_count: u32, instance_count: u32) {
        unsafe {
            self.DrawInstanced(vertex_count, instance_count, 0, 0);
        }
    }

    pub fn dispatch(&self, x: u32, y: u32, z: u32) {
        unsafe {
            self.Dispatch(x, y, z);
        }
    }

    pub fn dispatch_mesh(&self, x: u32, y: u32, z: u32) {
        unsafe {
            self.DispatchMesh(x, y, z);
        }
    }

    pub fn resolve_query(
        &self,
        query_heap: &ID3D12QueryHeap,
        query_type: D3D12_QUERY_TYPE,
        num_queries: u32,
        buffer: &ID3D12Resource,
    ) {
        unsafe {
            self.ResolveQueryData(query_heap, query_type, 0, num_queries, buffer, 0);
        }
    }

    pub fn close(self) -> Result<ClosedCommandList> {
        unsafe {
            self.Close()?;

            Ok(ClosedCommandList(self.0.cast()?))
        }
    }
}

impl AsRef<ID3D12GraphicsCommandList6> for CommandList {
    fn as_ref(&self) -> &ID3D12GraphicsCommandList6 {
        &self.0
    }
}

impl std::ops::Deref for CommandList {
    type Target = ID3D12GraphicsCommandList6;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub struct ClosedCommandList(ID3D12CommandList);
