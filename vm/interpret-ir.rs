use elf::endian::AnyEndian;
use elf::ElfBytes;

use vm::aarch64::{compile_code, AArch64Compiler};
use vm::register::{FprRegister, GprRegister, RegId};
use vm::VmContext;

use slab::Slab;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let path = std::path::PathBuf::from(filename);

    let file_data = std::fs::read(path).unwrap();

    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    let text_section = file
        .section_header_by_name(".text")
        .expect("section table should be parseable")
        .expect("file should have a .text section");

    let ep_offset = text_section.sh_offset as usize;
    let ep_size = text_section.sh_size as usize;

    let buf = &slice[ep_offset..(ep_offset + ep_size)];

    let mut gpr_storage = Slab::new();
    let mut fpr_storage = Slab::new();

    // initialize AArch64 registers.
    let pstate_reg = RegId(gpr_storage.insert(GprRegister::new("pstate", 8)) as u8);
    let stack_reg = RegId(gpr_storage.insert(GprRegister::new("sp", 8)) as u8);
    let btype_next = RegId(gpr_storage.insert(GprRegister::new("btype_next", 8)) as u8);

    let gpr_registers: [RegId; 32] = (0..32)
        .map(|i| RegId(gpr_storage.insert(GprRegister::new(format!("x{i}"), 8)) as u8))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let fpr_registers: [RegId; 32] = (0..32)
        .map(|i| RegId(fpr_storage.insert(FprRegister::new(format!("f{i}"), 8)) as u8))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let compiler = AArch64Compiler::new(
        gpr_registers,
        fpr_registers,
        pstate_reg,
        stack_reg,
        btype_next,
    );
    let mut vm_ctx = VmContext::new();
    compile_code(text_section.sh_addr, buf, &compiler, &mut vm_ctx);

    let v = vm_ctx.get_instr(0).len();
    let mut offs = 0;
    while offs < v {
        let ir = vm_ctx.get_instr(offs);
        println!("{ir}");

        offs += ir.curr_size() as usize;
    }
}
