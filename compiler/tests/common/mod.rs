//! What the run tests share: build a program with the compiler, run it, and compare.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Run {
    pub status: i32,
    pub out: String,
    pub err: String,
}

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

pub fn program(name: &str) -> PathBuf {
    repo().join("design").join("programs").join(format!("{name}.cleat"))
}

/// A directory of this test's own, emptied.
pub fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Compiles the sources as one program and returns the executable.
pub fn build(test: &str, dir: &Path, roots: &[PathBuf], entry: &str) -> PathBuf {
    let exe = dir.join(format!("{test}.exe"));
    let roots: Vec<&Path> = roots.iter().map(|p| p.as_path()).collect();
    if let Err(errors) = cleatc::build_roots(&roots, entry, &exe) {
        let text: Vec<String> = errors.iter().map(|d| format!("{}:{}:{}: {}", d.file.display(), d.line, d.column, d.message)).collect();
        panic!("{test} does not build:\n{}", text.join("\n"));
    }
    exe
}

/// Compiles one source text. The file is named for the entry class.
pub fn build_text(test: &str, entry: &str, text: &str) -> (PathBuf, PathBuf) {
    let dir = scratch(test);
    let simple = entry.rsplit('.').next().unwrap();
    let file = dir.join(format!("{simple}.cleat"));
    std::fs::write(&file, text).unwrap();
    let exe = build(test, &dir, &[file], entry);
    (dir, exe)
}

/// Runs a program. With `stress`, the collector runs at every allocation, which shows
/// a pointer the compiled code failed to keep where the collector looks.
pub fn run(exe: &Path, args: &[&str], stress: bool) -> Run {
    let mut cmd = Command::new(exe);
    cmd.args(args);
    if stress {
        cmd.env("CLEAT_GC_STRESS", "1");
    } else {
        cmd.env_remove("CLEAT_GC_STRESS");
    }
    let output = cmd.output().unwrap_or_else(|e| panic!("cannot run {}: {e}", exe.display()));
    Run {
        status: output.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n"),
        err: String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n"),
    }
}

/// Runs twice, plainly and under collector stress, and expects the same from both.
pub fn expect(exe: &Path, args: &[&str], status: i32, out: &str) -> Run {
    let mut last = None;
    for stress in [false, true] {
        let r = run(exe, args, stress);
        let how = if stress { " (with a collection at every allocation)" } else { "" };
        assert_eq!(r.out, out, "the output of {}{how}\nstderr: {}", exe.display(), r.err);
        assert_eq!(r.status, status, "the exit status of {}{how}\nstderr: {}", exe.display(), r.err);
        last = Some(r);
    }
    last.unwrap()
}
