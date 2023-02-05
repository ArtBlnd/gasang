use elf::endian::AnyEndian;
use elf::ElfBytes;

use std::path::PathBuf;

use vm::image::*;

fn main() {
    // Get first argument
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];

    let file_buf = std::fs::read(PathBuf::from(filename)).unwrap();
    let file_elf = ElfBytes::<AnyEndian>::minimal_parse(&file_buf).unwrap();

    let mut image = Image::from_image(file_buf.to_vec());

    let Ok((Some(sec_headers), Some(strtbl))) = file_elf.section_headers_with_strtab() else {
        panic!("Bad elf file!");
    };

    for sec in sec_headers {
        let name = strtbl.get(sec.sh_name as usize).unwrap();
        let beg = sec.sh_offset;
        let end = sec.sh_offset + sec.sh_size;

        if name == ".text" {
            image.set_entrypoint(sec.sh_addr);
        }

        image.add_section(name, sec.sh_addr, beg as usize, end as usize);
    }
}
