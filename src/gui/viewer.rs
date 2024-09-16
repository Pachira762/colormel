use anyhow::Result;
use windows::{
    core::{s, PCSTR, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, TRUE, WPARAM},
        Graphics::Dwm::{
            DWMNCRP_ENABLED, DWMWA_NCRENDERING_POLICY, DWMWA_USE_IMMERSIVE_DARK_MODE,
            DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
        },
        System::SystemServices::MK_LBUTTON,
        UI::{Input::KeyboardAndMouse::VK_ESCAPE, WindowsAndMessaging::*},
    },
};

use crate::{
    cast,
    gui::utils::{self, module_handle},
    GET_X_LPARAM, GET_Y_LPARAM,
};

use super::{
    app::App,
    hwnd::{CheckBox, Hwnd, Trackbar},
    menu::Menu,
    utils::{quit, Rect as _, Word},
    window::{wndproc, Window},
};

const EX_STYLE: WINDOW_EX_STYLE =
    WINDOW_EX_STYLE(WS_EX_NOREDIRECTIONBITMAP.0 | WS_EX_APPWINDOW.0 | WS_EX_TOPMOST.0);

pub struct Viewer<T: App> {
    app: Option<T>,
    hwnd: HWND,
    transparent: bool,
    hittest: HitTest,
    menu: &'static mut Menu,
    mx: i32,
    my: i32,
}

impl<T: App> Viewer<T> {
    pub fn create<'a>() -> Result<&'a mut Self> {
        unsafe {
            const CLASS_NAME: PCSTR = s!("Viewer");

            utils::register_window_class(
                CS_HREDRAW | CS_VREDRAW,
                Some(wndproc::<Self>),
                Some(LoadIconW(module_handle(), PCWSTR(1 as _))?),
                None,
                None,
                CLASS_NAME,
            )?;

            let hwnd = utils::create_window(
                EX_STYLE,
                CLASS_NAME,
                s!("Colormel"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                800,
                800,
                None,
                None,
                None,
            )?;

            hwnd.update();
            hwnd.show(SW_SHOW);

            if let Some(mut this) = std::ptr::NonNull::new(hwnd.user_data() as *mut Self) {
                Ok(this.as_mut())
            } else {
                anyhow::bail!(windows::core::Error::from_win32())
            }
        }
    }

    fn set_transparency(&mut self, transparent: bool) {
        self.transparent = transparent;

        if transparent {
            self.hwnd
                .set_ex_style(EX_STYLE | WS_EX_LAYERED | WS_EX_TRANSPARENT);
        } else {
            self.hwnd.set_ex_style(EX_STYLE);
        }
    }

    fn on_create(&mut self, _wp: WPARAM, _lp: LPARAM) -> Result<()> {
        self.hwnd.set_display_affinity(WDA_EXCLUDEFROMCAPTURE);

        let rect = self
            .app
            .as_mut()
            .expect("no app when on_create")
            .window_rect();

        self.hwnd.set_pos(
            rect.left,
            rect.top,
            rect.width(),
            rect.height(),
            SWP_FRAMECHANGED,
        );

        self.hwnd.dwm_extend_frame(-1);
        self.hwnd.dwm_enable_blur_behind(true);
        self.hwnd
            .dwm_set_attribute(DWMWA_NCRENDERING_POLICY, &DWMNCRP_ENABLED);
        self.hwnd
            .dwm_set_attribute(DWMWA_USE_IMMERSIVE_DARK_MODE, &TRUE);
        self.hwnd
            .dwm_set_attribute(DWMWA_WINDOW_CORNER_PREFERENCE, &DWMWCP_DONOTROUND);

        self.hwnd.set_timer(0x01, 100);

        if let Some(app) = &mut self.app {
            let mut builder = self.menu.get_builder()?;
            app.build_menu(&mut builder)?;
            self.menu.build(builder);
        }

        Ok(())
    }

    fn on_close(&mut self, _wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        if let Some(mut app) = self.app.take() {
            _ = app.on_destroy();
        }

        self.hwnd.destroy();

        Some(LRESULT(0))
    }

    fn on_destroy(&mut self, _wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        if let Some(mut app) = self.app.take() {
            _ = app.on_destroy();
        }

        quit(0);

        Some(LRESULT(0))
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
        self.menu.adjust_rect(x, y, width, height);

        if width > 0 && height > 0 {
            if let Some(app) = &mut self.app {
                _ = app.on_pos_changed(x, y, width, height);
            }
        }

        Some(LRESULT(0))
    }

    fn on_timer(&mut self, _wp: WPARAM, _lp: LPARAM) -> Option<LRESULT> {
        if self.transparent && self.hittest.on_frame() {
            self.set_transparency(false);
        }

        Some(LRESULT(0))
    }

    fn on_nc_hit_test(&mut self, _wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        let mx = GET_X_LPARAM!(lp);
        let my = GET_Y_LPARAM!(lp);
        let hit = self.hittest.nc_hit_test(mx, my);
        Some(LRESULT(hit as _))
    }

    fn on_control(&mut self, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        if let Some(app) = self.app.as_mut() {
            let code = wp.hi();
            let id = wp.lo();
            let ctrl = HWND::from_lparam(lp);

            if code == BN_CLICKED {
                app.on_button(id, ctrl.checkbox_checked());
            }
        }

        Some(LRESULT(0))
    }

    fn on_hscroll(&mut self, _wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        if let Some(app) = self.app.as_mut() {
            let trackbar = HWND::from_lparam(lp);
            let id = trackbar.menu().0 as u32;
            let val = trackbar.trackbar_pos();

            app.on_slider(id, val);
        }

        Some(LRESULT(0))
    }

    fn on_mouse_move(&mut self, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        let mx = GET_X_LPARAM!(lp);
        let my = GET_Y_LPARAM!(lp);

        if wp == WPARAM(MK_LBUTTON.0 as _) {
            let dx = mx - self.mx;
            let dy = my - self.my;

            if let Some(app) = self.app.as_mut() {
                app.on_drag(dx, dy);
            }
        }

        self.mx = mx;
        self.my = my;

        Some(LRESULT(0))
    }
}

impl<T: App> Window for Viewer<T> {
    fn new(hwnd: HWND, _cs: &mut CREATESTRUCTA) -> Result<Box<Self>> {
        let hittest = HitTest::new(hwnd, WINDOW_EX_STYLE::default(), WS_OVERLAPPEDWINDOW);
        let menu = Menu::create(hwnd)?;
        let app = Some(T::new(hwnd)?);

        Ok(Box::new(Self {
            app,
            hwnd,
            transparent: false,
            hittest,
            menu,
            mx: 0,
            my: 0,
        }))
    }

