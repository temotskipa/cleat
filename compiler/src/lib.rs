//! The Cleat compiler: source to a native executable, through LLVM IR and clang.

use std::path::{Path, PathBuf};

pub mod ast;
pub mod emit;
pub mod lex;
pub mod parse;
pub mod sema;

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
    analyze(&[root]).map(|_| ())
}

/// Parses and checks the `.cleat` files under every root as one program, with the
/// prelude. A file of the package `cleat` among the roots stands in place of the
/// prelude's file of the same name.
pub fn analyze(roots: &[&Path]) -> Result<sema::program::Program, Vec<Diagnostic>> {
    let mut units = Vec::new();
    let mut errors = Vec::new();
    for root in roots {
        match parse_project(root) {
            Ok(u) => units.extend(u),
            Err(e) => errors.extend(e),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    analyze_units(units)
}

/// Checks source given as text, each with the name of its file, as one program.
pub fn analyze_sources(sources: &[(PathBuf, String)]) -> Result<sema::program::Program, Vec<Diagnostic>> {
    let mut units = Vec::new();
    let mut errors = Vec::new();
    for (path, text) in sources {
        match parse::parse_unit(path, text) {
            Ok(u) => units.push(u),
            Err(e) => errors.push(e),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    analyze_units(units)
}

fn analyze_units(mut units: Vec<ast::Unit>) -> Result<sema::program::Program, Vec<Diagnostic>> {
    static PRELUDE: std::sync::OnceLock<Result<Vec<ast::Unit>, Vec<Diagnostic>>> = std::sync::OnceLock::new();
    let prelude = PRELUDE.get_or_init(|| parse_project(&prelude_dir())).clone()?;
    let same = |a: &Path, b: &Path| match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    };
    for r in &units {
        if r.package.first().map(|s| s.as_str()) != Some("cleat") {
            continue;
        }
        // A file of the prelude itself, or one that stands in place of a prelude file.
        let known = r.package.len() == 1 && prelude.iter().any(|u| same(&r.file, &u.file) || r.file.file_name() == u.file.file_name());
        if !known {
            return Err(vec![Diagnostic::general(
                &r.file,
                "a program does not declare the package `cleat`, or a package whose name begins with `cleat.`",
            )]);
        }
    }
    for u in prelude {
        let replaced = units.iter().any(|r| {
            same(&r.file, &u.file) || (r.package == ["cleat"] && r.file.file_name() == u.file.file_name())
        });
        if !replaced {
            units.push(u);
        }
    }
    let mut p = sema::analyze(units);
    if p.diags.is_empty() {
        Ok(p)
    } else {
        Err(std::mem::take(&mut p.diags))
    }
}

/// Where the prelude's source is: `CLEAT_PRELUDE`, or the `prelude` directory beside
/// the compiler's own source.
pub fn prelude_dir() -> PathBuf {
    match std::env::var_os("CLEAT_PRELUDE") {
        Some(p) => PathBuf::from(p),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("prelude"),
    }
}

/// Typechecks `root` and writes a native executable for the entry class `entry`.
pub fn build(root: &Path, entry: &str, output: &Path) -> Result<(), Vec<Diagnostic>> {
    build_roots(&[root], entry, output).map(|_| ())
}

/// Compiles the files under every root as one program. The result is the reports that
/// do not reject the program.
pub fn build_roots(roots: &[&Path], entry: &str, output: &Path) -> Result<Vec<Diagnostic>, Vec<Diagnostic>> {
    build_with(roots, entry, output, &[])
}

/// As `build_roots`, linking the given files too: the libraries or objects that supply
/// the foreign methods of the program.
pub fn build_with(roots: &[&Path], entry: &str, output: &Path, link: &[PathBuf]) -> Result<Vec<Diagnostic>, Vec<Diagnostic>> {
    let mut p = analyze(roots)?;
    let first = roots.first().copied().unwrap_or(Path::new("."));
    let fail = |message: String| vec![Diagnostic::general(first, message)];
    let mut found: Vec<u32> = p.classes.iter().filter(|c| c.qname == entry).map(|c| c.id).collect();
    if found.is_empty() {
        found = p.classes.iter().filter(|c| c.name == entry && c.lambda.is_none()).map(|c| c.id).collect();
    }
    if found.len() != 1 {
        return Err(fail(format!("the entry class `{entry}` names {} classes of the program", found.len())));
    }
    let (ir, shim) = emit::emit(&mut p, found[0]).map_err(|errors| errors.into_iter().map(|m| Diagnostic::general(first, m)).collect::<Vec<_>>())?;
    emit::link::link(&ir, &shim, output, link).map_err(fail)?;
    Ok(std::mem::take(&mut p.warnings))
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
