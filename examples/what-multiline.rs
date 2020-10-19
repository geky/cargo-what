use cargo_what::what;

#[allow(unreachable_code)]
#[allow(unused_variables)]
fn main() {
    //= `_`
    //= 0 is `std::iter::Successors<\(u128, u128\), .*closure.*>`
    let x = what!(
        std::iter::successors(Some((1u128, 1u128)), |(a, b)| {
            a.checked_add(*b).map(|c| (*b, c))
        })
    );
}
