use crate::instr::VmInstr;
use crate::jump_table::JumpTable;
use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;
use crate::{Interrupt, MMUError};

use slab::Slab;
use std::collections::HashMap;
use std::sync::Arc;

pub const CARRY_FLAG: u64 = 1 << 0;
pub const OVERFLOW_FLAG: u64 = 1 << 1;

#[derive(Debug)]
pub struct VmContext {
    pub vm_instr: Vec<VmInstr>,
    pub jump_table: JumpTable,
    pub mmu: Mmu,
}

impl VmContext {
    pub fn new() -> Self {
        Self {
            vm_instr: Vec::new(),
            jump_table: JumpTable::new(0x1000),
            mmu: todo!(),
        }
    }
}

#[derive(Debug)]
pub struct Vm {
    ctx: Arc<VmContext>,

    gpr_registers: Slab<GprRegister>,
    fpr_registers: Slab<FprRegister>,

    // instruction pointer
    ipv: usize,
    ipr: u64,
    ipr2ipv_cache: HashMap<u64, usize>,
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
            let instr = &ctx.vm_instr[self.ipv];

            // executing vm is unsafe because it modifies memory in unsafe way
            // do not try to modify memory from mmu outsize of vm execution.
            unsafe {
                instr.op.execute(self, ctx)?;
            }

            // check control flow has been modified
            // if its modified do not increase instruction pointer.
            if !self.ip_modified {
                self.inc_ip(instr.size as u64);
            } else {
                self.ip_modified = false;
            }
        }

        Ok(0)
    }

    pub fn ipr(&self) -> u64 {
        self.ipr
    }

    pub fn inc_ip(&mut self, offset: u64) {
        self.ipv += 1;
        self.ipr += offset;
    }

    pub fn jump2ipv(&mut self, ipv: usize, ipr: u64) {
        self.ip_modified = true;
        self.ipv = ipv;
        self.ipr = ipr as u64;
    }

    pub fn jump2ipr(&mut self, ctx: &VmContext, ipr: u64) {
        if let Some(ipv) = self.ipr2ipv_cache.get(&ipr) {
            self.jump2ipv(*ipv, ipr);
            return;
        }

        // Get checkpoint and its real instruction pointer
        let cp = ctx.jump_table.get_checkpoint(ipr);

        let mut cp_ipv = cp.ipv;
        let mut cp_ipr = cp.ipr;

        // find ipv that has same ipr
        loop {
            assert!(cp_ipr > ipr, "Bad instruction size and its checkpoint!");
            if cp_ipr == ipr {
                break;
            }

            let instr = &ctx.vm_instr[cp_ipv];
            cp_ipv -= 1;
            cp_ipr -= instr.size as u64;
        }

        self.ipr2ipv_cache.insert(cp_ipr, cp_ipv);

        self.ip_modified = true;
        self.ipv = cp_ipv;
        self.ipr = cp_ipr;
    }

    pub fn jump2ipr_rel(&mut self, ctx: &VmContext, ipr: i64) {
        let ipr = (self.ipr as i128 + ipr as i128) as u64;
        self.jump2ipr(ctx, ipr);
    }

    pub fn mem(&self, addr: usize) -> Result<MemoryFrame, MMUError> {
        self.ctx.mmu.frame(addr)
    }

    pub fn gpr(&mut self, id: RegId) -> &mut GprRegister {
        self.gpr_registers
            .get_mut(id.0 as usize)
            .expect("Bad gpr register id")
    }

    pub fn fpr(&mut self, id: RegId) -> &mut FprRegister {
        self.fpr_registers
            .get_mut(id.0 as usize)
            .expect("Bad fpr register id")
    }

    pub fn dump(&self) {
        println!("ip: {}", self.ipv);
        println!("ip_real: {}", self.ipr);
        println!("gpr:");
        for (id, reg) in self.gpr_registers.iter() {
            println!("  {}: {} = 0x{:016x}", id, reg.name(), reg.get());
        }
        println!("fpr:");
        for (id, reg) in self.fpr_registers.iter() {
            println!("  {}: {} = {}", id, reg.name, reg.value);
        }
    }
}
