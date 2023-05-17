use machineinstr::aarch64::AArch64InstrParserRule;
use machineinstr::MachineInstrParserRule;

use core::board::Board;
use core::codegen::flag_policy::AArch64FlagPolicy;
use core::codegen::rustjit::InterpretCodegen;
use core::codegen::Codegen;
use core::compiler::aarch64::AArch64Compiler;
use core::compiler::Compiler;
use core::debug::aarch64::AArch64;
use core::softmmu::BasicPage;
use core::softmmu::Mmu;
use core::Cpu;
use core::debug::*;

use std::convert::Infallible;
use std::path::PathBuf;
use std::net::{TcpListener, TcpStream};

use gdbstub::conn::ConnectionExt;
use gdbstub::stub::{DisconnectReason, GdbStub, GdbStubError};
type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

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

    let config = Configuration {
        ram_size: 2 * 1024 * 1024,
    };

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
    mmu.mmap(
        addr_flash,
        size_flash,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap(); // flash is read-only
    mmu.write(addr_flash, &image).unwrap();

    let addr_lowmem_peripherals = 0x0800_0000u64;
    let size_lowmem_peripherals = 0x3800_0000u64;
    mmu.mmap(
        addr_lowmem_peripherals,
        size_lowmem_peripherals,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap();

    let addr_ram = 0x4000_0000u64;
    let size_ram = config.ram_size;
    mmu.mmap(
        addr_ram,
        size_ram,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap();
    {
        let dtb = std::fs::read("../binaries/virt-dtb.dtb").unwrap();
        mmu.write(addr_ram, &dtb).unwrap();
    }

    let board = Board::new(comp, cgen, mci_parser, (), mmu, cpu);
    board.run().unwrap()
}


unsafe fn init_and_debug<C, G, P>(
    config: Configuration,
    cpu: Cpu,
    mmu: Mmu,
    comp: C,
    cgen: G,
    mci_parser: P,
    image: Vec<u8>,
) -> DynResult<()>
where
    C: Compiler,
    P: MachineInstrParserRule<MachineInstr = C::Item>,
    G: Codegen,
{
    // https://qemu.readthedocs.io/en/latest/system/arm/virt.html

    let addr_flash = 0x0000_0000u64;
    let size_flash = 0x0800_0000u64;
    mmu.mmap(
        addr_flash,
        size_flash,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap(); // flash is read-only
    mmu.write(addr_flash, &image).unwrap();

    let addr_lowmem_peripherals = 0x0800_0000u64;
    let size_lowmem_peripherals = 0x3800_0000u64;
    mmu.mmap(
        addr_lowmem_peripherals,
        size_lowmem_peripherals,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap();

    let addr_ram = 0x4000_0000u64;
    let size_ram = config.ram_size;
    mmu.mmap(
        addr_ram,
        size_ram,
        Box::new(BasicPage::new(true, true, true)),
    )
    .unwrap();
    {
        let dtb = std::fs::read("../binaries/virt-dtb.dtb").unwrap();
        mmu.write(addr_ram, &dtb).unwrap();
    }

    let mut board = Board::new(comp, cgen, mci_parser, AArch64, mmu, cpu);

    
    let connection: Box<dyn ConnectionExt<Error = std::io::Error>> = Box::new(wait_for_tcp(9001).unwrap());

    let gdb = GdbStub::new(connection);

    match gdb.run_blocking::<GdbEventLoop<_, _, _, _>>(&mut board) {
        Ok(disconnect_reason) => match disconnect_reason {
            DisconnectReason::Disconnect => {
                println!("GDB client has disconnected.");
            }
            DisconnectReason::TargetExited(code) => {
                println!("Target exited with code {}!", code)
            }
            DisconnectReason::TargetTerminated(sig) => {
                println!("Target terminated with signal {}!", sig)
            }
            DisconnectReason::Kill => println!("GDB sent a kill command!"),
        },
        Err(GdbStubError::TargetError(e)) => {
            println!("target encountered a fatal error: {}", e)
        }
        Err(e) => {
            println!("gdbstub encountered a fatal error: {}", e)
        }
    }

    Ok(())
}

fn wait_for_tcp(port: u16) -> DynResult<TcpStream> {
    let sockaddr = format!("127.0.0.1:{}", port);
    eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

    let sock = TcpListener::bind(sockaddr)?;
    let (stream, addr) = sock.accept()?;
    eprintln!("Debugger connected from {}", addr);

    Ok(stream)
}