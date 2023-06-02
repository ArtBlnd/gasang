use std::fmt::Debug;

use crate::{Instruction, Primitive, Register, RegisterFileDesc, RegisterId};

// The representation of an architecture
pub trait Architecture: Default + Clone + Copy + PartialEq + Eq {
    type Inst: Instruction;

    type Reg: Register;
    type RegId: RegisterId;

    fn get_register_by_name(name: impl AsRef<str>) -> Self::RegId;

    type MnemonicHint: Debug;
    fn get_register_by_mnemonic(hint: Self::MnemonicHint, mnemonic: impl Primitive) -> Self::RegId;

    /// Get pc register id.
    /// This panics if the arch does not have a pc register.
    fn get_pc_register() -> Self::RegId;

    /// Get flag register id.
    /// This panics if the arch does not have a flag register.
    fn get_flag_register() -> Self::RegId;

    /// Get register file description.
    fn get_register_file_desc() -> RegisterFileDesc;
}
