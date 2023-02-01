use crate::{register::RegId, SlotId};

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

pub struct Reg1Imm8 {
    pub op1: RegId,
    pub imm8: u8,
}

impl Reg1Imm8 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        [opcode, self.op1.0, self.imm8]
    }

    pub fn u8(&self) -> u8 {
        self.imm8
    }

    pub fn i8(&self) -> i8 {
        self.imm8 as i8
    }
}

pub struct Reg1Imm16 {
    pub op1: RegId,
    pub imm16: u16,
}

impl Reg1Imm16 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        let imm = self.imm16.to_le_bytes();
        [opcode, self.op1.0, imm[0], imm[1]]
    }

    pub fn u16(&self) -> u16 {
        self.imm16
    }

    pub fn i16(&self) -> i16 {
        self.imm16 as i16
    }
}

pub struct Reg1Imm32 {
    pub op1: RegId,
    pub imm32: u32,
}

impl Reg1Imm32 {
    pub fn build(self, opcode: u8) -> [u8; 6] {
        let imm = self.imm32.to_le_bytes();
        [opcode, self.op1.0, imm[0], imm[1], imm[2], imm[3]]
    }

    pub fn u32(&self) -> u32 {
        self.imm32
    }

    pub fn i32(&self) -> i32 {
        self.imm32 as i32
    }
}

pub struct Reg1Imm64 {
    pub op1: RegId,
    pub imm64: u64,
}

impl Reg1Imm64 {
    pub fn build(self, opcode: u8) -> [u8; 10] {
        let imm = self.imm64.to_le_bytes();
        [
            opcode, self.op1.0, imm[0], imm[1], imm[2], imm[3], imm[4], imm[5], imm[6], imm[7],
        ]
    }

    pub fn u64(&self) -> u64 {
        self.imm64
    }

    pub fn i64(&self) -> i64 {
        self.imm64 as i64
    }
}

pub struct Reg2Imm8 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm8: u8,
}

impl Reg2Imm8 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        [opcode, self.op1.0, self.op2.0, self.imm8]
    }

    pub fn u8(&self) -> u8 {
        self.imm8
    }

    pub fn i8(&self) -> i8 {
        self.imm8 as i8
    }
}

pub struct Reg2Imm16 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm16: u16,
}

impl Reg2Imm16 {
    pub fn build(self, opcode: u8) -> [u8; 5] {
        let imm = self.imm16.to_le_bytes();
        [opcode, self.op1.0, self.op2.0, imm[0], imm[1]]
    }

    pub fn u16(&self) -> u16 {
        self.imm16
    }

    pub fn i16(&self) -> i16 {
        self.imm16 as i16
    }
}

pub struct Reg2Imm32 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm32: u32,
}

impl Reg2Imm32 {
    pub fn build(self, opcode: u8) -> [u8; 7] {
        let imm = self.imm32.to_le_bytes();
        [
            opcode, self.op1.0, self.op2.0, imm[0], imm[1], imm[2], imm[3],
        ]
    }

    pub fn u32(&self) -> u32 {
        self.imm32
    }

    pub fn i32(&self) -> i32 {
        self.imm32 as i32
    }
}

pub struct Reg2Imm64 {
    pub op1: RegId,
    pub op2: RegId,
    pub imm64: u64,
}

impl Reg2Imm64 {
    pub fn build(self, opcode: u8) -> [u8; 11] {
        let imm = self.imm64.to_le_bytes();
        [
            opcode, self.op1.0, self.op2.0, imm[0], imm[1], imm[2], imm[3], imm[4], imm[5], imm[6],
            imm[7],
        ]
    }

    pub fn u64(&self) -> u64 {
        self.imm64
    }

    pub fn i64(&self) -> i64 {
        self.imm64 as i64
    }
}

pub struct Imm16 {
    pub imm16: u16,
}

impl Imm16 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        let imm = self.imm16.to_le_bytes();
        [opcode, imm[0], imm[1]]
    }

    pub fn u16(&self) -> u16 {
        self.imm16
    }

    pub fn i16(&self) -> i16 {
        self.imm16 as i16
    }
}

pub struct Imm32 {
    pub imm32: u32,
}

impl Imm32 {
    pub fn build(self, opcode: u8) -> [u8; 5] {
        let imm = self.imm32.to_le_bytes();
        [opcode, imm[0], imm[1], imm[2], imm[3]]
    }

    pub fn u32(&self) -> u32 {
        self.imm32
    }

    pub fn i32(&self) -> i32 {
        self.imm32 as i32
    }
}

pub struct Reg1Slot1 {
    pub op1: RegId,
    pub slot_id: SlotId,
}

impl Reg1Slot1 {
    pub fn build(self, opcode: u8) -> [u8; 3] {
        [opcode, self.op1.0, self.slot_id.0]
    }
}

pub struct SlotImm32 {
    pub slot_id: SlotId,
    pub imm32: u32,
}

impl SlotImm32 {
    pub fn build(self, opcode: u8) -> [u8; 6] {
        let imm = self.imm32.to_le_bytes();
        [opcode, self.slot_id.0, imm[0], imm[1], imm[2], imm[3]]
    }

    pub fn u32(&self) -> u32 {
        self.imm32
    }

    pub fn i32(&self) -> i32 {
        self.imm32 as i32
    }
}

pub struct Slot3 {
    pub slot1: SlotId,
    pub slot2: SlotId,
    pub slot3: SlotId,
}

impl Slot3 {
    pub fn build(self, opcode: u8) -> [u8; 4] {
        [opcode, self.slot1.0, self.slot2.0, self.slot3.0]
    }
}
