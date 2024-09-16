use anyhow::Result;
use windows::Win32::{
    Foundation::HWND,
    Graphics::{
        Direct3D12::*,
        Dxgi::{
            CreateDXGIFactory2, IDXGIFactory2, DXGI_CREATE_FACTORY_DEBUG, DXGI_CREATE_FACTORY_FLAGS,
        },
    },
};

use crate::gui::hwnd::Hwnd;

use super::{
    composite::CompositionHost,
    core::{
        command_queue::CommandQueue,
        descriptor::{DescriptorHeap, NonShaderVisibleSrvHeap, ShaderVisibleSrvHeap},
        device::Device,
        fence::Fence,
        query::TimestampQueryPool,
        root_signature::RootSignature,
        swap_chain::SwapChain,
    },
    initializer::Initializer,
    renderer::Renderer,
};

pub struct Context {
    #[allow(unused)]
    compositor: CompositionHost,
    _debug: Option<ID3D12Debug6>,
    device: Device,
    command_queue: CommandQueue,
    fence: Fence,
    swap_chain: SwapChain,
    root_signature: RootSignature,
    shader_visible_srv_heap: ShaderVisibleSrvHeap,
    non_shader_visible_srv_heap: NonShaderVisibleSrvHeap,
    timestamp_query_heap: TimestampQueryPool,
}

impl Context {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let mut compositor = CompositionHost::new()?;

        let _debug = unsafe {
            if cfg!(debug_assertions) {
                let mut debug: Option<ID3D12Debug6> = None;
                D3D12GetDebugInterface(&mut debug)?;
                debug.inspect(|debug| {
                    debug.EnableDebugLayer();
                })
            } else {
                None
            }
        };

        let factory: IDXGIFactory2 = unsafe {
            let flags = if cfg!(debug_assertions) {
                DXGI_CREATE_FACTORY_DEBUG
            } else {
                DXGI_CREATE_FACTORY_FLAGS::default()
            };
            CreateDXGIFactory2(flags)?
        };

        let adatper = unsafe { factory.EnumAdapters1(0) }?;
        let device = Device::new(adatper)?;

        let command_queue = CommandQueue::new(&device)?;

        let fence = Fence::new(&device)?;

        let (width, height) = hwnd.size();
        let swap_chain = SwapChain::new(&factory, &device, &command_queue, width, height)?;

        compositor.bind_swap_chain(hwnd, &swap_chain)?;

        let root_signature = RootSignature::new(&device)?;

        let shader_visible_srv_heap = DescriptorHeap::new(&device, 16)?;
        let non_shader_visible_srv_heap = DescriptorHeap::new(&device, 16)?;

        let timestamp_query_heap = TimestampQueryPool::new(&device)?;

        Ok(Self {
            compositor,
            _debug,
            device,
            command_queue,
            fence,
            swap_chain,
            root_signature,
            shader_visible_srv_heap,
            non_shader_visible_srv_heap,
            timestamp_query_heap,
        })
    }

    pub fn create_initializer(&mut self) -> Result<Initializer> {
        Initializer::new(
            self.device.clone(),
            self.root_signature.clone(),
            self.non_shader_visible_srv_heap.iter(),
        )
    }

    pub fn create_renderer(
        &mut self,
        width: u32,
        height: u32,
        clear_color: &[f32; 4],
    ) -> Result<Renderer> {
        self.swap_chain.resize(&self.device, width, height)?;

        let command_list = self.command_queue.command_list()?;
        let render_target = self.swap_chain.render_target()?;

        Renderer::new(
            &self.device,
            &self.root_signature,
            command_list,
            render_target,
            &self.shader_visible_srv_heap,
            &self.timestamp_query_heap,
            clear_color,
        )
    }

    pub fn execute(&mut self, mut renderer: Renderer) -> Result<()> {
        let mut labels = renderer.resolve_query(self.timestamp_query_heap.buffer());

        let command_list = renderer.close()?;
        self.command_queue.execute(command_list)?;

        self.swap_chain.present()?;
        self.fence.wait(&self.command_queue)?;

        let freq = unsafe { self.command_queue.GetTimestampFrequency()? };

        if let Some(labels) = labels.take_if(|labels| !labels.is_empty()) {
            self.timestamp_query_heap.dump(freq, &labels)?;
        }

        Ok(())
    }
}

impl AsRef<Device> for Context {
    fn as_ref(&self) -> &Device {
        &self.device
    }
}

impl std::ops::Deref for Context {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
