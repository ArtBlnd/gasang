use crate::{FlagId, RegId, VmCpu};

pub type VmInstr = Box<dyn Fn(&mut VmCpu) -> Result<(), usize>>;

pub fn gen_instr(instr: impl Fn(&mut VmCpu) -> Result<(), usize> + 'static) -> VmInstr {
    Box::new(instr)
}

pub fn gen_add_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let rhs = state.get_gpr_register(rhs).unwrap().value;
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.value += rhs;
        return Ok(());
    })
}

pub fn gen_sub_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let rhs = state.get_gpr_register(rhs).unwrap().value;
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.value -= rhs;
        return Ok(());
    })
}

pub fn gen_div_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let rhs = state.get_gpr_register(rhs).unwrap().value;
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.value /= rhs;
        return Ok(());
    })
}

pub fn gen_mul_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let rhs = state.get_gpr_register(rhs).unwrap().value;
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.value *= rhs;
        return Ok(());
    })
}

pub fn gen_mov_reg2reg(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let rhs = state.get_gpr_register(rhs).unwrap().value;
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.value = rhs;
        return Ok(());
    })
}

pub fn gen_move_reg2cst(lhs: RegId, rhs: usize) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let lhs = state.get_gpr_register(lhs).unwrap();
        lhs.value = rhs;
        return Ok(());
    })
}

pub fn gen_jmp(instr_n: usize) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        state.ip = instr_n;
        state.cf_modified = true;
        return Ok(());
    })
}

pub fn gen_jmp_if(instr_n: usize, flag: FlagId) -> VmInstr {
    gen_instr(move |state: &mut VmCpu| {
        let flag = state.get_flag(flag).unwrap();
        if flag {
            state.ip = instr_n;
            state.cf_modified = true;
        }

        return Ok(());
    })
}

pub fn gen_interrupt(value: usize) -> VmInstr {
    gen_instr(move |_: &mut VmCpu| return Err(value))
}
