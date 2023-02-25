use std::collections::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Section {
    sec_addr: u64,

    beg: usize,
    end: usize,

    writable: bool,
    executable: bool,
}

pub struct Image {
    image: Vec<u8>,
    image_code_entry: u64,

    sections: HashMap<String, Section>,
}

impl Image {
    pub fn from_image(image: Vec<u8>) -> Self {
        Self {
            image,
            image_code_entry: 0,

            sections: HashMap::new(),
        }
    }

    pub fn set_entrypoint(&mut self, ep: u64) -> &mut Self {
        self.image_code_entry = ep;
        self
    }

    pub fn entrypoint(&self) -> u64 {
        self.image_code_entry
    }

    // Add secment into image
    pub fn add_section(
        &mut self,
        sec_name: impl AsRef<str>,
        sec_addr: u64,
        writable: bool,
        executable: bool,
        beg: usize,
        end: usize,
    ) -> &mut Self {
        self.sections.insert(
            sec_name.as_ref().to_string(),
            Section {
                sec_addr,
                writable,
                executable,
                beg,
                end,
            },
        );

        self
    }

    pub fn sections(&self) -> impl Iterator<Item = &str> {
        self.sections.keys().map(|v| v.as_str())
    }

    pub fn section_addr(&self, name: impl AsRef<str>) -> u64 {
        self.sections.get(name.as_ref()).unwrap().sec_addr
    }

    pub fn section_data(&self, name: impl AsRef<str>) -> &[u8] {
        let sec = self.sections.get(name.as_ref()).unwrap();
        &self.image[sec.beg..sec.end]
    }

    pub fn section_access_info(&self, name: impl AsRef<str>) -> (bool, bool) {
        let sec = self.sections.get(name.as_ref()).unwrap();

        (sec.writable, sec.executable)
    }
}
