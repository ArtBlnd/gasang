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
            Architecture::AArch64Elf => new_aarch64_unknown_linux(image),
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
        //println!("{eip:x}");
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

fn new_aarch64_unknown_linux(image: &[u8]) -> Cpu {
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

    // emulate enviornment variables.
    let buf: &[u8] = &[
        1, 0, 0, 0, 0, 0, 0, 0, 39, 254, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 53, 254,
        255, 255, 255, 255, 0, 0, 69, 254, 255, 255, 255, 255, 0, 0, 79, 254, 255, 255, 255, 255,
        0, 0, 92, 254, 255, 255, 255, 255, 0, 0, 113, 254, 255, 255, 255, 255, 0, 0, 128, 254, 255,
        255, 255, 255, 0, 0, 143, 254, 255, 255, 255, 255, 0, 0, 152, 254, 255, 255, 255, 255, 0,
        0, 163, 254, 255, 255, 255, 255, 0, 0, 176, 254, 255, 255, 255, 255, 0, 0, 187, 254, 255,
        255, 255, 255, 0, 0, 234, 254, 255, 255, 255, 255, 0, 0, 1, 255, 255, 255, 255, 255, 0, 0,
        12, 255, 255, 255, 255, 255, 0, 0, 22, 255, 255, 255, 255, 255, 0, 0, 30, 255, 255, 255,
        255, 255, 0, 0, 47, 255, 255, 255, 255, 255, 0, 0, 75, 255, 255, 255, 255, 255, 0, 0, 98,
        255, 255, 255, 255, 255, 0, 0, 114, 255, 255, 255, 255, 255, 0, 0, 214, 255, 255, 255, 255,
        255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 33, 0, 0, 0, 0, 0, 0, 0, 0, 240, 255, 247, 255, 255, 0,
        0, 51, 0, 0, 0, 0, 0, 0, 0, 112, 18, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 255, 8, 0,
        0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 17, 0, 0, 0, 0, 0, 0, 0,
        100, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 64, 0, 64, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0,
        0, 0, 0, 56, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9,
        0, 0, 0, 0, 0, 0, 0, 16, 36, 64, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 0, 8, 254, 255, 255, 255, 255, 0, 0, 26, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 31, 0, 0, 0, 0, 0, 0, 0, 234, 255, 255, 255, 255, 255,
        0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 24, 254, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 119, 79, 173, 13, 97, 178, 109, 63, 162, 234, 96, 206, 255, 222,
        50, 225, 97, 97, 114, 99, 104, 54, 52, 0, 0, 0, 0, 0, 0, 0, 0, 47, 114, 111, 111, 116, 47,
        99, 111, 110, 116, 101, 110, 116, 0, 83, 72, 69, 76, 76, 61, 47, 98, 105, 110, 47, 98, 97,
        115, 104, 0, 80, 87, 68, 61, 47, 114, 111, 111, 116, 0, 76, 79, 71, 78, 65, 77, 69, 61,
        114, 111, 111, 116, 0, 88, 68, 71, 95, 83, 69, 83, 83, 73, 79, 78, 95, 84, 89, 80, 69, 61,
        116, 116, 121, 0, 95, 61, 47, 117, 115, 114, 47, 98, 105, 110, 47, 103, 100, 98, 0, 77, 79,
        84, 68, 95, 83, 72, 79, 87, 78, 61, 112, 97, 109, 0, 76, 73, 78, 69, 83, 61, 50, 52, 0, 72,
        79, 77, 69, 61, 47, 114, 111, 111, 116, 0, 76, 65, 78, 71, 61, 67, 46, 85, 84, 70, 45, 56,
        0, 67, 79, 76, 85, 77, 78, 83, 61, 56, 48, 0, 73, 78, 86, 79, 67, 65, 84, 73, 79, 78, 95,
        73, 68, 61, 54, 55, 54, 98, 102, 56, 101, 50, 101, 102, 55, 50, 52, 57, 56, 52, 56, 53, 54,
        51, 98, 55, 100, 102, 101, 102, 100, 51, 102, 97, 100, 100, 0, 88, 68, 71, 95, 83, 69, 83,
        83, 73, 79, 78, 95, 67, 76, 65, 83, 83, 61, 117, 115, 101, 114, 0, 84, 69, 82, 77, 61, 118,
        116, 50, 50, 48, 0, 85, 83, 69, 82, 61, 114, 111, 111, 116, 0, 83, 72, 76, 86, 76, 61, 49,
        0, 88, 68, 71, 95, 83, 69, 83, 83, 73, 79, 78, 95, 73, 68, 61, 49, 0, 88, 68, 71, 95, 82,
        85, 78, 84, 73, 77, 69, 95, 68, 73, 82, 61, 47, 114, 117, 110, 47, 117, 115, 101, 114, 47,
        48, 0, 74, 79, 85, 82, 78, 65, 76, 95, 83, 84, 82, 69, 65, 77, 61, 56, 58, 49, 48, 54, 54,
        53, 0, 72, 85, 83, 72, 76, 79, 71, 73, 78, 61, 70, 65, 76, 83, 69, 0, 80, 65, 84, 72, 61,
        47, 117, 115, 114, 47, 108, 111, 99, 97, 108, 47, 115, 98, 105, 110, 58, 47, 117, 115, 114,
        47, 108, 111, 99, 97, 108, 47, 98, 105, 110, 58, 47, 117, 115, 114, 47, 115, 98, 105, 110,
        58, 47, 117, 115, 114, 47, 98, 105, 110, 58, 47, 115, 98, 105, 110, 58, 47, 98, 105, 110,
        58, 47, 114, 111, 111, 116, 47, 46, 108, 111, 99, 97, 108, 47, 98, 105, 110, 58, 47, 114,
        111, 111, 116, 47, 46, 108, 111, 99, 97, 108, 47, 98, 105, 110, 0, 77, 65, 73, 76, 61, 47,
        118, 97, 114, 47, 109, 97, 105, 108, 47, 114, 111, 111, 116, 0, 47, 114, 111, 111, 116, 47,
        99, 111, 110, 116, 101, 110, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    let stack = cpu.reg_by_name("sp").unwrap();
    *cpu.gpr_mut(stack).u64_mut() = 0xfffffffffbf0;

    unsafe {
        cpu.mmu.frame(0xfffffffffbf0).write(buf).unwrap();
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

    const REG_ADDR: u64 = 0x1000000000000;
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
