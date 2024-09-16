use anyhow::Result;
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D12::*,
        Dxgi::{Common::*, *},
    },
};

use super::{
    command_queue::CommandQueue,
    descriptor::{Descriptor, DsvHeap, RtvHeap},
    device::Device,
    resource::Resource,
    wrap::{ClearValue, DsvDesc, HeapProps, ResourceDesc, RtvDesc, SwapChainDesc},
};

pub struct RenderTarget {
    pub buffer: Resource,
    pub rtv: Descriptor,
    pub depth: Resource,
    pub dsv: Descriptor,
}

pub struct SwapChain {
    swap_chain: IDXGISwapChain4,

    buffers: Vec<Resource>,
    depth: Resource,

    #[allow(unused)]
    rtv_heap: RtvHeap,
    rtvs: Vec<Descriptor>,

    #[allow(unused)]
    dsv_heap: DsvHeap,
    dsv: Descriptor,

    size: (u32, u32),
}

impl SwapChain {
    const BUFFER_COUNT: u32 = 2;

    pub fn new(
        factory: &IDXGIFactory2,
        device: &Device,
        command_queue: &CommandQueue,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        unsafe {
            let swap_chain: IDXGISwapChain4 = factory
                .CreateSwapChainForComposition(
                    command_queue.as_ref(),
                    &SwapChainDesc::composited(
                        width as _,
                        height as _,
                        DXGI_FORMAT_R16G16B16A16_FLOAT,
                        Self::BUFFER_COUNT,
                    ),
                    None,
                )?
                .cast()?;

            let rtv_heap = RtvHeap::new(device, Self::BUFFER_COUNT)?;
            let rtvs: Vec<_> = (0..Self::BUFFER_COUNT)
                .map(|i| rtv_heap.descriptor(i))
                .collect();

            let dsv_heap = DsvHeap::new(device, 1)?;
            let dsv = dsv_heap.descriptor(0);

            let buffers = Self::get_buffers(&swap_chain)?;
            Self::create_rtvs(device, &buffers, &rtvs);

            let depth = Self::create_depth(device, width, height)?;
            Self::create_dsv(device, &depth, dsv);

            Ok(Self {
                swap_chain,
                buffers,
                depth,
                rtv_heap,
                rtvs,
                dsv_heap,
                dsv,
                size: (width, height),
            })
        }
    }

    pub fn render_target(&self) -> Result<RenderTarget> {
        let index = unsafe { self.GetCurrentBackBufferIndex() as usize };

        Ok(RenderTarget {
            buffer: self.buffers[index].clone(),
            rtv: self.rtvs[index],
            depth: self.depth.clone(),
            dsv: self.dsv,
        })
    }

    pub fn present(&self) -> Result<()> {
        unsafe {
            self.Present(1, DXGI_PRESENT::default())
                .ok()
                .map_err(anyhow::Error::msg)
        }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) -> Result<()> {
        if (width, height) == self.size {
            return Ok(());
        }

        self.buffers.clear();

        unsafe {
            self.swap_chain.ResizeBuffers(
                0,
                width,
                height,
                DXGI_FORMAT_UNKNOWN,
                DXGI_SWAP_CHAIN_FLAG::default(),
            )?;
        }

        self.buffers = Self::get_buffers(&self.swap_chain)?;
        Self::create_rtvs(device, &self.buffers, &self.rtvs);

        self.depth = Self::create_depth(device, width, height)?;
        Self::create_dsv(device, &self.depth, self.dsv);

        Ok(())
    }

    fn get_buffers(swap_chain: &IDXGISwapChain4) -> Result<Vec<Resource>> {
        let mut buffers = vec![];
        for i in 0..Self::BUFFER_COUNT {
            let buffer = unsafe { swap_chain.GetBuffer::<ID3D12Resource>(i)? };
            let buffer = Resource::from(buffer);
            buffers.push(buffer);
        }
        Ok(buffers)
    }

    fn create_rtvs(device: &Device, buffers: &[Resource], rtvs: &[Descriptor]) {
        for (buffer, &rtv) in buffers.iter().zip(rtvs.iter()) {
            unsafe {
                device.CreateRenderTargetView(
                    buffer.as_ref(),
                    //Some(&RtvDesc::default(DXGI_FORMAT_B8G8R8A8_UNORM_SRGB)),
                    Some(&RtvDesc::default(DXGI_FORMAT_R16G16B16A16_FLOAT)),
                    rtv.cpu,
                );
            }
        }
    }

    fn create_depth(device: &Device, width: u32, height: u32) -> Result<Resource> {
        Resource::new(
            device,
            &HeapProps::default(),
            None,
            &ResourceDesc::texture2d(
                width,
                height,
                DXGI_FORMAT_D16_UNORM,
                D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL | D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE,
            ),
            D3D12_RESOURCE_STATE_DEPTH_WRITE,
            Some(&ClearValue::depth(DXGI_FORMAT_D16_UNORM, 1.0)),
        )
    }

    fn create_dsv(device: &Device, depth: &Resource, dsv: Descriptor) {
        unsafe {
            device.CreateDepthStencilView(
                depth.as_ref(),
                Some(&DsvDesc::default(DXGI_FORMAT_D16_UNORM)),
                dsv.cpu,
            );
        }
    }
}

impl AsRef<IDXGISwapChain4> for SwapChain {
    fn as_ref(&self) -> &IDXGISwapChain4 {
        &self.swap_chain
    }
}

impl std::ops::Deref for SwapChain {
    type Target = IDXGISwapChain4;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
