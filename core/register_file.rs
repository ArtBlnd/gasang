use std::collections::HashMap;

use crate::RawRegisterId;

#[derive(Clone)]
pub struct RegisterFileDesc {
    register: HashMap<RawRegisterId, RegisterDesc>,
}

impl RegisterFileDesc {
    pub fn register(&self, id: RawRegisterId) -> &RegisterDesc {
        self.register.get(&id).unwrap()
    }

    pub fn total_size(&self) -> usize {
        self.register
            .values()
            .map(|r| r.offset + r.size)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone)]
pub struct RegisterDesc {
    pub is_read_only: bool,
    pub size: usize,
    pub offset: usize,
}
