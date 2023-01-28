use crate::instruction::{execute_instr, VmIr};
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
    vm_instr: Vec<u8>,
    jump_table: JumpTable,
    mmu: Mmu,
}

impl VmContext {
    pub fn new() -> Self {
        Self {
            vm_instr: Vec::new(),
            jump_table: JumpTable::new(0x1000),
            mmu: Mmu::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Vm {
    pub(crate) ctx: Arc<VmContext>,

    pub(crate) gpr_registers: Slab<GprRegister>,
    pub(crate) fpr_registers: Slab<FprRegister>,
    pub(crate) regs_byname: HashMap<String, RegId>,

    // instruction pointer
    pub(crate) ipv: usize,
    pub(crate) ipr: u64,
    pub(crate) ipr2ipv_cache: HashMap<u64, usize>,
    pub(crate) ip_modified: bool,

    pub(crate) flags: u64,
}

impl Vm {
    pub fn run(&mut self) -> Result<u64, Interrupt> {
        let r = self.run_inner(&self.ctx.clone());
        return r;
    }

    fn current_instr<'r>(&self, ctx: &'r VmContext) -> VmIr<'r> {
        VmIr::from_ref(&ctx.vm_instr[self.ipv..])
    }

    fn run_inner(&mut self, ctx: &VmContext) -> Result<u64, Interrupt> {
        while self.ipv < ctx.vm_instr.len() {
            let ir = self.current_instr(ctx);

            let curr_size = ir.curr_size() as usize;
            let orgn_size = ir.real_size();

            // executing vm is unsafe because it modifies memory in unsafe way
            // do not try to modify memory from mmu outsize of vm execution.
            unsafe {
                execute_instr(ir, self, ctx)?;
            }

            // check control flow has been modified
            // if its modified do not increase instruction pointer.
            if !self.ip_modified {
                self.inc_ip(curr_size, orgn_size as u64);
            } else {
                self.ip_modified = false;
            }
        }

        Ok(0)
    }

    pub fn ipr(&self) -> u64 {
        self.ipr
    }

    pub fn inc_ip(&mut self, ipv_offset: usize, ipr_offset: u64) {
        self.ipv += ipv_offset;
        self.ipr += ipr_offset;
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
            // cp_ipr -= instr.size as u64;
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

    pub fn mem(&self, addr: u64) -> Result<MemoryFrame, MMUError> {
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

    pub fn reg_by_name(&self, name: impl AsRef<str>) -> Option<RegId> {
        self.regs_byname.get(name.as_ref()).copied()
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
