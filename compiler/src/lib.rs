mod ast;
mod check;
mod lex;
mod llvm;
mod parse;

use std::path::{Path, PathBuf};

pub use check::Diagnostic;

/// Typecheck every `.cleat` file under `root` as one project.
pub fn check(root: &Path) -> Result<(), Vec<Diagnostic>> {
    let files = collect(root)?;
    let parsed = parse_all(&files)?;
    check::check_project(&parsed)?;
    Ok(())
}

/// Typecheck `root` and write a native executable for `entry` (`package.Type`).
pub fn build(root: &Path, entry: &str, output: &Path) -> Result<(), Vec<Diagnostic>> {
    let files = collect(root)?;
    let parsed = parse_all(&files)?;
    let project = check::check_project(&parsed)?;
    let ir = llvm::emit(&project, entry)?;
    llvm::link(&ir, output).map_err(|message| {
        vec![Diagnostic {
            file: output.to_path_buf(),
            line: 1,
            column: 1,
            message,
        }]
    })
}

fn collect(root: &Path) -> Result<Vec<(PathBuf, String)>, Vec<Diagnostic>> {
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|message| {
        vec![Diagnostic {
            file: root.to_path_buf(),
            line: 1,
            column: 1,
            message,
        }]
    })?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    if files.is_empty() {
        return Err(vec![Diagnostic {
            file: root.to_path_buf(),
            line: 1,
            column: 1,
            message: "the project contains no .cleat files".into(),
        }]);
    }
    Ok(files)
}

fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|err| format!("cannot read {}: {err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("cleat") {
            let text = std::fs::read_to_string(&path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
            if text.as_bytes().iter().any(|b| *b > 127) {
                // UTF-8 was already validated by read_to_string. Non-ASCII is legal.
            }
            out.push((path, text));
        }
    }
    Ok(())
}

fn parse_all(files: &[(PathBuf, String)]) -> Result<Vec<ast::Unit>, Vec<Diagnostic>> {
    let mut units = Vec::new();
    let mut errors = Vec::new();
    for (path, text) in files {
        match parse::parse_unit(path, text) {
            Ok(unit) => units.push(unit),
            Err(err) => errors.push(err),
        }
    }
    if errors.is_empty() {
        Ok(units)
    } else {
        Err(errors)
    }
}
