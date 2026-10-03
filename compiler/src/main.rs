use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(command) = args.next() else {
        eprintln!(
            "usage: cleatc check <project> | cleatc build <project> --entry pkg.Type -o out.exe"
        );
        return ExitCode::from(2);
    };
    let result = match command.as_str() {
        "check" => {
            let Some(root) = args.next() else {
                eprintln!("usage: cleatc check <project>");
                return ExitCode::from(2);
            };
            cleatc::check(PathBuf::from(root).as_path())
        }
        "build" => {
            let Some(root) = args.next() else {
                eprintln!("usage: cleatc build <project> --entry pkg.Type -o out.exe");
                return ExitCode::from(2);
            };
            let mut entry = None;
            let mut output = None;
            let rest: Vec<String> = args.collect();
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
                    other => {
                        eprintln!("unknown argument {other}");
                        return ExitCode::from(2);
                    }
                }
                i += 1;
            }
            let (Some(entry), Some(output)) = (entry, output) else {
                eprintln!("usage: cleatc build <project> --entry pkg.Type -o out.exe");
                return ExitCode::from(2);
            };
            cleatc::build(
                PathBuf::from(root).as_path(),
                &entry,
                PathBuf::from(output).as_path(),
            )
        }
        other => {
            eprintln!("unknown command {other}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::from(0),
        Err(errors) => {
            for err in errors {
                eprintln!(
                    "{}:{}:{}: {}",
                    err.file.display(),
                    err.line,
                    err.column,
                    err.message
                );
            }
            ExitCode::from(1)
        }
    }
}
