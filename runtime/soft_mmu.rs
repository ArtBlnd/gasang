use core::{Architecture, ArchitectureCompat};
use device::IoDevice;
use std::{marker::PhantomData, sync::Arc};

use thread_local::ThreadLocal;

#[derive(Clone)]
pub struct DeviceBlock {
    pub base: u64,
    pub size: u64,
    pub device: Arc<dyn IoDevice + Sync + Send>,
}

pub struct SoftMmu {
    map: Vec<DeviceBlock>,
    last_access: ThreadLocal<DeviceBlock>,
}

impl SoftMmu {
    pub fn new() -> Self {
        Self {
            map: Vec::new(),
            last_access: ThreadLocal::new(),
        }
    }

    pub fn map<I>(&mut self, base: u64, size: u64, device: I)
    where
        I: IoDevice + Sync + Send + 'static,
    {
        self.map.push(DeviceBlock {
            base,
            size,
            device: Arc::new(device),
        })
    }
}

impl IoDevice for SoftMmu {
    unsafe fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let device_block = self.last_access.get_or(|| {
            self.map
                .iter()
                .find(|block| (block.base..block.base + block.size).contains(&offset))
                .cloned()
                .unwrap()
        });

        device_block.device.read_at(offset - device_block.base, buf)
    }

    unsafe fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let device_block = self.last_access.get_or(|| {
            self.map
                .iter()
                .find(|block| (block.base..block.base + block.size).contains(&offset))
                .cloned()
                .unwrap()
        });

        device_block
            .device
            .write_at(offset - device_block.base, buf)
    }
}
