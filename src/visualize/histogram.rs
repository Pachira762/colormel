use anyhow::Result;
use windows::Win32::{
    Foundation::RECT,
    Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
        Direct3D12::*,
        Dxgi::Common::{DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32_UINT},
    },
};

use crate::{
    config::{Config, HISTOGRAM_MODE_HUE, HISTOGRAM_MODE_RGB, HISTOGRAM_MODE_RGBL},
    graphics::{
        core::{
            pso::PipelineState,
            wrap::{BlendDesc, DepthStencilDesc, RasterizerDesc, RtvFormats},
        },
        initializer::Initializer,
        math,
        renderer::Renderer,
        resource::RwBuffer,
    },
    gui::utils::Rect as _,
};

pub struct Histogram {
    compute_pso: PipelineState,
    draw_pso: PipelineState,
    buffers: [RwBuffer; 4],
}

impl Histogram {
    pub fn new(ctx: &mut Initializer) -> Result<Self> {
        let compute_pso =
            ctx.create_compute_pipeline(include_bytes!("../shaders/bin/HistogramCs.bin"), None)?;

        let draw_pso = ctx.create_graphics_pipeline(
            include_bytes!("../shaders/bin/HistogramVs.bin"),
            include_bytes!("../shaders/bin/HistogramPs.bin"),
            BlendDesc::mul(),
            RasterizerDesc::none(),
            DepthStencilDesc::none(),
            &[],
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            RtvFormats::single(DXGI_FORMAT_R16G16B16A16_FLOAT),
            None,
            None,
        )?;

        const NUM_ELEMS: u32 = 256;
        let buffers = [
            RwBuffer::new(ctx, NUM_ELEMS, DXGI_FORMAT_R32_UINT)?,
            RwBuffer::new(ctx, NUM_ELEMS, DXGI_FORMAT_R32_UINT)?,
            RwBuffer::new(ctx, NUM_ELEMS, DXGI_FORMAT_R32_UINT)?,
            RwBuffer::new(ctx, NUM_ELEMS, DXGI_FORMAT_R32_UINT)?,
        ];

        Ok(Self {
            compute_pso,
            draw_pso,
            buffers,
        })
    }

    pub fn process(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        if config.enable_histogram {
            self.clear(ctx)?;
            self.compute(config, ctx)?;
            self.draw(config, ctx)?;
        }
        Ok(())
    }

    fn clear(&mut self, ctx: &mut Renderer) -> Result<()> {
        let barriers: Vec<_> = self
            .buffers
            .iter()
            .map(|buffer| {
                buffer.transition_barrier(
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                )
            })
            .collect();
        ctx.resource_barrier(&barriers);

        for buffer in &self.buffers {
            ctx.clear_uav(buffer.raw_uav, buffer);
        }

        Ok(())
    }

    fn compute(&mut self, config: &Config, ctx: &mut Renderer) -> Result<()> {
        ctx.set_pipeline_state(&self.compute_pso);

        #[repr(C)]
        struct Params {
            rect: RECT,
            mode: u32,
            ch: u32,
        }
        let ch = match config.histogram_mode {
            HISTOGRAM_MODE_RGB => 3,
            HISTOGRAM_MODE_RGBL => 4,
            _ => 1,
        };
        let params = Params {
            rect: config.window_rect,
            mode: config.histogram_mode,
            ch,
        };
        ctx.set_compute_constants(&params);
        ctx.set_uavs(&[
            self.buffers[0].uav,
            self.buffers[1].uav,
            self.buffers[2].uav,
            self.buffers[3].uav,
        ]);

        let threads = 2 * 8;
        ctx.dispatch(
            math::div_round_up(config.window_rect.width() as u32, threads),
            math::div_round_up(config.window_rect.height() as u32, threads),
            1,
        );

        Ok(())
    }

    fn draw(&mut self, config: &Config, ctx: &mut Renderer) -> Result<()> {
        ctx.set_pipeline_state(&self.draw_pso);
        ctx.set_viewport(crate::graphics::renderer::ViewportKind::Full);

        ctx.set_primitive_topology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);

        let barriers: Vec<_> = self
            .buffers
            .iter()
            .map(|buf| {
                buf.transition_barrier(
                    D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                    D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                )
            })
            .collect();
        ctx.resource_barrier(&barriers);

        #[repr(C)]
        struct Params {
            colors: [[f32; 4]; 4],
            mode: u32,
            scale: f32,
        }

        let (width, height) = config.window_rect.size();
        let scale = config.histogram_scale
            * if config.histogram_mode == HISTOGRAM_MODE_HUE {
                0.20 / ((width * height) as f32)
            } else {
                10.0 / ((width * height) as f32)
            };

        let params = Params {
            colors: [
                [0.0, 1.0, 0.0, 0.8],
                [1.0, 0.0, 0.0, 0.8],
                [0.0, 0.0, 1.0, 0.8],
                [1.0, 1.0, 1.0, 0.8],
            ],
            mode: config.histogram_mode as _,
            scale,
        };

        ctx.set_graphics_constants(&params);
        ctx.set_graphics_srvs(&[
            self.buffers[0].srv,
            self.buffers[1].srv,
            self.buffers[2].srv,
            self.buffers[3].srv,
        ]);

        let ch = match config.histogram_mode {
            HISTOGRAM_MODE_RGB => 3,
            HISTOGRAM_MODE_RGBL => 4,
            _ => 1,
        };
        ctx.draw(2 * 256, ch);

        Ok(())
    }
}
