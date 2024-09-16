use anyhow::Result;
use windows::Win32::Foundation::{HWND, RECT};

use super::control::Builder;

pub trait App: Sized {
    fn new(hwnd: HWND) -> Result<Self>;

    fn on_destroy(&mut self) -> Result<()>;

    fn on_pos_changed(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()>;

    fn on_button(&mut self, id: u32, checked: bool);

    fn on_slider(&mut self, id: u32, val: i32);

    fn on_drag(&mut self, dx: i32, dy: i32);

    fn window_rect(&mut self) -> RECT;

    fn transparency(&mut self) -> bool;

    fn build_menu(&mut self, builder: &mut Builder) -> Result<()>;
}
