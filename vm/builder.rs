use crate::ir::*;

pub struct VMBuilder {
    
}

impl VMBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn allocate_flag(&mut self, name: &str) -> FlagId {
        todo!()
    }

    pub fn allocate_register(&mut self, name: &str) -> RegId {
        todo!()
    }

    pub fn allocate_literal(&mut self, loc: usize, name: &str) -> usize {
        todo!()
    }
}