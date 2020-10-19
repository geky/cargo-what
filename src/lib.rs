//! Tells you what type things are.
//!
//! This crate provides the `what!` macro. This is functionally similar to the
//! [`todo!`] macro, except that it also tells you type information.
//!
//! ``` rust
//! # use cargo_what::what;
//! # use std::error::Error;
//! fn hello() -> Result<(), Box<dyn Error>> {
//!     what!()
//! }
//! ```
//!
//! Just like [`todo!`], `what!` passes all type-checks and makes it easy to
//! write/build/test unfinished code. If it ever ends up in a compiled program,
//! attempted to execute a `what!` will panic.
//!
//! The fun part happens when you run `cargo what`.
//!
//! ``` bash
//! $ cargo what
//! hole: expecting `std::result::Result<(), Box<dyn std::error::Error>>`
//!  --> src/hello.rs
//!   |
//! 2 |     what!()
//!   |     ^^^^^^^
//! ```
//!
//! Unfortunately, custom diagnostics aren't really available to Rust libraries,
//! requiring the extra command. `cargo what` can be installed with `cargo`:
//!
//! ``` bash
//! $ cargo install cargo-what
//! ```
//!
//! `cargo what` wraps `cargo build` to show the type-info of any `what!`s
//! you have in your code.
//!
//! `what!` also accepts arguments and shows their types, which can be useful
//! for reducing the "unused variable" noise.
//!
//! ``` rust
//! # use cargo_what::what;
//! fn hello(a: usize, b: usize) -> usize {
//!     let c = a..b;
//!     what!(a, b, c)
//! }
//! ```
//!
//! And with `cargo what`:
//!
//! ```bash
//! $ cargo what
//! hole: expecting `usize`
//!  --> src/hello.rs
//!   |
//! 3 |     what!(a, b, c)
//!   |     ^^^^^^^^^^^^^^
//!   |
//!   = note: a is `usize`
//!   = note: b is `usize`
//!   = note: c is `std::ops::Range<usize>`
//! ```
//!
//! Emacs keybindings left as an exercise to the reader.
//!
//! [`todo!`]: https://doc.rust-lang.org/std/macro.todo.html
//!

// we need this for token pasting
#[doc(hidden)]
pub use paste as __paste;


/// This is the core `what!` macro.
///
/// It behaves similarly to [`todo!`], passes all type-checks,
/// and panics if executed.
///
/// ```rust
/// # use cargo_what::what;
/// # use std::error::Error;
/// fn hello() -> Result<(), Box<dyn Error>> {
///     what!()
/// }
/// ```
///
/// One difference from [`todo!`] is that `what!` also accepts
/// arbitrary arguments, which can help reduce "unused variable"
/// noise.
///
/// ``` rust
/// # use cargo_what::what;
/// fn hello(a: usize, b: usize) -> usize {
///     let c = a..b;
///     what!(a, b, c)
/// }
/// ```
///
/// See the [crate-level documentation](/cargo_what) for more info
/// on how you can show the type-info of `what!`s in a program.
///
/// [`todo!`]: https://doc.rust-lang.org/std/macro.todo.html
///

// cargo build => todo
#[cfg(not(cargo_what_query))]
#[macro_export]
macro_rules! what {
    ($($args:expr),* $(,)*) => {
        ({
            $(
                let _ = $args;
            )*
            todo!()
        })
    };
}

// cargo what => query type info
#[cfg(cargo_what_query)]
#[macro_export]
macro_rules! what {
    (@[$($n:expr)*] $(,)*) => {};
    (@[$($n:expr)*] $arg:ident, $($args:tt)*) => {
        // we can use ident directly as a special case
        $crate::__paste::paste! {
            trait [<What_ $arg>] {};
            let _: &dyn [<What_ $arg>] = &$arg;
        }
        what!(@[$($n)* 0] $($args)*);
    };
    (@[$($n:expr)*] $arg:expr, $($args:tt)*) => {
        $crate::__paste::paste! {
            trait [<What_ $($n)*>] {};
            // yes you need these parens (tt munching?)
            let _: &dyn [<What_ $($n)*>] = &($arg);
        }
        what!(@[$($n)* 0] $($args)*);
    };
    // actual macro
    ($($args:tt)*) => {
        ({
            todo!();

            #[allow(unreachable_code)]
            {
                what!(@[0] $($args)*,);

                trait What {};
                match true {
                    true => {
                        let what;
                        let _: &dyn What = &what;
                        what
                    }
                    false => {
                        struct WhatTrait {};
                        impl What for WhatTrait {};
                        WhatTrait{}
                    }
                }
            }
        })
    };
}
