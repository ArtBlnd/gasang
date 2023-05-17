use crate::error::MmuError;

use super::host_memory::HostMemory;
use super::MmuEvent;
use super::PAGE_SIZE;

fn offset(addr: u64) -> usize {
    addr as usize & (PAGE_SIZE - 1)
}

pub trait Page {
    unsafe fn try_write(&self, addr: u64, buf: &[u8]) -> Result<(), MmuError>;
    unsafe fn try_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError>;

    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;
    fn is_executable(&self) -> bool;

    unsafe fn into_inner(self: Box<Self>) -> HostMemory;
}

#[derive(Clone)]
pub struct BasicPage {
    memory: HostMemory,
    readable: bool,
    writable: bool,
    executable: bool,
}

impl BasicPage {
    pub fn new(readable: bool, writable: bool, executable: bool) -> Self {
        Self {
            memory: HostMemory::new(PAGE_SIZE),
            readable,
            writable,
            executable,
        }
    }

    pub fn from_page(page: Box<dyn Page>) -> Self {
        let readable = page.is_readable();
        let writable = page.is_writable();
        let executable = page.is_executable();
        let memory = unsafe { page.into_inner() };
        Self {
            memory,
            readable,
            writable,
            executable,
        }
    }
}

impl Page for BasicPage {
    unsafe fn try_write(&self, addr: u64, buf: &[u8]) -> Result<(), MmuError> {
        if self.writable == false {
            return Err(MmuError::AccessViolation(addr));
        }

        let start = offset(addr);
        let end = start + buf.len();

        self.memory.slice()[start..end].copy_from_slice(buf);

        debug_assert!(end <= PAGE_SIZE, "buffer is too large");

        Ok(())
    }

    unsafe fn try_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError> {
        if self.readable == false {
            return Err(MmuError::AccessViolation(addr));
        }

        let start = offset(addr);
        let end = start + buf.len();

        debug_assert!(end <= PAGE_SIZE, "buffer is too large");

        buf.copy_from_slice(&self.memory.slice()[start..end]);

        Ok(())
    }

    fn is_readable(&self) -> bool {
        self.readable
    }

    fn is_writable(&self) -> bool {
        self.writable
    }

    fn is_executable(&self) -> bool {
        self.executable
    }

    unsafe fn into_inner(self: Box<Self>) -> HostMemory {
        self.memory
    }
}

pub struct PageWithCallback {
    memory: HostMemory,
    callback: Box<dyn Fn(MmuEvent) -> ()>,
    readable: bool,
    writable: bool,
    executable: bool,
}

impl PageWithCallback {
    pub fn from_page(page: Box<dyn Page>, callback: Box<dyn Fn(MmuEvent) -> ()>) -> Self {
        let readable = page.is_readable();
        let writable = page.is_writable();
        let executable = page.is_executable();
        let memory = unsafe { page.into_inner() };
        Self {
            memory,
            callback,
            readable,
            writable,
            executable,
        }
    }
}

impl Page for PageWithCallback {
    unsafe fn try_write(&self, addr: u64, buf: &[u8]) -> Result<(), MmuError> {
        if self.writable == false {
            return Err(MmuError::AccessViolation(addr));
        }

        debug_assert!(buf.len() <= PAGE_SIZE, "buffer is too large");

        let start = offset(addr);
        let end = start + buf.len();

        self.memory.slice()[start..end].copy_from_slice(buf);

        (self.callback)(MmuEvent::Write(addr..addr + buf.len() as u64));

        Ok(())
    }

    unsafe fn try_read(&self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError> {
        if self.readable == false {
            return Err(MmuError::AccessViolation(addr));
        }

        debug_assert!(buf.len() <= PAGE_SIZE, "buffer is too large");

        let start = offset(addr);
        let end = start + buf.len();

        buf.copy_from_slice(&self.memory.slice()[start..end]);

        (self.callback)(MmuEvent::Read(addr..addr + buf.len() as u64));

        Ok(())
    }

    fn is_readable(&self) -> bool {
        self.readable
    }

    fn is_writable(&self) -> bool {
        self.writable
    }

    fn is_executable(&self) -> bool {
        self.executable
    }

    unsafe fn into_inner(self: Box<Self>) -> HostMemory {
        self.memory
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn basic_page_write_test() {
        let page = BasicPage::new(true, true, true);
        let content = "Hello World".as_bytes();
        unsafe {
            page.try_write(0xbee, content).unwrap();
        }

        let result = unsafe { &page.memory.slice()[0xbee..0xbee + content.len()] };

        assert_eq!(content, result)
    }

    #[test]
    fn basic_page_read_test() {
        let page = BasicPage::new(true, true, true);
        let content = "Hello World".as_bytes();

        unsafe {
            page.memory.slice()[0xbee..0xbee + content.len()].copy_from_slice(content);
        }

        let mut result: Vec<_> = (0..content.len()).map(|_| 0u8).collect();

        unsafe {
            page.try_read(0xbee, &mut result).unwrap();
        }

        assert_eq!(content, &result)
    }

    #[test]
    fn page_with_callback_write_test() {
        let page = BasicPage::new(true, true, true);
        let event_list = Arc::new(Mutex::new(Vec::new()));
        let e_list = event_list.clone();

        let page = PageWithCallback::from_page(
            Box::new(page),
            Box::new(move |e| {
                let mut e_list = e_list.lock().unwrap();
                e_list.push(e);
            }),
        );

        let content = "Hello World".as_bytes();

        unsafe {
            page.try_write(0xbee, content).unwrap();
        }

        let result = unsafe { &page.memory.slice()[0xbee..0xbee + content.len()] };

        assert_eq!(content, result);

        let event_list = event_list.lock().unwrap();
        assert_eq!(
            event_list[0],
            MmuEvent::Write(0xbee..0xbee + content.len() as u64)
        )
    }

    #[test]
    fn page_with_callback_read_test() {
        let page = BasicPage::new(true, true, true);
        let event_list = Arc::new(Mutex::new(Vec::new()));
        let e_list = event_list.clone();

        let page = PageWithCallback::from_page(
            Box::new(page),
            Box::new(move |e| {
                let mut e_list = e_list.lock().unwrap();
                e_list.push(e);
            }),
        );

        let content = "Hello World".as_bytes();

        unsafe {
            page.memory.slice()[0xbee..0xbee + content.len()].copy_from_slice(content);
        }

        let mut result: Vec<_> = (0..content.len()).map(|_| 0u8).collect();

        unsafe {
            page.try_read(0xbee, &mut result).unwrap();
        }

        assert_eq!(content, &result);

        let event_list = event_list.lock().unwrap();
        assert_eq!(
            event_list[0],
            MmuEvent::Read(0xbee..0xbee + content.len() as u64)
        )
    }
}
