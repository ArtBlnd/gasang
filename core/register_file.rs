use std::collections::HashMap;

use crate::RawRegisterId;

#[derive(Clone)]
pub struct RegisterFileDesc {
    total_size: usize,
    register: HashMap<RawRegisterId, RegisterDesc>,
}

impl RegisterFileDesc {
    pub fn size(&self) -> usize {
        self.total_size
    }

    pub fn register(&self, id: RawRegisterId) -> &RegisterDesc {
        self.register.get(&id).unwrap()
    }
}

#[derive(Clone)]
pub struct RegisterDesc {
    /// The name of the register
    pub size: usize,

    /// The offset of the register in the register file
    pub offset: usize,
}
