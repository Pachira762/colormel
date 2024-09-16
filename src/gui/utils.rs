#![allow(unused, non_snake_case)]

use std::mem::size_of;

use anyhow::{Error as E, Result};
use windows::{
    core::{Param, PCSTR},
    Win32::{
        Foundation::*,
        Graphics::{
            Dwm::{DwmGetWindowAttribute, DWMWA_CAPTION_BUTTON_BOUNDS},
            Gdi::{COLOR_WINDOW, HBRUSH},
        },
        System::LibraryLoader::GetModuleHandleA,
        UI::WindowsAndMessaging::*,
    },
};

#[macro_export]
macro_rules! LOWORD {
    ($dw:expr) => {
        ($dw & 0xffff) as u32
    };
}

#[macro_export]
macro_rules! HIWORD {
    ($dw:expr) => {
        (($dw >> 16) & 0xffff) as u32
    };
}

#[macro_export]
macro_rules! GET_X_LPARAM {
    ($lp:ident) => {
        (($lp.0 & 0xffff) as i16) as i32
    };
}

#[macro_export]
macro_rules! GET_Y_LPARAM {
    ($lp:ident) => {
        ((($lp.0 >> 16) & 0xffff) as i16) as i32
    };
}

#[macro_export]
macro_rules! GET_WHEEL_DELTA_WPARAM {
    ($wp:ident) => {
        ((($wp.0 >> 16) & 0xffff) as i16) as i32
    };
}

#[macro_export]
macro_rules! cast {
    ($src:expr, $type:ty) => {
        unsafe { ($src as *mut $type).as_mut().unwrap() }
    };
}

pub trait Word: Sized {
    fn dw(self) -> u64;

    fn lo(self) -> u32 {
        LOWORD!(self.dw())
    }

    fn hi(self) -> u32 {
        HIWORD!(self.dw())
    }
}

impl Word for WPARAM {
    fn dw(self) -> u64 {
        self.0 as _
    }
}

impl Word for LPARAM {
    fn dw(self) -> u64 {
        self.0 as _
    }
}

pub trait Rect {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self;
    fn inner(&self, x: i32, y: i32) -> Self;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn size(&self) -> (i32, i32);
    fn usize(&self) -> (u32, u32);
    fn is_in(&self, x: i32, y: i32) -> bool;
}

impl Rect for RECT {
    fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    fn inner(&self, width: i32, height: i32) -> Self {
        Self {
            left: self.left + width,
            top: self.top + height,
            right: self.right - width,
            bottom: self.bottom - height,
        }
    }

    fn width(&self) -> i32 {
        self.right - self.left
    }

    fn height(&self) -> i32 {
        self.bottom - self.top
    }

    fn size(&self) -> (i32, i32) {
        (self.width(), self.height())
    }

    fn usize(&self) -> (u32, u32) {
        (self.width() as u32, self.height() as u32)
    }

    fn is_in(&self, x: i32, y: i32) -> bool {
        self.left <= x && x < self.right && self.top <= y && y < self.bottom
    }
}

pub fn module_handle() -> HINSTANCE {
    unsafe { HINSTANCE(GetModuleHandleA(None).expect("failed GetModuleHandleA").0) }
}

pub fn register_window_class(
    style: WNDCLASS_STYLES,
    proc: WNDPROC,
    icon: Option<HICON>,
    cursor: Option<HCURSOR>,
    bg: Option<HBRUSH>,
    name: PCSTR,
) -> Result<()> {
    unsafe {
        let wc = WNDCLASSA {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: proc,
            hInstance: module_handle(),
            hIcon: icon.unwrap_or_else(|| {
                LoadIconW(None, IDI_APPLICATION).expect("failed to get default app icon")
            }),
            hCursor: cursor.unwrap_or_else(|| {
                LoadCursorW(None, IDC_ARROW).expect("failed to get default cursor.")
            }),
            hbrBackground: bg.unwrap_or(HBRUSH((COLOR_WINDOW.0 as isize + 1) as _)),
            lpszClassName: name,
            ..Default::default()
        };
        if RegisterClassA(&wc) == 0 {
            anyhow::bail!(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn create_window<P0: Param<HWND>, P1: Param<HMENU>>(
    ex_style: WINDOW_EX_STYLE,
    class_name: PCSTR,
    window_name: PCSTR,
    style: WINDOW_STYLE,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    parent: P0,
    menu: P1,
    param: Option<*const std::ffi::c_void>,
) -> Result<HWND> {
    unsafe {
        CreateWindowExA(
            ex_style,
            class_name,
            window_name,
            style,
            x,
            y,
            width,
            height,
            parent,
            menu,
            module_handle(),
            param,
        )
        .map_err(E::msg)
    }
}

pub fn adjust_window_rect(
    ex_style: WINDOW_EX_STYLE,
    style: WINDOW_STYLE,
    width: u32,
    height: u32,
) -> RECT {
    unsafe {
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        AdjustWindowRectEx(&mut rc, style, None, ex_style)
            .expect("failed to adjust window rect ex.");
        rc
    }
}

pub fn cursor_pos() -> (i32, i32) {
    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point);
        (point.x, point.y)
    }
}

pub fn quit(code: i32) {
    unsafe {
        PostQuitMessage(code);
    }
}

pub fn system_metrics(index: SYSTEM_METRICS_INDEX) -> i32 {
    unsafe { GetSystemMetrics(index) }
}
