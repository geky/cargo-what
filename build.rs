use std::env;

// we need this to get any useful config into cargo
fn main() {
    // catch env changes
    println!("cargo:rerun-if-env-changed=WHAT_QUERY");
    // set cfg
    if env::var("WHAT_QUERY").is_ok() {
        println!("cargo:rustc-cfg=what_query");
    }
}
