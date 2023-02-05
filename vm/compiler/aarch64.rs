use crate::compiler::Compiler;
use crate::error::CompileError;
use crate::ir::*;
use crate::register::RegId;

use machineinstr::aarch64::AArch64Instr;
use utility::extract_bits16;

use smallvec::SmallVec;

pub struct AArch64Compiler {
    gpr_registers: [RegId; 31],
    fpr_registers: [RegId; 31],
    stack_reg: RegId,
}

impl AArch64Compiler {
    pub fn new(gpr_registers: [RegId; 31], fpr_registers: [RegId; 31], stack_reg: RegId) -> Self {
        Self {
            gpr_registers,
            fpr_registers,
            stack_reg,
        }
    }
    pub fn gpr(&self, index: u8) -> RegId {
        self.gpr_registers[index as usize]
    }

    pub fn fpr(&self, index: u8) -> RegId {
        self.fpr_registers[index as usize]
    }
}

impl Compiler for AArch64Compiler {
    type Item = AArch64Instr;

    fn compile(&self, item: Self::Item) -> Result<IrBlock, CompileError> {
        let mut block = IrBlock::new(4);

        match item {
            AArch64Instr::MovzVar32(operand) | AArch64Instr::MovzVar64(operand) => {
                let pos = operand.hw << 4;

                let ir = Ir::Value(Operand::Immediate((operand.imm16 as u64) << pos, Type::U64));
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd));
                block.append(ir, ds);
            }
            AArch64Instr::Nop => {
                let ir = Ir::Nop;
                let ds = BlockDestination::None;
                block.append(ir, ds);
            }
            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let ir = Ir::Add(
                    Type::U64,
                    Operand::Eip,
                    Operand::Immediate(imm as u64, Type::I64),
                );
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd));
                block.append(ir, ds);
            }
            AArch64Instr::OrrShiftedReg32(operand) | AArch64Instr::OrrShiftedReg64(operand) => {
                let rm = self.gpr(operand.rm);
                let rd = self.gpr(operand.rd);

                if operand.imm6 == 0 && operand.shift == 0 && operand.rn == 0b11111 {
                    let ir = Ir::Value(Operand::Register(rm, Type::U64));
                    let ds = BlockDestination::GprRegister(rd);

                    block.append(ir, ds);
                } else {
                    let rn = self.gpr(operand.rn);

                    todo!()
                }
            }

            AArch64Instr::LdrImm64(operand) => {
                let (mut wback, post_index, _scale, offset) = decode_operand_for_ld_st_imm(operand);

                if wback && operand.rn == operand.rt && operand.rn != 31 {
                    wback = false;
                }

                let dst = self.gpr(operand.rt);
                let src = if operand.rn == 31 {
                    // If rn is 31, we use stack register instead of gpr registers.
                    Operand::Register(self.stack_reg, Type::U64)
                } else {
                    Operand::Register(self.gpr(operand.rn), Type::U64)
                };

                let offset_temp =
                    Operand::Immediate(if !post_index { offset } else { 0 } as u64, Type::I16);
                let offset_temp = Operand::Ir(Box::new(Ir::SextCast(Type::I64, offset_temp)));

                let ir = Ir::Load(
                    Type::U64,
                    Operand::Ir(Box::new(Ir::Add(Type::U64, src, offset_temp))),
                );
                let ds = BlockDestination::GprRegister(dst);

                block.append(ir, ds);
            }

            AArch64Instr::Svc(operand) => {
                let ir = Ir::Value(Operand::Immediate(operand.imm16 as u64, Type::U16));
                let ds = BlockDestination::SystemCall;

                block.append(ir, ds);
            }

            AArch64Instr::Brk(operand) => {
                let ir = Ir::Value(Operand::Immediate(operand.imm16 as u64, Type::U16));
                let ds = BlockDestination::Exit;

                block.append(ir, ds);
            }
            _ => unimplemented!("unimplemented instruction: {:?}", item),
        }

        Ok(block)
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

const fn decode_operand_for_ld_st_imm(
    operand: machineinstr::aarch64::SizeImm12RnRt,
) -> (bool, bool, u8, i16) {
    if extract_bits16(11..12, operand.imm12) == 0b0 {
        let imm9 = extract_bits16(2..11, operand.imm12) as i64;
        let post = extract_bits16(0..2, operand.imm12) == 0b01;

        (true, post, operand.size, sign_extend(imm9, 9) as i16)
    } else {
        //Unsigned offset
        (
            false,
            false,
            operand.size,
            (operand.imm12 << operand.size) as i16,
        )
    }
}
