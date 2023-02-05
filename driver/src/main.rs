use elf::endian::AnyEndian;
use elf::ElfBytes;
use machineinstr::aarch64::AArch64InstrParserRule;
use vm::codegen::interpret::InterpretCodegen;
use vm::compiler::aarch64::AArch64Compiler;
use vm::register::{FprRegister, GprRegister, RegId};
use vm::vm_builder::VmBuilder;

use std::path::PathBuf;

use vm::engine::Engine;
use vm::image::*;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let file_buf = std::fs::read(PathBuf::from(filename)).unwrap();
    let file_elf = ElfBytes::<AnyEndian>::minimal_parse(&file_buf).unwrap();

    let mut image = Image::from_image(file_buf.to_vec());

    let Ok((Some(sec_headers), Some(strtbl))) = file_elf.section_headers_with_strtab() else {
        panic!("Bad elf file!");
    };

    for sec in sec_headers {
        let name = strtbl.get(sec.sh_name as usize).unwrap();
        let beg = sec.sh_offset;
        let end = sec.sh_offset + sec.sh_size;

        if name == ".text" {
            image.set_entrypoint(sec.sh_addr);
        }

        let (writable, executable) = get_access_info(sec.sh_flags);

        image.add_section(
            name,
            sec.sh_addr,
            writable,
            executable,
            beg as usize,
            end as usize,
        );
    }

    let mut vm_state = VmBuilder::new(&image);
    let gpr_registers: [RegId; 31] = std::array::from_fn(|idx| {
        vm_state.add_gpr_register(GprRegister::new(format!("x{}", idx), 8))
    });
    let fpr_registers: [RegId; 31] = std::array::from_fn(|idx| {
        vm_state.add_fpr_register(FprRegister::new(format!("x{}", idx), 8))
    });

    let mut vm_state = vm_state.build(image.entrypoint());
    let compiler = AArch64Compiler::new(gpr_registers, fpr_registers);
    let parse_rule = AArch64InstrParserRule;
    let codegen = InterpretCodegen;

    let mut engine = Engine::new(compiler, parse_rule, codegen);

    unsafe {
        engine.run(&mut vm_state).unwrap();
    }
}

fn get_access_info(sh_flags: u64) -> (bool, bool) {
    let writable = (sh_flags & 0x1) != 0;
    let executable = (sh_flags & 0x4) != 0;

    (writable, executable)
}
