use elf::abi::{PF_R, PF_W, PF_X, SHF_EXECINSTR, SHF_WRITE};
use elf::endian::AnyEndian;
use elf::ElfBytes;

use crate::loader::Loader;

pub struct ElfLoader<'d> {
    raw: ElfBytes<'d, AnyEndian>,
}

impl<'d> Loader for ElfLoader<'d> {
    fn load(&self, mmu: &crate::mmu::Mmu) {
        for seg in self.raw.segments().unwrap() {
            let addr = seg.p_paddr;
            let size = seg.p_memsz;
            let data = self.raw.segment_data(&seg).unwrap();

            mmu.mmap(addr, size, true, true, false);
            unsafe {
                mmu.frame(addr).write(data).expect("Failed VM Initialize");
            }

            let readable = seg.p_flags & PF_R != 0;
            let writable = seg.p_flags & PF_W != 0;
            let executable = seg.p_flags & PF_X != 0;

            assert!(seg.p_flags & elf::abi::PF_MASKPROC == 0);

            assert!(readable);

            mmu.mmap(addr, size, readable, writable, executable)
        }

        for sec in self.raw.section_headers().unwrap() {
            let addr = sec.sh_addr;
            let size = sec.sh_size;

            let writable = sec.sh_flags & SHF_WRITE as u64 != 0;
            let executable = sec.sh_flags & SHF_EXECINSTR as u64 != 0;

            mmu.mmap(addr, size, true, writable, executable)
        }
    }

    fn entry(&self) -> u64 {
        self.raw.ehdr.e_entry
    }
}

impl<'d> ElfLoader<'d> {
    pub fn new(file_buf: &'d [u8]) -> Self {
        let file_elf = ElfBytes::<AnyEndian>::minimal_parse(file_buf).unwrap();

        Self { raw: file_elf }
    }
}
