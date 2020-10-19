use cargo_what::what;

macro_rules! macro1 {
    ($arg:expr) => {
        let a: u32 = $arg;
    }
}

macro_rules! macro2 {
    ($arg:expr) => {
        let a: u32 = what!($arg);
    }
}


#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    let a = 1u8;
    //= `u32`
    //= a is `u8`
    macro1!(what!(a));

    //= `u32`
    //= . is `u8`
    macro2!(a);
}
