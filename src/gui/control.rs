#![allow(unused, clippy::too_many_arguments)]

use std::collections::HashMap;

use anyhow::Result;
use windows::{
    core::{s, w, PCSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::{CreateFontIndirectA, HFONT},
        UI::{Controls::*, WindowsAndMessaging::*},
    },
};

use super::{
    hwnd::{CheckBox, Hwnd, Trackbar},
    utils::create_window,
};

pub enum Ctrl {
    Space {
        size: i32,
    },
    Row {
        indent: i32,
        elems: Vec<Ctrl>,
    },
    Col {
        indent: i32,
        elems: Vec<Ctrl>,
    },
    Text {
        width: i32,
        height: i32,
        text: PCSTR,
    },
    Check {
        width: i32,
        height: i32,
        id: u32,
        text: PCSTR,
        checked: bool,
    },
    Radio {
        width: i32,
        height: i32,
        id: u32,
        text: PCSTR,
        checked: bool,
        group: u32,
    },
    Slider {
        width: i32,
        height: i32,
        id: u32,
        min: i32,
        max: i32,
        val: i32,
    },
}

pub struct Builder {
    parent: HWND,
    font: HFONT,
    groups: HashMap<u32, Vec<RadioParam>>,
    width: u32,
    height: u32,
}

impl Builder {
    pub fn new(parent: HWND) -> Result<Self> {
        unsafe {
            let size = std::mem::size_of::<NONCLIENTMETRICSA>() as u32;
            let mut ncm = NONCLIENTMETRICSA {
                cbSize: size,
                ..Default::default()
            };
            SystemParametersInfoA(
                SPI_GETNONCLIENTMETRICS,
                size,
                Some(&mut ncm as *mut _ as _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
            )?;

            ncm.lfCaptionFont.lfHeight *= 125;
            ncm.lfCaptionFont.lfHeight /= 100;
            let font = CreateFontIndirectA(&ncm.lfCaptionFont);

            Ok(Self {
                parent,
                font,
                groups: HashMap::new(),
                width: 0,
                height: 0,
            })
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn build(&mut self, root: Ctrl) -> Result<()> {
        let x = 12;
        let y = 8;
        let (w, h) = self.build_control(12, 8, root)?;

        self.create_radios()?;

        self.width = 2 * x + w as u32;
        self.height = 2 * y + h as u32;

        Ok(())
    }

    fn build_control(&mut self, x: i32, y: i32, ctrl: Ctrl) -> Result<(i32, i32)> {
        use Ctrl::*;

        match ctrl {
            Space { size } => Ok((size, size)),
            Row { indent, elems } => self.build_row(x, y, indent, elems),
            Col { indent, elems } => self.build_col(x, y, indent, elems),
            Text {
                width,
                height,
                text,
            } => self.create_text(x, y, width, height, text),
            Check {
                width,
                height,
                id,
                text,
                checked,
            } => self.create_check(x, y, width, height, id, text, checked),
            Radio {
                width,
                height,
                id,
                text,
                checked,
                group,
            } => self.add_radio(x, y, width, height, id, text, checked, group),
            Slider {
                width,
                height,
                id,
                min,
                max,
                val,
            } => self.create_slider(x, y, width, height, id, min, max, val),
        }
    }

    fn build_row(
        &mut self,
        mut x: i32,
        y: i32,
        indent: i32,
        elems: Vec<Ctrl>,
    ) -> Result<(i32, i32)> {
        let mut width = 0;
        let mut height = 0;

        for ctrl in elems {
            let (w, h) = self.build_control(x + indent, y, ctrl)?;
            x += w;
            width += w;
            height = height.max(h);
        }

        Ok((width + indent, height))
    }

    fn build_col(
        &mut self,
        x: i32,
        mut y: i32,
        indent: i32,
        elems: Vec<Ctrl>,
    ) -> Result<(i32, i32)> {
        let mut width = 0;
        let mut height = 0;

        for ctrl in elems {
            let (w, h) = self.build_control(x + indent, y, ctrl)?;
            y += h;
            width = width.max(w);
            height += h;
        }

        Ok((width + indent, height))
    }

    fn create_text(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        text: PCSTR,
    ) -> Result<(i32, i32)> {
        let hwnd =
            self.create_control(WINDOW_STYLE(0), s!("STATIC"), text, x, y, width, height, 0)?;

        Ok((width, height))
    }

    fn create_check(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: u32,
        text: PCSTR,
        checked: bool,
    ) -> Result<(i32, i32)> {
        let style = WINDOW_STYLE(BS_AUTOCHECKBOX as _);
        let hwnd = self.create_control(style, s!("BUTTON"), text, x, y, width, height, id)?;

        hwnd.checkbox_set_check(checked);

        Ok((width, height))
    }

    fn add_radio(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: u32,
        text: PCSTR,
        checked: bool,
        group: u32,
    ) -> Result<(i32, i32)> {
        let radio = RadioParam {
            text,
            x,
            y,
            width,
            height,
            id,
            checked,
        };

        self.groups
            .entry(group)
            .and_modify(|v| v.push(radio))
            .or_insert(vec![radio]);

        Ok((width, height))
    }

    fn create_radio(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: u32,
        text: PCSTR,
        checked: bool,
        group: bool,
    ) -> Result<()> {
        let style = WINDOW_STYLE(BS_AUTORADIOBUTTON as _)
            | if group {
                WS_GROUP
            } else {
                WINDOW_STYLE::default()
            };
        let hwnd = self.create_control(style, s!("BUTTON"), text, x, y, width, height, id)?;

        hwnd.checkbox_set_check(checked);

        Ok(())
    }

    fn create_radios(&self) -> Result<()> {
        for radios in self.groups.values() {
            let mut group = true;
            for radio in radios {
                self.create_radio(
                    radio.x,
                    radio.y,
                    radio.width,
                    radio.height,
                    radio.id,
                    radio.text,
                    radio.checked,
                    group,
                )?;
                group = false;
            }
        }

        Ok(())
    }

    fn create_slider(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: u32,
        min: i32,
        max: i32,
        val: i32,
    ) -> Result<(i32, i32)> {
        let hwnd = self.create_control(
            WINDOW_STYLE(0),
            TRACKBAR_CLASSA,
            s!("Trackbar"),
            x,
            y,
            width,
            height,
            id,
        )?;

        hwnd.trackbar_set_min_max(min, max);
        hwnd.trackbar_set_pos(val);

        Ok((width, height))
    }

    fn create_control(
        &self,
        style: WINDOW_STYLE,
        class_name: PCSTR,
        window_name: PCSTR,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        id: u32,
    ) -> Result<HWND> {
        let hwnd = create_window(
            WINDOW_EX_STYLE::default(),
            class_name,
            window_name,
            style | WS_CHILD | WS_VISIBLE,
            x,
            y,
            width,
            height,
            self.parent,
            HMENU(id as _),
            None,
        )?;

        hwnd.send_message(WM_SETFONT, WPARAM(self.font.0 as _), LPARAM(1));
        hwnd.set_theme(w!("DarkMode_Explorer"));

        Ok(hwnd)
    }
}

#[macro_export]
macro_rules! space {
    ($size:expr) => {
        $crate::gui::control::Ctrl::Space { size: $size }
    };
}

#[macro_export]
macro_rules! col {
    (indent: $indent:expr, $($elem:expr),* $(,)?) => {
        $crate::gui::control::Ctrl::Col {
            indent: $indent,
            elems: vec![$($elem),*],
        }
    };
    ($($elem:expr),* $(,)?) => {
        $crate::gui::control::Ctrl::Col {
            indent: 0,
            elems: vec![$($elem),*],
        }
    };
}

#[macro_export]
macro_rules! row {
    (indent: $indent:expr, $($elem:expr),* $(,)?) => {
        $crate::gui::control::Ctrl::Row {
            indent: $indent,
            elems: vec![$($elem),*],
        }
    };
    ($($elem:expr),* $(,)?) => {
        $crate::gui::control::Ctrl::Row {
            indent: 0,
            elems: vec![$($elem),*],
        }
    };
}

#[macro_export]
macro_rules! text {
    ($text:literal) => {
        $crate::gui::control::Ctrl::Text {
            width: 10 * $text.len() as i32,
            height: 24,
            text: ::windows::core::s!($text),
        }
    };
}

#[macro_export]
macro_rules! check {
    ($id:expr, $text:literal, $checked:expr) => {
        $crate::gui::control::Ctrl::Check {
            width: 80.max(10 * $text.len() as i32),
            height: 24,
            id: $id,
            text: ::windows::core::s!($text),
            checked: $checked,
        }
    };
    (width: $width:expr, $id:expr, $text:literal, $checked:expr) => {
        $crate::gui::control::Ctrl::Check {
            width: $width,
            height: 24,
            id: $id,
            text: ::windows::core::s!($text),
            checked: $checked,
        }
    };
}

#[macro_export]
macro_rules! radio {
    ($id:expr, $text:literal, $checked:expr, $group:expr) => {
        $crate::gui::control::Ctrl::Radio {
            width: 80.max(10 * $text.len() as i32),
            height: 24,
            id: $id,
            text: ::windows::core::s!($text),
            checked: $checked,
            group: $group,
        }
    };
}

#[macro_export]
macro_rules! slider {
    ($id:expr, $min:expr, $max:expr, $val:expr) => {
        $crate::gui::control::Ctrl::Slider {
            width: 100,
            height: 24,
            id: $id,
            min: $min,
            max: $max,
            val: $val,
        }
    };
}

#[derive(Clone, Copy)]
struct RadioParam {
    text: PCSTR,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    id: u32,
    checked: bool,
}
