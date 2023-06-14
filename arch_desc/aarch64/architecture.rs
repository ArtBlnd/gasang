use core::{Architecture, Primitive, RegisterFileDesc};

use super::{AArch64Inst, AArch64MnemonicHint, AArch64Register};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct AArch64Architecture;

impl Architecture for AArch64Architecture {
    type Inst = AArch64Inst;
    type Register = AArch64Register;

    fn get_register_by_name(name: impl AsRef<str>) -> Self::Register {
        let name = name.as_ref();

        match name {
            // handle special registers
            "sp" => return AArch64Register::Sp,
            "pc" => return AArch64Register::Pc,
            "pstate" => return AArch64Register::Pstate,
            _ => {}
        }

        let reg_number: u8 = name[1..].parse().unwrap();
        let reg_prefix = &name[0..1];

        assert!(reg_number < 32, "invalid register number {}", reg_number);
        match reg_prefix {
            // Handle scalar registers
            "x" => AArch64Register::X(reg_number),
            "w" => AArch64Register::W(reg_number),

            // Handle vector registers
            "v" => AArch64Register::V(reg_number),
            "q" => AArch64Register::Q(reg_number),
            "d" => AArch64Register::D(reg_number),
            "s" => AArch64Register::S(reg_number),
            "h" => AArch64Register::H(reg_number),
            "b" => AArch64Register::B(reg_number),
            _ => unreachable!("invalid register name {}", name),
        }
    }

    type MnemonicHint = AArch64MnemonicHint;
    fn get_register_by_mnemonic(
        hint: Self::MnemonicHint,
        mnemonic: impl Primitive,
    ) -> Self::Register {
        let raw = mnemonic.to_u8().unwrap();

        match (hint, raw) {
            (AArch64MnemonicHint::X, 31) => AArch64Register::Xzr,
            (AArch64MnemonicHint::X, v) if v < 31 => AArch64Register::X(v),
            (AArch64MnemonicHint::X_PC, 31) => AArch64Register::Pc,
            (AArch64MnemonicHint::X_PC, v) if v < 31 => AArch64Register::X(v),
            (AArch64MnemonicHint::X_SP, 31) => AArch64Register::Sp,
            (AArch64MnemonicHint::X_SP, v) if v < 31 => AArch64Register::X(v),
            (AArch64MnemonicHint::V, v) => AArch64Register::V(v),
            _ => unreachable!("invalid mnemonic with mnemonic hint {:?} {}", hint, raw),
        }
    }

    fn get_pc_register() -> Self::Register {
        AArch64Register::Pc
    }

    fn get_flag_register() -> Self::Register {
        AArch64Register::Pstate
    }

    fn get_register_file_desc() -> RegisterFileDesc {
        todo!()
    }
}
