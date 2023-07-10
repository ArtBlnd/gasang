use std::convert::Infallible;

use arch_desc::aarch64::AArch64Architecture;
use execution::{abi::AArch64UnknownLinux, codegen::rustjit::RustjitCodegen, Runtime};

fn main() {
    let file = std::fs::read("./examples/main").unwrap();

    unsafe {
        execute_runtime(&file);
    }
}

unsafe fn execute_runtime(file: &[u8]) -> Infallible {
    type Arch = AArch64Architecture;
    type Codegen = RustjitCodegen;
    type Abi = AArch64UnknownLinux;

    Runtime::run::<Arch, Codegen, Abi>(&file, |_, _| {})
}
