use std::env;

// we need this to get any useful config into cargo
fn main() {
    // catch env changes
    println!("cargo:rerun-if-env-changed=CARGO_WHAT_QUERY");
    // set cfg
    if env::var("CARGO_WHAT_QUERY").is_ok() {
        println!("cargo:rustc-cfg=cargo_what_query");
    }
}
