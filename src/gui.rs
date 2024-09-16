use anyhow::Result;
use windows::Win32::{
    Foundation::BOOL,
    System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
    UI::{
        Controls::{InitCommonControlsEx, ICC_WIN95_CLASSES, INITCOMMONCONTROLSEX},
        WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
    },
};

use self::{app::App, viewer::Viewer};

pub mod app;
pub mod control;
pub mod hwnd;
mod menu;
mod scroll;
pub mod utils;
mod viewer;
mod window;

pub fn run<T: App>() -> Result<()> {
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;

        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_WIN95_CLASSES,
        })
        .expect("failed InitCommonControlsEx");

        let mut _viewer = Viewer::<T>::create()?;

        loop {
            let mut msg = MSG::default();
            match GetMessageA(&mut msg, None, 0, 0) {
                BOOL(0) | BOOL(-1) => break,
                _ => {
                    _ = TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }
            }
        }

        RoUninitialize();

        Ok(())
    }
}
