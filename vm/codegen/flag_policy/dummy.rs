use crate::codegen::flag_policy::FlagPolicy;

pub struct DummyFlagPolicy;
impl FlagPolicy for DummyFlagPolicy {
    fn carry(&self, _vm: &crate::VmState) -> bool {
        todo!()
    }

    fn add_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &crate::VmState) {
        todo!()
    }

    fn sub_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &crate::VmState) {
        todo!()
    }
}
