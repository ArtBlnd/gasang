use crate::error::MmuError;
use crate::Cpu;

use crate::softmmu::Mmu;

pub struct ExecutionContext<'a> {
    pub cpu: &'a mut Cpu,
    pub mmu: &'a Mmu,
}

impl<'a> ExecutionContext<'a> {
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    pub unsafe fn mem_read(&mut self, addr: u64, buf: &mut [u8]) -> Result<(), MmuError> {
        self.mmu.read(addr, buf)
    }

    pub unsafe fn mem_read_u8(&mut self, addr: u64) -> Result<u8, MmuError> {
        let mut buf = [0u8; 1];
        self.mem_read(addr, &mut buf)?;

        Ok(buf[0])
    }

    pub unsafe fn mem_read_u16(&mut self, addr: u64) -> Result<u16, MmuError> {
        let mut buf = [0u8; 2];
        self.mem_read(addr, &mut buf)?;
        let result = u16::from_le_bytes(buf);

        Ok(result)
    }

    pub unsafe fn mem_read_u32(&mut self, addr: u64) -> Result<u32, MmuError> {
        let mut buf = [0u8; 4];
        self.mem_read(addr, &mut buf)?;
        let result = u32::from_le_bytes(buf);

        Ok(result)
    }

    pub unsafe fn mem_read_u64(&mut self, addr: u64) -> Result<u64, MmuError> {
        let mut buf = [0u8; 8];
        self.mem_read(addr, &mut buf)?;
        let result = u64::from_le_bytes(buf);

        Ok(result)
    }

    pub unsafe fn mem_write(&mut self, addr: u64, buf: &[u8]) -> Result<(), MmuError> {
        self.mmu.write(addr, buf)
    }

    pub unsafe fn mem_write_u8(&mut self, addr: u64, val: u8) -> Result<(), MmuError> {
        self.mem_write(addr, &[val])
    }

    pub unsafe fn mem_write_u16(&mut self, addr: u64, val: u16) -> Result<(), MmuError> {
        self.mem_write(addr, &val.to_le_bytes())
    }

    pub unsafe fn mem_write_u32(&mut self, addr: u64, val: u32) -> Result<(), MmuError> {
        self.mem_write(addr, &val.to_le_bytes())
    }

    pub unsafe fn mem_write_u64(&mut self, addr: u64, val: u64) -> Result<(), MmuError> {
        self.mem_write(addr, &val.to_le_bytes())
    }
}

pub trait Executable {
    type Output;

    unsafe fn execute<'a>(&self, ctx: &mut ExecutionContext<'a>) -> Self::Output;
}

pub struct FnExec<O> {
    func: Box<dyn for<'a> Fn(&mut ExecutionContext<'a>) -> O>,
}

impl<O> FnExec<O> {
    pub fn new<F>(func: F) -> Self
    where
        F: for<'a> Fn(&mut ExecutionContext<'a>) -> O + 'static,
    {
        Self {
            func: Box::new(func),
        }
    }
}

impl<O> Executable for FnExec<O> {
    type Output = O;

    unsafe fn execute<'a>(&self, ctx: &mut ExecutionContext<'a>) -> O {
        (self.func)(ctx)
    }
}
