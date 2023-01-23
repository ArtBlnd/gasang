mod compiler;
pub use compiler::*;

use crate::instr::VmInstrOp;
use crate::register::RegId;
use crate::Vm;

use machineinstr::aarch64::*;
use machineinstr::utils::BitReader;
use machineinstr::MachineInstParser;

pub struct AArch64VM<I> {
    vm_state: Vm,

    parser: MachineInstParser<I, AArch64InstrParserRule>,
    compiler: AArch64Compiler,
}

impl<I> AArch64VM<I>
where
    I: Iterator<Item = u8>,
{
    pub fn from_binary(binary: I) -> Self {
        todo!();
    }
}
