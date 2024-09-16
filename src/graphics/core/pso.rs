use anyhow::Result;

use windows::Win32::Graphics::Direct3D12::{ID3D12PipelineState, D3D12_PIPELINE_STATE_STREAM_DESC};

use super::device::Device;

pub struct PipelineState(ID3D12PipelineState);

impl PipelineState {
    pub fn new(device: &Device, desc: &D3D12_PIPELINE_STATE_STREAM_DESC) -> Result<Self> {
        unsafe { Ok(Self(device.CreatePipelineState(desc)?)) }
    }
}

impl AsRef<ID3D12PipelineState> for PipelineState {
    fn as_ref(&self) -> &ID3D12PipelineState {
        &self.0
    }
}

impl std::ops::Deref for PipelineState {
    type Target = ID3D12PipelineState;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
