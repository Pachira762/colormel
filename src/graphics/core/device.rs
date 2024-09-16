use anyhow::Result;
use windows::Win32::{
    Foundation::HANDLE,
    Graphics::{Direct3D::D3D_FEATURE_LEVEL_12_0, Direct3D12::*, Dxgi::*},
};

#[derive(Clone)]
pub struct Device {
    adapter: IDXGIAdapter1,
    device: ID3D12Device5,
}

impl Device {
    pub fn new(adapter: IDXGIAdapter1) -> Result<Self> {
        unsafe {
            let mut device = None;
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_12_0, &mut device)?;
            let device: ID3D12Device5 = device.unwrap();

            Ok(Self { adapter, device })
        }
    }

    pub fn open_shared_handle(&self, handle: HANDLE) -> Result<ID3D12Resource> {
        unsafe {
            let mut resource: Option<ID3D12Resource> = None;
            self.OpenSharedHandle(handle, &mut resource)?;
            Ok(resource.unwrap())
        }
    }

    pub fn create_srv(
        &self,
        resource: &ID3D12Resource,
        desc: Option<*const D3D12_SHADER_RESOURCE_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateShaderResourceView(resource, desc, descriptor);
        }
    }

    pub fn create_uav(
        &self,
        resource: &ID3D12Resource,
        desc: Option<*const D3D12_UNORDERED_ACCESS_VIEW_DESC>,
        descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) {
        unsafe {
            self.device
                .CreateUnorderedAccessView(resource, None, desc, descriptor);
        }
    }

    pub fn adapter(&self) -> &IDXGIAdapter1 {
        &self.adapter
    }
}

impl AsRef<ID3D12Device5> for Device {
    fn as_ref(&self) -> &ID3D12Device5 {
        &self.device
    }
}

impl std::ops::Deref for Device {
    type Target = ID3D12Device5;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
