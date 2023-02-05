use crate::compiler::Compiler;
use crate::error::CompileError;
use crate::ir::*;
use crate::register::RegId;

use machineinstr::aarch64::AArch64Instr;

pub struct AArch64Compiler {
    gpr_registers: [RegId; 31],
    fpr_registers: [RegId; 31],
}

impl AArch64Compiler {
    pub fn new(gpr_registers: [RegId; 31], fpr_registers: [RegId; 31]) -> Self {
        Self {
            gpr_registers,
            fpr_registers,
        }
    }
    pub fn gpr(&self, index: usize) -> RegId {
        self.gpr_registers[index]
    }

    pub fn fpr(&self, index: usize) -> RegId {
        self.fpr_registers[index]
    }
}

impl Compiler for AArch64Compiler {
    type Item = AArch64Instr;

    fn compile(&self, item: Self::Item) -> Result<Block, CompileError> {
        match item {
            AArch64Instr::MovzVar32(operand) | AArch64Instr::MovzVar64(operand) => {
                let pos = operand.hw << 4;

                let ir = Ir::Value(Operand::Immediate((operand.imm16 as u64) << pos, Type::U64));
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd as usize));
                Ok(Block::new(ir, ds, 4))
            }
            AArch64Instr::Nop => {
                let ir = Ir::Nop;
                let ds = BlockDestination::None;
                Ok(Block::new(ir, ds, 4))
            }
            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let ir = Ir::Add(Type::U64, Operand::Eip, Operand::Immediate(imm as u64, Type::I64));
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd as usize));
                Ok(Block::new(ir, ds, 4))
            }
            _ => unimplemented!("unimplemented instruction: {:?}", item),
        }
    }
}

const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}