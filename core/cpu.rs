use crate::register::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use gdbstub::arch::Registers;
use slab::Slab;

#[derive(Debug)]
pub struct Cpu {
    gpr_registers: Slab<GprRegister>,
    fpr_registers: Slab<FprRegister>,
    sys_registers: Slab<SysRegister>,
    reg_name_map: HashMap<String, RegId>,

    flags: AtomicU64,
    pc: u64,
    arch: Architecture,
}

impl PartialEq for Cpu {
    fn eq(&self, other: &Self) -> bool {
        for (k, v) in self.gpr_registers.iter() {
            if other.gpr_registers.get(k) == Some(v) {
                continue;
            } else {
                return false;
            }
        }
        for (k, v) in self.fpr_registers.iter() {
            if other.fpr_registers.get(k) == Some(v) {
                continue;
            } else {
                return false;
            }
        }
        for (k, v) in self.sys_registers.iter() {
            if other.sys_registers.get(k) == Some(v) {
                continue;
            } else {
                return false;
            }
        }

        self.flag() == other.flag() && self.pc == other.pc
    }
}

impl Default for Cpu {
    fn default() -> Self {
        panic!("No default cpu");
    }
}

impl Clone for Cpu {
    fn clone(&self) -> Self {
        Self {
            gpr_registers: self.gpr_registers.clone(),
            fpr_registers: self.fpr_registers.clone(),
            sys_registers: self.sys_registers.clone(),
            reg_name_map: self.reg_name_map.clone(),

            flags: AtomicU64::new(self.flags.load(Ordering::Relaxed)),
            pc: self.pc,
            arch: self.arch.clone(),
        }
    }
}

impl Cpu {
    pub fn new(arch: Architecture) -> Self {
        match arch {
            Architecture::AArch64Bin => init_base_aarch64_cpu(),
            _ => unreachable!(),
        }
    }

    pub(crate) fn new_for_test() -> Self {
        let mut gpr_registers = Slab::new();
        let mut reg_name_map = HashMap::new();

        let id = gpr_registers.insert(GprRegister::new("x0", 8));
        reg_name_map.insert(format!("x0"), RegId(id as u8));

        Self {
            gpr_registers,
            fpr_registers: Slab::new(),
            sys_registers: Slab::new(),
            reg_name_map,

            flags: AtomicU64::new(0),
            pc: 0,
            arch: Architecture::Test,
        }
    }

    pub fn reg_by_name(&self, name: impl AsRef<str>) -> Option<RegId> {
        self.reg_name_map.get(name.as_ref()).copied()
    }

    pub fn get_register_info(&self) -> HashMap<String, RegId> {
        self.reg_name_map.clone()
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

    #[inline]
    pub fn sys(&self, id: RegId) -> &SysRegister {
        &self.sys_registers[id.0 as usize]
    }

    #[inline]
    pub fn sys_mut(&mut self, id: RegId) -> &mut SysRegister {
        &mut self.sys_registers[id.0 as usize]
    }

    pub fn pc(&self) -> u64 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u64) {
        self.pc = pc;
    }

    pub fn flag(&self) -> u64 {
        self.flags.load(Ordering::SeqCst)
    }

    pub fn set_flag(&self, flag: u64) {
        self.flags.store(flag, Ordering::SeqCst);
    }

    pub fn add_flag(&self, flag: u64) {
        self.flags.fetch_or(flag, Ordering::SeqCst);
    }

    pub fn del_flag(&self, flag: u64) {
        self.flags.fetch_and(!flag, Ordering::SeqCst);
    }

    pub fn arch(&self) -> &Architecture {
        &self.arch
    }

    pub fn dump(&self) {
        self.dump_gpr();
        self.dump_fpr();
        self.dump_sys();
        self.dump_pc();
    }

    pub fn dump_gpr(&self) {
        for reg in self.gpr_registers.iter() {
            println!("{:14} 0x{:x} ", reg.1.name(), reg.1.u64());
        }
    }

    pub fn dump_fpr(&self) {
        for reg in self.fpr_registers.iter() {
            println!("{:14} 0x{:x} ", reg.1.name(), reg.1.u64());
        }
    }

    pub fn dump_sys(&self) {
        for reg in self.sys_registers.iter() {
            println!("{:14} 0x{:x} ", reg.1.name(), reg.1.u64());
        }
    }

    pub fn dump_flags(&self) {
        println!("{:14} 0x{:b}", "flags", self.flags.load(Ordering::SeqCst));
    }

    pub fn dump_pc(&self) {
        println!("{:14} 0x{:x}", "pc", self.pc);
    }
}

#[derive(Debug, Clone)]
pub enum Architecture {
    Test,
    AArch64Bin,
}

impl Architecture {
    pub fn gprs(&self) -> Vec<String> {
        let mut result = Vec::new();
        for i in 0..31 {
            let name = format!("x{}", i);
            result.push(name);
        }

        result.push("sp".to_string());

        result
    }
}

fn init_base_aarch64_cpu() -> Cpu {
    let mut cpu = Cpu {
        gpr_registers: Slab::new(),
        fpr_registers: Slab::new(),
        sys_registers: Slab::new(),
        reg_name_map: HashMap::new(),
        flags: AtomicU64::new(0),
        pc: 0,
        arch: Architecture::AArch64Bin,
    };

    for i in 0..31 {
        let id = cpu
            .gpr_registers
            .insert(GprRegister::new(format!("x{}", i), 8));
        cpu.reg_name_map.insert(format!("x{}", i), RegId(id as u8));
    }

    for i in 0..31 {
        let id = cpu
            .fpr_registers
            .insert(FprRegister::new(format!("v{}", i), 16));
        cpu.reg_name_map.insert(format!("v{}", i), RegId(id as u8));
    }

    let id = cpu.gpr_registers.insert(GprRegister::new("sp", 8));
    cpu.reg_name_map.insert("sp".to_string(), RegId(id as u8));

    let id = cpu.sys_registers.insert(SysRegister::new("tpidr_el0", 8));
    cpu.reg_name_map
        .insert("tpidr_el0".to_string(), RegId(id as u8));

    let id = cpu.sys_registers.insert(SysRegister::new("vbar_el1", 8));
    cpu.reg_name_map
        .insert("vbar_el1".to_string(), RegId(id as u8));

    let id = cpu.sys_registers.insert(SysRegister::new("cpacr_el1", 8));
    cpu.reg_name_map
        .insert("cpacr_el1".to_string(), RegId(id as u8));

    let id = cpu.sys_registers.insert(SysRegister::new("mpidr_el1", 8));
    cpu.reg_name_map
        .insert("mpidr_el1".to_string(), RegId(id as u8));

    cpu
}
