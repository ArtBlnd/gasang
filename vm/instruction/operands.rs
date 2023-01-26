use crate::instruction::VmIr;
use crate::register::RegId;

use std::convert::From;

pub const VMIR_REG1: u8 = 0b0000;
pub const VMIR_REG2: u8 = 0b0001;
pub const VMIR_REG3: u8 = 0b0010;
pub const VMIR_REG1U8: u8 = 0b0011;
pub const VMIR_REG2U8: u8 = 0b0100;
pub const VMIR_REG1U64: u8 = 0b0101;
pub const VMIR_REG2I64: u8 = 0b0110;

pub struct Reg1 {
    pub op1: RegId,
}
impl Reg1 {
    pub fn build(self, opcode: u8) -> [u8; 2] {
        [opcode, self.op1.0]
    }
}

pub struct Reg2 {
    pub op1: RegId,
    pub op2: RegId,
}

impl Reg2 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        [opcode, self.op1.0, self.op2.0]
    }
}

pub struct Reg3 {
    pub op1: RegId,
    pub op2: RegId,
    pub op3: RegId,
}

impl Reg3 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        [opcode, self.op1.0, self.op2.0, self.op3.0]
    }
}

pub struct Reg1U8 {
    pub op1: RegId,
    pub imm8: u8,
}

impl Reg1U8 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        [opcode, self.op1.0, self.imm8]
    }
}

pub struct Reg2U8 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm8: u8,
}

impl Reg2U8 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        [opcode, self.op1.0, self.op2.0, self.imm8]
    }
}

pub struct Reg1U32 {
    pub op1: RegId,
    pub imm32: u32,
}

impl Reg1U32 {
    pub fn build(self, opcode: u8) -> [u8; 6] {
        let imm = self.imm32.to_le_bytes();
        [opcode, self.op1.0, imm[0], imm[1], imm[2], imm[3]]
    }
}

pub struct Reg1U64 {
    pub op1: RegId,
    pub imm64: u64,
}

impl Reg1U64 {
    pub fn build(self, opcode: u8) -> [u8; 10] {
        let imm = self.imm64.to_le_bytes();
        [
            opcode, self.op1.0, imm[0], imm[1], imm[2], imm[3], imm[4], imm[5], imm[6], imm[7],
        ]
    }
}

pub struct Reg1U16 {
    pub op1: RegId,
    pub imm16: u16,
}

impl Reg1U16 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        let imm = self.imm16.to_le_bytes();
        [opcode, self.op1.0, imm[0], imm[1]]
    }
}

pub struct U16 {
    pub imm16: u16,
}

impl U16 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        let imm = self.imm16.to_le_bytes();
        [opcode, imm[0], imm[1]]
    }
}

pub struct Reg2I64 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm64: i64,
}

impl Reg2I64 {
    pub fn build(self, opcode: u8) -> [u8; 11] {
        let imm = self.imm64.to_le_bytes();
        [
            opcode, self.op1.0, self.op2.0, imm[0], imm[1], imm[2], imm[3], imm[4], imm[5], imm[6],
            imm[7],
        ]
    }
}

pub struct Reg1I64 {
    pub op1: RegId,
    pub imm64: i64,
}

impl Reg1I64 {
    pub fn build(self, opcode: u8) -> [u8; 10] {
        let imm = self.imm64.to_le_bytes();
        [
            opcode, self.op1.0, imm[0], imm[1], imm[2], imm[3], imm[4], imm[5], imm[6], imm[7],
        ]
    }
}
