use crate::register::RegId;
use crate::mmu::MemoryFrame;


pub trait Block {
    fn compile(&self);
}