mod error;
pub use error::*;
mod register;
pub use register::*;
mod instr;
pub use instr::*;
mod interrupt;
pub use interrupt::*;
mod aarch64;
pub use aarch64::*;
mod mmu;
pub use mmu::*;

use slab::Slab;

pub type RegId = usize;
pub type FlagId = usize;

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
    pub fn run(&mut self, instr: &Vec<VmInstr>) -> Result<(), Interrupt> {
        while self.ip < instr.len() {
            instr[self.ip](self)?;

            // check control flow has been modified
            // if its modified do not increase instruction pointer.
            if !self.cf_modified {
                self.ip += 1;
            } else {
                self.cf_modified = false;
            }
        }

        Ok(())
    }

    pub fn new_gpr_register(&mut self, name: impl AsRef<str>, size: u8) -> RegId {
        self.gpr_register.insert(GprRegister::new(name, size))
    }

    pub fn get_gpr_register(&mut self, id: RegId) -> Option<&mut GprRegister> {
        self.gpr_register.get_mut(id)
    }

    pub fn new_fpr_register(&mut self, name: impl AsRef<str>, size: u8) -> RegId {
        self.fpr_register.insert(FprRegister {
            name: name.as_ref().to_string(),
            size,
            value: 0.0,
        })
    }

    pub fn get_fpr_register(&mut self, id: RegId) -> Option<&mut FprRegister> {
        self.fpr_register.get_mut(id)
    }

    pub fn get_flag(&mut self, id: FlagId) -> Option<bool> {
        self.flags.get(id).cloned()
    }

    pub fn set_flag(&mut self, id: FlagId, value: bool) {
        if let Some(v) = self.flags.get_mut(id) {
            *v = value;
        }
    }

    pub fn new_flag(&mut self, name: impl AsRef<str>) -> FlagId {
        self.flags.insert(false)
    }

    pub fn set_cf_modified(&mut self) {
        self.cf_modified = true;
    }
}
