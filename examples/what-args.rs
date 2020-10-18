use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    let a = 1u8;
    let b = 1u32;
    let c = 1usize;
    what!(a, b, c);

    let a: Result<&dyn std::any::Any, Box<dyn std::error::Error>> = Ok(&1u8);
    what!(a);

    what!(1u8, 2u16, 3u32, 4u64, 5u128);

    what!(1u8+1, Some(2u16), &3u32);
}
