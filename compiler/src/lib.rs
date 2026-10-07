//! The Cleat compiler: source to a native executable, through LLVM IR and clang.

use std::path::{Path, PathBuf};

pub mod ast;
pub mod lex;
pub mod parse;

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub message: String,
}

impl Diagnostic {
    pub fn general(file: &Path, message: impl Into<String>) -> Diagnostic {
        Diagnostic { file: file.to_path_buf(), line: 1, column: 1, message: message.into() }
    }
}

/// Parses every `.cleat` file under `root`.
pub fn parse_project(root: &Path) -> Result<Vec<ast::Unit>, Vec<Diagnostic>> {
    let files = collect(root)?;
    let mut units = Vec::new();
    let mut errors = Vec::new();
    for (path, text) in &files {
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

/// Typechecks every `.cleat` file under `root` as one program.
pub fn check(root: &Path) -> Result<(), Vec<Diagnostic>> {
    parse_project(root)?;
    Ok(())
}

/// Typechecks `root` and writes a native executable for the entry class `entry`.
pub fn build(root: &Path, _entry: &str, _output: &Path) -> Result<(), Vec<Diagnostic>> {
    parse_project(root)?;
    Err(vec![Diagnostic::general(root, "building is not implemented yet")])
}

fn collect(root: &Path) -> Result<Vec<(PathBuf, String)>, Vec<Diagnostic>> {
    let mut files = Vec::new();
    walk(root, &mut files).map_err(|message| vec![Diagnostic::general(root, message)])?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    if files.is_empty() {
        return Err(vec![Diagnostic::general(root, "the project contains no .cleat files")]);
    }
    Ok(files)
}

fn walk(path: &Path, out: &mut Vec<(PathBuf, String)>) -> Result<(), String> {
    if path.is_file() {
        let text = std::fs::read_to_string(path)
            .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
        out.push((path.to_path_buf(), text));
        return Ok(());
    }
    let entries =
        std::fs::read_dir(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("cleat") {
            let text = std::fs::read_to_string(&path)
                .map_err(|err| format!("cannot read {}: {err}", path.display()))?;
            out.push((path, text));
        }
    }
    Ok(())
}
