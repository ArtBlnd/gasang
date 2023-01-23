use elf::endian::AnyEndian;
use elf::ElfBytes;

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

    todo!();
}
