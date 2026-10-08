//! From LLVM IR to an executable: clang compiles the module and links it with the
//! runtime library.

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(cmd: &mut Command) -> Result<(), String> {
    let output = cmd.output().map_err(|err| format!("cannot run {}: {err}", cmd.get_program().to_string_lossy()))?;
    if output.status.success() {
        return Ok(());
    }
    let mut detail = format!("{}{}", String::from_utf8_lossy(&output.stderr), String::from_utf8_lossy(&output.stdout));
    if detail.len() > 6000 {
        detail = format!("{}\n...", &detail[..6000]);
    }
    Err(format!("{} failed with {}:\n{detail}", cmd.get_program().to_string_lossy(), output.status))
}

/// clang: where `CLEAT_CLANG` points, the usual install, a copy under
/// `%LOCALAPPDATA%\cleat-llvm`, or the one on the path.
pub fn clang() -> PathBuf {
    if let Some(p) = std::env::var_os("CLEAT_CLANG") {
        return PathBuf::from(p);
    }
    let usual = Path::new(r"C:\Program Files\LLVM\bin\clang.exe");
    if usual.exists() {
        return usual.to_path_buf();
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let root = Path::new(&local).join("cleat-llvm");
        if let Ok(entries) = std::fs::read_dir(&root) {
            for e in entries.flatten() {
                let candidate = e.path().join("bin").join("clang.exe");
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from("clang")
}

/// The runtime library: where `CLEAT_RT` points, or the newest build beside this program.
pub fn runtime() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("CLEAT_RT") {
        return Ok(PathBuf::from(p));
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut dir = exe.parent();
    for _ in 0..3 {
        let Some(d) = dir else { break };
        dirs.push(d.to_path_buf());
        dirs.push(d.join("deps"));
        dir = d.parent();
    }
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for d in &dirs {
        let Ok(entries) = std::fs::read_dir(d) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let is_lib = (name.starts_with("cleatrt") && name.ends_with(".lib")) || (name.starts_with("libcleatrt") && name.ends_with(".a"));
            if !is_lib {
                continue;
            }
            let modified = e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            if best.as_ref().map(|(t, _)| modified >= *t).unwrap_or(true) {
                best = Some((modified, e.path()));
            }
        }
    }
    best.map(|(_, p)| p).ok_or_else(|| format!("the runtime library cleatrt was not found near {}; build it with `cargo build`, or set CLEAT_RT", exe.display()))
}

pub fn link(ir: &str, output: &Path) -> Result<(), String> {
    let dir = output.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    let ll = output.with_extension("ll");
    std::fs::write(&ll, ir).map_err(|err| format!("cannot write {}: {err}", ll.display()))?;
    let rt = runtime()?;
    let mut cmd = Command::new(clang());
    cmd.arg("-O1").arg("-Wno-override-module").arg(&ll).arg(&rt).arg("-o").arg(output);
    if cfg!(windows) {
        for lib in ["kernel32", "ntdll", "userenv", "ws2_32", "dbghelp", "advapi32", "bcrypt", "synchronization"] {
            cmd.arg(format!("-l{lib}"));
        }
        // The main thread recurses as deep as any other.
        cmd.arg("-Wl,/STACK:16777216");
    } else {
        cmd.args(["-lpthread", "-ldl", "-lm"]);
    }
    run(&mut cmd)
}
