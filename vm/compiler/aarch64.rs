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

                let ir = Ir::Value(Operand::imm(Type::U64, (operand.imm16 as u64) << pos));
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd));
                block.append(ir, ds);
            }
            AArch64Instr::Adr(operand) => {
                let imm = sign_extend((operand.immhi as i64) << 2 | (operand.immlo as i64), 20);

                let ir = gen_ip_relative(imm);
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd));
                block.append(ir, ds);
            }
            AArch64Instr::OrrShiftedReg64(operand) => {
                let rm = self.gpr(operand.rm);
                let rd = self.gpr(operand.rd);

                if operand.imm6 == 0 && operand.shift == 0 && operand.rn == 0b11111 {
                    let ir = Ir::Value(Operand::reg(Type::U64, rm));
                    let ds = BlockDestination::GprRegister(rd);

                    block.append(ir, ds);
                } else {
                    // let rn = self.gpr(operand.rn);

                    todo!()
                }
            }

            AArch64Instr::LdrImm64(operand) => {
                let (mut wback, post_index, _scale, mut offset) =
                    decode_operand_for_ld_st_imm(operand);
                if post_index {
                    offset = 0;
                }

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

                let ir = Ir::Load(
                    Type::U64,
                    Operand::ir(Ir::Add(
                        Type::U64,
                        Operand::reg(Type::U64, src),
                        Operand::imm(Type::U64, offset as u64),
                    )),
                );
                let ds = BlockDestination::GprRegister(dst);

                block.append(ir, ds);

                if wback {
                    let offset = Operand::ir(Ir::SextCast(
                        Type::I64,
                        Operand::imm(Type::I16, offset as u64),
                    ));

                    let ir = Ir::Add(Type::U64, Operand::reg(Type::U64, src), offset);
                    let ds = BlockDestination::GprRegister(src);

                    block.append(ir, ds);
                }
            }

            // Arithmetic instructions
            AArch64Instr::AddImm64(operand) => {
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
                    Operand::reg(Type::U64, rn),
                    Operand::imm(Type::U64, imm),
                );
                let ds = BlockDestination::GprRegister(rd);

                block.append(ir, ds);
            }

            AArch64Instr::AddShiftedReg64(operand) => {
                let rn = self.gpr(operand.rn);
                let rm = self.gpr(operand.rm);
                let rd = self.gpr(operand.rd);

                let sh = shift_reg(
                    rm,
                    decode_shift(operand.shift),
                    operand.imm6 as u64,
                    Type::U64,
                );
                let ir = Ir::Add(
                    Type::U64,
                    Operand::reg(Type::U64, rn),
                    Operand::Ir(Box::new(sh)),
                );

                let ds = BlockDestination::GprRegister(rd);
                block.append(ir, ds);
            }

            AArch64Instr::SubImm64(operand) => {
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
                    Operand::reg(Type::U64, rn),
                    Operand::imm(Type::U64, imm),
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
                    Operand::reg(Type::U64, rn),
                    Operand::imm(Type::U64, imm),
                );

                block.append(ir, ds);
            }

            // bitwise isntructions
            AArch64Instr::AndsImm64(operand) => {
                let (imm, _) = decode_bit_masks(operand.n, operand.imms, operand.immr, true, 64);
                let rn = Operand::reg(Type::U64, self.gpr(operand.rn));

                let ir = Ir::And(Type::U64, rn, Operand::imm(Type::U64, imm));
                let ds = BlockDestination::GprRegister(self.gpr(operand.rd));
                block.append(ir.clone(), ds);

                let ds = BlockDestination::None;
                let ir = Ir::Addc(Type::U64, Operand::ir(ir), Operand::imm(Type::U64, 0)); // Only for flag setting
                block.append(ir, ds);
            }

            // Branch instructions
            AArch64Instr::BlImm(operand) => {
                let ir = Ir::Add(Type::U64, Operand::Ip, Operand::imm(Type::U64, 4));
                let ds = BlockDestination::GprRegister(self.gpr(30));

                block.append(ir, ds);

                let imm = sign_extend((operand.imm26 << 2) as i64, 28);

                let ir = gen_ip_relative(imm);
                let ds = BlockDestination::Ip;

                block.append(ir, ds);
            }
            AArch64Instr::BImm(operand) => {
                let imm = sign_extend((operand.imm26 << 2) as i64, 28);

                let ir = gen_ip_relative(imm);
                let ds = BlockDestination::Ip;

                block.append(ir, ds);
            }
            AArch64Instr::Br(operand) => {
                let ir = Ir::Value(Operand::reg(Type::U64, self.gpr(operand.rn)));
                let ds = BlockDestination::Ip;

                block.append(ir, ds);
            }
            AArch64Instr::BCond(operand) => {
                let offset = operand.imm19 << 2;
                let ir = Ir::If(Type::U64, condition_holds(operand.cond), Operand::ir(gen_ip_relative(offset as i64)), Operand::ir(gen_ip_relative(4)));
                let ds = BlockDestination::Ip;

                block.append(ir, ds);
            }

            // Conditional Instructions
            AArch64Instr::CcmpImmVar32(operand) => {
                let rn = self.gpr(operand.rn);

                let subc = Operand::void_ir(Ir::Subc(
                    Type::U32,
                    Operand::reg(Type::U32, rn),
                    Operand::imm(Type::U32, operand.imm5 as u64),
                ));

                let ir = Ir::If(
                    Type::Void,
                    condition_holds(operand.cond),
                    subc,
                    Operand::Ir(Box::new(Ir::Nop)),
                );
                let ds = BlockDestination::None;

                block.append(ir, ds);
            }

            AArch64Instr::Csel32(operand) => {
                let rn = if operand.rn == 31 {
                    Operand::imm(Type::U32, 0)
                } else {
                    Operand::reg(Type::U32, self.gpr(operand.rn))
                };

                let rm = if operand.rm == 31 {
                    Operand::imm(Type::U32, 0)
                } else {
                    Operand::reg(Type::U32, self.gpr(operand.rm))
                };
                let rd = self.gpr(operand.rd);

                let ir = Ir::If(Type::U32, condition_holds(operand.cond), rn, rm);
                let ds = BlockDestination::GprRegister(rd);

                block.append(ir, ds);
            }

            // Interrupt Instructions
            AArch64Instr::Svc(operand) => {
                let ir = Ir::Value(Operand::imm(Type::U16, operand.imm16 as u64));
                let ds = BlockDestination::SystemCall;

                block.append(ir, ds);
            }

            AArch64Instr::Brk(operand) => {
                let ir = Ir::Value(Operand::imm(Type::U16, operand.imm16 as u64));
                let ds = BlockDestination::Exit;

                block.append(ir, ds);
            }

            // Speical instructions
            AArch64Instr::Nop | AArch64Instr::Wfi => {
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

enum ShiftType {
    LSL, // Logical shift left
    LSR, // Logical shift right
    ASR, // Arithmetic shift right
    ROR, // Rotate right
}

const fn decode_shift(shift: u8) -> ShiftType {
    match shift {
        0b00 => ShiftType::LSL,
        0b01 => ShiftType::LSR,
        0b10 => ShiftType::ASR,
        0b11 => ShiftType::ROR,
        _ => unreachable!(),
    }
}

const fn decode_operand_for_ld_st_imm(
    operand: machineinstr::aarch64::SizeImm12RnRt,
) -> (bool, bool, u8, i64) {
    if extract_bits16(11..12, operand.imm12) == 0b0 {
        let imm9 = extract_bits16(2..11, operand.imm12) as i64;
        let post = extract_bits16(0..2, operand.imm12) == 0b01;

        (true, post, operand.size, sign_extend(imm9, 9))
    } else {
        //Unsigned offset
        (
            false,
            false,
            operand.size,
            (operand.imm12 << operand.size) as i64,
        )
    }
}

const fn gen_ip_relative(offset: i64) -> Ir {
    if offset > 0 {
        Ir::Add(
            Type::U64,
            Operand::Ip,
            Operand::imm(Type::U64, offset as u64),
        )
    } else {
        Ir::Sub(
            Type::U64,
            Operand::Ip,
            Operand::imm(Type::U64, (-offset) as u64),
        )
    }
}

fn condition_holds(cond: u8) -> Operand {
    let masked_cond = (cond & 0b1110) >> 1;
    let cond0 = cond & 1;

    let result = match masked_cond {
        0b000 => cmp_eq_op_imm(zero_flag(), 1),
        0b001 => cmp_eq_op_imm(carry_flag(), 1),
        0b010 => cmp_eq_op_imm(negative_flag(), 1),
        0b011 => cmp_eq_op_imm(overflow_flag(), 1),
        0b100 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            cmp_eq_op_imm(carry_flag(), 1),
            cmp_eq_op_imm(zero_flag(), 0),
        ))),
        0b101 => Operand::Ir(Box::new(Ir::CmpEq(negative_flag(), overflow_flag()))),
        0b110 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            Operand::Ir(Box::new(Ir::CmpEq(negative_flag(), overflow_flag()))),
            cmp_eq_op_imm(zero_flag(), 0),
        ))),
        0b111 => Operand::imm(Type::Bool, 0b1u64),
        _ => unreachable!(),
    };

    if cond0 == 1 && cond != 0b1111 {
        Operand::Ir(Box::new(Ir::Not(Type::Bool, result)))
    } else {
        result
    }
}

