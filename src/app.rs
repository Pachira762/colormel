use core::f32;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use windows::Win32::Foundation::{HWND, RECT};

use crate::{
    check, col,
    config::*,
    graphics::math::Matrix,
    gui::{control::Builder, hwnd::Hwnd, utils::Rect as _},
    radio, row, slider, space, text,
    visualize::Visualizer,
};

const ID_ENABLE_FILTER: u32 = 0x0100;
const ID_FILTER_RGB: u32 = 0x0101;
const ID_FILTER_CH_R: u32 = 0x0111;
const ID_FILTER_CH_G: u32 = 0x0112;
const ID_FILTER_CH_B: u32 = 0x0113;
const ID_FILTER_HUE: u32 = 0x0102;
const ID_FILTER_SAT: u32 = 0x0103;
const ID_FILTER_LUMA: u32 = 0x0104;
const ID_ENABLE_HISTOGRAM: u32 = 0x0200;
const ID_HISTOGRAM_RGB: u32 = 0x0201;
const ID_HISTOGRAM_RGBL: u32 = 0x0202;
const ID_HISTOGRAM_LUMA: u32 = 0x0203;
const ID_HISTOGRAM_HUE: u32 = 0x0204;
const ID_HISTOGRAM_SCALE: u32 = 0x0211;
const ID_ENABLE_COLORCLOUD: u32 = 0x0300;
const ID_COLORCLOUD_RGB: u32 = 0x0301;
const ID_COLORCLOUD_HSL: u32 = 0x0302;
const ID_COLORCLOUD_BG: u32 = 0x0311;
const ID_COLORCLOUD_GRID: u32 = 0x0312;

const CONFIG_PATH: &str = "colormel.ini";

pub struct App {
    hwnd: HWND,

    config: Arc<Mutex<Config>>,

    transparency: bool,

    #[allow(unused)]
    visualizer: Visualizer,
}

impl crate::gui::app::App for App {
    fn new(hwnd: HWND) -> Result<Self> {
        let config = Arc::new(Mutex::new(Config::load(CONFIG_PATH)));
        let transparency = config
            .lock()
            .map_or(true, |config| !config.enable_color_cloud);

        let visualizer = Visualizer::new(hwnd, Arc::clone(&config))?;

        Ok(Self {
            hwnd,
            config,
            transparency,
            visualizer,
        })
    }

    fn on_destroy(&mut self) -> Result<()> {
        if let Ok(config) = self.config.lock() {
            config.save(CONFIG_PATH);
        }
        Ok(())
    }

    fn on_pos_changed(&mut self, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
        if let Ok(mut config) = self.config.lock() {
            config.window_rect = RECT::new(x, y, width, height);
        }
        Ok(())
    }

    fn on_button(&mut self, id: u32, checked: bool) {
        let mut config = match self.config.lock() {
            Ok(config) => config,
            _ => return,
        };

        match id {
            ID_ENABLE_FILTER => {
                config.enable_filter = checked;
            }
            ID_FILTER_RGB => {
                config.filter_mode = FILTER_MODE_RGB;
            }
            ID_FILTER_HUE => {
                config.filter_mode = FILTER_MODE_HUE;
            }
            ID_FILTER_SAT => {
                config.filter_mode = FILTER_MODE_SAT;
            }
            ID_FILTER_LUMA => {
                config.filter_mode = FILTER_MODE_LUMA;
            }
            ID_FILTER_CH_R => {
                config.filter_channels[0] = checked;
            }
            ID_FILTER_CH_G => {
                config.filter_channels[1] = checked;
            }
            ID_FILTER_CH_B => {
                config.filter_channels[2] = checked;
            }
            ID_ENABLE_HISTOGRAM => {
                config.enable_histogram = checked;
            }
            ID_HISTOGRAM_RGB => {
                config.histogram_mode = HISTOGRAM_MODE_RGB;
            }
            ID_HISTOGRAM_RGBL => {
                config.histogram_mode = HISTOGRAM_MODE_RGBL;
            }
            ID_HISTOGRAM_LUMA => {
                config.histogram_mode = HISTOGRAM_MODE_LUMA;
            }
            ID_HISTOGRAM_HUE => {
                config.histogram_mode = HISTOGRAM_MODE_HUE;
            }
            ID_ENABLE_COLORCLOUD => {
                config.enable_color_cloud = checked;
                self.transparency = !config.enable_color_cloud;
            }
            ID_COLORCLOUD_RGB => {
                config.color_cloud_mode = COLORCLOUD_MODE_RGB;
            }
            ID_COLORCLOUD_HSL => {
                config.color_cloud_mode = COLORCLOUD_MODE_HSL;
            }
            ID_COLORCLOUD_GRID => {
                config.show_grid = checked;
            }
            _ => {}
        }
    }

