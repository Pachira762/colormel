use anyhow::Result;
use windows::Win32::{
    Foundation::RECT,
    Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
        Direct3D12::{
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        },
        Dxgi::Common::{
            DXGI_FORMAT_D16_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32_UINT,
        },
    },
};

use crate::{
    config::Config,
    graphics::{
        core::{pso::PipelineState, wrap::*},
        initializer::Initializer,
        math,
        renderer::Renderer,
        resource::RwBuffer,
    },
    gui::utils::Rect as _,
};

pub struct ColorCloud {
    compute_pso: PipelineState,
    draw_pso: PipelineState,
    counter: RwBuffer,
}

impl ColorCloud {
    pub fn new(ctx: &mut Initializer) -> Result<Self> {
        let compute_pso =
            ctx.create_compute_pipeline(include_bytes!("../shaders/bin/ColorCloudCs.bin"), None)?;

        let draw_pso = ctx.create_mesh_pipeline(
            include_bytes!("../shaders/bin/ColorCloudAs.bin"),
            include_bytes!("../shaders/bin/ColorCloudMs.bin"),
            include_bytes!("../shaders/bin/ColorCloudPs.bin"),
            BlendDesc::none(),
            RasterizerDesc::none(),
            DepthStencilDesc::depth(),
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            RtvFormats::single(DXGI_FORMAT_R16G16B16A16_FLOAT),
            Some(DXGI_FORMAT_D16_UNORM),
            None,
        )?;

        const NUM_ELEMS: u32 = 256 * 256 * 256;
        let counter = RwBuffer::new(ctx, NUM_ELEMS, DXGI_FORMAT_R32_UINT)?;

        Ok(Self {
            compute_pso,
            draw_pso,
            counter,
        })
    }

    pub fn process(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        if config.enable_color_cloud {
            self.clear(ctx, config)?;
            self.compute(ctx, config)?;
            self.draw(ctx, config)?;
        }
        Ok(())
    }

    fn clear(&mut self, ctx: &mut Renderer, _config: &Config) -> Result<()> {
        ctx.resource_barrier(&[self.counter.transition_barrier(
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        )]);

        ctx.clear_uav(self.counter.raw_uav, &self.counter);

        Ok(())
    }

    fn compute(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        ctx.set_pipeline_state(&self.compute_pso);

        #[repr(C)]
        struct Params {
            rect: RECT,
        }

        const THREAD: u32 = 8;
        let dim_x = math::div_round_up(config.window_rect.width() as u32, THREAD);
        let dim_y = math::div_round_up(config.window_rect.height() as u32, THREAD);

        let params = Params {
            rect: config.window_rect,
        };
        ctx.set_uavs(&[self.counter.uav]);
        ctx.set_compute_constants(&params);
        ctx.dispatch(dim_x, dim_y, 1);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        ctx.set_pipeline_state(&self.draw_pso);
        ctx.set_viewport(crate::graphics::renderer::ViewportKind::Adjust);
        ctx.set_primitive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);

        ctx.resource_barrier(&[self.counter.transition_barrier(
            D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        )]);

        let width = config.window_rect.width();
        let height = config.window_rect.height();
        let min_count = 0;
        let max_count = width * height / 9;

        #[repr(C)]
        struct Params {
            projection: [f32; 12],
            min_count: u32,
            inv_max_count: f32,
            color_space: u32,
        }

        let params = Params {
            projection: config.projection_matrix().as_4x3(),
            min_count,
            inv_max_count: 1.0 / (max_count as f32),
            color_space: config.color_cloud_mode,
        };
        ctx.set_graphics_constants(&params);
        ctx.set_graphics_srvs(&[self.counter.srv]);

        const GRID: u32 = 8;
        ctx.dispatch_mesh(256 / GRID, 256 / GRID, 256 / GRID);

        Ok(())
    }
}