    #[allow(unused_variables)]
    fn wndproc(&mut self, hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
        match msg {
            WM_CREATE => match self.on_create(wp, lp) {
                Ok(_) => Some(LRESULT(0)),
                Err(e) => {
                    println!("{e:?}");
                    Some(LRESULT(-1))
                }
            },
            WM_KEYDOWN if wp.0 == VK_ESCAPE.0 as usize => self.on_close(wp, lp),
            WM_CLOSE => self.on_close(wp, lp),
            WM_DESTROY => self.on_destroy(wp, lp),
            WM_NCCALCSIZE if wp == WPARAM(1) => Some(LRESULT(0)),
            WM_WINDOWPOSCHANGED => self.on_window_pos_changed(wp, lp),
            WM_NCHITTEST => self.on_nc_hit_test(wp, lp),
            WM_TIMER => self.on_timer(wp, lp),
            WM_COMMAND => self.on_control(wp, lp),
            WM_HSCROLL => self.on_hscroll(wp, lp),
            WM_MOUSEMOVE => self.on_mouse_move(wp, lp),
            _ => None,
        }
    }
}

struct HitTest {
    window: RECT,
    caption: RECT,
    client: RECT,
    frame_x: i32,
    frame_y: i32,
    caption_y: i32,
}

impl HitTest {
    fn new(hwnd: HWND, ex_style: WINDOW_EX_STYLE, style: WINDOW_STYLE) -> Self {
        let frame = utils::adjust_window_rect(ex_style, style, 0, 0);
        let frame_x = frame.right;
        let frame_y = frame.bottom;
        let caption_y = -frame.top;

        let window = hwnd.rect();
        let caption = window.inner(frame_x, frame_y);
        let client = window.inner(caption_y, caption_y);

        Self {
            window,
            caption,
            client,
            frame_x,
            frame_y,
            caption_y,
        }
    }

    fn update(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.window.left = x;
        self.window.top = y;
        self.window.right = x + width;
        self.window.bottom = y + height;

        self.caption = self.window.inner(self.frame_x, self.frame_y);

        self.client = self.window.inner(self.caption_y, self.caption_y);
    }

    fn on_frame(&self) -> bool {
        let (mx, my) = utils::cursor_pos();
        self.window.is_in(mx, my) && !self.client.is_in(mx, my)
    }

    fn nc_hit_test(&self, x: i32, y: i32) -> u32 {
        if self.client.is_in(x, y) {
            HTCLIENT
        } else if self.caption.is_in(x, y) {
            HTCAPTION
        } else {
            let row = if y < self.caption.top {
                0
            } else if self.caption.bottom <= y {
                2
            } else {
                1
            };

            let col = if x < self.caption.left {
                0
            } else if self.caption.right <= x {
                2
            } else {
                1
            };

            match (row, col) {
                (0, 0) => HTTOPLEFT,
                (0, 1) => HTTOP,
                (0, 2) => HTTOPRIGHT,
                (1, 0) => HTLEFT,
                (1, 2) => HTRIGHT,
                (2, 0) => HTBOTTOMLEFT,
                (2, 1) => HTBOTTOM,
                (2, 2) => HTBOTTOMRIGHT,
                _ => unreachable!("unreachable hit test pattern!"),
            }
        }
    }
}
