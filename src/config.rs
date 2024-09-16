#![allow(unused)]

use std::path::{Path, PathBuf};

use ini::{Ini, SectionSetter};
use windows::Win32::Foundation::RECT;

use crate::{graphics::math::Matrix, gui::utils::Rect};

pub const FILTER_MODE_RGB: u32 = 0;
pub const FILTER_MODE_HUE: u32 = 1;
pub const FILTER_MODE_SAT: u32 = 2;
pub const FILTER_MODE_LUMA: u32 = 3;
pub const HISTOGRAM_MODE_RGB: u32 = 0;
pub const HISTOGRAM_MODE_RGBL: u32 = 1;
pub const HISTOGRAM_MODE_LUMA: u32 = 2;
pub const HISTOGRAM_MODE_HUE: u32 = 3;
pub const COLORCLOUD_MODE_RGB: u32 = 0;
pub const COLORCLOUD_MODE_HSL: u32 = 1;

#[derive(Clone, Debug)]
pub struct Config {
    pub enable_filter: bool,
    pub filter_mode: u32,
    pub filter_channels: [bool; 4],
    pub enable_histogram: bool,
    pub histogram_mode: u32,
    pub histogram_scale: f32,
    pub enable_color_cloud: bool,
    pub color_cloud_mode: u32,
    pub show_grid: bool,
    pub bg_opacity: f32,
    pub window_rect: RECT,
    pub rotation: Matrix,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Self {
        if let Ok(conf) = Ini::load_from_file_noescape(&path) {
            let window_x = conf.get_i32("window-x", 100).max(0);
            let window_y = conf.get_i32("window-y", 100).max(0);
            let window_width = conf.get_i32("window-width", 640).max(0);
            let window_height = conf.get_i32("window-height", 480).max(0);

            Self {
                enable_filter: conf.get_bool("enable-filter"),
                filter_mode: conf.get_u32("filter-mode", 0),
                filter_channels: [true; 4],
                enable_histogram: conf.get_bool("enable-histogram"),
                histogram_mode: conf.get_u32("histogram-mode", 0),
                histogram_scale: conf.get_f32("histogram-scale", 0.5),
                enable_color_cloud: conf.get_bool("enable-color-cloud"),
                color_cloud_mode: conf.get_u32("color-cloud-mode", 0),
                show_grid: conf.get_bool("show-grid"),
                bg_opacity: conf.get_f32("bg-opacity", 1.0),
                window_rect: RECT::new(window_x, window_y, window_width, window_height),
                rotation: Matrix::identity(),
            }
        } else {
            Self {
                enable_filter: false,
                filter_mode: 0,
                filter_channels: [true; 4],
                enable_histogram: false,
                histogram_mode: 0,
                histogram_scale: 0.5,
                enable_color_cloud: false,
                color_cloud_mode: 0,
                show_grid: false,
                bg_opacity: 1.0,
                window_rect: RECT::new(100, 100, 1280, 720),
                rotation: Matrix::identity(),
            }
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) {
        let mut conf = Ini::new();

        conf.with_general_section()
            .set_bool("enable-filter", self.enable_filter)
            .set_u32("filter-mode", self.filter_mode)
            .set_bool("enable-histogram", self.enable_histogram)
            .set_u32("histogram-mode", self.histogram_mode)
            .set_f32("histogram-scale", self.histogram_scale)
            .set_bool("enable-color-cloud", self.enable_color_cloud)
            .set_u32("color-cloud-mode", self.color_cloud_mode)
            .set_bool("show-grid", self.show_grid)
            .set_f32("bg-opacity", self.bg_opacity)
            .set_i32("window-x", self.window_rect.left)
            .set_i32("window-y", self.window_rect.top)
            .set_i32("window-width", self.window_rect.width())
            .set_i32("window-height", self.window_rect.height());

        _ = conf.write_to_file(path);
    }

    pub fn projection_matrix(&self) -> Matrix {
        let (width, height) = self.window_rect.size();
        let scale = 0.9 * width.min(height) as f32 / width.max(height) as f32;

        self.rotation
            .mul(&Matrix::scale(scale, scale, 0.25))
            .mul(&Matrix::translate(0.0, 0.0, 0.5))
    }
}

trait IniSetter<'a> {
    fn set_bool(&'a mut self, key: &str, value: bool) -> &'a mut SectionSetter<'a>;
    fn set_i32(&'a mut self, key: &str, value: i32) -> &'a mut SectionSetter<'a>;
    fn set_u32(&'a mut self, key: &str, value: u32) -> &'a mut SectionSetter<'a>;
    fn set_f32(&'a mut self, key: &str, value: f32) -> &'a mut SectionSetter<'a>;
}

impl<'a> IniSetter<'a> for SectionSetter<'a> {
    fn set_bool(&'a mut self, key: &str, value: bool) -> &'a mut SectionSetter<'a> {
        self.set(key, (value as u32).to_string())
    }

    fn set_i32(&'a mut self, key: &str, value: i32) -> &'a mut SectionSetter<'a> {
        self.set(key, value.to_string())
    }

    fn set_u32(&'a mut self, key: &str, value: u32) -> &'a mut SectionSetter<'a> {
        self.set(key, value.to_string())
    }

    fn set_f32(&'a mut self, key: &str, value: f32) -> &'a mut SectionSetter<'a> {
        self.set(key, value.to_string())
    }
}

trait IniGetter {
    fn get_bool(&self, key: &str) -> bool;
    fn get_i32(&self, key: &str, default: i32) -> i32;
    fn get_u32(&self, key: &str, default: u32) -> u32;
    fn get_f32(&self, key: &str, default: f32) -> f32;
}

impl IniGetter for Ini {
    fn get_bool(&self, key: &str) -> bool {
        matches!(self.get_from::<String>(None, key), Some("1"))
    }

    fn get_i32(&self, key: &str, default: i32) -> i32 {
        self.get_from::<String>(None, key)
            .unwrap_or_default()
            .parse::<i32>()
            .unwrap_or(default)
    }

    fn get_u32(&self, key: &str, default: u32) -> u32 {
        self.get_from::<String>(None, key)
            .unwrap_or_default()
            .parse::<u32>()
            .unwrap_or(default)
    }

    fn get_f32(&self, key: &str, default: f32) -> f32 {
        self.get_from::<String>(None, key)
            .unwrap_or_default()
            .parse::<f32>()
            .unwrap_or(default)
    }
}