fn cmp_eq_op_imm(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

fn cmp_ne_op_imm(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

fn negative_flag() -> Operand {
    let nf = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x8000_0000_0000_0000),
    )));

    Operand::Ir(Box::new(Ir::LShr(
        Type::U64,
        nf,
        Operand::imm(Type::U64, 63),
    )))
}

fn zero_flag() -> Operand {
    let zf = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x4000_0000_0000_0000),
    )));

    Operand::Ir(Box::new(Ir::LShr(
        Type::U64,
        zf,
        Operand::imm(Type::U64, 62),
    )))
}

fn carry_flag() -> Operand {
    let cf = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x2000_0000_0000_0000),
    )));

    Operand::Ir(Box::new(Ir::LShr(
        Type::U64,
        cf,
        Operand::imm(Type::U64, 61),
    )))
}

fn overflow_flag() -> Operand {
    let of = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x1000_0000_0000_0000),
    )));

    Operand::Ir(Box::new(Ir::LShr(
        Type::U64,
        of,
        Operand::imm(Type::U64, 60),
    )))
}

fn shift_reg(reg: RegId, shift_type: ShiftType, amount: u64, t: Type) -> Ir {
    let reg = Operand::reg(t, reg);
    let amount = Operand::imm(t, amount);

    match shift_type {
        ShiftType::LSL => Ir::LShl(t, reg, amount),
        ShiftType::LSR => Ir::LShr(t, reg, amount),
        ShiftType::ASR => Ir::AShr(t, reg, amount),
        ShiftType::ROR => Ir::Rotr(t, reg, amount),
    }
}

