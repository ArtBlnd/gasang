use elf::endian::AnyEndian;
use elf::ElfBytes;

use machineinstr::aarch64::AArch64InstrParserRule;
use machineinstr::MachineInstParser;
use utility::BitReader;

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

    let buf = &slice[ep_offset..(ep_offset + ep_size)];

    let reader = BitReader::new(buf.iter().cloned());
    let parser = MachineInstParser::new(reader, AArch64InstrParserRule);

    let mut n = 0;
    let start = std::time::Instant::now();
    for _inst in parser {
        n += 1;
        // println!("{:08}: {:?}", n, inst);
    }
    let end = std::time::Instant::now();

    println!("{n} {:?}", end - start);
}
