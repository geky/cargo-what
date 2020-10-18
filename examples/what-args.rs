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
}
