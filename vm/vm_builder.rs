use crate::image::Image;
use crate::mmu::Mmu;
use crate::register::*;
use crate::vm_state::VmState;

use slab::Slab;

pub struct VmBuilder {
    gpr_registers: Slab<GprRegister>,
    fpr_registers: Slab<FprRegister>,

    mmu: Mmu,
}

impl VmBuilder {
    pub fn new(image: &Image) -> Self {
        Self {
            gpr_registers: Slab::new(),
            fpr_registers: Slab::new(),

            mmu: Mmu::new(),
        }
    }

    pub fn build(self, entry_point: u64) -> VmState {
        VmState {
            gpr_registers: self.gpr_registers,
            fpr_registers: self.fpr_registers,

            mmu: self.mmu,
            eflags: 0,
            eip: entry_point,
        }
    }

    pub fn add_gpr_register(&mut self, register: GprRegister) -> RegId {
        RegId(self.gpr_registers.insert(register) as u8)
    }

    pub fn add_fpr_register(&mut self, register: FprRegister) -> RegId {
        RegId(self.fpr_registers.insert(register) as u8)
    }
}
