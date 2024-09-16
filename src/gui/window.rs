use anyhow::Result;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

use crate::cast;

use super::{hwnd::Hwnd, utils::quit};

pub trait Window: Sized {
    fn new(hwnd: HWND, cs: &mut CREATESTRUCTA) -> Result<Box<Self>>;

    fn wndproc(&mut self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT>;
}

pub unsafe extern "system" fn wndproc<T: Window>(
    hwnd: HWND,
    msg: u32,
    wp: WPARAM,
    lp: LPARAM,
) -> LRESULT {
    if let Some(result) = hwnd.dwm_def_proc(msg, wp, lp) {
        return result;
    }

    if let Some(mut window) = std::ptr::NonNull::new(hwnd.user_data() as *mut T) {
        if let Some(result) = window.as_mut().wndproc(hwnd, msg, wp, lp) {
            return result;
        }
    }

    default_window_proc::<T>(hwnd, msg, wp, lp)
}

fn default_window_proc<T: Window>(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match msg {
        WM_NCCREATE => match T::new(hwnd, cast!(lp.0, CREATESTRUCTA)) {
            Ok(window) => {
                let window = Box::leak(window);
                hwnd.set_user_data(window as *mut _ as _);

                LRESULT(1)
            }
            Err(e) => {
                println!("{e:?}");
                LRESULT(0)
            }
        },
        WM_CLOSE => {
            hwnd.destroy();
            LRESULT::default()
        }
        WM_DESTROY => {
            quit(0);
            LRESULT::default()
        }
        _ => hwnd.def_proc(msg, wp, lp),
    }
}
