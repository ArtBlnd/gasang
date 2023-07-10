use core::{
    ir::{IrType, IrValue},
    Architecture, ArchitectureCompat, Register,
};

use arch_desc::aarch64::AArch64Architecture;
use device::{devices::Memory, IoDevice};
use elf::{endian::AnyEndian, ElfBytes};

use crate::{codegen::Context, SoftMmu};

use super::Abi;

pub struct AArch64UnknownLinux {}
impl ArchitectureCompat<AArch64Architecture> for AArch64UnknownLinux {}

impl Abi for AArch64UnknownLinux {
    fn new() -> Self {
        Self {}
    }

    fn on_initialize<C: Context>(&mut self, binary: &[u8], ctx: &mut C, mmu: &mut SoftMmu) {
        let elf =
            ElfBytes::<AnyEndian>::minimal_parse(&binary).expect("Failed to parse ELF binary");
        for seg in elf.segments().unwrap() {
            let addr = seg.p_paddr;
            let size = seg.p_memsz;
            let data = elf.segment_data(&seg).expect("Bad segment data");

            mmu.map(addr, size, Memory::allocate(size as usize));
            unsafe {
                mmu.write_all_at(addr, &data);
            }
        }

        for sec in elf.section_headers().unwrap() {
            let addr = sec.sh_addr;
            let size = sec.sh_size;

            mmu.map(addr, size, Memory::allocate(size as usize));
        }

        ctx.set(
            IrValue::Register(IrType::B64, AArch64Architecture::get_pc_register().raw()),
            elf.ehdr.e_entry,
        );
    }

    fn on_exception<C: Context>(&self, exception: u64, ctx: &C, mmu: &SoftMmu) {
        todo!()
    }

    fn on_interrupt<C: Context>(&self, interrupt: u64, ctx: &C, mmu: &SoftMmu) {
        todo!()
    }

    fn on_system_call<C: Context>(&self, system_call: u64, ctx: &C, mmu: &SoftMmu) {
        todo!()
    }

    fn on_irq<C: Context>(&self, id: usize, level: usize, ctx: &C, mmu: &SoftMmu) {
        // Do nothing, we are in the userland.
    }
}
