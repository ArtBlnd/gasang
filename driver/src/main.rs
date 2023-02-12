use machineinstr::aarch64::AArch64InstrParserRule;

use vm::codegen::flag_policy::AArch64FlagPolicy;
use vm::codegen::interpret::InterpretCodegen;
use vm::compiler::aarch64::AArch64Compiler;
use vm::engine::Engine;

use vm::interrupt::AArch64UnixInterruptModel;
use vm::loader::elf::ElfLoader;
use vm::loader::Loader;
use vm::register::{FprRegister, GprRegister, RegId};
use vm::vm_builder::VmBuilder;

use std::path::PathBuf;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let file_buf = std::fs::read(PathBuf::from(filename)).unwrap();
    let loader = ElfLoader::new(&file_buf);
    let entry = loader.entry();

    let mut vm_state = VmBuilder::new(loader);
    let gpr_registers: [RegId; 31] = (0..31)
        .map(|idx| vm_state.add_gpr_register(GprRegister::new(format!("x{idx}"), 8)))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let fpr_registers: [RegId; 31] = (0..31)
        .map(|idx| vm_state.add_fpr_register(FprRegister::new(format!("d{idx}"), 8)))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let stack_reg = vm_state.add_gpr_register(GprRegister::new("sp", 8));

    let compiler = AArch64Compiler::new(gpr_registers, fpr_registers, stack_reg);
    let parse_rule = AArch64InstrParserRule;
    let codegen = InterpretCodegen::new(AArch64FlagPolicy);

    let mut engine = Engine::new(compiler, parse_rule, AArch64UnixInterruptModel, codegen);
    let mut vm_state = vm_state.build(entry);

    // allocate stack
    const STACK_SIZE: u64 = 1024 * 1024 * 4;
    vm_state.gpr_mut(stack_reg).set(576460752303390720);
    vm_state.mmu().mmap(
        576460752303390720 - STACK_SIZE,
        STACK_SIZE,
        true,
        true,
        false,
    );

    unsafe {
        engine.run(&mut vm_state).unwrap();
    }
}

fn get_access_info(sh_flags: u64) -> (bool, bool) {
    let writable = (sh_flags & 0x1) != 0;
    let executable = (sh_flags & 0x4) != 0;

    (writable, executable)
}
