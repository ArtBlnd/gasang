pub mod elf;

use crate::mmu::Mmu;

pub trait Loader {
    fn load(&self, mmu: &Mmu);
    fn entry(&self) -> u64;
}
