use anyhow::Result;
use windows::Win32::{
    Foundation::*,
    Graphics::Direct3D12::*,
    System::Threading::{CreateEventA, WaitForSingleObject, INFINITE},
};

use super::device::Device;

pub struct Fence {
    fence: ID3D12Fence,
    fence_value: u64,
    fence_event: HANDLE,
}

impl Fence {
    pub fn new(device: &Device) -> Result<Self> {
        unsafe {
            let fence = device.CreateFence(0, D3D12_FENCE_FLAG_NONE)?;
            let fence_value = 1;
            let fence_event = CreateEventA(None, FALSE, FALSE, None)?;

            Ok(Self {
                fence,
                fence_value,
                fence_event,
            })
        }
    }

    pub fn wait(&mut self, command_queue: &ID3D12CommandQueue) -> Result<()> {
        unsafe {
            let fence = self.fence_value;
            command_queue.Signal(&self.fence, fence)?;
            self.fence_value += 1;

            if self.fence.GetCompletedValue() < fence {
                self.fence.SetEventOnCompletion(fence, self.fence_event)?;
                WaitForSingleObject(self.fence_event, INFINITE);
            }

            Ok(())
        }
    }
}

unsafe impl Send for Fence {}
