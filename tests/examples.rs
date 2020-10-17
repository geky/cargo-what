use std::fs;
use std::process;

use anyhow;

// get available examples
fn examples() -> anyhow::Result<Vec<String>> {
    let mut paths: Vec<_> = fs::read_dir("./examples")?
        .filter_map(|f| f.ok())
        .filter(|e| {
            e.path().is_file()
                && e.file_name().to_str()
                    .filter(|name| name.ends_with(".rs"))
                    .is_some()
        })
        .map(|e| {
            e.file_name().to_str()
                .map(|name| {
                    String::from(&name[..name.len()-3])
                })
                .unwrap()
        })
        .collect();
    paths.sort();
    Ok(paths)
}


// compile with what
//
// (compiling without what already happens with cargo's default
// example building)
#[cfg_attr(not(feature = "build-binary"), ignore)]
#[test]
fn cargo_what() -> anyhow::Result<()> {
    for example in examples()? {
        let status = process::Command::new("./target/debug/cargo-what")
            .arg(format!("--example={}", example))
            .status()?;
        assert!(status.success());
    }

    Ok(())
}

