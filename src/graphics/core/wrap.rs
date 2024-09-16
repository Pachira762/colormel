#![allow(unused)]

use std::mem::ManuallyDrop;

use windows::{
    core::PCSTR,
    Win32::{
        Foundation::*,
        Graphics::{
            Direct3D::ID3DBlob,
            Direct3D12::*,
            Dxgi::{Common::*, *},
        },
    },
};

pub trait DescParam: Sized {
    fn as_param(&self) -> std::mem::ManuallyDrop<Option<Self>> {
        unsafe { std::mem::transmute_copy(self) }
    }
}
impl DescParam for ID3D12Resource {}
impl DescParam for ID3D12RootSignature {}

pub trait Blob {
    fn as_bytes(&self) -> &[u8];
    fn as_str(&self) -> &str;
}

impl Blob for ID3DBlob {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.GetBufferPointer() as *const _, self.GetBufferSize())
        }
    }

    fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

pub struct SampleDesc {}
impl SampleDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> DXGI_SAMPLE_DESC {
        DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        }
    }
}

pub enum SwapChainDesc {}
impl SwapChainDesc {
    pub fn composited(
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        count: u32,
    ) -> DXGI_SWAP_CHAIN_DESC1 {
        DXGI_SWAP_CHAIN_DESC1 {
            Width: width,
            Height: height,
            Format: format,
            Stereo: FALSE,
            SampleDesc: SampleDesc::default(),
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: count,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: 0,
        }
    }
}

pub enum CommandQueueDesc {}
impl CommandQueueDesc {
    pub fn direct() -> D3D12_COMMAND_QUEUE_DESC {
        D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        }
    }
}

pub trait DescriptorHandle: Sized {
    fn offset(&mut self, count: u32, increment_size: u32) -> Self;
}

impl DescriptorHandle for D3D12_CPU_DESCRIPTOR_HANDLE {
    fn offset(&mut self, count: u32, increment_size: u32) -> Self {
        self.ptr += count as usize * increment_size as usize;
        *self
    }
}

impl DescriptorHandle for D3D12_GPU_DESCRIPTOR_HANDLE {
    fn offset(&mut self, count: u32, increment_size: u32) -> Self {
        if self.ptr != 0 {
            self.ptr += count as u64 * increment_size as u64;
        }

        *self
    }
}

pub enum DescriptorHeapDesc {}
impl DescriptorHeapDesc {
    pub fn default(
        heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
        num: u32,
        shader_visible: bool,
    ) -> D3D12_DESCRIPTOR_HEAP_DESC {
        D3D12_DESCRIPTOR_HEAP_DESC {
            Type: heap_type,
            NumDescriptors: num,
            Flags: if shader_visible {
                D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE
            } else {
                D3D12_DESCRIPTOR_HEAP_FLAG_NONE
            },
            ..Default::default()
        }
    }

    pub fn srv(num: u32, shader_visible: bool) -> D3D12_DESCRIPTOR_HEAP_DESC {
        D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: num,
            Flags: if shader_visible {
                D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE
            } else {
                D3D12_DESCRIPTOR_HEAP_FLAG_NONE
            },
            ..Default::default()
        }
    }

    pub fn sampler(num: u32) -> D3D12_DESCRIPTOR_HEAP_DESC {
        D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            NumDescriptors: num,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            ..Default::default()
        }
    }

    pub fn rtv(num: u32) -> D3D12_DESCRIPTOR_HEAP_DESC {
        D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: num,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        }
    }

    pub fn dsv(num: u32) -> D3D12_DESCRIPTOR_HEAP_DESC {
        D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_DSV,
            NumDescriptors: num,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            ..Default::default()
        }
    }
}

pub enum HeapProps {}
impl HeapProps {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            ..Default::default()
        }
    }

    pub fn upload() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_UPLOAD,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            ..Default::default()
        }
    }

    pub fn readback() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_READBACK,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            ..Default::default()
        }
    }

    pub fn custom() -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_CUSTOM,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            ..Default::default()
        }
    }
}

pub enum ResourceDesc {}
impl ResourceDesc {
    pub fn buffer(size: u64, flags: D3D12_RESOURCE_FLAGS) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: size,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: flags,
        }
    }

    pub fn texture2d(
        width: u32,
        height: u32,
        format: DXGI_FORMAT,
        flags: D3D12_RESOURCE_FLAGS,
    ) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: width as _,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: format,
            SampleDesc: SampleDesc::default(),
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: flags,
        }
    }
}

