use crate::instr::VmInstr;
use crate::jump_table::JumpTable;
use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;
use crate::{Interrupt, MMUError};

use slab::Slab;
use std::sync::Arc;

#[derive(Debug)]
pub struct VmContext {
    pub vm_instr: Vec<VmInstr>,
}

#[derive(Debug)]
pub struct Vm {
    ctx: Arc<VmContext>,
    mmu: Mmu,
    jump_table: JumpTable,

    gpr_register: Slab<GprRegister>,
    fpr_register: Slab<FprRegister>,

    // instruction pointer
    ipv: usize,
    ipr: u64,
    ip_modified: bool,

    flags: u64,
}

impl Vm {
    pub fn run(&mut self) -> Result<usize, Interrupt> {
        let r = self.run_inner(&self.ctx.clone());
        return r;
    }

    fn run_inner(&mut self, ctx: &VmContext) -> Result<usize, Interrupt> {
        while self.ipv < ctx.vm_instr.len() {
            unsafe {
                ctx.vm_instr[self.ipv].op.execute(self)?;
            }

            // check control flow has been modified
            // if its modified do not increase instruction pointer.
            if !self.ip_modified {
                // inc_ip
            } else {
                self.ip_modified = false;
            }
        }

        Ok(0)
    }

    pub fn inc_ip(&mut self, offset: u64) {
        self.ipv += 1;
        self.ipr += offset;
    }

    pub fn set_ipv(&mut self, ipv: usize, ipr: u64) {
        self.ip_modified = true;
        self.ipv = ipv;
        self.ipr = ipr as u64;
    }

    pub fn set_ipr(&mut self, ctx: &VmContext, ipr: u64) {
        // Get checkpoint and its real instruction pointer
        let cp = self.jump_table.get_checkpoint(ipr);

        let mut cp_ipv = cp.ipv;
        let mut cp_ipr = cp.ipr;

        // find ipv that has same ipr
        loop {
            assert!(cp_ipr <= ipr, "Bad instruction size and its checkpoint!");
            if cp_ipr == ipr {
                break;
            }

            let instr = &ctx.vm_instr[cp_ipv];
            cp_ipv += 1;
            cp_ipr += instr.size as u64;
        }

        self.ip_modified = true;
        self.ipv = cp_ipv;
        self.ipr = cp_ipr;
    }

    pub unsafe fn mem(&self, addr: usize) -> Result<MemoryFrame, MMUError> {
        self.mmu.frame(addr)
    }

    pub fn gpr(&mut self, id: RegId) -> &mut GprRegister {
        self.gpr_register
            .get_mut(id.0 as usize)
            .expect("Bad gpr register id")
    }

    pub fn fpr(&mut self, id: RegId) -> &mut FprRegister {
        self.fpr_register
            .get_mut(id.0 as usize)
            .expect("Bad fpr register id")
    }

    pub fn dump(&self) {
        println!("ip: {}", self.ipv);
        println!("ip_real: {}", self.ipr);
        println!("gpr:");
        for (id, reg) in self.gpr_register.iter() {
            println!("  {}: {} = 0x{:016x}", id, reg.name(), reg.get());
        }
        println!("fpr:");
        for (id, reg) in self.fpr_register.iter() {
            println!("  {}: {} = {}", id, reg.name, reg.value);
        }
    }
}
