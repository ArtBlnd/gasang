use crate::instruction::VmIr;
use crate::register::RegId;
use crate::{Interrupt, Vm, VmContext};

use crate::instruction::instructions::*;

#[inline]
pub unsafe fn execute_instr(ir: VmIr, vm: &mut Vm, vm_ctx: &VmContext) -> Result<(), Interrupt> {
    let mut offset = 2;
    while let Some(opcode) = ir.opcode(&mut offset) {
        execute_instr_inner(&mut offset, opcode, &ir, vm, vm_ctx)?;
    }

    Ok(())
}

#[inline]
pub unsafe fn execute_instr_inner(
    offset: &mut usize,
    opcode: u8,
    ir: &VmIr,
    vm: &mut Vm,
    vm_ctx: &VmContext,
) -> Result<(), Interrupt> {
    match opcode {
        IROP_UADD_REG3 => {
            todo!()
        }

        _ => unimplemented!("unknown opcode: "),
    }

    Ok(())
}
