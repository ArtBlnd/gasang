use core::{
    ir::{BasicBlock, IrConstant, IrInst, IrType, IrValue},
    Architecture, Register,
};
use std::ops::{BitAnd, BitOr, Range, Shl};

use num_traits::{Bounded, One, Zero};

use super::{AArch64Architecture, OpcSizeImm12RnRt};

pub fn gen_move_pc(bb: &mut BasicBlock) {
    bb.push_inst(IrInst::Add {
        dst: IrValue::Register(IrType::B64, AArch64Architecture::get_pc_register().raw()),
        lhs: IrValue::Register(IrType::B64, AArch64Architecture::get_pc_register().raw()),
        rhs: IrValue::Constant(IrConstant::B64(4)),
    });
}

pub fn gen_pc_rel(bb: &mut BasicBlock, offset: IrValue) -> IrValue {
    let rel = bb.new_variable(IrType::B64);
    bb.push_inst(IrInst::Add {
        dst: rel,
        lhs: IrValue::Register(IrType::B64, AArch64Architecture::get_pc_register().raw()),
        rhs: offset,
    });

    rel
}

pub fn ones<T>(len: u32) -> T
where
    T: Zero + One + Shl<u32, Output = T> + BitOr<T, Output = T>,
{
    assert!(len <= 8);
    (0..len).fold(T::zero(), |state, _| {
        let state = state << 1;
        state | T::one()
    })
}

pub fn sign_extend<T>(val: T, len: u32) -> T
where
    T: Zero
        + One
        + Shl<u32, Output = T>
        + BitOr<T, Output = T>
        + BitAnd<T, Output = T>
        + Eq
        + Copy
        + Bounded,
{
    let sign_bit = T::one() << (len - 1);
    if val & sign_bit == T::zero() {
        val
    } else {
        val | T::max_value() << len
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

pub fn shift_reg(
    reg: IrValue,
    shift_type: ShiftType,
    amount: IrValue,
    t: IrType,
    dst: IrValue,
) -> IrInst {
    match shift_type {
        ShiftType::LSL => IrInst::Shl {
            dst,
            lhs: reg,
            rhs: amount,
        },
        ShiftType::LSR => IrInst::Lshr {
            dst,
            lhs: reg,
            rhs: amount,
        },
        ShiftType::ASR => IrInst::Ashr {
            dst,
            lhs: reg,
            rhs: amount,
        },
        ShiftType::ROR => IrInst::Rotr {
            dst,
            lhs: reg,
            rhs: amount,
        },
    }
}

pub const fn bit8(val: u8, idx: u8) -> bool {
    assert!(idx <= 8);
    ((val >> idx) & 0b1) == 0b1
}

pub const fn extract_bits16(range: Range<usize>, value: u16) -> u16 {
    let lshift = 16 - range.end;
    let left_shifted = value << lshift;

    left_shifted >> (range.start + lshift)
}

pub fn decode_operand_for_ld_st_reg_imm(
    operand: &OpcSizeImm12RnRt,
    is_simd_fp: bool,
) -> (bool, bool, i64) {
    let opc1 = bit8(operand.opc, 1) as u8;
    if operand.idxt == 0b00 {
        let imm9 = extract_bits16(2..11, operand.imm12) as i64;
        let post = extract_bits16(0..2, operand.imm12) == 0b01;

        (true, post, sign_extend(imm9, 9))
    } else {
        //Unsigned offset
        let scale = (if is_simd_fp { 4 } else { 0 }) * opc1 + operand.size;
        (false, false, (operand.imm12 << scale) as i64)
    }
}

#[cfg(test)]
mod tests {
    use crate::aarch64::compiler_prelude::sign_extend;

    #[test]
    fn test_sign_extend() {
        for i in i16::MIN..i16::MAX {
            let rust_sext = i as i64;
            let val = sign_extend((i as u16) as i64, 16);
            assert_eq!(rust_sext, val);
        }
    }
}
