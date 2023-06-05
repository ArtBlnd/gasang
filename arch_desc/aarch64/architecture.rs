use core::{Architecture, Primitive, RegisterFileDesc};

use super::{AArch64Inst, AArch64MnemonicHint, AArch64Register, AArch64RegisterId};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct AArch64Architecture;

impl Architecture for AArch64Architecture {
    type Inst = AArch64Inst;

    type Reg = AArch64Register;
    type RegId = AArch64RegisterId;

    fn get_register_by_name(name: impl AsRef<str>) -> Self::RegId {
        let name = name.as_ref();

        match name {
            // handle special registers
            "sp" => return AArch64RegisterId::Sp,
            "pc" => return AArch64RegisterId::Pc,
            "pstate" => return AArch64RegisterId::Pstate,
            _ => {}
        }

        let reg_number: u8 = name[1..].parse().unwrap();
        let reg_prefix = &name[0..1];

        assert!(reg_number < 32, "invalid register number {}", reg_number);
        match reg_prefix {
            // Handle scalar registers
            "x" => AArch64RegisterId::X(reg_number),
            "w" => AArch64RegisterId::W(reg_number),

            // Handle vector registers
            "v" => AArch64RegisterId::V(reg_number),
            "q" => AArch64RegisterId::Q(reg_number),
            "d" => AArch64RegisterId::D(reg_number),
            "s" => AArch64RegisterId::S(reg_number),
            "h" => AArch64RegisterId::H(reg_number),
            "b" => AArch64RegisterId::B(reg_number),
            _ => unreachable!("invalid register name {}", name),
        }
    }

    type MnemonicHint = AArch64MnemonicHint;
    fn get_register_by_mnemonic(hint: Self::MnemonicHint, mnemonic: impl Primitive) -> Self::RegId {
        let raw = mnemonic.to_u8().unwrap();

        match (hint, raw) {
            (AArch64MnemonicHint::X, 31) => AArch64RegisterId::Xzr,
            (AArch64MnemonicHint::X, v) => AArch64RegisterId::X(v),
            (AArch64MnemonicHint::X_PC, 31) => AArch64RegisterId::Pc,
            (AArch64MnemonicHint::X_PC, v) => AArch64RegisterId::X(v),
            (AArch64MnemonicHint::X_SP, 31) => AArch64RegisterId::Sp,
            (AArch64MnemonicHint::X_SP, v) => AArch64RegisterId::X(v),
            (AArch64MnemonicHint::V, v) => AArch64RegisterId::V(v),
            _ => unreachable!("invalid mnemonic with mnemonic hint {:?} {}", hint, raw),
        }
    }

    fn get_pc_register() -> Self::RegId {
        AArch64RegisterId::Pc
    }

    fn get_flag_register() -> Self::RegId {
        AArch64RegisterId::Pstate
    }

    fn get_register_file_desc() -> RegisterFileDesc {
        todo!()
    }
}