pub enum ResourceBarrier {}
impl ResourceBarrier {
    pub fn transition(
        resource: &ID3D12Resource,
        before: D3D12_RESOURCE_STATES,
        after: D3D12_RESOURCE_STATES,
    ) -> D3D12_RESOURCE_BARRIER {
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: resource.as_param(),
                    StateBefore: before,
                    StateAfter: after,
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                }),
            },
        }
    }
}

pub enum ClearValue {}
impl ClearValue {
    pub fn color(format: DXGI_FORMAT, color: [f32; 4]) -> D3D12_CLEAR_VALUE {
        D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 { Color: color },
        }
    }

    pub fn depth(format: DXGI_FORMAT, depth: f32) -> D3D12_CLEAR_VALUE {
        D3D12_CLEAR_VALUE {
            Format: format,
            Anonymous: D3D12_CLEAR_VALUE_0 {
                DepthStencil: D3D12_DEPTH_STENCIL_VALUE {
                    Depth: depth,
                    Stencil: 0,
                },
            },
        }
    }
}

pub enum SrvDesc {}
impl SrvDesc {
    pub fn buffer(num: u32, format: DXGI_FORMAT) -> D3D12_SHADER_RESOURCE_VIEW_DESC {
        D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_NONE,
                },
            },
        }
    }

    pub fn raw(num: u32) -> D3D12_SHADER_RESOURCE_VIEW_DESC {
        D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_RAW,
                },
            },
        }
    }

    pub fn structured(num: u32, stride: u32) -> D3D12_SHADER_RESOURCE_VIEW_DESC {
        D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_SRV_DIMENSION_BUFFER,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_SRV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: 0,
                    Flags: D3D12_BUFFER_SRV_FLAG_NONE,
                },
            },
        }
    }

    pub fn texture2d(format: DXGI_FORMAT) -> D3D12_SHADER_RESOURCE_VIEW_DESC {
        D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_SRV {
                    MostDetailedMip: 0,
                    MipLevels: 1,
                    PlaneSlice: 0,
                    ResourceMinLODClamp: 0.0,
                },
            },
        }
    }
}

pub enum UavDesc {}
impl UavDesc {
    pub fn buffer(num: u32, format: DXGI_FORMAT) -> D3D12_UNORDERED_ACCESS_VIEW_DESC {
        D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE,
                },
            },
        }
    }

    pub fn raw(num: u32) -> D3D12_UNORDERED_ACCESS_VIEW_DESC {
        D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_R32_TYPELESS,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: 0,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_RAW,
                },
            },
        }
    }

    pub fn structured(num: u32, stride: u32) -> D3D12_UNORDERED_ACCESS_VIEW_DESC {
        D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: DXGI_FORMAT_UNKNOWN,
            ViewDimension: D3D12_UAV_DIMENSION_BUFFER,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Buffer: D3D12_BUFFER_UAV {
                    FirstElement: 0,
                    NumElements: num,
                    StructureByteStride: stride,
                    CounterOffsetInBytes: 0,
                    Flags: D3D12_BUFFER_UAV_FLAG_NONE,
                },
            },
        }
    }
}

pub enum RtvDesc {}
impl RtvDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default(format: DXGI_FORMAT) -> D3D12_RENDER_TARGET_VIEW_DESC {
        D3D12_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
            Anonymous: D3D12_RENDER_TARGET_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_RTV {
                    MipSlice: 0,
                    PlaneSlice: 0,
                },
            },
        }
    }
}

pub enum DsvDesc {}
impl DsvDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default(format: DXGI_FORMAT) -> D3D12_DEPTH_STENCIL_VIEW_DESC {
        D3D12_DEPTH_STENCIL_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
            Flags: D3D12_DSV_FLAG_NONE,
            Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_DSV { MipSlice: 0 },
            },
        }
    }
}

pub enum DescriptorRange {}
impl DescriptorRange {
    pub fn srv(
        num: u32,
        register: u32,
        space: u32,
        flags: D3D12_DESCRIPTOR_RANGE_FLAGS,
    ) -> D3D12_DESCRIPTOR_RANGE1 {
        D3D12_DESCRIPTOR_RANGE1 {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
            NumDescriptors: num,
            BaseShaderRegister: register,
            RegisterSpace: space,
            Flags: flags,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
        }
    }

    pub fn uav(
        num: u32,
        register: u32,
        space: u32,
        flags: D3D12_DESCRIPTOR_RANGE_FLAGS,
    ) -> D3D12_DESCRIPTOR_RANGE1 {
        D3D12_DESCRIPTOR_RANGE1 {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
            NumDescriptors: num,
            BaseShaderRegister: register,
            RegisterSpace: space,
            Flags: flags,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
        }
    }

