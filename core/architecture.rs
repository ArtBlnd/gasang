use std::fmt::Debug;

use crate::{Instruction, Primitive, Register, RegisterFileDesc};

// The representation of an architecture
pub trait Architecture: Default + Clone + Copy + PartialEq + Eq {
    type Inst: Instruction;
    type Register: Register;

    fn get_register_by_name(name: impl AsRef<str>) -> Self::Register;

    type MnemonicHint: Debug;
    fn get_register_by_mnemonic(
        hint: Self::MnemonicHint,
        mnemonic: impl Primitive,
    ) -> Self::Register;

    /// Get pc register id.
    /// This panics if the arch does not have a pc register.
    fn get_pc_register() -> Self::Register;

    /// Get flag register id.
    /// This panics if the arch does not have a flag register.
    fn get_flag_register() -> Self::Register;

    /// Get register file description.
    fn get_register_file_desc() -> RegisterFileDesc;
}
