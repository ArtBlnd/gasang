use crate::instruction::*;

//Dst = Val0 + Val1
pub const IROP_AL_INSTRUCTION_MASK: u8 = 0b00_000000;
pub const IROP_UADD_REG3: u8 = 0b0000_0000;
pub const IROP_USUB_REG3: u8 = 0b0000_0001;
pub const IROP_UMUL_REG3: u8 = 0b0000_0010;
pub const IROP_UDIV_REG3: u8 = 0b0000_0011;
//Dst += Const(1byte)
pub const IROP_UADD_CST8: u8 = 0b0000_0100;
pub const IROP_USUB_CST8: u8 = 0b0000_0101;
pub const IROP_UMUL_CST8: u8 = 0b0000_0110;
pub const IROP_UDIV_CST8: u8 = 0b0000_0111;
//Dst += Const(4byte)
pub const IROP_UADD_CST32: u8 = 0b0000_1000;
pub const IROP_USUB_CST32: u8 = 0b0000_1001;
pub const IROP_UMUL_CST32: u8 = 0b0000_1010;
pub const IROP_UDIV_CST32: u8 = 0b0000_1011;

pub const IROP_IADD_CST32: u8 = 0b1000_1000;
pub const IROP_ISUB_CST32: u8 = 0b1000_1001;
pub const IROP_IMUL_CST32: u8 = 0b1000_1010;
pub const IROP_IDIV_CST32: u8 = 0b1000_1011;
//Dst += Const(8byte)
pub const IROP_UADD_CST64: u8 = 0b0000_1100;
pub const IROP_USUB_CST64: u8 = 0b0000_1101;
pub const IROP_UMUL_CST64: u8 = 0b0000_1110;
pub const IROP_UDIV_CST64: u8 = 0b0000_1111;

//Dst = Val0 (|, &, ^) Val1
pub const IROP_OR_REG3: u8 = 0b0001_0000;
pub const IROP_AND_REG3: u8 = 0b0001_0001;
pub const IROP_XOR_REG3: u8 = 0b0001_0010;

//Shifts
pub const IROP_LLEFT_SHIFT_IMM8: u8 = 0b0001_0011;
pub const IROP_LRIGHT_SHIFT_IMM8: u8 = 0b0001_0100; //Logical Right Shift
pub const IROP_ROTATE_IMM8: u8 = 0b0001_0101;
pub const IROP_ARIGHT_SHIFT_IMM8: u8 = 0b0001_0110; // Arithmetic Right Shift

//Memory Instructions
pub const IROP_MEM_INSTRUCTION_MASK: u8 = 0b01_000000;
pub const IROP_MOV_REG2MEM_REG: u8 = 0b0101_0111;
pub const IROP_MOV_REG2MEM_CST: u8 = 0b0101_1000;

pub const IROP_MOV_64CST2REG: u8 = 0b0101_1001;
pub const IROP_MOV_16CST2REG: u8 = 0b0101_1010;

pub const IROP_MOV_IPR2REG: u8 = 0b0101_1100;
pub const IROP_MOV_REG2REG: u8 = 0b0101_1101;

// ControlFlow Instructions
pub const IROP_CF_INSTRUCTION_MASK: u8 = 0b10_000000;
pub const IROP_JMP: u8 = 0b1000_0000;

//Special Instructions
pub const IROP_SP_INSTRUCTION_MASK: u8 = 0b11_000000;
pub const IROP_NOP: u8 = 0b1100_1101;
pub const IROP_SVC: u8 = 0b1100_1110;
pub const IROP_BRK: u8 = 0b1100_1111;
