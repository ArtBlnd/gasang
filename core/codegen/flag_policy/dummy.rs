use crate::codegen::flag_policy::FlagPolicy;

#[derive(Clone)]
pub struct DummyFlagPolicy;
impl FlagPolicy for DummyFlagPolicy {
    fn carry(&self, _vm: &crate::Cpu) -> bool {
        todo!()
    }

    fn add_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &crate::Cpu) {
        todo!()
    }

    fn sub_carry(&self, _ty: crate::ir::Type, _a: u64, _b: u64, _vm: &crate::Cpu) {
        todo!()
    }
}
