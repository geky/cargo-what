#[allow(unused_imports)]
use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    // It turns out unresolved traits compile error even
    // with the todo! macro. what! can still find the trait
    // info, but the program won't be able to compile.
    #[cfg(cargo_what_query)]
    {
        let x = std::fs::File::create(what!());
        let y = format!("{}", what!());
        let z = std::iter::repeat_with(what!());
    }
}
