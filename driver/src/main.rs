use machineinstr::aarch64::AArch64InstrParserRule;

use vm::codegen::flag_policy::AArch64FlagPolicy;
use vm::codegen::interpret::InterpretCodegen;
use vm::compiler::aarch64::AArch64Compiler;
use vm::engine::Engine;
use vm::Cpu;

use vm::interrupt::AArch64UnixInterruptModel;

use std::path::PathBuf;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let image = std::fs::read(PathBuf::from(filename)).unwrap();

    let mut cpu = Cpu::new(vm::cpu::Architecture::AArch64Elf, &image);
    let compiler = AArch64Compiler::new(cpu.get_register_info());

    let mut engine = Engine::new(
        compiler,
        AArch64InstrParserRule,
        AArch64UnixInterruptModel,
        InterpretCodegen::new(AArch64FlagPolicy),
    );

    unsafe {
        engine.run(&mut cpu).unwrap();
    }
}