    fn on_slider(&mut self, id: u32, val: i32) {
        let mut config = match self.config.lock() {
            Ok(config) => config,
            _ => return,
        };

        match id {
            ID_HISTOGRAM_SCALE => {
                config.histogram_scale = val as f32 / 100.0;
            }
            ID_COLORCLOUD_BG => {
                config.bg_opacity = val as f32 / 100.0;
            }
            _ => {}
        }
    }

    fn on_drag(&mut self, dx: i32, dy: i32) {
        let rect = self.hwnd.rect();
        let div = rect.width().min(rect.height()) as f32;
        let dx = dx as f32 / div;
        let dy = dy as f32 / div;

        if let Ok(mut config) = self.config.lock() {
            let rot = Matrix::mul(
                &Matrix::rot_y(f32::consts::PI * -dx),
                &Matrix::rot_x(f32::consts::PI * -dy),
            );
            config.rotation = Matrix::mul(&config.rotation, &rot);
        }
    }

    fn window_rect(&mut self) -> RECT {
        if let Ok(config) = self.config.lock() {
            config.window_rect
        } else {
            RECT::new(100, 100, 1280, 720)
        }
    }

    fn transparency(&mut self) -> bool {
        self.transparency
    }

    fn build_menu(&mut self, builder: &mut Builder) -> Result<()> {
        let config = self.config.lock().unwrap().to_owned();

        builder.build(col!(
            check!(ID_ENABLE_FILTER, "Filter", config.enable_filter),
            col!(
                indent: 16,
                radio!(ID_FILTER_RGB, "RGB", config.filter_mode == FILTER_MODE_RGB, ID_FILTER_RGB),
                row!(
                    indent: 12,
                    check!(width: 36, ID_FILTER_CH_R, "R", true),
                    check!(width: 36, ID_FILTER_CH_G, "G", true),
                    check!(width: 36, ID_FILTER_CH_B, "B", true)
                ),
                radio!(ID_FILTER_HUE, "Hue", config.filter_mode == FILTER_MODE_HUE, ID_FILTER_RGB),
                radio!(ID_FILTER_SAT, "Saturat", config.filter_mode == FILTER_MODE_SAT, ID_FILTER_RGB),
                radio!(ID_FILTER_LUMA, "Luma", config.filter_mode == FILTER_MODE_LUMA, ID_FILTER_RGB)
            ),
            space!(8),
            check!(ID_ENABLE_HISTOGRAM, "Histogram", config.enable_histogram),
            col!(
                indent: 16,
                radio!(ID_HISTOGRAM_RGB, "RGB", config.histogram_mode == HISTOGRAM_MODE_RGB, ID_HISTOGRAM_RGB),
                radio!(ID_HISTOGRAM_RGBL, "RGBL", config.histogram_mode == HISTOGRAM_MODE_RGBL, ID_HISTOGRAM_RGB),
                radio!(ID_HISTOGRAM_LUMA, "Luma", config.histogram_mode == HISTOGRAM_MODE_LUMA, ID_HISTOGRAM_RGB),
                radio!(ID_HISTOGRAM_HUE, "Hue", config.histogram_mode == HISTOGRAM_MODE_HUE, ID_HISTOGRAM_RGB),
                text!(" Scale"),
                slider!(ID_HISTOGRAM_SCALE, 0, 100, (100.0 * config.histogram_scale) as i32),
            ),
            space!(8),
            check!(ID_ENABLE_COLORCLOUD, "Colod-Cloud", config.enable_color_cloud),
            col!(
                indent: 16,
                radio!(ID_COLORCLOUD_RGB, "RGB", config.color_cloud_mode == COLORCLOUD_MODE_RGB, ID_COLORCLOUD_RGB),
                radio!(ID_COLORCLOUD_HSL, "HSL", config.color_cloud_mode == COLORCLOUD_MODE_HSL, ID_COLORCLOUD_RGB),
                check!(ID_COLORCLOUD_GRID, "Show Grid", config.show_grid),
            ),
            space!(8),
            text!(" Transparency"),
            slider!(ID_COLORCLOUD_BG, 0, 100, (100.0 * config.bg_opacity) as i32),
        ))
    }
}
