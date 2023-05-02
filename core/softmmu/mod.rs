mod host_memory;
mod page;

use crate::debug::WatchKind;
use crate::debug::WatchPoint;

use crate::error::DebugError;
use crate::error::MmuError;

pub use page::BasicPage;
pub use page::Page;
pub use page::PageWithCallback;

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};

const PAGE_SIZE: usize = 0xFFF + 1;
const PAGE_ADDRESS_MASK: usize = usize::MAX - (PAGE_SIZE - 1);

#[derive(Debug, PartialEq)]
pub enum MmuEvent {
    Write(Range<u64>),
    Read(Range<u64>),
}

#[derive(Clone)]
pub struct Mmu {
    inner: Arc<RwLock<MmuData>>,
    watchpoints: Arc<RwLock<Vec<WatchPoint>>>,
    events: Arc<RwLock<Vec<MmuEvent>>>,
}

pub struct MmuData {
    mapped_pages: HashMap<u64, Box<dyn Page>>,
}

impl Mmu {
    pub fn new() -> Self {
        let inner = Arc::new(RwLock::new(MmuData {
            mapped_pages: HashMap::new(),
        }));

        let watchpoints = Arc::new(RwLock::new(Vec::new()));

        let events = Arc::new(RwLock::new(Vec::new()));

        Self {
            inner,
            watchpoints,
            events,
        }
    }

    pub unsafe fn write(&self, addr: u64, buf: &[u8]) -> Result<(), MmuError> {
        let inner = self.inner.read().unwrap();
        inner.write(addr, buf)
    }

    pub unsafe fn read(&self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError> {
        let inner = self.inner.read().unwrap();
        inner.read(addr, buf)
    }

    pub fn is_writable(&self, range: Range<u64>) -> bool {
        let inner = self.inner.read().unwrap();
        inner.is_writable(range)
    }

    pub fn is_readable(&self, range: Range<u64>) -> bool {
        let inner = self.inner.read().unwrap();
        inner.is_readable(range)
    }

    pub fn is_executable(&self, range: Range<u64>) -> bool {
        let inner = self.inner.read().unwrap();
        inner.is_executable(range)
    }

    // Iterating MMU is unsafe
    pub unsafe fn iter(&self, start_addr: u64) -> MmuIter {
        MmuIter {
            addr: start_addr,
            data: self.inner.clone(),
        }
    }

    pub fn add_watchpoint(&self, watchpoint: WatchPoint) -> Result<(), DebugError> {
        let mut watchpoints = self.watchpoints.write().unwrap();
        if watchpoints.contains(&watchpoint) {
            Err(DebugError::WatchpointAlreadyExist(watchpoint))
        } else {
            let events = self.events.clone();

            let mut inner = self.inner.write().unwrap();
            let page = inner.munmap(watchpoint.addr)?;

            let page = PageWithCallback::from_page(
                page,
                Box::new(move |event| {
                    let mut events = events.write().unwrap();

                    events.push(event)
                }),
            );

            inner.mmap(watchpoint.addr, Box::new(page))?;

            watchpoints.push(watchpoint);

            Ok(())
        }
    }

    pub fn remove_watchpoint(&self, watchpoint: WatchPoint) -> Result<(), DebugError> {
        let mut watchpoints = self.watchpoints.write().unwrap();
        if !watchpoints.contains(&watchpoint) {
            Err(DebugError::WatchpointNotExist(watchpoint))
        } else {
            let mut inner = self.inner.write().unwrap();
            let page = inner.munmap(watchpoint.addr)?;

            let page = BasicPage::from_page(page);

            inner.mmap(watchpoint.addr, Box::new(page))?;

            watchpoints.push(watchpoint);

            Ok(())
        }
    }

    pub fn check_watchpoint_hit(&self) -> Option<(u64, WatchKind)> {
        let wps = self.watchpoints.read().unwrap();
        let evs = self.events.read().unwrap();

        wps.iter().find_map(|wp| {
            evs.iter().find_map(|ev| match ev {
                MmuEvent::Write(range) => wp
                    .is_hit(range, WatchKind::Write)
                    .then_some((wp.addr, WatchKind::Write)),
                MmuEvent::Read(range) => wp
                    .is_hit(range, WatchKind::Read)
                    .then_some((wp.addr, WatchKind::Read)),
            })
        })
    }

    pub fn clear_events(&self) {
        let mut evs = self.events.write().unwrap();
        evs.clear();
    }

    pub fn mmap<P>(&self, addr: u64, size: u64, page: Box<P>) -> Result<(), MmuError>
    where
        P: Page + Clone + 'static,
    {
        let mut inner = self.inner.write().unwrap();
        let bias = if (addr + size) % PAGE_SIZE as u64 == 0 {
            0
        } else {
            PAGE_SIZE as u64
        };
        let range = (addr..addr + size + bias).step_by(PAGE_SIZE);

        for addr in range {
            inner.mmap(addr, page.clone())?;
        }

        Ok(())
    }
}

impl MmuData {
    unsafe fn write(&self, addr: u64, buf: &[u8]) -> Result<(), MmuError> {
        let bias = if (addr + buf.len() as u64) % PAGE_SIZE as u64 == 0 {
            0
        } else {
            PAGE_SIZE as u64
        };
        let range = (addr..addr + buf.len() as u64 + bias)
            .step_by(PAGE_SIZE)
            .map(|v| {
                if v != addr {
                    page_initial_address(v)
                } else {
                    v
                }
            });

        let mut cursor = 0;

        for addr in range {
            let page = self.get_page(addr)?;
            let write_len = usize::min(PAGE_SIZE - offset(addr), buf.len() - cursor);

            page.try_write(addr, &buf[cursor..cursor + write_len])?;

            cursor += write_len;
        }

        Ok(())
    }

