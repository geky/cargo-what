use std::io::Write;

use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    //= `\[u8; 4\]`
    let x = u32::from_le_bytes(what!());

    //= `&\[std::io::IoSlice<'_>\]`
    let y = std::io::stdout().write_vectored(what!());

    //= `std::result::Result<&.*, std::boxed::Box<dyn std::error::Error>>
    let z: Result<&dyn std::any::Any, Box<dyn std::error::Error>> = what!();

    //= `\(\)`
    what!()
}
