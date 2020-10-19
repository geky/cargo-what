use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    //= `_`
    //= a is `u8`
    //= b is `u32`
    //= c is `usize`
    let a = 1u8;
    let b = 1u32;
    let c = 1usize;
    what!(a, b, c);

    //= `_`
    //= a is `std::result::Result<&.*, std::boxed::Box<dyn std::error::Error>>`
    let a: Result<&dyn std::any::Any, Box<dyn std::error::Error>> = Ok(&1u8);
    what!(a);

    //= `_`
    //= 0 is `u8`
    //= 1 is `u16`
    //= 2 is `u32`
    //= 3 is `u64`
    //= 4 is `u128`
    what!(1u8, 2u16, 3u32, 4u64, 5u128);

    //= `_`
    //= 0 is `u8`
    //= 1 is `std::option::Option<u16>`
    //= 2 is `&u32`
    what!(1u8+1, Some(2u16), &3u32);
}
