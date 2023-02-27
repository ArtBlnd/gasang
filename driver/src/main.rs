use machineinstr::aarch64::AArch64InstrParserRule;
use machineinstr::MachineInstrParserRule;

use core::board::Board;
use core::codegen::flag_policy::AArch64FlagPolicy;
use core::codegen::rustjit::InterpretCodegen;
use core::codegen::Codegen;
use core::compiler::aarch64::AArch64Compiler;
use core::compiler::Compiler;
use core::softmmu::Mmu;
use core::Cpu;

use std::convert::Infallible;
use std::path::PathBuf;

fn main() {
    // get file
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    // initialize basic components
    let cpu = Cpu::new(core::cpu::Architecture::AArch64Bin);
    let mmu = Mmu::new();
    let comp = AArch64Compiler::new(cpu.get_register_info());
    let cgen = InterpretCodegen::new(AArch64FlagPolicy);
    let parser_rule = AArch64InstrParserRule;

    // initialize image into mmu.
    let image = std::fs::read(PathBuf::from(filename)).unwrap();
    unsafe { init_image_and_run(cpu, mmu, comp, cgen, parser_rule, image) };
}

pub unsafe fn init_image_and_run<C, G, P>(
    cpu: Cpu,
    mmu: Mmu,
    comp: C,
    cgen: G,
    mci_parser: P,
    image: Vec<u8>,
) -> Infallible
where
    C: Compiler,
    P: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    mmu.mmap(0x0, image.len() as u64, true, true, true);
    unsafe {
        mmu.frame(0x0).write(&image).unwrap();
    }
    let board = Board::new(comp, cgen, mci_parser, mmu, cpu);

    board.run().unwrap()
}
