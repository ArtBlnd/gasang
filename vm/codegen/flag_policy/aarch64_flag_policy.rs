use crate::codegen::flag_policy::FlagPolicy;
use crate::ir::Type;
use crate::VmState;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AArch64FlagPolicy;

impl FlagPolicy for AArch64FlagPolicy {
    fn add_carry(&self, ty: Type, a: u64, b: u64, vm: &VmState) {
        let (n, z, c, v) = match ty {
            Type::U8 | Type::I8 => {
                let ua = a as u8;
                let ub = b as u8;
                let sa = a as i8;
                let sb = b as i8;

                let carry_in = if self.carry(vm) { 1 } else { 0 };

                let (ua, mut c) = ua.overflowing_add(carry_in);
                let (sa, mut v) = sa.overflowing_add_unsigned(carry_in);

                let (uresult, c2) = ua.overflowing_add(ub);
                let (sresult, v2) = sa.overflowing_add(sb);

                c |= c2;
                v |= v2;

                let n = sresult < 0;
                let z = uresult == 0;

                (n, z, c, v)
            }
            Type::U16 | Type::I16 => {
                let ua = a as u16;
                let ub = b as u16;
                let sa = a as i16;
                let sb = b as i16;

                let carry_in = if self.carry(vm) { 1 } else { 0 };

                let (ua, mut c) = ua.overflowing_add(carry_in);
                let (sa, mut v) = sa.overflowing_add_unsigned(carry_in);

                let (uresult, c2) = ua.overflowing_add(ub);
                let (sresult, v2) = sa.overflowing_add(sb);

                c |= c2;
                v |= v2;

                let n = sresult < 0;
                let z = uresult == 0;

                (n, z, c, v)
            }
            Type::U32 | Type::I32 => {
                let ua = a as u32;
                let ub = b as u32;
                let sa = a as i32;
                let sb = b as i32;

                let carry_in = if self.carry(vm) { 1 } else { 0 };

                let (ua, mut c) = ua.overflowing_add(carry_in);
                let (sa, mut v) = sa.overflowing_add_unsigned(carry_in);

                let (uresult, c2) = ua.overflowing_add(ub);
                let (sresult, v2) = sa.overflowing_add(sb);

                c |= c2;
                v |= v2;

                let n = sresult < 0;
                let z = uresult == 0;

                (n, z, c, v)
            }
            Type::U64 | Type::I64 => {
                let ua = a;
                let ub = b;
                let sa = a as i64;
                let sb = b as i64;

                let carry_in = if self.carry(vm) { 1 } else { 0 };

                let (ua, mut c) = ua.overflowing_add(carry_in);
                let (sa, mut v) = sa.overflowing_add_unsigned(carry_in);

                let (uresult, c2) = ua.overflowing_add(ub);
                let (sresult, v2) = sa.overflowing_add(sb);

                c |= c2;
                v |= v2;

                let n = sresult < 0;
                let z = uresult == 0;

                (n, z, c, v)
            }
            Type::F32 | Type::F64 => unimplemented!("Float type is not supported!"),
            Type::Void => panic!("Void type is not supported!"),
            Type::Bool => panic!("Bool type is not supported!"),
        };

        let (n, z, c, v): (u64, u64, u64, u64) = (n.into(), z.into(), c.into(), v.into());
        vm.add_flag(n << 63 | z << 62 | c << 61 | v << 60)
    }

    fn sub_carry(&self, ty: Type, a: u64, b: u64, vm: &VmState) {
        self.add_carry(ty, a, (-(b as i64)) as u64, vm)
    }

    fn carry(&self, vm: &VmState) -> bool {
        ((vm.flag() >> 61) & 1) == 1
    }
}
