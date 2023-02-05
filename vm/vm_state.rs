use crate::image::Image;
use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;

use slab::Slab;

pub struct VmState {
    pub(crate) gpr_registers: Slab<GprRegister>,
    pub(crate) fpr_registers: Slab<FprRegister>,

    pub(crate) eflags: u64,
    pub(crate) eip: u64,

    pub(crate) mmu: Mmu,
}

impl VmState {
    pub fn gpr(&self, id: RegId) -> &GprRegister {
        &self.gpr_registers[id.0 as usize]
    }

    pub fn gpr_mut(&mut self, id: RegId) -> &mut GprRegister {
        &mut self.gpr_registers[id.0 as usize]
    }

    pub fn fpr(&self, id: RegId) -> &FprRegister {
        &self.fpr_registers[id.0 as usize]
    }

    pub fn fpr_mut(&mut self, id: RegId) -> &mut FprRegister {
        &mut self.fpr_registers[id.0 as usize]
    }

    pub fn mem(&self, addr: u64) -> MemoryFrame {
        self.mmu.frame(addr)
    }

    pub fn eip(&self) -> u64 {
        self.eip
    }

    pub fn set_eip(&mut self, eip: u64) {
        self.eip = eip;
    }
}