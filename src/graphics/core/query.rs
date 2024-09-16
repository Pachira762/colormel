use std::collections::HashMap;

use anyhow::Result;
use windows::Win32::Graphics::Direct3D12::{
    ID3D12QueryHeap, D3D12_QUERY_HEAP_TYPE, D3D12_QUERY_HEAP_TYPE_TIMESTAMP,
    D3D12_RESOURCE_FLAG_NONE, D3D12_RESOURCE_STATE_COPY_DEST,
};

use super::{
    device::Device,
    resource::Resource,
    wrap::{HeapProps, QueryHeapDesc, ResourceDesc},
};

const QUERY_HEAP_TYPE_TIMESTAMP: i32 = D3D12_QUERY_HEAP_TYPE_TIMESTAMP.0;

#[derive(Clone)]
pub struct QueryHeap<const TYPE: i32> {
    heap: ID3D12QueryHeap,
    size: u32,
}

impl<const TYPE: i32> QueryHeap<TYPE> {
    fn new(device: &Device, size: u32) -> Result<Self> {
        unsafe {
            let desc = QueryHeapDesc::default(D3D12_QUERY_HEAP_TYPE(TYPE), size);
            let mut heap: Option<ID3D12QueryHeap> = None;
            device.CreateQueryHeap(&desc, &mut heap)?;

            Ok(Self {
                heap: heap.unwrap(),
                size,
            })
        }
    }

    fn len(&self) -> usize {
        self.size as _
    }
}

impl<const TYPE: i32> AsRef<ID3D12QueryHeap> for QueryHeap<TYPE> {
    fn as_ref(&self) -> &ID3D12QueryHeap {
        &self.heap
    }
}

impl<const TYPE: i32> std::ops::Deref for QueryHeap<TYPE> {
    type Target = ID3D12QueryHeap;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

pub type TimestampQueryHeap = QueryHeap<QUERY_HEAP_TYPE_TIMESTAMP>;

pub struct TimestampQueryPool {
    heap: TimestampQueryHeap,
    staging: Resource,
}

impl TimestampQueryPool {
    pub fn new(device: &Device) -> Result<Self> {
        let count = 64;

        let heap = QueryHeap::new(device, count)?;

        let staging = Resource::new(
            device,
            &HeapProps::readback(),
            None,
            &ResourceDesc::buffer(8 * count as u64, D3D12_RESOURCE_FLAG_NONE),
            D3D12_RESOURCE_STATE_COPY_DEST,
            None,
        )?;

        Ok(Self { heap, staging })
    }

    pub fn iter(&self) -> TimestampQueryIter {
        TimestampQueryIter {
            heap: self.heap.clone(),
            labels: Some(vec![]),
        }
    }

    pub fn buffer(&self) -> &Resource {
        &self.staging
    }

    pub fn dump(&self, freq: u64, labels: &[String]) -> Result<()> {
        let mut label_time = HashMap::<String, u64>::new();

        for (i, &time) in self
            .staging
            .read::<u64>(self.heap.len() as _)?
            .iter()
            .enumerate()
        {
            match label_time.entry(labels[i].clone()) {
                std::collections::hash_map::Entry::Occupied(v) => {
                    let t0 = *v.get();
                    let dt = time - t0;
                    println!(
                        "{}\t{:>4}us",
                        labels[i],
                        (1000.0 * 1000.0 * dt as f64 / freq as f64) as u32
                    );
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(time);
                }
            }
        }

        Ok(())
    }
}

pub struct TimestampQueryIter {
    heap: TimestampQueryHeap,
    labels: Option<Vec<String>>,
}

impl TimestampQueryIter {
    pub fn next(&mut self, label: &str) -> Option<u32> {
        if let Some(labels) = &mut self.labels {
            let index = labels.len();

            if index < self.heap.len() {
                labels.push(label.to_string());

                Some(index as u32)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn heap(&self) -> &TimestampQueryHeap {
        &self.heap
    }

    pub fn take_labels(&mut self) -> Option<Vec<String>> {
        self.labels.take()
    }

    pub fn count(&self) -> u32 {
        if let Some(labels) = &self.labels {
            labels.len() as u32
        } else {
            0
        }
    }
}
