#[doc(hidden)]
pub use paste as __paste;

// cargo build => todo
#[cfg(not(what_query))]
#[macro_export]
macro_rules! what {
    ($($args:ident),* $(,)*) => {
        ({
            $(
                let _ = $args;
            )*
            todo!()
        })
    }
}

// cargo what => query type info
#[cfg(what_query)]
#[macro_export]
macro_rules! what {
    ($($args:ident),* $(,)*) => {
        ({
            todo!();

            #[allow(unreachable_code)]
            {
                $(
                    $crate::__paste::paste! {
                        trait [<What_ $args>] {};
                        let _: &dyn [<What_ $args>] = &$args;
                    }
                )*

                trait What {};
                let what;
                let _: &dyn What = &what;
                what
            }
        })
    }
}
