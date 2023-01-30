/// Add | Op2 = Op1 + Op3
pub const UADD_REG3: u8 = 0b0000_0000;
/// Sub | Op2 = Op1 - Op3
pub const USUB_REG3: u8 = 0b0000_0001;
/// Mul | Op2 = Op1 * Op3
pub const UMUL_REG3: u8 = 0b0000_0010;
/// Div | Op2 = Op1 / Op3
pub const UDIV_REG3: u8 = 0b0000_0011;
///Add | Op2 = Op1 + Imm8
pub const UADD_REG2IMM8: u8 = 0b0000_0100;
///Sub | Op2 = Op1 - Imm8
pub const USUB_REG2IMM8: u8 = 0b0000_0101;
///Mul | Op2 = Op1 * Imm8
pub const UMUL_REG2IMM8: u8 = 0b0000_0110;
///Div | Op2 = Op1 / Imm8
pub const UDIV_REG2IMM8: u8 = 0b0000_0111;

/// Add | Op2 = Op1 + Imm32
pub const UADD_REG2IMM32: u8 = 0b0001_0000;
/// Sub | Op2 = Op1 - Imm32
pub const USUB_REG2IMM32: u8 = 0b0001_0001;
/// Mul | Op2 = Op1 * Imm32
pub const UMUL_REG2IMM32: u8 = 0b0001_0010;
/// Div | Op2 = Op1 / Imm32
pub const UDIV_REG2IMM32: u8 = 0b0001_0011;
/// Add | Op2 = Op1 + Imm32
pub const IADD_REG2IMM32: u8 = 0b0001_0100;
/// Sub | Op2 = Op1 - Imm32
pub const ISUB_REG2IMM32: u8 = 0b0001_0101;
/// Mul | Op2 = Op1 * Imm32
pub const IMUL_REG2IMM32: u8 = 0b0001_0110;
/// Div | Op2 = Op1 / Imm32
pub const IDIV_REG2IMM32: u8 = 0b0001_0111;

/// Add | Op2 = Op1 + Imm64
pub const UADD_REG2IMM64: u8 = 0b0001_1000;
/// Sub | Op2 = Op1 - Imm64
pub const USUB_REG2IMM64: u8 = 0b0001_1001;
/// Mul | Op2 = Op1 * Imm64
pub const UMUL_REG2IMM64: u8 = 0b0001_1010;
/// Div | Op2 = Op1 / Imm64
pub const UDIV_REG2IMM64: u8 = 0b0001_1011;

/// Add | Op2 = Op1 + Imm64
pub const IADD_REG2IMM64: u8 = 0b0001_1100;
/// Sub | Op2 = Op1 - Imm64
pub const ISUB_REG2IMM64: u8 = 0b0001_1101;
/// Mul | Op2 = Op1 * Imm64
pub const IMUL_REG2IMM64: u8 = 0b0001_1110;
/// Div | Op2 = Op1 / Imm64
pub const IDIV_REG2IMM64: u8 = 0b0001_1111;

/// Or | Op2 = Op1 | Op3
pub const OR_REG3: u8 = 0b0010_0000;
/// And | Op2 = Op1 & Op3
pub const AND_REG3: u8 = 0b0010_0001;
/// Xor | Op2 = Op1 ^ Op3
pub const XOR_REG3: u8 = 0b0010_0010;

/// Or | Op2 = Op1 | Imm64
pub const OR_REG2IMM64: u8 = 0b0010_0100;
/// And | Op2 = Op1 & Imm64
pub const AND_REG2IMM64: u8 = 0b0010_0101;
/// Xor | Op2 = Op1 ^ Imm64
pub const XOR_REG2IMM64: u8 = 0b0010_0110;

/// LSHL | Op2 = Op1 << Imm8
pub const LSHL_REG2IMM8: u8 = 0b0010_1000;
/// LSHR | Op2 = Op1 >> Imm8
pub const LSHR_REG2IMM8: u8 = 0b0010_1001; //Logical Right Shift
/// RROT | Op2 = rotate(Op1, Imm8)
pub const RROT_REG2IMM8: u8 = 0b0010_1010;
/// ASHR | Op2 = arithmetic_rshift(Op1, Imm8)
pub const ASHR_REG2IMM8: u8 = 0b0010_1011; // Arithmetic Right Shift

/// Push | push(Op1)
pub const PSH_REG: u8 = 0b0100_0000;
/// Pop | pop(Op1)
pub const POP_REG: u8 = 0b0100_0001;

/// Mov | Op2 = Op1
pub const MOV_REG2: u8 = 0b0100_1101;
/// Mov | Op1 = IPR(Real Program Counter)
pub const MOV_IPR_REG: u8 = 0b0100_1100;
/// Mov | Op1 = Imm64
pub const MOV_REG1IMM64: u8 = 0b0100_0110;
/// Mov | Op1 = Imm16
pub const MOV_REG1IMM16: u8 = 0b0100_0111;

/// Mov | Op2 = *(Op1 + Imm32)
pub const SLOAD_REL_REG2IMM32: u8 = 0b0100_0011;
/// Mov | *(Op2 + Imm32) = Op1
pub const SSTORE_REL_REG2IMM32: u8 = 0b0100_0010;
/// Mov | Op1 = *(Imm64)
pub const ULOAD_REG1IMM64: u8 = 0b0100_0101;
/// Mov | *(Imm64) = Op1
pub const USTORE_REG1IMM64: u8 = 0b0100_0100;

/// Jmp | Jump to JumpTable[Imm32]
pub const BR_IPV_IMM32: u8 = 0b1000_0000;
/// Jmp | Jump to Ipr + Imm32
pub const BR_IPR_IMM32_REL: u8 = 0b1000_0010;
/// JMP | Jump to Ipr + Imm32 if ( Op1 == Imm64 )
pub const BR_IPR_IMM32_REL_IF_OP1_EQ_IMM64: u8 = 0b1000_0011;
/// JMP | Jump to Ipr + Imm32 if ( Op1 != Imm64 )
pub const BR_IPR_IMM32_REL_IF_OP1_NE_IMM64: u8 = 0b1000_0100;

///NOP | No-op
pub const NOP: u8 = 0b1100_1101;
///SVC | SystemCall(Imm16)
pub const SVC_IMM16: u8 = 0b1100_1110;
///BRK | Break(Imm16)
pub const BRK_IMM16: u8 = 0b1100_1111;
