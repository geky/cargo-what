#[doc(hidden)]
pub use paste as __paste;

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
