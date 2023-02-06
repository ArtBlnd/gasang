use crate::compiler::Compiler;
use crate::error::CompileError;
use crate::ir::*;
use crate::register::RegId;

use machineinstr::aarch64::AArch64Instr;
use utility::extract_bits16;

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
            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let ir = gen_ip_relative(imm);
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
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let offset_temp =
                    Operand::Immediate(if !post_index { offset } else { 0 } as u64, Type::I16);
                let offset_temp = Operand::Ir(Box::new(Ir::SextCast(Type::I64, offset_temp)));

                let ir = Ir::Load(
                    Type::U64,
                    Operand::Ir(Box::new(Ir::Add(
                        Type::U64,
                        Operand::Register(src, Type::U64),
                        offset_temp.clone(),
                    ))),
                );
                let ds = BlockDestination::GprRegister(dst);

                block.append(ir, ds);

                if wback {
                    let offset = Operand::Ir(Box::new(Ir::SextCast(
                        Type::I64,
                        Operand::Immediate(offset as u64, Type::I16),
                    )));

                    let ir = Ir::Add(Type::U64, Operand::Register(src, Type::U64), offset);
                    let ds = BlockDestination::GprRegister(src);

                    block.append(ir, ds);
                }
            }

            // Arithmetic instructions
            AArch64Instr::AddImm32(operand) | AArch64Instr::AddImm64(operand) => {
                let rd = self.gpr(operand.rd);
                let rn = if operand.rn == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let imm = match operand.sh {
                    0b00 => operand.imm12 as u64,
                    0b01 => (operand.imm12 as u64) << 12,
                    _ => unreachable!(),
                };

                let ir = Ir::Add(
                    Type::U64,
                    Operand::Register(rn, Type::U64),
                    Operand::Immediate(imm, Type::U64),
                );
                let ds = BlockDestination::GprRegister(rd);

                block.append(ir, ds);
            }

            AArch64Instr::SubImm32(operand) | AArch64Instr::SubImm64(operand) => {
                let rd = self.gpr(operand.rd);
                let rn = if operand.rn == 31 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                let imm = match operand.sh {
                    0b00 => operand.imm12 as u64,
                    0b01 => (operand.imm12 as u64) << 12,
                    _ => unreachable!(),
                };

                let ir = Ir::Sub(
                    Type::U64,
                    Operand::Register(rn, Type::U64),
                    Operand::Immediate(imm, Type::U64),
                );
                let ds = BlockDestination::GprRegister(rd);

                block.append(ir, ds);
            }

            AArch64Instr::SubsImm32(operand) | AArch64Instr::SubsImm64(operand) => {
                let imm = match operand.sh {
                    0b00 => operand.imm12 as u64,
                    0b01 => (operand.imm12 as u64) << 12,
                    _ => unreachable!(),
                };

                let rn = if operand.rn == 0b11111 {
                    self.stack_reg
                } else {
                    self.gpr(operand.rn)
                };

                // If rd is 31, its alias is CMP(immediate).
                let ds = if operand.rd == 0b11111 {
                    BlockDestination::None
                } else {
                    BlockDestination::GprRegister(self.gpr(operand.rd))
                };

                let ir = Ir::Subc(
                    Type::U64,
                    Operand::Register(rn, Type::U64),
                    Operand::Immediate(imm, Type::U64),
                );

                block.append(ir, ds);
            }

            // Branch instructions
            AArch64Instr::BlImm(operand) => {
                let ir = Ir::Add(Type::U64, Operand::Ip, Operand::Immediate(4, Type::U64));
                let ds = BlockDestination::GprRegister(self.gpr(30));

                block.append(ir, ds);

                let imm = sign_extend(operand.imm26 as i64, 28);

                let ir = gen_ip_relative(imm);
                let ds = BlockDestination::Eip;

                block.append(ir, ds);
            }
            AArch64Instr::BImm(operand) => {
                let imm = sign_extend(operand.imm26 as i64, 28);

                let ir = gen_ip_relative(imm);
                let ds = BlockDestination::Eip;

                block.append(ir, ds);
            }

            // Compare Instructions
            AArch64Instr::CcmpImmVar32(operand) | AArch64Instr::CcmpImmVar64(operand) => {}

            // Interrupt Instructions
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

            // Speical instructions
            AArch64Instr::Nop => {
                let ir = Ir::Nop;
                let ds = BlockDestination::None;
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

const fn gen_ip_relative(offset: i64) -> Ir {
    if offset > 0 {
        Ir::Add(
            Type::U64,
            Operand::Ip,
            Operand::Immediate(offset as u64, Type::U64),
        )
    } else {
        Ir::Sub(
            Type::U64,
            Operand::Ip,
            Operand::Immediate((-offset) as u64, Type::U64),
        )
    }
}

const fn condition_holds(cond: u8) -> Ir {
    let masked_cond = (cond & 0b1110) >> 1;

    match masked_cond {
        0b000 => todo!(),
        0b001 => todo!(),
        0b010 => todo!(),
        0b011 => todo!(),
        0b100 => todo!(),
        0b101 => todo!(),
        0b110 => todo!(),
        0b111 => todo!(),
        _ => unreachable!(),
    };
}
