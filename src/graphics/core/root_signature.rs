use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::*;

use super::{
    device::Device,
    wrap::{Blob, DescriptorRange, RootParameter, RootSignatureDesc},
};

pub const ROOT_PARAM_INDEX_CONSTANTS: u32 = 0;
pub const ROOT_PARAM_INDEX_SRV: u32 = 1;
pub const ROOT_PARAM_INDEX_UAV: u32 = 2;
pub const ROOT_PARAM_INDEX_DIRECT_SRV: u32 = 3;

#[derive(Clone)]
pub struct RootSignature(ID3D12RootSignature);

impl RootSignature {
    pub fn new(device: &Device) -> Result<Self> {
        unsafe {
            let ranges_srv = [DescriptorRange::srv(
                4,
                0,
                0,
                D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_VOLATILE
                    | D3D12_DESCRIPTOR_RANGE_FLAG_DATA_VOLATILE,
            )];
            let ranges_uav = [DescriptorRange::uav(
                4,
                0,
                0,
                D3D12_DESCRIPTOR_RANGE_FLAG_DESCRIPTORS_VOLATILE
                    | D3D12_DESCRIPTOR_RANGE_FLAG_DATA_VOLATILE,
            )];
            let ranges_direct = [DescriptorRange::srv(
                1,
                0,
                1,
                D3D12_DESCRIPTOR_RANGE_FLAG_DATA_STATIC,
            )];
            let params = [
                RootParameter::constants(0, 0, 32, D3D12_SHADER_VISIBILITY_ALL),
                RootParameter::table(&ranges_srv, D3D12_SHADER_VISIBILITY_ALL),
                RootParameter::table(&ranges_uav, D3D12_SHADER_VISIBILITY_ALL),
                RootParameter::table(&ranges_direct, D3D12_SHADER_VISIBILITY_ALL),
            ];

            let mut blob = None;
            let mut error = None;
            let desc = RootSignatureDesc::default(
                &params,
                &[],
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            );
            match D3D12SerializeVersionedRootSignature(&desc, &mut blob, Some(&mut error)) {
                Ok(_) => Ok(Self(
                    device.CreateRootSignature(0, blob.unwrap().as_bytes())?,
                )),
                Err(e) => {
                    println!("{:?}", error.unwrap().as_str());
                    anyhow::bail!(e)
                }
            }
        }
    }
}

impl AsRef<ID3D12RootSignature> for RootSignature {
    fn as_ref(&self) -> &ID3D12RootSignature {
        &self.0
    }
}

impl std::ops::Deref for RootSignature {
    type Target = ID3D12RootSignature;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