    pub fn cbv(
        num: u32,
        register: u32,
        space: u32,
        flags: D3D12_DESCRIPTOR_RANGE_FLAGS,
    ) -> D3D12_DESCRIPTOR_RANGE1 {
        D3D12_DESCRIPTOR_RANGE1 {
            RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
            NumDescriptors: num,
            BaseShaderRegister: register,
            RegisterSpace: space,
            Flags: flags,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
        }
    }
}

pub enum RootParameter {}
impl RootParameter {
    pub fn table(
        ranges: &[D3D12_DESCRIPTOR_RANGE1],
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_ROOT_PARAMETER1 {
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE1 {
                    NumDescriptorRanges: ranges.len() as _,
                    pDescriptorRanges: ranges.as_ptr(),
                },
            },
            ShaderVisibility: visibility,
        }
    }

    pub fn constants(
        register: u32,
        space: u32,
        num: u32,
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_ROOT_PARAMETER1 {
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                Constants: D3D12_ROOT_CONSTANTS {
                    ShaderRegister: register,
                    RegisterSpace: space,
                    Num32BitValues: num,
                },
            },
            ShaderVisibility: visibility,
        }
    }

    pub fn cbv(
        register: u32,
        space: u32,
        flags: D3D12_ROOT_DESCRIPTOR_FLAGS,
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_ROOT_PARAMETER1 {
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR1 {
                    ShaderRegister: register,
                    RegisterSpace: space,
                    Flags: flags,
                },
            },
            ShaderVisibility: visibility,
        }
    }

    pub fn srv(
        register: u32,
        space: u32,
        flags: D3D12_ROOT_DESCRIPTOR_FLAGS,
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_ROOT_PARAMETER1 {
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_SRV,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR1 {
                    ShaderRegister: register,
                    RegisterSpace: space,
                    Flags: flags,
                },
            },
            ShaderVisibility: visibility,
        }
    }

    pub fn uav(
        register: u32,
        space: u32,
        flags: D3D12_ROOT_DESCRIPTOR_FLAGS,
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_ROOT_PARAMETER1 {
        D3D12_ROOT_PARAMETER1 {
            ParameterType: D3D12_ROOT_PARAMETER_TYPE_UAV,
            Anonymous: D3D12_ROOT_PARAMETER1_0 {
                Descriptor: D3D12_ROOT_DESCRIPTOR1 {
                    ShaderRegister: register,
                    RegisterSpace: space,
                    Flags: flags,
                },
            },
            ShaderVisibility: visibility,
        }
    }
}

pub enum StaticSamplerDesc {}
impl StaticSamplerDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default(
        filter: D3D12_FILTER,
        address: D3D12_TEXTURE_ADDRESS_MODE,
        register: u32,
        space: u32,
        visibility: D3D12_SHADER_VISIBILITY,
    ) -> D3D12_STATIC_SAMPLER_DESC {
        D3D12_STATIC_SAMPLER_DESC {
            Filter: filter,
            AddressU: address,
            AddressV: address,
            AddressW: address,
            MipLODBias: 0.0,
            MaxAnisotropy: 0,
            ComparisonFunc: D3D12_COMPARISON_FUNC_NONE,
            BorderColor: D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
            MinLOD: 0.0,
            MaxLOD: D3D12_FLOAT32_MAX,
            ShaderRegister: register,
            RegisterSpace: space,
            ShaderVisibility: visibility,
        }
    }
}

pub enum RootSignatureDesc {}
impl RootSignatureDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default(
        params: &[D3D12_ROOT_PARAMETER1],
        samplers: &[D3D12_STATIC_SAMPLER_DESC],
        flags: D3D12_ROOT_SIGNATURE_FLAGS,
    ) -> D3D12_VERSIONED_ROOT_SIGNATURE_DESC {
        D3D12_VERSIONED_ROOT_SIGNATURE_DESC {
            Version: D3D_ROOT_SIGNATURE_VERSION_1_1,
            Anonymous: D3D12_VERSIONED_ROOT_SIGNATURE_DESC_0 {
                Desc_1_1: D3D12_ROOT_SIGNATURE_DESC1 {
                    NumParameters: params.len() as _,
                    pParameters: params.as_ptr(),
                    NumStaticSamplers: samplers.len() as _,
                    pStaticSamplers: samplers.as_ptr(),
                    Flags: flags,
                },
            },
        }
    }
}

pub enum BlendDesc {}
impl BlendDesc {
    pub fn none() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [
                D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: FALSE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_ONE,
                    DestBlend: D3D12_BLEND_ZERO,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_ZERO,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }

