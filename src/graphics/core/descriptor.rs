use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::*;

use super::{device::Device, wrap::*};

#[derive(Clone, Copy)]
pub struct Descriptor {
    pub cpu: D3D12_CPU_DESCRIPTOR_HANDLE,
    pub gpu: D3D12_GPU_DESCRIPTOR_HANDLE,
}

impl Descriptor {
    pub fn offset(&mut self, count: u32, increment_size: u32) -> Self {
        self.cpu.offset(count, increment_size);
        self.gpu.offset(count, increment_size);

        *self
    }
}

pub const DESCRIPTOR_HEAP_TYPE_SRV: i32 = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV.0;
pub const DESCRIPTOR_HEAP_TYPE_SAMPLER: i32 = D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER.0;
pub const DESCRIPTOR_HEAP_TYPE_RTV: i32 = D3D12_DESCRIPTOR_HEAP_TYPE_RTV.0;
pub const DESCRIPTOR_HEAP_TYPE_DSV: i32 = D3D12_DESCRIPTOR_HEAP_TYPE_DSV.0;

pub struct DescriptorHeap<const TYPE: i32, const SHADER_VISIBLE: bool> {
    heap: ID3D12DescriptorHeap,
    num_descriptors: u32,
    increment_size: u32,
}

impl<const TYPE: i32, const SHADER_VISIBLE: bool> DescriptorHeap<TYPE, SHADER_VISIBLE> {
    pub fn new(device: &Device, num_descriptors: u32) -> Result<Self> {
        unsafe {
            let desc = match TYPE {
                DESCRIPTOR_HEAP_TYPE_SRV => {
                    DescriptorHeapDesc::srv(num_descriptors, SHADER_VISIBLE)
                }
                DESCRIPTOR_HEAP_TYPE_SAMPLER => DescriptorHeapDesc::sampler(num_descriptors),
                DESCRIPTOR_HEAP_TYPE_RTV => DescriptorHeapDesc::rtv(num_descriptors),
                DESCRIPTOR_HEAP_TYPE_DSV => DescriptorHeapDesc::dsv(num_descriptors),
                _ => unreachable!("invalid descriptor heap type {TYPE}"),
            };
            let heap = device.CreateDescriptorHeap(&desc)?;

            let increment_size =
                device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE(TYPE));

            Ok(Self {
                heap,
                num_descriptors,
                increment_size,
            })
        }
    }

    pub fn descriptor(&self, index: u32) -> Descriptor {
        debug_assert!(index < self.num_descriptors);

        unsafe {
            let cpu = self
                .heap
                .GetCPUDescriptorHandleForHeapStart()
                .offset(index, self.increment_size);

            let gpu = if SHADER_VISIBLE {
                self.heap
                    .GetGPUDescriptorHandleForHeapStart()
                    .offset(index, self.increment_size)
            } else {
                D3D12_GPU_DESCRIPTOR_HANDLE::default()
            };

            Descriptor { cpu, gpu }
        }
    }

    pub fn iter(&self) -> DescriptorIter {
        let cur = self.descriptor(0);
        let end = self
            .descriptor(0)
            .offset(self.num_descriptors, self.increment_size);

        DescriptorIter {
            cur,
            end,
            increment_size: self.increment_size,
        }
    }
}

impl<const TYPE: i32, const SHADER_VISIBLE: bool> AsRef<ID3D12DescriptorHeap>
    for DescriptorHeap<TYPE, SHADER_VISIBLE>
{
    fn as_ref(&self) -> &ID3D12DescriptorHeap {
        &self.heap
    }
}

impl<const TYPE: i32, const SHADER_VISIBLE: bool> std::ops::Deref
    for DescriptorHeap<TYPE, SHADER_VISIBLE>
{
    type Target = ID3D12DescriptorHeap;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub type ShaderVisibleSrvHeap = DescriptorHeap<DESCRIPTOR_HEAP_TYPE_SRV, true>;
pub type NonShaderVisibleSrvHeap = DescriptorHeap<DESCRIPTOR_HEAP_TYPE_SRV, false>;
pub type SamplerViewHeap = DescriptorHeap<DESCRIPTOR_HEAP_TYPE_SAMPLER, false>;
pub type RtvHeap = DescriptorHeap<DESCRIPTOR_HEAP_TYPE_RTV, false>;
pub type DsvHeap = DescriptorHeap<DESCRIPTOR_HEAP_TYPE_DSV, false>;

pub struct DescriptorIter {
    cur: Descriptor,
    end: Descriptor,
    increment_size: u32,
}

impl Iterator for DescriptorIter {
    type Item = Descriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur.cpu.ptr < self.end.cpu.ptr {
            let descriptor = self.cur;

            self.cur.offset(1, self.increment_size);

            Some(descriptor)
        } else {
            None
        }
    }
}
