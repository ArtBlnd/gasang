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
/// Not | Op2 = !Op1
pub const NOT_REG2: u8 = 0b0010_0011;

/// Or | Op2 = Op1 | Imm64
pub const OR_REG2IMM64: u8 = 0b0010_0100;
/// And | Op2 = Op1 & Imm64
pub const AND_REG2IMM64: u8 = 0b0010_0101;
/// Xor | Op2 = Op1 ^ Imm64
pub const XOR_REG2IMM64: u8 = 0b0010_0110;
/// NOT | Op1 = !Imm64
pub const NOT_REG1IMM64: u8 = 0b0010_0111;

/// LSHL | Op2 = Op1 << Imm8
pub const LSHL_REG2IMM8: u8 = 0b0010_1000;
/// LSHR | Op2 = Op1 >> Imm8
pub const LSHR_REG2IMM8: u8 = 0b0010_1001; //Logical Right Shift
/// RROT | Op2 = rotate(Op1, Imm8)
pub const RROT_REG2IMM8: u8 = 0b0010_1010;
/// ASHR | Op2 = arithmetic_rshift(Op1, Imm8)
pub const ASHR_REG2IMM8: u8 = 0b0010_1011; // Arithmetic Right Shift

/// REPL | Op2 = Replicate(Op1, Imm16Hi, Imm16Lo)
pub const REPL_REG2IMM16: u8 = 0b0011_1111;

/// Push | push(Op1)
pub const PSH_REG: u8 = 0b0100_0000;
/// Pop | pop(Op1)
pub const POP_REG: u8 = 0b0100_0001;

/// Mov | Op2 = Op1
pub const MOV_REG2: u8 = 0b0100_0010;
/// Mov | Op1 = IPR(Real Program Counter)
pub const MOV_IPR_REG: u8 = 0b0100_0011;

/// Mov | Slot = Op1
pub const STORE_SLOT_REG: u8 = 0b0100_0100;
/// Mov | Op1 = Slot
pub const LOAD_SLOT_REG: u8 = 0b0100_0101;

/// Mov | Op1 = Imm64
pub const MOV_REG1IMM64: u8 = 0b0100_0110;
/// Mov | Op1 = Imm16
pub const MOV_REG1IMM16: u8 = 0b0100_0111;

/// Mov | Op2 = *(Op1 + Imm32)
pub const SLOAD_REL_REG2IMM32: u8 = 0b0100_1100;
/// Mov | *(Op2 + Imm32) = Op1
pub const SSTORE_REL_REG2IMM32: u8 = 0b0100_1101;
/// Mov | Op2 = *(Op1 + Imm32)
pub const ULOAD_REL_REG2IMM32: u8 = 0b0100_1110;
/// Mov | *(Op2 + Imm32) = Op1
pub const USTORE_REL_REG2IMM32: u8 = 0b0100_1111;
/// Mov | Op1 = *(Imm64)
pub const ULOAD_REG1IMM64: u8 = 0b0101_0000;
/// Mov | *(Imm64) = Op1
pub const USTORE_REG1IMM64: u8 = 0b0101_0001;

/// Mov | Op2 = Op1\<Imm8\>
pub const MOV_BIT_REG2IMM8: u8 = 0b0111_1111;

/// Jmp | Jump to JumpTable[Imm32]
pub const BR_IPV_IMM32: u8 = 0b1000_0000;
/// Jmp | Jump to Imm32
pub const BR_IPR_IMM32: u8 = 0b1000_0001;
/// Jmp | Jump to Ipr + Imm32
pub const BR_IPR_IMM32_REL: u8 = 0b1000_0010;
/// Jmp | Jump to Ipr + Imm32 If slots[SlotID] == 0
pub const BR_IPR_IMM32_REL_IF_SLOT_ZERO: u8 = 0b1000_0011;
/// Jmp | Jump to Op1
pub const BR_IPR_REG1: u8 = 0b1000_0100;

///NOP | No-op
pub const NOP: u8 = 0b1100_1101;
///SVC | SystemCall(Imm16)
pub const SVC_IMM16: u8 = 0b1100_1110;
///BRK | Break(Imm16)
pub const BRK_IMM16: u8 = 0b1100_1111;

///Add | Slot2 = Slot1 + Slot3
pub const UADD_SLOT3: u8 = 0b1111_0000;
///Sub | Slot2 = Slot1 - Slot3
pub const USUB_SLOT3: u8 = 0b1111_0001;
///Mul | Slot2 = Slot1 * Slot3
pub const UMUL_SLOT3: u8 = 0b1111_0010;
///Div | Slot2 = Slot1 / Slot3
pub const UDIV_SLOT3: u8 = 0b1111_0011;

///Or | Slot2 = Slot1 | Slot3
pub const OR_SLOT3: u8 = 0b1111_0100;
///And | Slot2 = Slot1 & Slot3
pub const AND_SLOT3: u8 = 0b1111_0101;
///Xor | Slot2 = Slot1 ^ Slot3
pub const XOR_SLOT3: u8 = 0b1111_0110;

///Add | Op1 = Op1 + Slot1
pub const UADD_REG1SLOT1: u8 = 0b1111_1000;
///Sub | Op1 = Op1 - Slot1
pub const USUB_REG1SLOT1: u8 = 0b1111_1001;
///Mul | Op1 = Op1 * Slot1
pub const UMUL_REG1SLOT1: u8 = 0b1111_1010;
///Div | Op1 = Op1 / Slot1
pub const UDIV_REG1SLOT1: u8 = 0b1111_1011;

///Or | Op1 = Op1 | Slot1
pub const OR_REG1SLOT1: u8 = 0b1111_1100;
///And | Op1 = Op1 & Slot1
pub const AND_REG1SLOT1: u8 = 0b1111_1101;
///Xor | Op1 = Op1 ^ Slot1
pub const XOR_REG1SLOT1: u8 = 0b1111_1110;
