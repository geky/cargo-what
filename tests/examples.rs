use std::fs;
use std::process;
use std::io;
use std::io::BufRead;
use std::io::Write;

use anyhow;
use regex;
use lazy_static::lazy_static;

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

// read expected output in the examples, these
// are specified by any comments preceded by `//=`
fn read_expected(example: &str) -> anyhow::Result<Vec<regex::Regex>> {
    lazy_static! {
        static ref EXPECTED_PATTERN: regex::Regex = regex::Regex::new(
            "//= *(.*?) *$"
        ).unwrap();
    }

    let mut expected = Vec::new();
    let f = fs::File::open(format!("./examples/{}.rs", example))?;
    for line in io::BufReader::new(f).lines() {
        if let Some(cap) = EXPECTED_PATTERN.captures(&line?) {
            expected.push(regex::Regex::new(cap.get(1).unwrap().as_str())?);
        }
    }

    Ok(expected)
}

// compile with what
//
// (compiling without what already happens with cargo's default
// example building)
#[cfg_attr(not(feature = "build-bin"), ignore)]
#[test]
fn cargo_what() -> anyhow::Result<()> {
    for example in examples()? {
        let expected = read_expected(&example)?;

        let output = process::Command::new("./target/debug/cargo-what")
            .arg("-q")
            .arg(format!("--example={}", example))
            .output()?;

        // print output for debugging (these tests are opt-in anyways)
        io::stdout().write_all(&output.stderr)?;

        // scan for expected outputs in order
        for pattern in expected {
            assert!(
                output.stderr.lines()
                    .filter_map(|line| line.ok())
                    .any(|line| pattern.is_match(&line)),
                "expected {:?}",
                format!("{}", pattern)
            );
        }

        assert!(output.status.success());
    }

    Ok(())
}

