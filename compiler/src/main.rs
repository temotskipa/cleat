use cleatc::Diagnostic;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: cleatc check <path>...\n       cleatc build <path>... --entry pkg.Type -o out.exe [--link file]...";

fn show(d: &Diagnostic, warning: bool) {
    let kind = if warning { "warning: " } else { "" };
    eprintln!("{}:{}:{}: {kind}{}", d.file.display(), d.line, d.column, d.message);
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let rest: Vec<String> = args.collect();
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut entry = None;
    let mut output = None;
    let mut link: Vec<PathBuf> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--entry" => {
                i += 1;
                entry = rest.get(i).cloned();
            }
            "-o" => {
                i += 1;
                output = rest.get(i).cloned();
            }
            "--link" => {
                i += 1;
                match rest.get(i) {
                    Some(p) => link.push(PathBuf::from(p)),
                    None => {
                        eprintln!("{USAGE}");
                        return ExitCode::from(2);
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("unknown argument {other}\n{USAGE}");
                return ExitCode::from(2);
            }
            path => roots.push(PathBuf::from(path)),
        }
        i += 1;
    }
    if roots.is_empty() {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let paths: Vec<&Path> = roots.iter().map(|p| p.as_path()).collect();
    let result = match command.as_str() {
        "check" => cleatc::analyze(&paths).map(|mut p| std::mem::take(&mut p.warnings)),
        "build" => {
            let (Some(entry), Some(output)) = (entry, output) else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            cleatc::build_with(&paths, &entry, Path::new(&output), &link)
        }
        other => {
            eprintln!("unknown command {other}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(warnings) => {
            for w in &warnings {
                show(w, true);
            }
            ExitCode::from(0)
        }
        Err(errors) => {
            for err in &errors {
                show(err, false);
            }
            ExitCode::from(1)
        }
    }
}
