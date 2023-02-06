use crate::codegen::flag_policy::FlagPolicy;

pub struct DummyFlagPolicy;
impl FlagPolicy for DummyFlagPolicy {
    fn carry(&self, vm: &mut crate::VmState) -> bool {
        todo!()
    }

    fn add_carry(&self, ty: crate::ir::Type, a: u64, b: u64, vm: &mut crate::VmState) {
        todo!()
    }

    fn sub_carry(&self, ty: crate::ir::Type, a: u64, b: u64, vm: &mut crate::VmState) {
        todo!()
    }
}
