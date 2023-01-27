mod compiler;
pub use compiler::*;
mod engine;
pub use engine::*;

use crate::jump_table::{JumpId, JumpTable};
use crate::register::RegId;
use crate::Vm;
use crate::VmContext;

use machineinstr::aarch64::*;
use machineinstr::utils::BitReader;
use machineinstr::MachineInstParser;

use crate::instruction::*;

use std::collections::HashMap;

pub fn compile_text_segment(
    addr: u64,
    data: &[u8],
    compiler: &AArch64Compiler,
    vm_ctx: &mut VmContext,
) {
    // Compile instructions into VMIR and insert it to context.
    let mut ipr = addr as u64;
    let parser =
        MachineInstParser::new(BitReader::new(data.iter().cloned()), AArch64InstrParserRule);

    let mut prev_size = 0u8;
    for native_instr in parser {
        let instr = compiler.compile_instr(native_instr.size, prev_size, native_instr.op);
        // vm_ctx.vm_instr.extend_from_slice(&instr);

        ipr += native_instr.size as u64;
        prev_size = instr.len() as u8;
    }
}
