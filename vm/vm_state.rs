use crate::interrupt::InterruptModel;
use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;

use std::collections::HashMap;

use slab::Slab;

pub struct VmState {
    pub(crate) gpr_registers: Slab<GprRegister>,
    pub(crate) fpr_registers: Slab<FprRegister>,
    pub(crate) reg_name_map: HashMap<String, RegId>,

    pub(crate) flags: u64,
    pub(crate) ip: u64,

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

    pub fn mmu(&self) -> &Mmu {
        &self.mmu
    }

    pub fn ip(&self) -> u64 {
        self.ip
    }

    pub fn set_ip(&mut self, eip: u64) {
        self.ip = eip;
        println!("Jump to ip: {eip:x}");
    }

    pub fn flag(&self) -> u64 {
        self.flags
    }

    pub fn set_flag(&mut self, flag: u64) {
        self.flags = flag;
    }

    pub fn add_flag(&mut self, flag: u64) {
        self.flags |= flag;
    }

    pub fn del_flag(&mut self, flag: u64) {
        self.flags &= !flag;
    }

    pub fn interrupt_model(&self) -> &dyn InterruptModel {
        self.interrupt_model.as_ref()
    }

    pub fn dump(&self) {
        println!("EIP: 0x{:x}", self.ip);
        println!("EFLAGS: {:064b}", self.flags);

        for reg in &self.gpr_registers {
            print!("({}: 0x{:x}), ", reg.1.name(), reg.1.get());
        }
        println!();
    }
}
