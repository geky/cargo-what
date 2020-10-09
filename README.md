What!
=====

This crate is a sort of expiriment in type-driven development.

This crate provides the `what!` macro, which very similar to Rust's `todo!`
macro.

``` rust
fn hello() -> Result<(), Box<dyn Error>> {
    what!()
}
```

Just like the `todo!` macro, the `what!` macro passes all typechecks to make
it easy to write/build/test unfinished code. If it ever ends up in a compiled
program, attempting to execute a `what!` macro will panic.

The fun part happens when you use the related `cargo-what` binary (this may
require a `cargo install`).

``` bash
$ cargo what
hole: expecting `std::result::Result<(), Box<dyn std::error::Error>>`
 --> src/hello.rs
  |
2 |     what!()
  |     ^^^^^^^
```

Now you can see the type-info of any `what!` macros you have in your program.

The `what!` macro can also except arguments, which helps reduce
the "unused variable" noise common to traditional `todo!` macros

``` rust
fn hello(a: usize, b: usize) -> usize {
    let c = a..b;
    what!(a, b, c)
}
```

And with `cargo what`:

```bash
$ cargo what
hole: expecting `usize`
 --> src/hello.rs
  |
3 |     what!(a, b, c)
  |     ^^^^^^^^^^^^^^
  = note: a is `usize`
  = note: b is `usize`
  = note: c is `std::ops::Range<usize>`
```

Emacs keybindings left as an exercise to the reader~
