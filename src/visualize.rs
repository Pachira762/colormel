mod colorcloud;
mod filter;
mod grid;
mod histogram;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Duration,
};

use anyhow::Result;
use colorcloud::ColorCloud;
use filter::Filter;
use grid::Grids;
use histogram::Histogram;
use windows::Win32::Foundation::HWND;

use crate::{
    config::Config,
    graphics::{context::Context, duplicate::Duplication},
    gui::utils::Rect,
};

pub struct Visualizer {
    keep_running: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl Visualizer {
    pub fn new(hwnd: HWND, config: Arc<Mutex<Config>>) -> Result<Self> {
        let mut pipeline = Pipeline::new(hwnd)?;

        let keep_running = Arc::new(AtomicBool::new(true));
        let keep_running2 = Arc::clone(&keep_running);

        let join_handle = std::thread::spawn(move || {
            while keep_running2.load(Ordering::Relaxed) {
                let config = if let Ok(config) = config.lock() {
                    config.to_owned()
                } else {
                    break;
                };

                if let Err(e) = pipeline.process(config) {
                    println!("{e:?}");
                    break;
                }
            }
        });

        Ok(Self {
            keep_running,
            join_handle: Some(join_handle),
        })
    }

    pub fn terminate(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);

        if let Some(join_handle) = self.join_handle.take() {
            _ = join_handle.join();
        }
    }
}

impl Drop for Visualizer {
    fn drop(&mut self) {
        self.terminate();
    }
}

struct Pipeline {
    ctx: Context,
    dupl: Duplication,
    colorcloud: ColorCloud,
    filter: Filter,
    histogram: Histogram,
    grids: Grids,
}

impl Pipeline {
    fn new(hwnd: HWND) -> Result<Self> {
        let mut ctx = Context::new(hwnd)?;
        let mut initializer = ctx.create_initializer()?;

        let dupl = Duplication::new(&mut initializer)?;
        let colorcloud = ColorCloud::new(&mut initializer)?;
        let filter = Filter::new(&mut initializer)?;
        let histogram = Histogram::new(&mut initializer)?;
        let grids = Grids::new(&mut initializer)?;

        Ok(Self {
            ctx,
            dupl,
            colorcloud,
            filter,
            histogram,
            grids,
        })
    }

    fn process(&mut self, config: Config) -> Result<()> {
        let srv = if let Some(srv) = self.dupl.duplicate(&self.ctx)? {
            srv
        } else {
            std::thread::sleep(Duration::from_millis(10));
            return Ok(());
        };

        let opacity = 1.0 - config.bg_opacity;
        let mut renderer = self.ctx.create_renderer(
            config.window_rect.width() as _,
            config.window_rect.height() as _,
            &[0.0, 0.0, 0.0, opacity],
        )?;

        renderer.set_shared_srv(srv);

        self.filter.process(&mut renderer, &config)?;
        self.colorcloud.process(&mut renderer, &config)?;
        self.grids.process(&mut renderer, &config)?;
        self.histogram.process(&mut renderer, &config)?;

        self.ctx.execute(renderer)?;

        Ok(())
    }
}
