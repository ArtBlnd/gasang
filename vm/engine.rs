use crate::image::Image;

pub trait Engine {
    fn init(&mut self, image: Image);
    fn run(&mut self) -> u64;
}
