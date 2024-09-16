use anyhow::Result;
use windows::Win32::{
    Foundation::CloseHandle,
    Graphics::{
        Direct3D12::*,
        Dxgi::{IDXGIResource1, DXGI_SHARED_RESOURCE_READ},
    },
};

use super::{device::Device, wrap::*};

#[derive(Clone)]
#[repr(transparent)]
pub struct Resource(ID3D12Resource);

impl Resource {
    pub fn new(
        device: &Device,
        heap_props: &D3D12_HEAP_PROPERTIES,
        heap_flags: Option<D3D12_HEAP_FLAGS>,
        desc: &D3D12_RESOURCE_DESC,
        initial_state: D3D12_RESOURCE_STATES,
        clear_value: Option<*const D3D12_CLEAR_VALUE>,
    ) -> Result<Self> {
        unsafe {
            let mut resource: Option<ID3D12Resource> = None;
            device.CreateCommittedResource(
                heap_props,
                heap_flags.unwrap_or_default(),
                desc,
                initial_state,
                clear_value,
                &mut resource,
            )?;
            Ok(Self(resource.unwrap()))
        }
    }

    pub fn new_buffer(
        device: &Device,
        heap_props: &D3D12_HEAP_PROPERTIES,
        heap_flags: Option<D3D12_HEAP_FLAGS>,
        size: u64,
        flags: D3D12_RESOURCE_FLAGS,
        initial_state: D3D12_RESOURCE_STATES,
    ) -> Result<Self> {
        Self::new(
            device,
            heap_props,
            heap_flags,
            &ResourceDesc::buffer(size, flags),
            initial_state,
            None,
        )
    }

    pub fn from_dxgi(resource: &IDXGIResource1, device: &Device) -> Result<Self> {
        unsafe {
            let handle = resource.CreateSharedHandle(None, DXGI_SHARED_RESOURCE_READ.0, None)?;

            let mut result: Option<ID3D12Resource> = None;
            match device.OpenSharedHandle(handle, &mut result) {
                Ok(_) => {
                    CloseHandle(handle)?;
                    Ok(Self(result.unwrap()))
                }
                Err(e) => {
                    CloseHandle(handle)?;
                    anyhow::bail!(e)
                }
            }
        }
    }

    pub fn read<T: Clone>(&self, len: usize) -> Result<Vec<T>> {
        unsafe {
            let mut ptr = std::ptr::null_mut();
            self.Map(0, None, Some(&mut ptr))?;

            let data: Vec<_> = std::slice::from_raw_parts(ptr as *mut T, len).to_vec();

            self.Unmap(0, None);
            Ok(data)
        }
    }

    pub fn write<T>(&self, src: &[T]) -> Result<()> {
        unsafe {
            let mut dest: *mut T = std::ptr::null_mut();
            self.Map(0, None, Some(&mut dest as *mut _ as _))?;

            dest.copy_from_nonoverlapping(src.as_ptr(), src.len());

            self.Unmap(0, None);
        }

        Ok(())
    }

    pub fn transition_barrier(
        &self,
        before: D3D12_RESOURCE_STATES,
        after: D3D12_RESOURCE_STATES,
    ) -> D3D12_RESOURCE_BARRIER {
        ResourceBarrier::transition(self, before, after)
    }

    pub fn desc(&self) -> D3D12_RESOURCE_DESC {
        unsafe { self.GetDesc() }
    }

    pub fn size(&self) -> (u32, u32) {
        let desc = self.desc();
        (desc.Width as u32, desc.Height)
    }
}

impl From<ID3D12Resource> for Resource {
    fn from(value: ID3D12Resource) -> Self {
        Self(value)
    }
}

impl AsRef<ID3D12Resource> for Resource {
    fn as_ref(&self) -> &ID3D12Resource {
        &self.0
    }
}

impl std::ops::Deref for Resource {
    type Target = ID3D12Resource;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
