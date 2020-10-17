use std::io::Write;

use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    let x = u32::from_le_bytes(what!());
    let y = std::io::stdout().write_vectored(what!());
    let z: Result<&dyn std::any::Any, Box<dyn std::error::Error>> = what!();
}
