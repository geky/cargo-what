What?
=====

Tells you what type things are.

This crate provides the `what!` macro. This is functionally similar to the
[`todo!`] macro, except that it also tells you type information.

``` rust
fn hello() -> Result<(), Box<dyn Error>> {
    what!()
}
```

Just like [`todo!`], `what!` passes all type-checks and makes it easy to
write/build/test unfinished code. If it ever ends up in a compiled program,
attempted to execute a `what!` will panic.

The fun part happens when you run `cargo what`.

``` bash
$ cargo what
hole: expecting `std::result::Result<(), Box<dyn std::error::Error>>`
 --> src/hello.rs
  |
2 |     what!()
  |     ^^^^^^^
```

Unfortunately, custom diagnostics aren't really available to Rust libraries,
requiring the extra command. `cargo what` can be installed with `cargo`:

``` bash
$ cargo install cargo-what
```

`cargo what` wraps `cargo build` to show the type-info of any `what!`s
you have in your code.

`what!` also accepts arguments and shows their types, which can be useful
for reducing the "unused variable" noise.

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
  |
  = note: a is `usize`
  = note: b is `usize`
  = note: c is `std::ops::Range<usize>`
```

Emacs keybindings left as an exercise to the reader.

[`todo!`]: https://doc.rust-lang.org/std/macro.todo.html
