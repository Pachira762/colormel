#![allow(unused)]

use std::fmt::Debug;

use anyhow::Result;
use windows::{
    core::{Param, PCSTR, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::UpdateWindow},
        UI::{
            Controls::{
                SetScrollInfo, SetWindowTheme, BST_CHECKED, BST_UNCHECKED, MARGINS, TBM_SETPOS,
                TBM_SETRANGEMAX, TBM_SETRANGEMIN,
            },
            WindowsAndMessaging::*,
        },
    },
};

use super::utils::Rect;

pub trait Hwnd: Copy + Into<HWND> {
    fn from_lparam(lp: LPARAM) -> HWND {
        HWND(lp.0 as _)
    }

    fn rect(self) -> RECT {
        unsafe {
            let mut rc = RECT::default();
            _ = GetWindowRect(self.into(), &mut rc);
            rc
        }
    }

    fn size(self) -> (u32, u32) {
        let rc = self.rect();
        ((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32)
    }

    fn set_pos(self, x: i32, y: i32, width: i32, height: i32, flags: SET_WINDOW_POS_FLAGS) {
        unsafe {
            SetWindowPos(self.into(), None, x, y, width, height, flags);
        }
    }

    fn client_size(self) -> (u32, u32) {
        unsafe {
            let mut rc = RECT::default();
            GetClientRect(self.into(), &mut rc);
            rc.usize()
        }
    }

    fn style(self) -> WINDOW_STYLE {
        unsafe { WINDOW_STYLE(GetWindowLongA(self.into(), GWL_STYLE) as _) }
    }

    fn set_style(self, style: WINDOW_STYLE) {
        unsafe {
            SetWindowLongA(self.into(), GWL_STYLE, style.0 as _);
        }
    }

    fn ex_style(self) -> WINDOW_EX_STYLE {
        unsafe { WINDOW_EX_STYLE(GetWindowLongA(self.into(), GWL_EXSTYLE) as _) }
    }

    fn set_ex_style(self, ex_style: WINDOW_EX_STYLE) {
        unsafe {
            SetWindowLongA(self.into(), GWL_EXSTYLE, ex_style.0 as _);
        }
    }

    fn text(self) -> String {
        unsafe {
            let len = GetWindowTextLengthA(self.into());

            if len > 0 {
                let mut buf = vec![0; len as usize + 1];
                GetWindowTextA(self.into(), &mut buf);
                buf.pop();
                String::from_utf8_unchecked(buf)
            } else {
                String::new()
            }
        }
    }

    fn user_data(self) -> isize {
        unsafe { GetWindowLongPtrA(self.into(), GWLP_USERDATA) }
    }

    fn set_user_data(self, data: isize) {
        unsafe {
            SetWindowLongPtrA(self.into(), GWLP_USERDATA, data);
        }
    }

    fn titlebar_info_ex(self) -> TITLEBARINFOEX {
        unsafe {
            let mut info = TITLEBARINFOEX {
                cbSize: std::mem::size_of::<TITLEBARINFOEX>() as u32,
                ..Default::default()
            };
            _ = self.send_message(
                WM_GETTITLEBARINFOEX,
                WPARAM::default(),
                LPARAM(&mut info as *mut _ as _),
            );
            info
        }
    }

    fn send_message(self, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        unsafe { SendMessageA(self.into(), msg, wp, lp) }
    }

    fn post_message(self, msg: u32, wp: WPARAM, lp: LPARAM) {
        unsafe {
            PostMessageA(self.into(), msg, wp, lp);
        }
    }

    fn window(self, cmd: GET_WINDOW_CMD) -> HWND {
        unsafe { GetWindow(self.into(), cmd).unwrap_or_default() }
    }

    fn owner(self) -> HWND {
        self.window(GW_OWNER)
    }

    fn parent(self) -> HWND {
        unsafe { GetParent(self.into()).unwrap_or_default() }
    }

    fn menu(self) -> HMENU {
        unsafe { GetMenu(self.into()) }
    }

    fn destroy(self) {
        unsafe {
            _ = DestroyWindow(self.into());
        }
    }

    fn update(self) {
        unsafe {
            UpdateWindow(self.into());
        }
    }

    fn show(self, cmd: SHOW_WINDOW_CMD) {
        unsafe {
            ShowWindow(self.into(), cmd);
        }
    }

    fn set_timer(self, id: usize, elapse: u32) {
        unsafe {
            SetTimer(self.into(), id, elapse, None);
        }
    }

    fn scroll_info(
        self,
        bar: SCROLLBAR_CONSTANTS,
        range: bool,
        page: bool,
        pos: bool,
        track: bool,
    ) -> SCROLLINFO {
        unsafe {
            let mut info = SCROLLINFO::default();
            info.cbSize = std::mem::size_of_val(&info) as u32;
            if range {
                info.fMask |= SIF_RANGE;
            }
            if page {
                info.fMask |= SIF_PAGE;
            }
            if pos {
                info.fMask |= SIF_POS;
            }
            if track {
                info.fMask |= SIF_TRACKPOS;
            }
            GetScrollInfo(self.into(), bar, &mut info);
            info
        }
    }

    fn set_scroll_info(
        self,
        bar: SCROLLBAR_CONSTANTS,
        range: Option<(i32, i32)>,
        page: Option<u32>,
        pos: Option<i32>,
        track: Option<i32>,
    ) -> i32 {
        unsafe {
            let mut info = SCROLLINFO {
                cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                ..Default::default()
            };

            if let Some((min, max)) = range {
                info.fMask |= SIF_RANGE;
                info.nMin = min;
                info.nMax = max;
            }
            if let Some(page) = page {
                info.fMask |= SIF_PAGE;
                info.nPage = page;
            }
            if let Some(pos) = pos {
                info.fMask |= SIF_POS;
                info.nPos = pos;
            }
            if let Some(track) = track {
                info.fMask |= SIF_TRACKPOS;
                info.nTrackPos = track;
            }

            SetScrollInfo(self.into(), bar, &info, TRUE)
        }
    }

    fn scroll(self, dx: i32, dy: i32) {
        unsafe {
            ScrollWindowEx(
                self.into(),
                dx,
                dy,
                None,
                None,
                None,
                None,
                SW_ERASE | SW_INVALIDATE | SW_SCROLLCHILDREN,
            );
        }
    }

    fn def_proc(self, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
        unsafe { DefWindowProcA(self.into(), msg, wp, lp) }
    }

    fn set_theme(self, theme: PCWSTR) {
        unsafe {
            SetWindowTheme(self.into(), theme, None).expect("failed SetWindowTheme.");
        }
    }

    fn set_display_affinity(self, affinity: WINDOW_DISPLAY_AFFINITY) {
        unsafe {
            SetWindowDisplayAffinity(self.into(), affinity);
        }
    }

    fn dwm_extend_frame(self, margin: i32) {
        unsafe {
            let margins = MARGINS {
                cxLeftWidth: margin,
                cxRightWidth: margin,
                cyTopHeight: margin,
                cyBottomHeight: margin,
            };
            DwmExtendFrameIntoClientArea(self.into(), &margins);
        }
    }

    fn dwm_enable_blur_behind(self, enable: bool) {
        unsafe {
            DwmEnableBlurBehindWindow(
                self.into(),
                &DWM_BLURBEHIND {
                    dwFlags: DWM_BB_ENABLE,
                    fEnable: enable.into(),
                    ..Default::default()
                },
            );
        }
    }

    fn dwm_attribute<T: Default>(self, attr: DWMWINDOWATTRIBUTE) -> T {
        unsafe {
            let mut value = T::default();
            DwmGetWindowAttribute(
                self.into(),
                attr,
                &mut value as *mut _ as _,
                std::mem::size_of::<T>() as u32,
            );
            value
        }
    }

    fn dwm_set_attribute<T>(self, attr: DWMWINDOWATTRIBUTE, value: *const T) {
        unsafe {
            DwmSetWindowAttribute(
                self.into(),
                attr,
                value as _,
                std::mem::size_of::<T>() as u32,
            );
        }
    }

    fn dwm_def_proc(self, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        unsafe {
            let mut result = LRESULT::default();
            if DwmDefWindowProc(self.into(), msg, wp, lp, &mut result).as_bool() {
                Some(result)
            } else {
                None
            }
        }
    }

    fn caption_button_bounds(self) -> RECT {
        unsafe { self.dwm_attribute(DWMWA_CAPTION_BUTTON_BOUNDS) }
    }
}

impl Hwnd for HWND {}

pub trait CheckBox: Hwnd {
    fn checkbox_checked(self) -> bool {
        self.send_message(BM_GETCHECK, WPARAM::default(), LPARAM::default())
            .0
            == BST_CHECKED.0 as isize
    }

    fn checkbox_set_check(self, checked: bool) {
        let state = if checked { BST_CHECKED } else { BST_UNCHECKED };
        self.send_message(BM_SETCHECK, WPARAM(state.0 as _), LPARAM::default());
    }
}

impl CheckBox for HWND {}

pub trait Trackbar: Hwnd {
    fn trackbar_set_min_max(self, min: i32, max: i32) {
        self.send_message(TBM_SETRANGEMIN, WPARAM(0), LPARAM(min as _));
        self.send_message(TBM_SETRANGEMAX, WPARAM(1), LPARAM(max as _));
    }

    fn trackbar_set_pos(self, pos: i32) {
        self.send_message(TBM_SETPOS, WPARAM(1), LPARAM(pos as _));
    }

    fn trackbar_pos(self) -> i32 {
        const TBM_GETPOS: u32 = WM_USER;
        self.send_message(TBM_GETPOS, WPARAM(0), LPARAM(0)).0 as _
    }
}

impl Trackbar for HWND {}