const fn highest_set_bit(x: u64) -> u64 {
    63 - x.leading_zeros() as u64
}

const fn ones(n: u64) -> u64 {
    replicate(1, n, 1)
}

const fn ror(x: u64, shift: u64, size: u64) -> u64 {
    let m = shift % size;
    x >> m | ((x << (size - m)) & ones(shift))
}

const fn replicate(x: u64, n: u64, size: u64) -> u64 {
    let mut result = 0b0;
    let mut i = n;

    while i > 0 {
        result |= x;
        result <<= size;
        i -= 1;
    }

    result
}

const fn decode_bit_masks(immn: u8, imms: u8, immr: u8, immediate: bool, m: u8) -> (u64, u64) {
    let len = highest_set_bit(((immn << 6) as u16 | extract_bits16(0..6, !imms as u16)) as u64);
    assert!(len >= 1, "UNDEFINED");
    assert!(m >= (1 << len), "UNDEFINED");

    let levels = ones(len);

    assert!(
        !(immediate && (imms as u64 & levels) == levels),
        "UNDEFINED"
    );

    let s = imms & levels as u8;
    let r = immr & levels as u8;
    let diff = s - r;

    let esize = 1 << len;
    let d = extract_bits16(0..len as usize, diff as u16);

    let welem = ones(s as u64 + 1);
    let telem = ones(d as u64 + 1);

    let wmask = replicate(ror(welem, r as u64, esize), m as u64, esize);
    let tmask = replicate(telem, m as u64, esize);

    (wmask, tmask)
}
