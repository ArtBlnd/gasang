use std::collections::HashMap;

use crate::loader::Loader;
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
    pub fn new(loader: impl Loader) -> Self {
        let mmu = Mmu::new();

        loader.load(&mmu);

        Self {
            gpr_registers: Slab::new(),
            fpr_registers: Slab::new(),

            mmu,
        }
    }

    pub fn build(self, entry_point: u64) -> VmState {
        let mut reg_name_map = HashMap::new();
        for gpr in &self.gpr_registers {
            let k = gpr.1.name();
            let v = gpr.0;

            reg_name_map.insert(k.to_string(), RegId(v as u8));
        }
        for fpr in &self.fpr_registers {
            let k = fpr.1.name();
            let v = fpr.0;

            reg_name_map.insert(k.to_string(), RegId(v as u8));
        }

        VmState {
            gpr_registers: self.gpr_registers,
            fpr_registers: self.fpr_registers,

            reg_name_map,

            mmu: self.mmu,
            flags: Default::default(),
            ip: entry_point,
        }
    }

    pub fn add_gpr_register(&mut self, register: GprRegister) -> RegId {
        RegId(self.gpr_registers.insert(register) as u8)
    }

    pub fn add_fpr_register(&mut self, register: FprRegister) -> RegId {
        RegId(self.fpr_registers.insert(register) as u8)
    }
}
