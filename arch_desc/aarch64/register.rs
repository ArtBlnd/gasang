use core::{RawRegisterId, Register, RegisterId};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AArch64MnemonicHint {
    X,
    X_SP,
    X_PC,
    V,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AArch64Register {
    // Speical registers
    Xzr,
    Pstate([bool; 64]),
    Sp(u64),
    Pc(u64),

    // Scalar registers
    X(u64),
    W(u32),

    // Vector registers
    V(Vector),
    Q(u128),
    D(u64),
    S(u32),
    H(u16),
    B(u8),
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct Vector([u8; 16]);

impl Register for AArch64Register {
    fn is_read_only(&self) -> bool {
        match self {
            Self::Xzr => true,
            _ => false,
        }
    }

    type Id = AArch64RegisterId;

    fn parent(&self) -> Self::Id {
        todo!()
    }

    fn id(&self) -> Self::Id {
        todo!()
    }

    fn size(&self) -> usize {
        todo!()
    }
}

impl From<Vector> for [u8; 16] {
    fn from(vec: Vector) -> Self {
        vec.0
    }
}

impl From<Vector> for [u16; 8] {
    fn from(vec: Vector) -> Self {
        let mut result = [0; 8];
        for i in 0..8 {
            result[i] = u16::from_le_bytes([vec.0[i * 2], vec.0[i * 2 + 1]]);
        }
        result
    }
}

impl From<Vector> for [u32; 4] {
    fn from(vec: Vector) -> Self {
        let mut result = [0; 4];
        for i in 0..4 {
            result[i] = u32::from_le_bytes([
                vec.0[i * 4],
                vec.0[i * 4 + 1],
                vec.0[i * 4 + 2],
                vec.0[i * 4 + 3],
            ]);
        }
        result
    }
}

impl From<Vector> for [u64; 2] {
    fn from(vec: Vector) -> Self {
        let mut result = [0; 2];
        for i in 0..2 {
            result[i] = u64::from_le_bytes([
                vec.0[i * 8],
                vec.0[i * 8 + 1],
                vec.0[i * 8 + 2],
                vec.0[i * 8 + 3],
                vec.0[i * 8 + 4],
                vec.0[i * 8 + 5],
                vec.0[i * 8 + 6],
                vec.0[i * 8 + 7],
            ]);
        }
        result
    }
}

impl From<Vector> for u128 {
    fn from(vec: Vector) -> Self {
        u128::from_le_bytes(vec.0)
    }
}

impl From<[u8; 16]> for Vector {
    fn from(arr: [u8; 16]) -> Self {
        Self(arr)
    }
}

impl From<[u16; 8]> for Vector {
    fn from(arr: [u16; 8]) -> Self {
        let mut result = [0; 16];
        for i in 0..8 {
            result[i * 2] = arr[i].to_le_bytes()[0];
            result[i * 2 + 1] = arr[i].to_le_bytes()[1];
        }
        Self(result)
    }
}

impl From<[u32; 4]> for Vector {
    fn from(arr: [u32; 4]) -> Self {
        let mut result = [0; 16];
        for i in 0..4 {
            result[i * 4] = arr[i].to_le_bytes()[0];
            result[i * 4 + 1] = arr[i].to_le_bytes()[1];
            result[i * 4 + 2] = arr[i].to_le_bytes()[2];
            result[i * 4 + 3] = arr[i].to_le_bytes()[3];
        }
        Self(result)
    }
}

impl From<[u64; 2]> for Vector {
    fn from(arr: [u64; 2]) -> Self {
        let mut result = [0; 16];
        for i in 0..2 {
            result[i * 8] = arr[i].to_le_bytes()[0];
            result[i * 8 + 1] = arr[i].to_le_bytes()[1];
            result[i * 8 + 2] = arr[i].to_le_bytes()[2];
            result[i * 8 + 3] = arr[i].to_le_bytes()[3];
            result[i * 8 + 4] = arr[i].to_le_bytes()[4];
            result[i * 8 + 5] = arr[i].to_le_bytes()[5];
            result[i * 8 + 6] = arr[i].to_le_bytes()[6];
            result[i * 8 + 7] = arr[i].to_le_bytes()[7];
        }
        Self(result)
    }
}

impl From<u128> for Vector {
    fn from(val: u128) -> Self {
        Self(val.to_le_bytes())
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum AArch64RegisterId {
    // General purpose registers
    X(u8),
    W(u8),

    // Vector registers
    V(u8),
    Q(u8),
    D(u8),
    S(u8),
    H(u8),
    B(u8),

    // Special registers
    Sp,
    Pc,
    Pstate,
    Xzr,
}

impl RegisterId for AArch64RegisterId {
    fn raw(&self) -> RawRegisterId {
        let raw = match self {
            &Self::W(v) => 0x00FF + v as usize,
            &Self::X(v) => 0x0000 + v as usize,
            &Self::V(v) => 0x01FF + v as usize,
            &Self::Q(v) => 0x02FF + v as usize,
            &Self::D(v) => 0x03FF + v as usize,
            &Self::S(v) => 0x04FF + v as usize,
            &Self::H(v) => 0x05FF + v as usize,
            &Self::B(v) => 0x06FF + v as usize,

            Self::Sp => 0x0800,
            Self::Pc => 0x0801,
            Self::Pstate => 0x0802,
            Self::Xzr => 0x0803,
        };

        RawRegisterId::new(raw)
    }
}
