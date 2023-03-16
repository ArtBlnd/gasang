mod aarch64;

use crate::{cpu::Architecture, Cpu};
use gdbstub::arch::Registers;
use gdbstub::target::TargetError;

use aarch64::*;

impl Registers for Cpu {
    type ProgramCounter = u64;

    fn pc(&self) -> Self::ProgramCounter {
        self.pc()
    }

    fn gdb_serialize(&self, mut write_byte: impl FnMut(Option<u8>)) {
        let vec = match self.arch() {
            Architecture::AArch64Bin => serialize_aarch64(self).unwrap(),
            _ => unreachable!(),
        };

        for byte in vec {
            write_byte(Some(byte));
        }

        write_byte(None);
    }

    fn gdb_deserialize(&mut self, bytes: &[u8]) -> Result<(), ()> {
        match self.arch() {
            Architecture::AArch64Bin => deserialize_aarch64(bytes, self),
            _ => unreachable!(),
        };

        Ok(())
    }
}