    pub fn alpha() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [
                D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: TRUE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_SRC_ALPHA,
                    DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_ZERO,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }

    pub fn mul() -> D3D12_BLEND_DESC {
        D3D12_BLEND_DESC {
            AlphaToCoverageEnable: FALSE,
            IndependentBlendEnable: FALSE,
            RenderTarget: [
                D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: TRUE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_SRC_ALPHA,
                    DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_INV_DEST_ALPHA,
                    DestBlendAlpha: D3D12_BLEND_ONE,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_NOOP,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as _,
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
        }
    }
}

pub enum RasterizerDesc {}
impl RasterizerDesc {
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            DepthClipEnable: FALSE,
            ..Default::default()
        }
    }

    pub fn none() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_NONE,
            DepthClipEnable: FALSE,
            ..Default::default()
        }
    }
}

pub enum DepthStencilDesc {}
impl DepthStencilDesc {
    pub fn none() -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: FALSE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_NONE,
            StencilEnable: FALSE,
            ..Default::default()
        }
    }

    pub fn depth() -> D3D12_DEPTH_STENCIL_DESC {
        D3D12_DEPTH_STENCIL_DESC {
            DepthEnable: TRUE,
            DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
            DepthFunc: D3D12_COMPARISON_FUNC_LESS,
            StencilEnable: FALSE,
            ..Default::default()
        }
    }
}

pub enum InputElementDesc {}
impl InputElementDesc {
    pub fn per_vertex(name: PCSTR, format: DXGI_FORMAT) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: name,
            SemanticIndex: 0,
            Format: format,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        }
    }

    pub fn per_instance(name: PCSTR, format: DXGI_FORMAT, step: u32) -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: name,
            SemanticIndex: 0,
            Format: format,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_INSTANCE_DATA,
            InstanceDataStepRate: step,
        }
    }
}

pub enum InputLayoutDesc {}
impl InputLayoutDesc {
    pub fn from_slice(elems: &[D3D12_INPUT_ELEMENT_DESC]) -> D3D12_INPUT_LAYOUT_DESC {
        D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: elems.as_ptr(),
            NumElements: elems.len() as _,
        }
    }
}

pub enum RtvFormats {}
impl RtvFormats {
    pub fn single(format: DXGI_FORMAT) -> D3D12_RT_FORMAT_ARRAY {
        D3D12_RT_FORMAT_ARRAY {
            RTFormats: [
                format,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
                DXGI_FORMAT_UNKNOWN,
            ],
            NumRenderTargets: 1,
        }
    }
}

pub enum ShaderBytecode {}
impl ShaderBytecode {
    pub fn from_bytes(bytes: &[u8]) -> D3D12_SHADER_BYTECODE {
        D3D12_SHADER_BYTECODE {
            pShaderBytecode: bytes.as_ptr() as _,
            BytecodeLength: bytes.len(),
        }
    }
}

#[repr(C, align(8))]
pub struct PipelineStateStreamSubobject<T, const U: i32> {
    pub pss_type: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE,
    pub pss_value: T,
}

impl<T, const U: i32> From<T> for PipelineStateStreamSubobject<T, U> {
    fn from(value: T) -> Self {
        Self {
            pss_type: D3D12_PIPELINE_STATE_SUBOBJECT_TYPE(U),
            pss_value: value,
        }
    }
}

pub type PipelineStateStreamRootSignature = PipelineStateStreamSubobject<
    ManuallyDrop<Option<ID3D12RootSignature>>,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_ROOT_SIGNATURE.0 },
>;
pub type PipelineStateStreamVs = PipelineStateStreamSubobject<
    D3D12_SHADER_BYTECODE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_VS.0 },
>;
pub type PipelineStateStreamPs = PipelineStateStreamSubobject<
    D3D12_SHADER_BYTECODE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PS.0 },
>;
pub type PipelineStateStreamCs = PipelineStateStreamSubobject<
    D3D12_SHADER_BYTECODE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_CS.0 },
>;
pub type PipelineStateStreamAs = PipelineStateStreamSubobject<
    D3D12_SHADER_BYTECODE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_AS.0 },
>;
pub type PipelineStateStreamMs = PipelineStateStreamSubobject<
    D3D12_SHADER_BYTECODE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_MS.0 },
>;
pub type PipelineStateStreamBlend =
    PipelineStateStreamSubobject<D3D12_BLEND_DESC, { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_BLEND.0 }>;
pub type PipelineStateStreamSampleMask =
    PipelineStateStreamSubobject<u32, { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_MASK.0 }>;
