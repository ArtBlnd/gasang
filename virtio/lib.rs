mod console;
mod fdt;

pub trait VirtIo {
    fn init(&self);
    fn irq(&self, irq: u32);
}
