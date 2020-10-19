use std::env;
use std::process;
use std::io;
use std::io::BufRead;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::thread;
use std::cmp;
use std::ffi;

use anyhow::anyhow;
use atty;
use serde_json;
use regex;
use lazy_static::lazy_static;
use colored::Colorize;


#[derive(Debug)]
struct Hole {
    ty: Option<String>,
    tr: Option<String>,
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
            "(?:\
                `(.*): .*::What(?:_([a-zA-Z_][a-zA-Z_0-9]*)|_(0+))?`\
                |`.*::WhatTrait: (.*)`\
                |`.*::WhatTrait` doesn't implement `(.*)`\
                |expected.*`(.*)`.*found `.*::WhatTrait`\
            )"
        ).unwrap();
        static ref SITE_PATTERN: regex::Regex = regex::Regex::new(
            "what!\\((?:[^@]|$)"
        ).unwrap();
    }

    let short = message.get("message")?.as_str()?;
    let ma = TYPE_PATTERN.captures(short)?;
    let arg = ma.get(2)
        .map(|arg| format!("{}", arg.as_str()))
        .or_else(|| {
            ma.get(3)
                .map(|arg| format!("{}", arg.range().len()-1))
        });
    let ty = ma.get(1);
    let tr = ma.get(4).or(ma.get(5)).or(ma.get(6));

    let long = message.get("rendered")?.as_str()?;
    let long = long.lines()
        .skip(1).take(4)
        .collect::<Vec<_>>()
        .join("\n");

    let mut span = message.get("spans")?.get(0)?;
    let is_call_site = |span: &serde_json::Value| -> bool {
        span.get("text")
            .and_then(|span| span.as_array())
            .filter(|span| {
                span.iter()
                    .any(|text| {
                        text.get("text")
                            .and_then(|text| text.as_str())
                            .filter(|text| SITE_PATTERN.is_match(text))
                            .is_some()
                    })
            })
            .is_some()
    };
    while !is_call_site(span) {
        span = span.get("expansion")?.get("span")?;
    }

    let file = span.get("file_name")?.as_str()?;
    let line = usize::try_from(span.get("line_start")?.as_u64()?).ok()?;

    // lookup hole?
    let hole = holes.entry((String::from(file), line)).or_insert_with(|| {
        Hole {
            ty: None,
            tr: None,
            args: Vec::new(),
            file: String::from(file),
            line: line,
            context: long,
        }
    });

    match (arg, ty, tr) {
        (Some(arg), Some(ty), _) => {
            hole.args.push((arg, String::from(ty.as_str())));
        }
        (_, Some(ty), _) => {
            hole.ty = Some(String::from(ty.as_str()));
        }
        (_, _, Some(tr)) => {
            hole.tr = Some(String::from(tr.as_str()));
        }
        _ => {
            // do nothing (bad parse?)
        }
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
        format!(
            ": expecting `{}`",
            match (&hole.ty, &hole.tr) {
                (Some(ty), _) => format!("{}", ty),
                (_, Some(tr)) => format!("impl {}", tr),
                _             => format!("_"),
            }
        ).bright_white()
    );

    let lines: Vec<_> = hole.context.lines().collect();
    let mut max_span_len = 0;
    for (i, line) in lines.iter().take(3).enumerate() {
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

    let squiggle = lines.get(3)
        .and_then(|line| {
            SQUIGGLE_PATTERN.find(line)
                .map(|ma| match ma.range() {
                    squiggle if squiggle.len() == 1 => {
                        squiggle.start..lines[2].len()
                    }
                    squiggle => {
                        squiggle
                    }
                })
        });

    if let Some(squiggle) = squiggle {
        eprintln!(
            "{:>max_span_len$}{:>squiggle_len$}",
            "|".bright_blue(),
            "^".repeat(squiggle.len()).bright_magenta(),
            max_span_len=max_span_len,
            squiggle_len=squiggle.end-max_span_len
        );
    }

    if hole.args.len() > 0 {
        eprintln!(
            "{:>max_span_len$}",
            "|".bright_blue(),
            max_span_len=max_span_len
        );
    }

    for (name, ty) in &hole.args {
        eprintln!(
            "{:>max_span_len$} {}{}",
            "=".bright_blue(),
            "note".bright_white(),
            format!(": {} is `{}`", name, ty),
            max_span_len=max_span_len
        );
    }

    eprintln!();

    Some(())
}

// helper to add conditional args
trait CommandExt {
    fn arg_if<S: AsRef<ffi::OsStr>>(
        &mut self,
        pred: bool,
        arg: S
    ) -> &mut Self;
}

impl CommandExt for process::Command {
    fn arg_if<S: AsRef<ffi::OsStr>>(
        &mut self,
        pred: bool,
        arg: S
    ) -> &mut Self {
        if pred {
            self.arg(arg)
        } else {
            self
        }
    }
}

fn what(args: std::env::Args) -> anyhow::Result<()> {
    // get args, but also remove "cargo" and "what"
    let mut seenwhat: bool = false;
    let mut dashdash: Option<usize> = None;
    let mut color: Option<bool> = None;
    let mut filteredargs: Vec<String> = Vec::new();
    for (i, arg) in args.skip(1).enumerate() {
        match arg.as_str() {
            _ if dashdash.is_some() => {
                filteredargs.push(arg);
            }
            "what" if !seenwhat => {
                // skip
                seenwhat = true;
            },
            "--" => {
                dashdash = Some(i);
            }
            "--version" => {
                // catch version
                println!("cargo-what {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "-h" | "--help" => {
                // catch help, kinda
                process::Command::new("cargo").arg("build")
                    .arg("--help")
                    .status()?;
                return Ok(());
            }
            arg if arg.starts_with("--color") => {
                if arg.find("always").is_some() {
                    color = Some(true);
                } else if arg.find("never").is_some() {
                    color = Some(false);
                }
            }
            arg if arg.starts_with("--messsage-format") => {
                // catch --message-format (breaks things)
                return Err(anyhow!("--message-format not supported"));
            }
            _ => {
                filteredargs.push(arg);
            }
        }
    }

    // need color?
    let color = color.unwrap_or_else(|| {
            atty::is(atty::Stream::Stderr)
        });
    colored::control::set_override(color);

    // we are running in cargo what
    env::set_var("CARGO_WHAT_QUERY", "1");

    // run cargo build
    let mut build = process::Command::new("cargo")
        .arg("build")
        .args(&filteredargs)
        .arg("--message-format=json")
        .arg_if(color, "--color=always")
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
