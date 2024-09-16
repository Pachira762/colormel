use anyhow::Result;
use windows::{
    core::{s, w},
    Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreateSolidBrush, DrawTextA, SetBkMode, SetTextColor, DT_SINGLELINE, DT_VCENTER,
            HBRUSH, HDC, TRANSPARENT,
        },
        UI::{
            Controls::{CDDS_PREPAINT, CDRF_SKIPDEFAULT, NMCUSTOMDRAW, NM_CUSTOMDRAW},
            Input::KeyboardAndMouse::VK_ESCAPE,
            WindowsAndMessaging::*,
        },
    },
};

use crate::{cast, gui::hwnd::Hwnd};

use super::{
    control::Builder,
    scroll::ScrollBar,
    utils::{self, Rect},
    window::{wndproc, Window},
};

pub struct Menu {
    hwnd: HWND,
    parent: HWND,
    hittest: HitTest,
    scrollbar: ScrollBar,
    bg: HBRUSH,
    visible: bool,
}

impl Menu {
    pub fn create<'a>(parent: HWND) -> Result<&'a mut Self> {
        unsafe {
            let brush = CreateSolidBrush(COLORREF(0x171717));

            utils::register_window_class(
                CS_HREDRAW | CS_VREDRAW,
                Some(wndproc::<Self>),
                None,
                None,
                Some(brush),
                s!("Menu"),
            )?;

            let hwnd = utils::create_window(
                WINDOW_EX_STYLE::default(),
                s!("Menu"),
                s!("Menu"),
                WS_CLIPCHILDREN | WS_POPUP | WS_VSCROLL,
                0,
                0,
                0,
                0,
                parent,
                None,
                None,
            )?;

            if let Some(mut this) = std::ptr::NonNull::new(hwnd.user_data() as *mut Self) {
                Ok(this.as_mut())
            } else {
                anyhow::bail!(windows::core::Error::from_win32())
            }
        }
    }

    pub fn adjust_rect(&mut self, x: i32, y: i32, _width: i32, height: i32) {
        self.hwnd.set_pos(x, y, 168, height, SWP_NOZORDER);
    }

    pub fn get_builder(&mut self) -> Result<Builder> {
        Builder::new(self.hwnd)
    }

    pub fn build(&mut self, builder: Builder) {
        let (_width, height) = builder.size();
        let (_cx, cy) = self.hwnd.client_size();
        self.scrollbar.init(height as _, cy);
    }

    fn on_create(&mut self, _wp: WPARAM, _lp: LPARAM) -> Result<()> {
        self.hwnd.set_display_affinity(WDA_EXCLUDEFROMCAPTURE);
        self.hwnd.set_theme(w!("DarkMode_Explorer"));

        self.hwnd.set_timer(0x02, 100);

        Ok(())
    }

    fn on_window_pos_changed(&mut self, _wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        let WINDOWPOS {
            x,
            y,
            cx: width,
            cy: height,
            ..
        } = *cast!(lp.0, WINDOWPOS);

        self.hittest.update(x, y, width, height);
        self.scrollbar.set_page_size(height as _);

        Some(LRESULT(0))
    }

    fn on_nc_hit_test(&mut self, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        if self.hwnd.def_proc(WM_NCHITTEST, wp, lp) == LRESULT(HTVSCROLL as _) {
            Some(LRESULT(HTVSCROLL as _))
        } else {
            Some(LRESULT(HTTRANSPARENT as _))
        }
    }

    fn on_close(&mut self, _wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        self.hwnd.parent().destroy();
        Some(LRESULT(0))
    }

    fn on_static(&mut self, wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        unsafe {
            let hdc = HDC(wp.0 as _);
            SetTextColor(hdc, COLORREF(0xf0f0f0));
            SetBkMode(hdc, TRANSPARENT);
            Some(LRESULT(self.bg.0 as _))
        }
    }

