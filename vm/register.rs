pub struct GprRegister {
    pub name: String,
    pub size: u8,
    pub value: usize,
}

pub struct FprRegister {
    pub name: String,
    pub size: u8,
    pub value: f64,
}
