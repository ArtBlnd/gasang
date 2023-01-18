use crate::{FlagId, RegId, VmState, Interrupt};

pub type VmInstr = Box<dyn Fn(&mut VmState) -> Result<(), Interrupt>>;

pub fn gen_instr(instr: impl Fn(&mut VmState) -> Result<(), Interrupt> + 'static) -> VmInstr {
    Box::new(instr)
}

pub fn gen_add_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let rhs = state.get_gpr_register(rhs).unwrap().get();
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.add(rhs)?;
        return Ok(());
    })
}

pub fn gen_sub_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let rhs = state.get_gpr_register(rhs).unwrap().get();
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.sub(rhs)?;
        return Ok(());
    })
}

pub fn gen_div_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let rhs = state.get_gpr_register(rhs).unwrap().get();
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.div(rhs)?;
        return Ok(());
    })
}

pub fn gen_mul_reg_instr(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let rhs = state.get_gpr_register(rhs).unwrap().get();
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.mul(rhs)?;
        return Ok(());
    })
}

pub fn gen_mov_reg2reg(lhs: RegId, rhs: RegId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let rhs = state.get_gpr_register(rhs).unwrap().get();
        let lhs = state.get_gpr_register(lhs).unwrap();

        lhs.set(rhs);
        return Ok(());
    })
}

pub fn gen_move_reg2cst(lhs: RegId, rhs: usize) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let lhs = state.get_gpr_register(lhs).unwrap();
        lhs.set(rhs);
        return Ok(());
    })
}

pub fn gen_jmp(instr_n: usize) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        state.ip = instr_n;
        state.set_cf_modified();
        return Ok(());
    })
}

pub fn gen_jmp_if(instr_n: usize, flag: FlagId) -> VmInstr {
    gen_instr(move |state: &mut VmState| {
        let flag = state.get_flag(flag).unwrap();
        if flag {
            state.ip = instr_n;
            state.set_cf_modified();
        }

        return Ok(());
    })
}

pub fn gen_interrupt(value: usize) -> VmInstr {
    gen_instr(move |_: &mut VmState| return Err(Interrupt::Interrupt(value)))
}
