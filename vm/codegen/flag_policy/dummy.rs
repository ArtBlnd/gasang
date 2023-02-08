use crate::codegen::flag_policy::FlagPolicy;

pub struct DummyFlagPolicy;
impl FlagPolicy for DummyFlagPolicy {
    fn carry(&self, _vm: &mut crate::VmState) -> bool {
        todo!()
    }

    fn add_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &mut crate::VmState) {
        todo!()
    }

    fn sub_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &mut crate::VmState) {
        todo!()
    }
}
