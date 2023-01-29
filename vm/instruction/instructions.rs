//Dst = Val0 + Val1
pub const IROP_AL_INSTRUCTION_MASK: u8 = 0b0000_0000;
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

pub const IROP_IADD_CST32: u8 = 0b0001_0000;
pub const IROP_ISUB_CST32: u8 = 0b0001_0001;
pub const IROP_IMUL_CST32: u8 = 0b0001_0010;
pub const IROP_IDIV_CST32: u8 = 0b0001_0011;
//Dst += Const(8byte)
pub const IROP_UADD_CST64: u8 = 0b0001_0100;
pub const IROP_USUB_CST64: u8 = 0b0001_0101;
pub const IROP_UMUL_CST64: u8 = 0b0001_0110;
pub const IROP_UDIV_CST64: u8 = 0b0001_0111;

//Dst = Val0 (|, &, ^) Val1
pub const IROP_OR_REG3: u8 = 0b0001_1000;
pub const IROP_AND_REG3: u8 = 0b0001_1001;
pub const IROP_XOR_REG3: u8 = 0b0001_1010;

//Shifts
pub const IROP_LLEFT_SHIFT_IMM8: u8 = 0b0001_1100;
pub const IROP_LRIGHT_SHIFT_IMM8: u8 = 0b0001_1101; //Logical Right Shift
pub const IROP_ROTATE_IMM8: u8 = 0b0001_1110;
pub const IROP_ARIGHT_SHIFT_IMM8: u8 = 0b0001_1111; // Arithmetic Right Shift

//Memory Instructions
pub const IROP_MEM_INSTRUCTION_MASK: u8 = 0b0100_0000;
pub const IROP_PUSH_REG: u8 = 0b0100_0000;
pub const IROP_POP_REG: u8 = 0b0100_0001;

pub const IROP_USTORE_REG2REG: u8 = 0b0100_0010;
pub const IROP_ULOAD_REG2REG: u8 = 0b0100_0011;
pub const IROP_SSTORE_REG2REG: u8 = 0b0100_0100;
pub const IROP_SLOAD_REG2REG: u8 = 0b0100_0101;

pub const IROP_MOV_64CST2REG: u8 = 0b0100_0110;
pub const IROP_MOV_16CST2REG: u8 = 0b0100_0111;

pub const IROP_MOV_IPR2REG: u8 = 0b0100_1100;
pub const IROP_MOV_REG2REG: u8 = 0b0100_1101;

// ControlFlow Instructions
pub const IROP_CF_INSTRUCTION_MASK: u8 = 0b1000_0000;
pub const IROP_BR_IPV: u8 = 0b1000_0001;
pub const IROP_BR_IPR_REL32: u8 = 0b1000_0010;

//Special Instructions
pub const IROP_SP_INSTRUCTION_MASK: u8 = 0b1100_0000;
pub const IROP_NOP: u8 = 0b1100_1101;
pub const IROP_SVC: u8 = 0b1100_1110;
pub const IROP_BRK: u8 = 0b1100_1111;
