#![windows_subsystem = "windows"]

use anyhow::Result;

pub mod app;
pub mod config;
pub mod graphics;
pub mod gui;
pub mod visualize;

fn main() -> Result<()> {
    gui::run::<app::App>()
}
