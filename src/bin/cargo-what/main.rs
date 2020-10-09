use std::env;
use std::process;
use std::io;
use std::io::BufRead;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::cmp;

use anyhow::anyhow;
use atty;
use serde_json;
use regex;
use lazy_static::lazy_static;
use colored::Colorize;


#[derive(Debug)]
struct Hole {
    ty: Option<String>,
    args: Vec<(String, String)>,
    file: String,
    line: usize,
    context: String,
}


fn parse_message(line: &str) -> Option<serde_json::Value> {
    if !line.starts_with("{") {
        return None;
    }

    let mut message: serde_json::Value
        = serde_json::from_str(line).ok()?;
    if message.get("reason")? == "compiler-message" {
        let message = message.get_mut("message")?.take();
        if message.get("code")?.get("code")? == "E0277" {
            return Some(message)
        }
    }

    None
}

fn parse_hole(
    message: serde_json::Value,
    holes: &mut HashMap<(String, usize), Hole>
) -> Option<()> {
    // parse out expected type
    lazy_static! {
        static ref TYPE_PATTERN: regex::Regex = regex::Regex::new(
            r"`(.*): .*::What(?:_([[:word:]]*))?`"
        ).unwrap();
    }

    let short = message.get("message")?.as_str()?;
    let ma = TYPE_PATTERN.captures(short)?;
    let ty = ma.get(1)?.as_str();
    let arg = ma.get(2).map(|arg| arg.as_str());

    let long = message.get("rendered")?.as_str()?;
    let long = long.lines()
        .skip(1).take(4)
        .collect::<Vec<_>>()
        .join("\n");

    let span = message.get("spans")?.get(0)?.get("expansion")?.get("span")?;
    let file = span.get("file_name")?.as_str()?;
    let line = usize::try_from(span.get("line_start")?.as_u64()?).ok()?;

    // lookup hole?
    let hole = holes.entry((String::from(file), line)).or_insert_with(|| {
        Hole {
            ty: None,
            args: Vec::new(),
            file: String::from(file),
            line: line,
            context: long,
        }
    });

    if let Some(arg) = arg {
        hole.args.push((String::from(arg), String::from(ty)));
    } else {
        hole.ty = Some(String::from(ty));
    }

    Some(())
}

fn render_hole(hole: &Hole) -> Option<()> {
    lazy_static! {
        static ref SPAN_PATTERN: regex::Regex = regex::Regex::new(
            r"[ 0-9-]+[|>]"
        ).unwrap();
        static ref SQUIGGLE_PATTERN: regex::Regex = regex::Regex::new(
            r"\^+"
        ).unwrap();
    }

    eprintln!(
        "{}{}",
        "hole".bright_magenta(),
        format!(": expecting `{}`", hole.ty.as_ref()?).bright_white()
    );

    let lines: Vec<_> = hole.context.lines().collect();
    let mut max_span_len = 0;
    for (i, line) in lines.iter().enumerate() {
        let squiggle = if i == lines.len()-1 {
            SQUIGGLE_PATTERN.find(line)
        } else {
            None
        };

        if let Some(ma) = SPAN_PATTERN.find(line) {
            max_span_len = cmp::max(max_span_len, ma.end());
            eprintln!(
                "{}{}{}",
                ma.as_str().bright_blue(),
                &line[
                    ma.end()..squiggle
                        .map(|ma| ma.start())
                        .unwrap_or(line.len())
                ],
                squiggle
                    .map(|ma| ma.as_str())
                    .unwrap_or("")
                    .bright_magenta()
            );
        } else {
            eprintln!("{}", line);
        }
    }
    max_span_len -= 1;

    for (name, ty) in &hole.args {
        eprintln!(
            "{:>max_span_len$} {}{}",
            "=".bright_blue(),
            "note".bright_white(),
            format!(": {} is `{}`", name, ty),
            max_span_len=max_span_len
        )
    }

    eprintln!();

    Some(())
}


fn what(args: std::env::Args) -> anyhow::Result<()> {
    // get args, but also remove "cargo" and "what"
    let mut seen_dash_dash = false;
    let args: Vec<_> = args
        .skip(1)
        .filter(|arg| {
            if arg == "--" {
                seen_dash_dash = true;
            }
            seen_dash_dash || arg != "what"
        })
        .collect();

    // catch version
    if args.iter().any(|arg| arg == "--version") {
        println!("cargo-what {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // catch --message-format (breaks things)
    if args.iter().any(|arg| arg.starts_with("--message-format")) {
        return Err(anyhow!("--message-format not supported"));
    }

    // need color?
    let color = args.iter().find_map(|arg| {
            if arg.starts_with("--color") {
                if arg.find("always").is_some() {
                    Some(true)
                } else if arg.find("never").is_some() {
                    Some(false)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            atty::is(atty::Stream::Stderr)
        });
    colored::control::set_override(color);

    // we are running in cargo what
    env::set_var("WHAT_QUERY", "1");

    // run cargo build
    let mut build = process::Command::new("cargo")
        .arg("build")
        .args(&args)
        .arg("--message-format=json")
        .args(
            if color && !args.iter().any(|arg| arg.starts_with("--color")) {
                &["--color=always"][..]
            } else {
                &[][..]
            }
        )
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()?;
    let stdout = build.stdout.take().unwrap();
    let stderr = build.stderr.take().unwrap();

    // sort of a hack, we filter output so "could not compile" doesn't
    // get printed all the time
    let stdout_thread = thread::spawn(|| {
        for line in io::BufReader::new(stderr).lines() {
            let line = match line {
                Ok(line) => line,
                _ => break,
            };

            if line.find("  ").is_some() {
                eprintln!("{}", line);
            }
        }
    });

    let mut holes: HashMap::<(String, usize), Hole> = HashMap::new();
    for line in io::BufReader::new(stdout).lines() {
        let line = match line {
            Ok(line) => line,
            _ => break,
        };

        if let Some(message) = parse_message(&line) {
            // yes we ignore errors here
            let _ = parse_hole(message, &mut holes);
        }
    }

    build.wait()?;
    stdout_thread.join().unwrap();

    // display holes
    let mut holes: Vec<_> = holes.values().collect();
    holes.sort_by_key(|hole| (&hole.file, hole.line));
    for hole in holes {
        let _ = render_hole(hole);
    }

    Ok(())
}

fn main() {
    let res = what(env::args());
    match res {
        Ok(()) => {
            process::exit(0);
        }
        Err(err) => {
            eprintln!("error: {}", err);
            process::exit(1);
        }
    }
}
