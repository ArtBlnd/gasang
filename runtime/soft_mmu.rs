use device::IoDevice;
use parking_lot::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use std::{cell::RefCell, ops::Range, sync::Arc};

use thread_local::ThreadLocal;

#[derive(Clone)]
struct DeviceBlock {
    base: u64,
    size: u64,
    link_state: Arc<LinkState>,
    device: Arc<dyn IoDevice + Sync + Send>,
}

impl DeviceBlock {
    fn range(&self) -> Range<u64> {
        self.base..self.base + self.size
    }
}

pub struct SoftMmu {
    // map is sorted based on base address
    map: Vec<DeviceBlock>,
    last_access: ThreadLocal<RefCell<(usize, DeviceBlock)>>,
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
            link_state: Arc::new(LinkState::new()),
            device: Arc::new(device),
        })
    }

    fn get_device_block(&self, offset: u64) -> DeviceBlock {
        fn is_block_avail(block: &DeviceBlock, offset: u64) -> bool {
            block.range().contains(&offset)
        }

        let (idx, block) = &*self
            .last_access
            .get_or(|| RefCell::new(self.get_device_block_slow(offset)))
            .borrow();

        if is_block_avail(block, offset) {
            return block.clone();
        }

        // Optimization: if the next block is within 4KB, then it is likely that the next access
        if block.base + block.size + 4096 >= offset {
            let next_block = &self.map[idx + 1];
            if is_block_avail(&next_block, offset) {
                return next_block.clone();
            }
        }

        let (_, block) = self.get_device_block_slow(offset);
        block
    }

    fn get_device_block_slow(&self, offset: u64) -> (usize, DeviceBlock) {
        fn get_device_binary(
            map: &[DeviceBlock],
            range: Range<usize>,
            finding: u64,
        ) -> (usize, DeviceBlock) {
            assert_ne!(range.start, range.end, "not exist");
            let mid = (range.start + range.end) / 2;

            if map[mid].range().contains(&finding) {
                (mid, map[mid].clone())
            } else if map[mid].base > finding {
                get_device_binary(map, range.start..mid, finding)
            } else {
                get_device_binary(map, mid..range.end, finding)
            }
        }

        get_device_binary(&self.map, 0..self.map.len(), offset)
    }

    pub unsafe fn ll64(&self, offset: u64) -> u64 {
        let device_block = self.get_device_block(offset);
        let _ = device_block.link_state.link(offset);

        let mut buf = [0u8; 8];
        device_block
            .device
            .read_all_at(offset - device_block.base, &mut buf);
        std::mem::transmute(buf)
    }

    pub unsafe fn sc64(&self, offset: u64, value: u64) -> bool {
        let device_block = self.get_device_block(offset);
        let hold = device_block.link_state.hold(offset);

        if hold.is_none() {
            return false;
        }

        let mut buf: [u8; 8] = std::mem::transmute(value);
        device_block
            .device
            .write_all_at(offset - device_block.base, &mut buf);
        true
    }

    pub unsafe fn ll32(&self, offset: u64) -> u32 {
        let device_block = self.get_device_block(offset);
        let _ = device_block.link_state.link(offset);

        let mut buf = [0u8; 4];
        device_block
            .device
            .read_all_at(offset - device_block.base, &mut buf);
        std::mem::transmute(buf)
    }

    pub unsafe fn sc32(&self, offset: u64, value: u32) -> bool {
        let device_block = self.get_device_block(offset);
        let hold = device_block.link_state.hold(offset);

        if hold.is_none() {
            return false;
        }

        let mut buf: [u8; 4] = std::mem::transmute(value);
        device_block
            .device
            .write_all_at(offset - device_block.base, &mut buf);
        true
    }
}

impl IoDevice for SoftMmu {
    unsafe fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let device_block = self.get_device_block(offset);
        device_block.device.read_at(offset - device_block.base, buf)
    }

    unsafe fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let device_block = self.get_device_block(offset);
        let _ = device_block.link_state.hold_normal();

        device_block
            .device
            .write_at(offset - device_block.base, buf)
    }
}

struct LinkState {
    state: RwLock<u64>,
}

impl LinkState {
    fn new() -> Self {
        Self {
            state: RwLock::new(0),
        }
    }

    /// Creates a new link state.
    fn link(&self, addr: u64) {
        // Wait until the link is available.
        // You can't link the state if it is already linked or writting.
        *self.state.write() = addr;
    }

    /// Returns true if the link is up.
    #[must_use]
    fn hold(&self, addr: u64) -> Option<LinkStateHold1<'_>> {
        let state = self.state.try_write()?;
        if *state == addr {
            Some(LinkStateHold1(state))
        } else {
            None
        }
    }

    #[must_use]
    fn hold_normal(&self) -> LinkStateHold2<'_> {
        let mut state = self.state.upgradable_read();
        if *state != 0 {
            state.with_upgraded(|state| {
                *state = 0;
            });
        }

        LinkStateHold2(state)
    }
}

struct LinkStateHold1<'a>(RwLockWriteGuard<'a, u64>);
struct LinkStateHold2<'a>(RwLockUpgradableReadGuard<'a, u64>);