pub type PipelineStateStreamRasterizer = PipelineStateStreamSubobject<
    D3D12_RASTERIZER_DESC,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RASTERIZER.0 },
>;
pub type PipelineStateStreamDepthStencil = PipelineStateStreamSubobject<
    D3D12_DEPTH_STENCIL_DESC,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL.0 },
>;
pub type PipelineStateStreamInputLayout = PipelineStateStreamSubobject<
    D3D12_INPUT_LAYOUT_DESC,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_INPUT_LAYOUT.0 },
>;
pub type PipelineStateStreamPrimitiveTopology = PipelineStateStreamSubobject<
    D3D12_PRIMITIVE_TOPOLOGY_TYPE,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_PRIMITIVE_TOPOLOGY.0 },
>;
pub type PipelineStateStreamRenderTargetFormats = PipelineStateStreamSubobject<
    D3D12_RT_FORMAT_ARRAY,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_RENDER_TARGET_FORMATS.0 },
>;
pub type PipelineStateStreamDepthStencilFormat = PipelineStateStreamSubobject<
    DXGI_FORMAT,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_DEPTH_STENCIL_FORMAT.0 },
>;
pub type PipelineStateStreamSampleDesc = PipelineStateStreamSubobject<
    DXGI_SAMPLE_DESC,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_SAMPLE_DESC.0 },
>;
pub type PipelineStateStreamFlags = PipelineStateStreamSubobject<
    D3D12_PIPELINE_STATE_FLAGS,
    { D3D12_PIPELINE_STATE_SUBOBJECT_TYPE_FLAGS.0 },
>;

macro_rules! impl_into_pipeline_state_stream_desc {
    ($T:ty) => {
        impl From<&$T> for D3D12_PIPELINE_STATE_STREAM_DESC {
            fn from(value: &$T) -> Self {
                D3D12_PIPELINE_STATE_STREAM_DESC {
                    SizeInBytes: std::mem::size_of_val(value),
                    pPipelineStateSubobjectStream: value as *const _ as _,
                }
            }
        }
    };
}

#[repr(C)]
pub struct GraphicsPipelineStateDesc {
    pub root_signature: PipelineStateStreamRootSignature,
    pub vs: PipelineStateStreamVs,
    pub ps: PipelineStateStreamPs,
    pub blend: PipelineStateStreamBlend,
    pub sample_mask: PipelineStateStreamSampleMask,
    pub rasterizer: PipelineStateStreamRasterizer,
    pub depth_stencil: PipelineStateStreamDepthStencil,
    pub input_layout: PipelineStateStreamInputLayout,
    pub primitive_topology: PipelineStateStreamPrimitiveTopology,
    pub rtv_formats: PipelineStateStreamRenderTargetFormats,
    pub dsv_format: PipelineStateStreamDepthStencilFormat,
    pub sample_desc: PipelineStateStreamSampleDesc,
    pub flags: PipelineStateStreamFlags,
}
impl_into_pipeline_state_stream_desc!(GraphicsPipelineStateDesc);

#[repr(C)]
pub struct ComputePipelineStateDesc {
    pub root_signature: PipelineStateStreamRootSignature,
    pub cs: PipelineStateStreamCs,
    pub flags: PipelineStateStreamFlags,
}
impl_into_pipeline_state_stream_desc!(ComputePipelineStateDesc);

#[repr(C)]
pub struct MeshPipelineStateDesc {
    pub root_signature: PipelineStateStreamRootSignature,
    pub amps: PipelineStateStreamAs,
    pub ms: PipelineStateStreamMs,
    pub ps: PipelineStateStreamPs,
    pub blend: PipelineStateStreamBlend,
    pub sample_mask: PipelineStateStreamSampleMask,
    pub rasterizer: PipelineStateStreamRasterizer,
    pub depth_stencil: PipelineStateStreamDepthStencil,
    pub primitive_topology: PipelineStateStreamPrimitiveTopology,
    pub rtv_formats: PipelineStateStreamRenderTargetFormats,
    pub dsv_format: PipelineStateStreamDepthStencilFormat,
    pub sample_desc: PipelineStateStreamSampleDesc,
    pub flags: PipelineStateStreamFlags,
}
impl_into_pipeline_state_stream_desc!(MeshPipelineStateDesc);

pub enum QueryHeapDesc {}

impl QueryHeapDesc {
    pub fn default(heap_type: D3D12_QUERY_HEAP_TYPE, count: u32) -> D3D12_QUERY_HEAP_DESC {
        D3D12_QUERY_HEAP_DESC {
            Type: heap_type,
            Count: count,
            NodeMask: 0,
        }
    }
}
