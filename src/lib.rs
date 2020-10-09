pub use paste as __paste;

// cargo build => todo
#[cfg(not(what_query))]
#[macro_export]
macro_rules! what {
    ($($args:expr),* $(,)*) => {
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
    ($($args:expr),* $(,)*) => {
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
                let _v;
                let _: &dyn What = &_v;
                _v
            }
        })
    }
}
