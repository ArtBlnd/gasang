mod error;
pub use error::*;

mod interrupt;
pub use interrupt::*;

pub mod aarch64;
pub mod instr;
pub mod memory;
pub mod mmu;
pub mod register;

use slab::Slab;

use crate::instr::VmInstr;
use crate::register::*;

#[derive(Debug, Clone, Copy)]
pub struct FlagId(pub usize);

pub struct VmState {
    gpr_register: Slab<GprRegister>,
    fpr_register: Slab<FprRegister>,
    flags: Slab<bool>,

    // instruction pointer
    ip: usize,

    // control flow modified.
    cf_modified: bool,
}

impl VmState {
    pub fn run(&mut self, instr: &Vec<VmInstr>) -> Result<usize, Interrupt> {
        while self.ip < instr.len() {
            instr[self.ip].execute(self)?;

            // check control flow has been modified
            // if its modified do not increase instruction pointer.
            if !self.cf_modified {
                self.ip += 1;
            } else {
                self.cf_modified = false;
            }
        }

        Ok(0)
    }

    pub fn new_gpr_register(&mut self, name: impl AsRef<str>, size: u8) -> RegId {
        RegId(self.gpr_register.insert(GprRegister::new(name, size)))
    }

    pub fn get_gpr_register(&mut self, id: RegId) -> Option<&mut GprRegister> {
        self.gpr_register.get_mut(id.0)
    }

    pub fn new_fpr_register(&mut self, name: impl AsRef<str>, size: u8) -> RegId {
        RegId(self.fpr_register.insert(FprRegister {
            name: name.as_ref().to_string(),
            size,
            value: 0.0,
        }))
    }

    pub fn get_fpr_register(&mut self, id: RegId) -> Option<&mut FprRegister> {
        self.fpr_register.get_mut(id.0)
    }

    pub fn get_flag(&mut self, id: FlagId) -> Option<bool> {
        self.flags.get(id.0).cloned()
    }

    pub fn set_flag(&mut self, id: FlagId, value: bool) {
        if let Some(v) = self.flags.get_mut(id.0) {
            *v = value;
        }
    }

    pub fn new_flag(&mut self, name: impl AsRef<str>) -> FlagId {
        FlagId(self.flags.insert(false))
    }

    pub fn set_cf_modified(&mut self) {
        self.cf_modified = true;
    }
}
