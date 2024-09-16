use anyhow::Result;
use windows::Win32::{
    Foundation::RECT,
    Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
        Direct3D12::D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        Dxgi::Common::DXGI_FORMAT_R16G16B16A16_FLOAT,
    },
};

use crate::{
    config::Config,
    graphics::{
        core::{pso::PipelineState, wrap::*},
        initializer::Initializer,
        renderer::Renderer,
    },
};

#[allow(unused)]
pub struct Filter {
    pso: PipelineState,
}

impl Filter {
    pub fn new(ctx: &mut Initializer) -> Result<Self> {
        let pso = ctx.create_graphics_pipeline(
            include_bytes!("../shaders/bin/FilterVs.bin"),
            include_bytes!("../shaders/bin/FilterPs.bin"),
            BlendDesc::none(),
            RasterizerDesc::none(),
            DepthStencilDesc::none(),
            &[],
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            RtvFormats::single(DXGI_FORMAT_R16G16B16A16_FLOAT),
            None,
            None,
        )?;

        Ok(Self { pso })
    }

    pub fn process(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        if config.enable_filter {
            self.draw(ctx, config)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        ctx.set_pipeline_state(&self.pso);
        ctx.set_viewport(crate::graphics::renderer::ViewportKind::Full);
        ctx.set_primitive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

        #[repr(C)]
        struct Params {
            rect: RECT,
            mode: u32,
            mask: [f32; 3],
        }

        let params = Params {
            rect: config.window_rect,
            mode: config.filter_mode,
            mask: channel_mask(&config.filter_channels),
        };
        ctx.set_graphics_constants(&params);

        ctx.draw(3, 1);

        Ok(())
    }
}

fn channel_mask(channels: &[bool]) -> [f32; 3] {
    fn mask(ch: bool) -> f32 {
        if ch {
            1.0
        } else {
            0.0
        }
    }

    [mask(channels[0]), mask(channels[1]), mask(channels[2])]
}
