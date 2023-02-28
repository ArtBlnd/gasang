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

struct Configuration {
    ram_size: u64,
}

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

    let config = Configuration { ram_size: 2 * 1024 * 1024 };

    // initialize image into mmu.
    let image = std::fs::read(PathBuf::from(filename)).unwrap();
    unsafe { init_and_run(config, cpu, mmu, comp, cgen, parser_rule, image) };
}

unsafe fn init_and_run<C, G, P>(
    config: Configuration,
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
    // https://qemu.readthedocs.io/en/latest/system/arm/virt.html

    let addr_flash = 0x0000_0000u64;
    let size_flash = 0x0800_0000u64;
    mmu.mmap(addr_flash, size_flash, true, true, true); // flash is read-only
    mmu.frame(addr_flash).write(&image).unwrap();

    let addr_lowmem_peripherals = 0x0800_0000u64;
    let size_lowmem_peripherals = 0x3800_0000u64;
    mmu.mmap(addr_lowmem_peripherals, size_lowmem_peripherals, true, true, true);

    let addr_ram = 0x4000_0000u64;
    let size_ram = config.ram_size;
    mmu.mmap(addr_ram, size_ram, true, true, true);
    {
        let dtb = std::fs::read("binaries/virt-dtb.dtb").unwrap();
        mmu.frame(addr_ram).write(&dtb).unwrap();
    }

    let board = Board::new(comp, cgen, mci_parser, mmu, cpu);
    board.run().unwrap()
}