    fn on_notify(&mut self, _wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        let nmc = cast!(lp.0, NMCUSTOMDRAW);

        if nmc.hdr.code == NM_CUSTOMDRAW
            && nmc.dwDrawStage == CDDS_PREPAINT
            && nmc.dwItemSpec >= 0x0100
        // not slider
        {
            let mut text = nmc.hdr.hwndFrom.text();
            if !text.is_empty() {
                unsafe {
                    SetBkMode(nmc.hdc, TRANSPARENT);
                    SetTextColor(nmc.hdc, COLORREF(0xf0f0f0));

                    nmc.rc.left += 17;

                    DrawTextA(
                        nmc.hdc,
                        text.as_bytes_mut(),
                        &mut nmc.rc,
                        DT_VCENTER | DT_SINGLELINE,
                    );
                }
                return Some(LRESULT(CDRF_SKIPDEFAULT as _));
            }
        }
        None
    }

    fn on_vscroll(&mut self, wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        self.scrollbar.on_scroll(wp);
        Some(LRESULT(0))
    }

    fn on_mouse_wheel(&mut self, wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        self.scrollbar.on_wheel(wp);
        Some(LRESULT(0))
    }

    fn on_timer(&mut self, _wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        let (x, y) = utils::cursor_pos();

        if self.visible && !self.hittest.on_window(x, y) {
            self.show(false);
        } else if !self.visible && self.hittest.on_toggle(x, y) {
            self.show(true);
        }

        Some(LRESULT(0))
    }

    fn on_show(&mut self, wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        self.visible = wp == WPARAM(1);
        Some(LRESULT(0))
    }

    fn show(&self, show: bool) {
        self.hwnd.show(if show { SW_SHOW } else { SW_HIDE });
    }
}

impl Window for Menu {
    fn new(hwnd: HWND, _cs: &mut CREATESTRUCTA) -> Result<Box<Self>> {
        let parent = hwnd.parent();
        let hittest = HitTest::new(hwnd);
        let scrollbar = ScrollBar::new_vert(hwnd);
        let bg = unsafe { CreateSolidBrush(COLORREF(0x171717)) };

        Ok(Box::new(Self {
            parent,
            hwnd,
            hittest,
            scrollbar,
            bg,
            visible: false,
        }))
    }

    #[allow(unused_variables)]
    fn wndproc(
        &mut self,
        hwnd: HWND,
        msg: u32,
        wp: windows::Win32::Foundation::WPARAM,
        lp: windows::Win32::Foundation::LPARAM,
    ) -> Option<windows::Win32::Foundation::LRESULT> {
        match msg {
            WM_CREATE => match self.on_create(wp, lp) {
                Ok(_) => Some(LRESULT::default()),
                Err(e) => {
                    println!("{e:?}");
                    Some(LRESULT(-1))
                }
            },
            WM_KEYDOWN if wp.0 == VK_ESCAPE.0 as usize => self.on_close(wp, lp),
            WM_WINDOWPOSCHANGED => self.on_window_pos_changed(wp, lp),
            WM_NCHITTEST => self.on_nc_hit_test(wp, lp),
            WM_NOTIFY => self.on_notify(wp, lp),
            WM_MOUSEWHEEL => self.on_mouse_wheel(wp, lp),
            WM_CTLCOLORSTATIC => self.on_static(wp, lp),
            WM_SHOWWINDOW => self.on_show(wp, lp),
            WM_TIMER => self.on_timer(wp, lp),
            WM_COMMAND | WM_HSCROLL => Some(self.parent.send_message(msg, wp, lp)),
            WM_VSCROLL => self.on_vscroll(wp, lp),
            _ => None,
        }
    }
}

struct HitTest {
    window: RECT,
    toggle: RECT,
}

impl HitTest {
    fn new(hwnd: HWND) -> Self {
        let window = hwnd.rect();

        let toggle = RECT {
            right: window.left + 32,
            ..window
        };

        Self { window, toggle }
    }

    fn update(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.window = RECT::new(x, y, width, height);

        self.toggle = RECT {
            right: self.window.left + 32,
            ..self.window
        };
    }

    fn on_window(&self, x: i32, y: i32) -> bool {
        self.window.is_in(x, y)
    }

    fn on_toggle(&self, x: i32, y: i32) -> bool {
        self.toggle.is_in(x, y)
    }
}
