
/// Add | Op2 = Op1 + Op3
pub const UADD_REG3: u8 = 0b0000_0000;
/// Sub | Op2 = Op1 + Op3
pub const USUB_REG3: u8 = 0b0000_0001; 
/// Mul | Op2 = Op1 + Op3
pub const UMUL_REG3: u8 = 0b0000_0010;
/// Div | Op2 = Op1 + Op3
pub const UDIV_REG3: u8 = 0b0000_0011;
///Add | Op2 = Op1 + Imm8
pub const UADD_REG2IMM8: u8 = 0b0000_0100;
///Sub | Op2 = Op1 + Imm8
pub const USUB_REG2IMM8: u8 = 0b0000_0101;
///Mul | Op2 = Op1 + Imm8
pub const UMUL_REG2IMM8: u8 = 0b0000_0110;
///Div | Op2 = Op1 + Imm8
pub const UDIV_REG2IMM8: u8 = 0b0000_0111;

//Dst += Const(4byte)
pub const UADD_REG2IMM32: u8 = 0b0001_0000;
pub const USUB_REG2IMM32: u8 = 0b0001_0001;
pub const UMUL_REG2IMM32: u8 = 0b0001_0010;
pub const UDIV_REG2IMM32: u8 = 0b0001_0011;

pub const IADD_REG2IMM32: u8 = 0b0001_0100;
pub const ISUB_REG2IMM32: u8 = 0b0001_0101;
pub const IMUL_REG2IMM32: u8 = 0b0001_0110;
pub const IDIV_REG2IMM32: u8 = 0b0001_0111;
//Dst += Const(8byte)
pub const UADD_REG2IMM64: u8 = 0b0001_1000;
pub const USUB_REG2IMM64: u8 = 0b0001_1001;
pub const UMUL_REG2IMM64: u8 = 0b0001_1010;
pub const UDIV_REG2IMM64: u8 = 0b0001_1011;

pub const IADD_REG2IMM64: u8 = 0b0001_1100;
pub const ISUB_REG2IMM64: u8 = 0b0001_1101;
pub const IMUL_REG2IMM64: u8 = 0b0001_1110;
pub const IDIV_REG2IMM64: u8 = 0b0001_1111;
//Dst = Val0 (|, &, ^) Val1
pub const OR_REG3: u8 = 0b0010_0000;
pub const AND_REG3: u8 = 0b0010_0001;
pub const XOR_REG3: u8 = 0b0010_0010;

pub const OR_REG2IMM64: u8 = 0b0010_0100;
pub const AND_REG2IMM64: u8 = 0b0010_0101;
pub const XOR_REG2IMM64: u8 = 0b0010_0110;

//Shifts
pub const LSHL_REG2IMM8: u8 = 0b0010_1000;
pub const LSHR_REG2IMM8: u8 = 0b0010_1001; //Logical Right Shift
pub const RROT_REG2IMM8: u8 = 0b0010_1010;
pub const ASHR_REG2IMM8: u8 = 0b0010_1011; // Arithmetic Right Shift

//Memory Instructions
pub const PSH_REG: u8 = 0b0100_0000;
pub const POP_REG: u8 = 0b0100_0001;

pub const MOV_REG2: u8 = 0b0100_1101;
pub const MOV_IPR_REG: u8 = 0b0100_1100;
pub const MOV_REG1IMM64: u8 = 0b0100_0110;
pub const MOV_REG1IMM16: u8 = 0b0100_0111;
pub const SLOAD_REL_REG2IMM32: u8 = 0b0100_0011;
pub const SSTORE_REL_REG2IMM32: u8 = 0b0100_0010;
pub const ULOAD_REG1IMM64: u8 = 0b0100_0101;
pub const USTORE_REG1IMM64: u8 = 0b0100_0100;

// ControlFlow Instructions
pub const BR_IPV_IMM32: u8 = 0b1000_0001;
pub const BR_IRP_IMM32_REL: u8 = 0b1000_0010;

//Special Instructions
pub const NOP: u8 = 0b1100_1101;
pub const SVC_IMM16: u8 = 0b1100_1110;
pub const BRK_IMM16: u8 = 0b1100_1111;
