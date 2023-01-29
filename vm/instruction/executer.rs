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
        IROP_MOV_16CST2REG => {
            let op = ir.reg1u16(offset);
            vm.gpr(op.op1).set(op.imm16 as u64);
        }
        IROP_MOV_REG2REG => {
            let op = ir.reg2(offset);
            let val = vm.gpr(op.op1).get();
            vm.gpr(op.op2).set(val);
        }
        IROP_MOV_IPR2REG => {
            let op = ir.reg1(offset);
            let ipr = vm.ipr;
            vm.gpr(op.op1).set(ipr);
        }
        IROP_IADD_CST32 => {
            let op = ir.reg1i32(offset);
            let reg = vm.gpr(op.op1);
            let val = reg.get() as i64;
            reg.set((val + op.imm32 as i64) as u64);
        }

        IROP_LLEFT_SHIFT_IMM8 => {
            let op = ir.reg2u8(offset);
            let reg = vm.gpr(op.op1);
            let val = reg.get();

            reg.set(val << op.imm8);
        }

        IROP_OR_REG3 => {
            let op = ir.reg3(offset);
            let src = vm.gpr(op.op1).get();
            let val = vm.gpr(op.op3).get();

            vm.gpr(op.op2).set(src | val);
        }

        IROP_SVC => {
            let op = ir.u16(offset);
            return Err(Interrupt::SystemCall(op.imm16 as usize));
        }

        IROP_BRK => {
            let op = ir.u16(offset);
            return Err(Interrupt::DebugBreakpoint(op.imm16 as usize));
        }
        _ => unimplemented!("unknown opcode: {:02x}", opcode),
    }

    Ok(())
}
