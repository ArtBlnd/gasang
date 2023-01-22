mod translate;
pub use translate::*;

use crate::{RegId, VmInstr, VmState};

use machineinstr::aarch64::*;
use machineinstr::utils::BitReader;
use machineinstr::MachineInstParser;

pub struct AArch64VM<I> {
    vm_state: VmState,

    parser: MachineInstParser<I, AArch64InstrParserRule>,
    translater: AArch64Translater,
}

impl<I> AArch64VM<I>
where
    I: Iterator<Item = u8>,
{
    pub fn new(binary: I) -> Self {
        let mut vm_state = VmState::new();

        // Initialize registers
        let mut gpr_registers: [RegId; 32] = [RegId(0); 32];
        let mut fpr_registers: [RegId; 32] = [RegId(0); 32];
        let pc_reg = vm_state.new_gpr_register("pc", 8);
        let pstate_reg = vm_state.new_gpr_register("pstate", 8);

        for i in 0..32 {
            gpr_registers[i] = vm_state.new_gpr_register(format!("x{}", i), 8);
        }

        for i in 0..32 {
            fpr_registers[i] = vm_state.new_fpr_register(format!("x{}", i), 8);
        }

        let parser = MachineInstParser::new(BitReader::new(binary), AArch64InstrParserRule);

        // Initialize translater
        let translater = AArch64Translater {
            gpr_registers,
            fpr_registers,
            pc_reg,
            pstate_reg,
        };

        Self {
            vm_state,
            parser,
            translater,
        }
    }
}
