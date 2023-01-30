use crate::instruction::VmIr;
use crate::instruction::*;
use crate::register::RegId;
use crate::{Interrupt, Vm, VmContext};

use crate::instruction::instructions::*;

#[inline]
pub unsafe fn execute_instr(ir: VmIr, vm: &mut Vm, vm_ctx: &VmContext) -> Result<(), Interrupt> {
    let mut executor = VmIrExecutor {
        vm,
        vm_ctx,
        interrupt: None,
    };

    ir.visit(&mut executor);

    if let Some(int) = executor.interrupt {
        return Err(int);
    }

    Ok(())
}

struct VmIrExecutor<'v> {
    vm: &'v mut Vm,
    vm_ctx: &'v VmContext,

    interrupt: Option<Interrupt>,
}

impl<'v> InstrVisitor for VmIrExecutor<'v> {
    fn visit_no_operand(&mut self, op: u8) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1(&mut self, op: u8, operand: Reg1) {
        match op {
            MOV_IPR_REG => {
                let ipr = self.vm.ipr();
                self.vm.gpr(operand.op1).set(ipr);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2(&mut self, op: u8, operand: Reg2) {
        match op {
            MOV_REG2 => {
                let src = self.vm.gpr(operand.op1).get();
                self.vm.gpr(operand.op2).set(src);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg3(&mut self, op: u8, operand: Reg3) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1imm8(&mut self, op: u8, operand: Reg1Imm8) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1imm16(&mut self, op: u8, operand: Reg1Imm16) {
        match op {
            MOV_REG1IMM16 => self.vm.gpr(operand.op1).set(operand.imm16 as u64),
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1imm32(&mut self, op: u8, operand: Reg1Imm32) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg1imm64(&mut self, op: u8, operand: Reg1Imm64) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2imm8(&mut self, op: u8, operand: Reg2Imm8) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2imm16(&mut self, op: u8, operand: Reg2Imm16) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2imm32(&mut self, op: u8, operand: Reg2Imm32) {
        match op {
            IADD_REG2IMM32 => {
                let src = self.vm.gpr(operand.op1).get() as i32;
                let dst = self.vm.gpr(operand.op2);
                let (result, cf) = src.overflowing_add(operand.i32());
                dst.set(result as u64);
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_reg2imm64(&mut self, op: u8, operand: Reg2Imm64) {
        match op {
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }

    fn visit_u16(&mut self, op: u8, operand: Imm16) {
        match op {
            SVC_IMM16 => {
                self.interrupt = Some(Interrupt::SystemCall(operand.imm16 as usize));
            }
            _ => unimplemented!("opcode: {:02x}", op),
        }
    }
}
