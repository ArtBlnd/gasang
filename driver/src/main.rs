use machineinstr::aarch64::AArch64InstrParserRule;

use core::codegen::flag_policy::AArch64FlagPolicy;
use core::codegen::rustjit::InterpretCodegen;
use core::compiler::aarch64::AArch64Compiler;
use core::engine::Engine;
use core::Cpu;

use std::path::PathBuf;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let image = std::fs::read(PathBuf::from(filename)).unwrap();

    let mut cpu = Cpu::new(core::cpu::Architecture::AArch64Bin, &image);
    let compiler = AArch64Compiler::new(cpu.get_register_info());

    let engine = Engine::new(
        compiler,
        AArch64InstrParserRule,
        InterpretCodegen::new(AArch64FlagPolicy),
    );

    unsafe {
        engine.run(&mut cpu).unwrap();
    }
}