    unsafe fn read(&self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError> {
        let bias = if (addr + buf.len() as u64) % PAGE_SIZE as u64 == 0 {
            0
        } else {
            PAGE_SIZE as u64
        };
        let range = (addr..addr + buf.len() as u64 + bias)
            .step_by(PAGE_SIZE)
            .map(|v| {
                if v != addr {
                    page_initial_address(v)
                } else {
                    v
                }
            });

        let mut cursor = 0;

        for addr in range {
            let page = self.get_page(addr)?;
            let read_len = usize::min(PAGE_SIZE - offset(addr), buf.len() - cursor);

            page.try_read(addr, &mut buf[cursor..cursor + read_len])?;

            cursor += read_len;
        }

        Ok(())
    }

    fn is_readable(&self, range: Range<u64>) -> bool {
        range.step_by(PAGE_SIZE).all(|addr| {
            if let Ok(page) = self.get_page(addr) {
                page.is_readable()
            } else {
                false
            }
        })
    }

    fn is_writable(&self, range: Range<u64>) -> bool {
        range.step_by(PAGE_SIZE).all(|addr| {
            if let Ok(page) = self.get_page(addr) {
                page.is_writable()
            } else {
                false
            }
        })
    }

    fn is_executable(&self, range: Range<u64>) -> bool {
        range.step_by(PAGE_SIZE).all(|addr| {
            if let Ok(page) = self.get_page(addr) {
                page.is_executable()
            } else {
                false
            }
        })
    }

    fn mmap(&mut self, addr: u64, page: Box<dyn Page>) -> Result<(), MmuError> {
        let init_addr = page_initial_address(addr);
        if self.mapped_pages.contains_key(&init_addr) {
            Err(MmuError::PageAlreadyMapped(addr))
        } else {
            self.mapped_pages.insert(init_addr, page);
            Ok(())
        }
    }

    fn munmap(&mut self, addr: u64) -> Result<Box<dyn Page>, MmuError> {
        let page = self
            .mapped_pages
            .remove(&page_initial_address(addr))
            .ok_or(MmuError::PageNotMapped(addr))?;

        Ok(page)
    }

    fn get_page(&self, addr: u64) -> Result<&Box<dyn Page>, MmuError> {
        self.mapped_pages
            .get(&page_initial_address(addr))
            .ok_or(MmuError::PageNotMapped(addr))
    }
}

pub struct MmuIter {
    addr: u64,
    data: Arc<RwLock<MmuData>>,
}

impl Iterator for MmuIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.data.read().unwrap();
        let mut buf = [0u8; 1];
        unsafe {
            inner.read(self.addr, &mut buf).ok()?;
        }

        self.addr += 1;

        Some(buf[0])
    }
}

fn page_initial_address(addr: u64) -> u64 {
    addr & PAGE_ADDRESS_MASK as u64
}

fn offset(addr: u64) -> usize {
    addr as usize & (PAGE_SIZE - 1)
}

mod test {
    use super::*;

    #[test]
    fn mmu_write_test() {
        let mmu = Mmu::new();
        let buf = "Hello World".as_bytes();
        mmu.mmap(
            0xbabe1234cafe5678,
            PAGE_SIZE as u64,
            Box::new(BasicPage::new(true, true, true)),
        )
        .unwrap();
        unsafe { mmu.write(0xbabe1234cafe5678, &buf).unwrap() }

        let page = mmu
            .inner
            .write()
            .unwrap()
            .munmap(0xbabe1234cafe5678)
            .unwrap();

        let mut result: Vec<_> = (0..buf.len()).map(|_| 0u8).collect();
        unsafe { page.try_read(0xbabe1234cafe5678, &mut result).unwrap() }
        assert_eq!(buf, result);
    }

    #[test]
    fn mmu_read_test() {
        mmu_write_test(); // to check write is not wrong

        let mmu = Mmu::new();
        mmu.mmap(
            0xbabe1234cafe5678,
            PAGE_SIZE as u64,
            Box::new(BasicPage::new(true, true, true)),
        )
        .unwrap();

        let hello = "Hello World".as_bytes();

        unsafe { mmu.write(0xbabe1234cafe5678, hello).unwrap() }

        let mut buf: Vec<_> = (0..hello.len()).map(|_| 0u8).collect();

        unsafe { mmu.read(0xbabe1234cafe5678, &mut buf).unwrap() }

        assert_eq!(hello, buf);
    }

    #[test]
    fn mmu_exceed_page_size_test() {
        mmu_read_test();

        let mmu = Mmu::new();
        mmu.mmap(
            0xbabe1234cafe5678,
            (PAGE_SIZE * 3) as u64,
            Box::new(BasicPage::new(true, true, true)),
        )
        .unwrap();

        let test_buf: Vec<_> = (0..(PAGE_SIZE * 3)).map(|_| 5u8).collect();

        unsafe {
            mmu.write(0xbabe1234cafe5678, &test_buf).unwrap();
        }

        let mut result: Vec<_> = (0..(PAGE_SIZE * 3)).map(|_| 0u8).collect();

        unsafe {
            mmu.read(0xbabe1234cafe5678, &mut result).unwrap();
        }

        assert_eq!(test_buf, result);
    }
}
