use anyhow::Result;
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::DXGI_FORMAT};

use super::core::{
    descriptor::{Descriptor, DescriptorIter},
    device::Device,
    pso::PipelineState,
    root_signature::RootSignature,
    wrap::*,
};

pub struct Initializer {
    device: Device,
    root_signature: RootSignature,
    descriptor_pool: DescriptorIter,
}

impl Initializer {
    pub fn new(
        device: Device,
        root_signature: RootSignature,
        descriptor_pool: DescriptorIter,
    ) -> Result<Self> {
        Ok(Self {
            device,
            root_signature,
            descriptor_pool,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_graphics_pipeline(
        &self,
        vs: &[u8],
        ps: &[u8],
        blend: D3D12_BLEND_DESC,
        rasterizer: D3D12_RASTERIZER_DESC,
        depth_stencil: D3D12_DEPTH_STENCIL_DESC,
        input_elements: &[D3D12_INPUT_ELEMENT_DESC],
        primitive_topology: D3D12_PRIMITIVE_TOPOLOGY_TYPE,
        rtv_formats: D3D12_RT_FORMAT_ARRAY,
        dsv_format: Option<DXGI_FORMAT>,
        flags: Option<D3D12_PIPELINE_STATE_FLAGS>,
    ) -> Result<PipelineState> {
        let desc = GraphicsPipelineStateDesc {
            root_signature: self.root_signature.as_param().into(),
            vs: ShaderBytecode::from_bytes(vs).into(),
            ps: ShaderBytecode::from_bytes(ps).into(),
            blend: blend.into(),
            sample_mask: D3D12_DEFAULT_SAMPLE_MASK.into(),
            rasterizer: rasterizer.into(),
            depth_stencil: depth_stencil.into(),
            input_layout: InputLayoutDesc::from_slice(input_elements).into(),
            primitive_topology: primitive_topology.into(),
            rtv_formats: rtv_formats.into(),
            dsv_format: dsv_format.unwrap_or_default().into(),
            sample_desc: SampleDesc::default().into(),
            flags: flags.unwrap_or_default().into(),
        };

        PipelineState::new(&self.device, &(&desc).into())
    }

    pub fn create_compute_pipeline(
        &self,
        cs: &[u8],
        flags: Option<D3D12_PIPELINE_STATE_FLAGS>,
    ) -> Result<PipelineState> {
        let desc = ComputePipelineStateDesc {
            root_signature: self.root_signature.as_param().into(),
            cs: ShaderBytecode::from_bytes(cs).into(),
            flags: flags.unwrap_or_default().into(),
        };

        PipelineState::new(&self.device, &(&desc).into())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_mesh_pipeline(
        &self,
        amps: &[u8],
        ms: &[u8],
        ps: &[u8],
        blend: D3D12_BLEND_DESC,
        rasterizer: D3D12_RASTERIZER_DESC,
        depth_stencil: D3D12_DEPTH_STENCIL_DESC,
        primitive_topology: D3D12_PRIMITIVE_TOPOLOGY_TYPE,
        rtv_formats: D3D12_RT_FORMAT_ARRAY,
        dsv_format: Option<DXGI_FORMAT>,
        flags: Option<D3D12_PIPELINE_STATE_FLAGS>,
    ) -> Result<PipelineState> {
        let desc = MeshPipelineStateDesc {
            root_signature: self.root_signature.as_param().into(),
            amps: ShaderBytecode::from_bytes(amps).into(),
            ms: ShaderBytecode::from_bytes(ms).into(),
            ps: ShaderBytecode::from_bytes(ps).into(),
            blend: blend.into(),
            sample_mask: D3D12_DEFAULT_SAMPLE_MASK.into(),
            rasterizer: rasterizer.into(),
            depth_stencil: depth_stencil.into(),
            primitive_topology: primitive_topology.into(),
            rtv_formats: rtv_formats.into(),
            dsv_format: dsv_format.unwrap_or_default().into(),
            sample_desc: SampleDesc::default().into(),
            flags: flags.unwrap_or_default().into(),
        };

        PipelineState::new(&self.device, &(&desc).into())
    }

    pub fn next_descriptor(&mut self) -> Descriptor {
        self.descriptor_pool.next().expect("descriptor size limit")
    }
}

impl AsRef<Device> for Initializer {
    fn as_ref(&self) -> &Device {
        &self.device
    }
}

impl std::ops::Deref for Initializer {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
