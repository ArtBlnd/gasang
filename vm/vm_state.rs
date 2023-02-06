use crate::interrupt::InterruptModel;
use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;

use std::collections::HashMap;

use slab::Slab;

pub struct VmState {
    pub(crate) gpr_registers: Slab<GprRegister>,
    pub(crate) fpr_registers: Slab<FprRegister>,
    pub(crate) reg_name_map: HashMap<String, RegId>,

    pub(crate) eflags: u64,
    pub(crate) eip: u64,

    pub(crate) mmu: Mmu,

    pub(crate) interrupt_model: Box<dyn InterruptModel>,
}

impl VmState {
    pub fn reg_by_name(&self, name: impl AsRef<str>) -> Option<RegId> {
        self.reg_name_map.get(name.as_ref()).copied()
    }

    #[inline]
    pub fn gpr(&self, id: RegId) -> &GprRegister {
        &self.gpr_registers[id.0 as usize]
    }

    #[inline]
    pub fn gpr_mut(&mut self, id: RegId) -> &mut GprRegister {
        &mut self.gpr_registers[id.0 as usize]
    }

    #[inline]
    pub fn fpr(&self, id: RegId) -> &FprRegister {
        &self.fpr_registers[id.0 as usize]
    }

    #[inline]
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

    pub fn interrupt_model(&self) -> &dyn InterruptModel {
        self.interrupt_model.as_ref()
    }

    pub fn dump(&self) {
        println!("EIP: 0x{:x}", self.eip);
        println!("EFLAGS: 0x{:x}", self.eflags);
        
        for reg in &self.gpr_registers {
            println!("{}: 0x{:x}", reg.1.name(), reg.1.get());
        }

        for reg in &self.fpr_registers {
            println!("{}: 0x{:x}", reg.1.name(), reg.1.get().to_bits());
        }
    }
}
