use anyhow::Result;
use windows::{
    core::Interface,
    Foundation::Numerics::Vector2,
    System::{DispatcherQueue, DispatcherQueueController},
    Win32::{
        Foundation::HWND,
        Graphics::Dxgi::IDXGISwapChain4,
        System::WinRT::{
            Composition::{ICompositorDesktopInterop, ICompositorInterop},
            CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_NONE,
            DQTYPE_THREAD_CURRENT,
        },
    },
    UI::Composition::{CompositionStretch, Compositor, Desktop::DesktopWindowTarget},
};

pub struct CompositionHost {
    #[allow(unused)]
    controller: DispatcherQueueController,
    #[allow(unused)]
    queue: DispatcherQueue,

    compositor: Compositor,
    targets: Vec<DesktopWindowTarget>,
}

impl CompositionHost {
    pub fn new() -> Result<Self> {
        unsafe {
            let controller = CreateDispatcherQueueController(DispatcherQueueOptions {
                dwSize: std::mem::size_of::<DispatcherQueueOptions>() as _,
                threadType: DQTYPE_THREAD_CURRENT,
                apartmentType: DQTAT_COM_NONE,
            })?;
            let queue = controller.DispatcherQueue()?;
            let compositor = Compositor::new()?;

            Ok(Self {
                controller,
                queue,
                compositor,
                targets: vec![],
            })
        }
    }

    pub fn bind_swap_chain(&mut self, hwnd: HWND, swap_chain: &IDXGISwapChain4) -> Result<()> {
        unsafe {
            let target = {
                let interop: ICompositorDesktopInterop = self.compositor.cast()?;
                interop.CreateDesktopWindowTarget(hwnd, true)?
            };
            let content = self.compositor.CreateSpriteVisual()?;
            let surface = {
                let interop: ICompositorInterop = self.compositor.cast()?;
                interop.CreateCompositionSurfaceForSwapChain(swap_chain)?
            };
            let brush = self.compositor.CreateSurfaceBrushWithSurface(&surface)?;
            brush.SetStretch(CompositionStretch::UniformToFill)?;

            content.SetRelativeSizeAdjustment(Vector2::one())?;
            content.SetBrush(&brush)?;
            target.SetRoot(&content)?;
            self.targets.push(target);

            Ok(())
        }
    }
}
