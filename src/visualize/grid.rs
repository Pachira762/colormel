use core::f32::consts::PI;

use anyhow::Result;
use windows::{
    core::s,
    Win32::Graphics::{
        Direct3D::D3D_PRIMITIVE_TOPOLOGY_LINELIST,
        Direct3D12::{D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE, D3D12_RASTERIZER_DESC},
        Dxgi::Common::{
            DXGI_FORMAT_D16_UNORM, DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_R32G32B32_FLOAT,
        },
    },
};

use crate::{
    config::Config,
    graphics::{
        core::{pso::PipelineState, wrap::*},
        initializer::Initializer,
        renderer::Renderer,
        resource::VertexBuffer,
    },
};

pub struct Grids {
    pso: PipelineState,
    grids: [VertexBuffer; 2],
}

impl Grids {
    pub fn new(ctx: &mut Initializer) -> Result<Self> {
        let pso = ctx.create_graphics_pipeline(
            include_bytes!("../shaders/bin/PrimitiveVs.bin"),
            include_bytes!("../shaders/bin/PrimitivePs.bin"),
            BlendDesc::none(),
            D3D12_RASTERIZER_DESC {
                AntialiasedLineEnable: true.into(),
                ..RasterizerDesc::none()
            },
            DepthStencilDesc::depth(),
            &[
                InputElementDesc::per_vertex(s!("POSITION"), DXGI_FORMAT_R32G32B32_FLOAT),
                InputElementDesc::per_vertex(s!("COLOR"), DXGI_FORMAT_R32G32B32_FLOAT),
            ],
            D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
            RtvFormats::single(DXGI_FORMAT_R16G16B16A16_FLOAT),
            Some(DXGI_FORMAT_D16_UNORM),
            None,
        )?;

        let grids = [
            VertexBuffer::new(ctx, &rgb_grid())?,
            VertexBuffer::new(ctx, &hsl_grid(6, 48))?,
        ];

        Ok(Self { pso, grids })
    }

    pub fn process(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        if config.enable_color_cloud && config.show_grid {
            self.show(ctx, config)?;
        }

        Ok(())
    }

    fn show(&mut self, ctx: &mut Renderer, config: &Config) -> Result<()> {
        ctx.set_pipeline_state(&self.pso);
        ctx.set_viewport(crate::graphics::renderer::ViewportKind::Adjust);
        ctx.set_primitive_topology(D3D_PRIMITIVE_TOPOLOGY_LINELIST);

        let vertex_buffer = &self.grids[config.color_cloud_mode as usize];
        ctx.set_vertex_buffers(&[vertex_buffer.view()]);

        #[repr(C)]
        struct Params {
            projection: [f32; 12],
        }

        let params = Params {
            projection: config.projection_matrix().as_4x3(),
        };

        ctx.set_graphics_constants(&params);
        ctx.draw(vertex_buffer.vertex_count(), 1);

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}

fn rgb_grid() -> Vec<Vertex> {
    fn rgb_position(r: f32, g: f32, b: f32) -> [f32; 3] {
        [1.25 * (r - 0.5), 1.25 * (g - 0.5), 1.25 * (b - 0.5)]
    }

    fn rgb_vertex(r: f32, g: f32, b: f32) -> Vertex {
        Vertex::new(rgb_position(r, g, b), [r, g, b])
    }

    let v0 = rgb_vertex(0.0, 0.0, 0.0);
    let r = rgb_vertex(1.0, 0.0, 0.0);
    let g = rgb_vertex(0.0, 1.0, 0.0);
    let b = rgb_vertex(0.0, 0.0, 1.0);
    let rg = rgb_vertex(1.0, 1.0, 0.0);
    let rb = rgb_vertex(1.0, 0.0, 1.0);
    let gb = rgb_vertex(0.0, 1.0, 1.0);
    let v1 = rgb_vertex(1.0, 1.0, 1.0);

    vec![
        v0, r, v0, g, v0, b, r, rg, r, rb, g, rg, g, gb, b, rb, b, gb, rg, v1, rb, v1, gb, v1,
    ]
}

fn hsl_grid(n_hue: u32, n_div: u32) -> Vec<Vertex> {
    fn hsl_to_position(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
        let h = 2.0 * PI * hue;
        let mut s = saturation;
        let mut l = 2.0 * lightness - 1.0;

        let a = s + l.abs();
        let b = (s * s + l * l).sqrt();
        if b > 0.0 {
            let n = a / b;
            s *= n;
            l *= n;
        }

        let y = l;

        let (mut z, mut x) = h.sin_cos();
        x *= s;
        z *= s;

        [x, y, -z]
    }

    fn hsl_to_rgb(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
        let max = lightness + saturation / 2.0;
        let min = lightness - saturation / 2.0;
        let del = max - min;

        let hue = 360.0 * hue;
        if hue < 60.0 {
            [max, min + del * hue / 60.0, min]
        } else if hue < 120.0 {
            [min + del * (120.0 - hue) / 60.0, max, min]
        } else if hue < 180.0 {
            [min, max, min + del * (hue - 120.0) / 60.0]
        } else if hue < 240.0 {
            [min, min + del * (240.0 - hue) / 60.0, max]
        } else if hue < 300.0 {
            [min + del * (hue - 240.0) / 60.0, min, max]
        } else {
            [max, min, min + del * (360.0 - hue) / 60.0]
        }
    }

    fn hsl_vertex(hue: f32, saturation: f32, lightness: f32) -> Vertex {
        Vertex::new(
            hsl_to_position(hue, saturation, lightness),
            hsl_to_rgb(hue, saturation, lightness),
        )
    }

    let n_edge = n_hue * n_div + 2 * n_div;
    let n_vertices = 2 * n_edge;
    let mut vertices = Vec::with_capacity(n_vertices as _);

    vertices.push(hsl_vertex(0.0, 0.0, 0.0));
    vertices.push(hsl_vertex(0.0, 0.0, 1.0));

    for hue in 0..n_hue {
        for i in 1..=n_div {
            vertices.push(if i == 1 {
                hsl_vertex(0.0, 0.0, 0.0)
            } else {
                *vertices.last().unwrap()
            });

            let hue = hue as f32 / n_hue as f32;
            let lightness = i as f32 / n_div as f32;
            let saturation = 1.0 - 2.0 * (lightness - 0.5).abs();
            vertices.push(hsl_vertex(hue, saturation, lightness));
        }
    }

    for i in 1..=(2 * n_div) {
        vertices.push(if i == 1 {
            hsl_vertex(0.0, 1.0, 0.5)
        } else {
            *vertices.last().unwrap()
        });

        let hue = i as f32 / (2 * n_div) as f32;
        let saturation = 1.0;
        let lightness = 0.5;
        vertices.push(hsl_vertex(hue, saturation, lightness));
    }

    vertices
}
