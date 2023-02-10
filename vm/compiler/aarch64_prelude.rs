use std::ops::Range;

use crate::ir::*;
use crate::register::RegId;
use utility::*;

pub const fn sign_extend(value: i64, size: u8) -> i64 {
    let mask = 1 << (size - 1);
    let sign = value & mask;
    if sign != 0 {
        value | !((1 << size) - 1)
    } else {
        value
    }
}

pub enum ShiftType {
    LSL, // Logical shift left
    LSR, // Logical shift right
    ASR, // Arithmetic shift right
    ROR, // Rotate right
}

pub fn decode_shift(shift: u8) -> ShiftType {
    match shift {
        0b00 => ShiftType::LSL,
        0b01 => ShiftType::LSR,
        0b10 => ShiftType::ASR,
        0b11 => ShiftType::ROR,
        _ => unreachable!("failed to decode shift: 0b{shift:0b}"),
    }
}

pub const fn decode_operand_for_ld_st_reg_imm(
    operand: machineinstr::aarch64::SizeImm12RnRt,
) -> (bool, bool, u8, i64) {
    if operand.idxt == 0b00 {
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

pub const fn decode_o_for_ld_st_pair_offset(o: u8) -> (bool, bool) {
    match o {
        0b001 => (true, true),
        0b011 => (true, false),
        0b010 => (false, false),
        _ => unreachable!(),
    }
}

pub const fn gen_ip_relative(offset: i64) -> Ir {
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

pub fn condition_holds(cond: u8) -> Operand {
    let masked_cond = (cond & 0b1110) >> 1;
    let cond0 = cond & 1;

    let result = match masked_cond {
        0b000 => cmp_eq_op_imm64(zero_flag(), 1),
        0b001 => cmp_eq_op_imm64(carry_flag(), 1),
        0b010 => cmp_eq_op_imm64(negative_flag(), 1),
        0b011 => cmp_eq_op_imm64(overflow_flag(), 1),
        0b100 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            cmp_eq_op_imm64(carry_flag(), 1),
            cmp_eq_op_imm64(zero_flag(), 0),
        ))),
        0b101 => Operand::Ir(Box::new(Ir::CmpEq(negative_flag(), overflow_flag()))),
        0b110 => Operand::Ir(Box::new(Ir::And(
            Type::Bool,
            Operand::Ir(Box::new(Ir::CmpEq(negative_flag(), overflow_flag()))),
            cmp_eq_op_imm64(zero_flag(), 0),
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

pub fn cmp_eq_op_imm64(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

pub fn cmp_eq_op_imm32(op: Operand, immediate: u32) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(
        op,
        Operand::imm(Type::U32, immediate as u64),
    )))
}

pub fn cmp_ne_op_imm(op: Operand, immediate: u64) -> Operand {
    Operand::Ir(Box::new(Ir::CmpEq(op, Operand::imm(Type::U64, immediate))))
}

pub fn negative_flag() -> Operand {
    let nf = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x8000_0000_0000_0000),
    )));

    Operand::ir(Ir::LShr(Type::U64, nf, Operand::imm(Type::U64, 63)))
}

pub fn zero_flag() -> Operand {
    let zf = Operand::Ir(Box::new(Ir::And(
        Type::U64,
        Operand::Flag,
        Operand::imm(Type::U64, 0x4000_0000_0000_0000),
    )));

    Operand::ir(Ir::LShr(Type::U64, zf, Operand::imm(Type::U64, 62)))
}

pub fn carry_flag() -> Operand {
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

pub fn overflow_flag() -> Operand {
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

pub fn shift_reg(reg: RegId, shift_type: ShiftType, amount: u64, t: Type) -> Ir {
    let reg = Operand::reg(t, reg);
    let amount = Operand::imm(t, amount);

    match shift_type {
        ShiftType::LSL => Ir::LShl(t, reg, amount),
        ShiftType::LSR => Ir::LShr(t, reg, amount),
        ShiftType::ASR => Ir::AShr(t, reg, amount),
        ShiftType::ROR => Ir::Rotr(t, reg, amount),
    }
}

pub const fn highest_set_bit(x: u64) -> u64 {
    63 - x.leading_zeros() as u64
}

pub const fn ones(n: u64) -> u64 {
    replicate(1, n, 1)
}

pub const fn ror(x: u64, shift: u64, size: u64) -> u64 {
    let m = shift % size;
    x >> m | ((x << (size - m)) & ones(shift))
}

pub const fn replicate(x: u64, n: u64, size: u64) -> u64 {
    let mut result = 0b0;
    let mut i = n;

    while i > 0 {
        result |= x;
        result <<= size;
        i -= 1;
    }

    result
}

pub fn replicate_reg64(val: RegId, n: u8) -> Ir {
    let val = Operand::reg(Type::U64, val);

    Ir::If(
        Type::U64,
        Operand::ir(Ir::BitCast(
            Type::Bool,
            Operand::ir(Ir::LShr(Type::U64, val, Operand::imm(Type::U64, n as u64))),
        )),
        Operand::imm(Type::U64, u64::MAX),
        Operand::imm(Type::U64, 0),
    )
}

pub const fn decode_bit_masks(immn: u8, imms: u8, immr: u8, immediate: bool, m: u8) -> (u64, u64) {
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

pub enum ExtendType {
    UXTB,
    UXTH,
    UXTW,
    UXTX,
    SXTB,
    SXTH,
    SXTW,
    SXTX,
}

pub const fn decode_reg_extend(op: u8) -> ExtendType {
    match op {
        0b000 => ExtendType::UXTB,
        0b001 => ExtendType::UXTH,
        0b010 => ExtendType::UXTW,
        0b011 => ExtendType::UXTX,
        0b100 => ExtendType::SXTB,
        0b101 => ExtendType::SXTH,
        0b110 => ExtendType::SXTW,
        0b111 => ExtendType::SXTX,
        _ => unreachable!(),
    }
}

pub fn extend_reg(reg: RegId, ext_type: ExtendType, shift: u8, n: u8) -> Ir {
    assert!(shift <= 4);
    let n = Type::uscalar_from_size(n as usize);

    let (unsigned, ty) = match ext_type {
        ExtendType::SXTB => (false, Type::I8),
        ExtendType::SXTH => (false, Type::I16),
        ExtendType::SXTW => (false, Type::I32),
        ExtendType::SXTX => (false, Type::I64),
        ExtendType::UXTB => (true, Type::U8),
        ExtendType::UXTH => (true, Type::U16),
        ExtendType::UXTW => (true, Type::U32),
        ExtendType::UXTX => (true, Type::U64),
    };

    let ir = Ir::LShl(
        ty,
        Operand::Register(ty, reg),
        Operand::Immediate(Type::U8, shift as u64),
    );

    if unsigned {
        Ir::ZextCast(n, Operand::ir(ir))
    } else {
        Ir::SextCast(n, Operand::ir(ir))
    }
}

pub fn replace_bits(val: Operand, imm: u64, range: Range<u64>) -> Ir {
    let imm = Operand::imm(val.get_type(), imm << range.start);
    let mask = Operand::imm(
        val.get_type(),
        !(ones(range.end - range.start) << range.start),
    );

    Ir::Or(
        val.get_type(),
        Operand::ir(Ir::And(val.get_type(), val, mask)),
        imm,
    )
}
