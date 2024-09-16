use anyhow::Result;
use windows::Win32::Graphics::{
    Direct3D12::{
        D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS, D3D12_RESOURCE_FLAG_NONE,
        D3D12_RESOURCE_STATE_COMMON, D3D12_VERTEX_BUFFER_VIEW,
    },
    Dxgi::Common::{
        DXGI_FORMAT, DXGI_FORMAT_R32_FLOAT, DXGI_FORMAT_R32_SINT, DXGI_FORMAT_R32_UINT,
        DXGI_FORMAT_R8_SINT, DXGI_FORMAT_R8_SNORM, DXGI_FORMAT_R8_UINT, DXGI_FORMAT_R8_UNORM,
    },
};

use super::{
    core::{
        descriptor::Descriptor,
        device::Device,
        resource::Resource,
        wrap::{HeapProps, SrvDesc, UavDesc},
    },
    initializer::Initializer,
};

pub struct Texture2D {
    pub resource: Resource,
    pub srv: Descriptor,
}

pub struct RwBuffer {
    pub resource: Resource,
    pub srv: Descriptor,
    pub uav: Descriptor,
    pub raw_uav: Descriptor,
}

impl RwBuffer {
    pub fn new(ctx: &mut Initializer, num_elems: u32, format: DXGI_FORMAT) -> Result<Self> {
        let elem_size = match format {
            DXGI_FORMAT_R8_SINT | DXGI_FORMAT_R8_UINT | DXGI_FORMAT_R8_SNORM
            | DXGI_FORMAT_R8_UNORM => 1,
            DXGI_FORMAT_R32_FLOAT | DXGI_FORMAT_R32_UINT | DXGI_FORMAT_R32_SINT => 4,

            _ => unreachable!("unsupported format {format:?}"),
        };

        let size = elem_size as usize * num_elems as usize;

        let resource = Resource::new_buffer(
            ctx,
            &HeapProps::default(),
            None,
            size as _,
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_STATE_COMMON,
        )?;

        let srv = ctx.next_descriptor();
        let desc = SrvDesc::buffer(num_elems, format);
        ctx.create_srv(&resource, Some(&desc), srv.cpu);

        let uav = ctx.next_descriptor();
        let desc = UavDesc::buffer(num_elems, format);
        ctx.create_uav(&resource, Some(&desc), uav.cpu);

        let raw_uav = ctx.next_descriptor();
        let desc = UavDesc::raw((size / 4) as _);
        ctx.create_uav(&resource, Some(&desc), raw_uav.cpu);

        Ok(Self {
            resource,
            srv,
            uav,
            raw_uav,
        })
    }
}

impl AsRef<Resource> for RwBuffer {
    fn as_ref(&self) -> &Resource {
        &self.resource
    }
}

impl std::ops::Deref for RwBuffer {
    type Target = Resource;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub struct VertexBuffer {
    #[allow(unused)]
    buffer: Resource,
    view: D3D12_VERTEX_BUFFER_VIEW,
    n_vertices: u32,
}

impl VertexBuffer {
    pub fn new<T>(device: &Device, vertices: &[T]) -> Result<Self> {
        let stride = std::mem::size_of::<T>();
        let n_vertices = vertices.len();
        let size = std::mem::size_of_val(vertices);

        let buffer = Resource::new_buffer(
            device,
            &HeapProps::upload(),
            None,
            size as _,
            D3D12_RESOURCE_FLAG_NONE,
            D3D12_RESOURCE_STATE_COMMON,
        )?;

        buffer.write(vertices)?;

        let view = D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: unsafe { buffer.GetGPUVirtualAddress() },
            SizeInBytes: size as _,
            StrideInBytes: stride as _,
        };

        Ok(Self {
            buffer,
            view,
            n_vertices: n_vertices as _,
        })
    }

    pub fn view(&self) -> D3D12_VERTEX_BUFFER_VIEW {
        self.view
    }

    pub fn vertex_count(&self) -> u32 {
        self.n_vertices
    }
}
