use std::mem::MaybeUninit;

pub fn make_array<T, const N: usize>(f: impl Fn(usize) -> T) -> [T; N] {
    let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for (i, v) in array.iter_mut().enumerate() {
        *v = MaybeUninit::new(f(i));
    }

    unsafe { std::mem::transmute_copy(&array) }
}
