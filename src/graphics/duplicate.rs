use anyhow::Result;
use windows::{
    core::Interface,
    Win32::Graphics::{
        Direct3D::*,
        Direct3D11::*,
        Dxgi::{Common::*, *},
    },
};

use super::{
    core::{descriptor::Descriptor, device::Device, resource::Resource, wrap::SrvDesc},
    initializer::Initializer,
};

pub struct Duplication {
    dupl: IDXGIOutputDuplication,
    #[allow(unused)]
    resource: Option<Resource>,
    srv: Descriptor,
    format: DXGI_FORMAT,
}

impl Duplication {
    pub fn new(ctx: &mut Initializer) -> Result<Self> {
        unsafe {
            let device: &Device = ctx;
            let adapter = device.adapter();

            let flags = if cfg!(debug_assertions) {
                D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG
            } else {
                D3D11_CREATE_DEVICE_BGRA_SUPPORT
            };

            let mut device_d3d11 = None;
            D3D11CreateDevice(
                adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                None,
                flags,
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&mut device_d3d11),
                None,
                None,
            )?;
            let device_d3d11 = device_d3d11.unwrap();

            let dupl = adapter
                .EnumOutputs(0)?
                .cast::<IDXGIOutput6>()?
                .DuplicateOutput1(
                    &device_d3d11,
                    0,
                    &[DXGI_FORMAT_R16G16B16A16_FLOAT, DXGI_FORMAT_B8G8R8A8_UNORM],
                )?;

            let format = match dupl.GetDesc().ModeDesc.Format {
                DXGI_FORMAT_B8G8R8A8_UNORM => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
                _ => DXGI_FORMAT_R16G16B16A16_FLOAT,
            };

            let srv = ctx.next_descriptor();

            Ok(Self {
                dupl,
                resource: None,
                srv,
                format,
            })
        }
    }

    pub fn duplicate(&mut self, device: &Device) -> Result<Option<Descriptor>> {
        unsafe {
            let _ = self.resource.take();

            match self.dupl.ReleaseFrame() {
                Err(e) if e.code() != DXGI_ERROR_INVALID_CALL => anyhow::bail!(e),
                _ => {}
            };

            let mut info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut resource = None;
            let hr = self.dupl.AcquireNextFrame(1000, &mut info, &mut resource);

            match hr {
                Ok(_) => {
                    if info.AccumulatedFrames == 0 {
                        Ok(None)
                    } else {
                        let resource: IDXGIResource1 = resource.unwrap().cast()?;
                        let resource = Resource::from_dxgi(&resource, device)?;

                        device.create_srv(
                            &resource,
                            Some(&SrvDesc::texture2d(self.format)),
                            self.srv.cpu,
                        );

                        self.resource = Some(resource);

                        Ok(Some(self.srv))
                    }
                }
                Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => Ok(None),
                Err(e) => anyhow::bail!(e),
            }
        }
    }
}
