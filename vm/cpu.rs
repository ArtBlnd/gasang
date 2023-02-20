use crate::mmu::{MemoryFrame, Mmu};
use crate::register::*;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use elf::abi::{PF_R, PF_W, PF_X, SHF_EXECINSTR, SHF_WRITE};
use elf::endian::AnyEndian;
use slab::Slab;

pub struct Cpu {
    gpr_registers: Slab<GprRegister>,
    fpr_registers: Slab<FprRegister>,
    sys_registers: Slab<SysRegister>,
    reg_name_map: HashMap<String, RegId>,

    flags: AtomicU64,
    ip: u64,

    mmu: Mmu,
}

impl Cpu {
    pub fn new(arch: Architecture, image: &[u8]) -> Self {
        match arch {
            Architecture::AArch64Elf => new_aarch64_elf(image),
            Architecture::AArch64Bin => new_aarch64_bin(image),
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
        //println!("<<<<<{eip:x}>>>>>");
        self.ip = eip;
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

    pub fn dump(&self) {
        for reg in self.gpr_registers.iter() {
            print!("{}: 0x{:x} ", reg.1.name(), reg.1.u64());
        }
        println!();

        for reg in self.fpr_registers.iter() {
            print!("{}: 0x{:x} ", reg.1.name(), reg.1.u64());
        }
        println!();

        for reg in self.sys_registers.iter() {
            print!("{}: 0x{:x} ", reg.1.name(), reg.1.u64());
        }
        println!();

        println!("ip: 0x{:016x}", self.ip);
        println!("flag: 0x{:064b}", self.flag());
    }
}

pub enum Architecture {
    AArch64Elf,
    AArch64Bin,
}

fn new_aarch64_bin(image: &[u8]) -> Cpu {
    todo!()
}

fn new_aarch64_elf(image: &[u8]) -> Cpu {
    let mut cpu = init_base_aarch64_cpu();
    let file = elf::ElfBytes::<AnyEndian>::minimal_parse(image).expect("Invalid image");

    for seg in file.segments().unwrap() {
        let addr = seg.p_paddr;
        let size = seg.p_memsz;
        let data = file.segment_data(&seg).unwrap();

        cpu.mmu.mmap(addr, size, true, true, false);
        unsafe {
            cpu.mmu
                .frame(addr)
                .write(data)
                .expect("Failed VM Initialize");
        }

        if seg.p_type == elf::abi::PT_TLS {
            let reg_id = cpu.reg_by_name("tpidr_el0").unwrap();
            *cpu.sys_mut(reg_id).u64_mut() = addr;
        }

        let readable = seg.p_flags & PF_R != 0;
        let writable = seg.p_flags & PF_W != 0;
        let executable = seg.p_flags & PF_X != 0;

        assert!(seg.p_flags & elf::abi::PF_MASKPROC == 0);

        assert!(readable);

        cpu.mmu.mmap(addr, size, readable, writable, executable)
    }

    for sec in file.section_headers().unwrap() {
        let addr = sec.sh_addr;
        let size = sec.sh_size;

        let writable = sec.sh_flags & SHF_WRITE as u64 != 0;
        let executable = sec.sh_flags & SHF_EXECINSTR as u64 != 0;

        cpu.mmu.mmap(addr, size, true, writable, executable)
    }

    cpu.ip = file.ehdr.e_entry;
    cpu
}

fn init_base_aarch64_cpu() -> Cpu {
    let mut cpu = Cpu {
        gpr_registers: Slab::new(),
        fpr_registers: Slab::new(),
        sys_registers: Slab::new(),
        reg_name_map: HashMap::new(),
        flags: AtomicU64::new(0),
        ip: 0,
        mmu: Mmu::new(),
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

    const REG_ADDR: u64 = 0x7ffffffffff8000;
    const REG_SIZE: u64 = 1024 * 1024 * 4;

    let mut sp_reg = GprRegister::new("sp", 8);
    *sp_reg.u64_mut() = REG_ADDR;
    cpu.mmu()
        .mmap(REG_ADDR - REG_SIZE, REG_SIZE, true, true, false);
    let id = cpu.gpr_registers.insert(sp_reg);
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

    cpu
}
