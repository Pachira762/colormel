use windows::Win32::{
    Foundation::{HWND, WPARAM},
    UI::WindowsAndMessaging::{
        SB_BOTTOM, SB_HORZ, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBTRACK, SB_TOP,
        SB_VERT, SCROLLBAR_COMMAND, SCROLLBAR_CONSTANTS, SCROLLINFO,
    },
};

use crate::GET_WHEEL_DELTA_WPARAM;

use super::{hwnd::Hwnd, utils::Word};

const LINE: i32 = 13;

pub struct ScrollBar {
    hwnd: HWND,
    bar: SCROLLBAR_CONSTANTS,
}

impl ScrollBar {
    pub fn new(hwnd: HWND, bar: SCROLLBAR_CONSTANTS) -> Self {
        Self { hwnd, bar }
    }

    #[allow(unused)]
    pub fn new_vert(hwnd: HWND) -> Self {
        Self::new(hwnd, SB_VERT)
    }

    #[allow(unused)]
    pub fn new_horz(hwnd: HWND) -> Self {
        Self::new(hwnd, SB_HORZ)
    }

    pub fn init(&self, range: i32, page: u32) {
        self.hwnd
            .set_scroll_info(self.bar, Some((0, range)), Some(page), Some(0), None);
    }

    pub fn set_page_size(&mut self, size: u32) {
        self.update(None, Some(size), None, None);
    }

    pub fn on_scroll(&mut self, wp: WPARAM) {
        let mut info = self.info(true, true, true, true);

        match SCROLLBAR_COMMAND(wp.lo() as _) {
            SB_LINEUP => {
                info.nPos -= LINE;
            }
            SB_LINEDOWN => {
                info.nPos += LINE;
            }
            SB_PAGEUP => {
                info.nPos -= info.nPage as i32;
            }
            SB_PAGEDOWN => {
                info.nPos += info.nPage as i32;
            }
            SB_TOP => {
                info.nPos = info.nMin;
            }
            SB_BOTTOM => {
                info.nPos = info.nMax;
            }
            SB_THUMBTRACK => {
                info.nPos = info.nTrackPos;
            }
            _ => {}
        }

        self.update(None, None, Some(info.nPos), None);
    }

    pub fn on_wheel(&mut self, wp: WPARAM) {
        let pos = self.pos();
        let delta = -GET_WHEEL_DELTA_WPARAM!(wp) * LINE / 120;
        self.update(None, None, Some(pos + delta), None);
    }

    fn update(
        &mut self,
        range: Option<i32>,
        page: Option<u32>,
        pos: Option<i32>,
        track: Option<i32>,
    ) {
        let prev = self.pos();
        let now = self.set_info(range, page, pos, track);
        self.scroll(now - prev);
    }

    fn info(&self, range: bool, page: bool, pos: bool, track: bool) -> SCROLLINFO {
        self.hwnd.scroll_info(self.bar, range, page, pos, track)
    }

    fn pos(&self) -> i32 {
        self.info(false, false, true, false).nPos
    }

    fn set_info(
        &self,
        range: Option<i32>,
        page: Option<u32>,
        pos: Option<i32>,
        track: Option<i32>,
    ) -> i32 {
        self.hwnd
            .set_scroll_info(self.bar, range.map(|max| (0, max)), page, pos, track)
    }

    fn scroll(&self, delta: i32) {
        let (dx, dy) = match self.bar {
            SB_VERT => (0, delta),
            SB_HORZ => (delta, 0),
            _ => unreachable!("no supported scroll bar!"),
        };

        self.hwnd.scroll(-dx, -dy);
    }
}
