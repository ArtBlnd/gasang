use core::{RawRegisterId, Register};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum AArch64MnemonicHint {
    X,
    X_SP,
    X_PC,
    V,
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub enum AArch64Register {
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

impl Register for AArch64Register {
    fn raw(&self) -> RawRegisterId {
        let raw = match self {
            &Self::W(v) => 0x00FF + v as usize,
            &Self::X(v) => v as usize,
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
