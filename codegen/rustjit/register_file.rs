use core::{RawRegisterId, RegisterFileDesc};

pub struct RegisterFile {
    desc: RegisterFileDesc,
    file: Box<[u8]>,
}

impl RegisterFile {
    pub fn new(desc: &RegisterFileDesc) -> Self {
        let mut file = Vec::new();
        file.resize(desc.size(), 0);
        Self {
            desc: desc.clone(),
            file: file.into_boxed_slice(),
        }
    }

    pub fn read(&self, id: RawRegisterId) -> &[u8] {
        let desc = self.desc.register(id);
        &self.file[desc.offset..desc.offset + desc.size]
    }

    pub fn write(&mut self, id: RawRegisterId, value: &[u8]) {
        let desc = self.desc.register(id);
        assert_eq!(desc.size, value.len());
        self.file[desc.offset..desc.offset + desc.size].copy_from_slice(value);
    }
}
