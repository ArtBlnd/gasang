use elf::endian::AnyEndian;
use elf::ElfBytes;

use machineinstr::aarch64::{AArch64Instr, AArch64InstrParserRule};
use machineinstr::utils::BitReader;
use machineinstr::MachineInstParser;

use vm::aarch64::AArch64Translater;
use vm::register::RegId;
use vm::VmState;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let path = std::path::PathBuf::from(filename);

    let file_data = std::fs::read(path).unwrap();

    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).unwrap();

    let text_section = file
        .section_header_by_name(".text")
        .expect("section table should be parseable")
        .expect("file should have a .text section");

    let ep_offset = text_section.sh_offset as usize;
    let ep_size = text_section.sh_size as usize;
    let ep_addr = text_section.sh_addr as u64;

    let buf = &slice[ep_offset..(ep_offset + ep_size)];

    // Initialize VM State.
    let mut vm_state = VmState::new();
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

    vm_state
        .get_gpr_register(pc_reg)
        .unwrap()
        .set(ep_addr as u64);

    let instr_translator = AArch64Translater {
        gpr_registers,
        fpr_registers,
        pc_reg,
        pstate_reg,
    };

    let mut vm_instr = Vec::new();
    for instr in MachineInstParser::new(BitReader::new(buf.iter().cloned()), AArch64InstrParserRule)
    {
        for instr in instr_translator.translate(instr) {
            vm_instr.push(instr);
        }
    }

    while true {
        println!("{:?}", vm_state.run(&vm_instr));
        vm_state.incrase_ip();
        vm_state.dump();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }

    todo!()
}
