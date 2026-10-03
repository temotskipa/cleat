use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("cleat-m2-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for (path, text) in files {
        let full = root.join(path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(full, text).unwrap();
    }
    root
}

fn messages(root: &Path) -> String {
    match cleatc::check(root) {
        Ok(()) => String::new(),
        Err(errors) => errors
            .iter()
            .map(|err| format!("{}:{}:{}: {}", err.file.display(), err.line, err.column, err.message))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[test]
fn references_arrays_identity_and_precise_collection() {
    let root = project(
        "refs",
        &[
            (
                "Box.cleat",
                r#"
                package demo;
                public class Box {
                    public Int32 n;
                    public Box other;
                }
                "#,
            ),
            (
                "Other.cleat",
                r#"
                package demo;
                public class Other {
                    public Int32 n;
                }
                "#,
            ),
            (
                "Main.cleat",
                r#"
                package demo;
                public class Main {
                    public Object item;
                    public static void main() {
                        Box keep = new Box();
                        keep.n = 7;
                        Box child = new Box();
                        child.n = 9;
                        keep.other = child;
                        Other stranger = new Other();
                        Box[] xs = new Box[2];
                        xs[0] = keep;
                        Main holder = new Main();
                        holder.item = 41;
                        Int32 i = 0;
                        while (i < 10000) {
                            Box trash = new Box();
                            trash.n = i;
                            i = i + 1;
                        }
                        Int32 one = 1;
                        Int32 big = 2147483647;
                        Int32 bad = 0;
                        if (keep.n != 7) { bad = big + one; }
                        if (keep.other.n != 9) { bad = big + one; }
                        if (xs[0] != keep) { bad = big + one; }
                        if (xs[0].n != 7) { bad = big + one; }
                        if (keep == child) { bad = big + one; }
                        if (keep.identical(keep)) { } else { bad = big + one; }
                        if (keep.getClass().isInstance(keep)) { } else { bad = big + one; }
                        if (keep.getClass().isInstance(stranger)) { bad = big + one; }
                        if (keep.getClass().has(keep.getClass())) { bad = big + one; }
                        if (keep.n == 7) { } else { bad = big + one; }
                    }
                }
                "#,
            ),
        ],
    );
    let exe = root.join("refs.exe");
    cleatc::build(&root, "demo.Main", &exe).unwrap_or_else(|err| {
        panic!(
            "{}",
            err.iter()
                .map(|e| format!("{}: {}", e.file.display(), e.message))
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(0), "reference program should exit 0");
    let ir = fs::read_to_string(root.join("refs.ll")).unwrap();
    assert!(
        ir.contains("call ptr @cleat_box_i32"),
        "storing an Int32 in Object boxes it: {ir}"
    );
    assert!(
        ir.contains("alloca i32"),
        "unboxed Int32 uses an i32 slot"
    );
    assert!(ir.contains("alloca ptr"), "references use pointer slots");
    let mut ptr_slots = Vec::new();
    let mut int_slots = Vec::new();
    for line in ir.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix('%') {
            if let Some(name) = rest.split_once(" = alloca ptr") {
                ptr_slots.push(name.0.to_string());
            } else if let Some(name) = rest.split_once(" = alloca i32") {
                int_slots.push(name.0.to_string());
            }
        }
        if let Some(slot) = line.strip_prefix("call void @cleat_push_root(ptr %") {
            let slot = slot.trim_end_matches(')');
            assert!(
                ptr_slots.iter().any(|name| name == slot),
                "root {slot} is not a reference slot"
            );
            assert!(
                int_slots.iter().all(|name| name != slot),
                "unboxed Int32 slot {slot} was scanned as a root"
            );
        }
    }
    assert!(!ptr_slots.is_empty(), "expected reference roots");
}

#[test]
fn identical_on_int32_is_rejected() {
    let root = project(
        "identical",
        &[(
            "Main.cleat",
            r#"
            package demo;
            public class Main {
                public static void main() {
                    Int32 n = 1;
                    if (n.identical(n)) { }
                }
            }
            "#,
        )],
    );
    let text = messages(&root);
    assert!(
        text.contains("Main.cleat")
            && text.contains("section 2.2")
            && text.contains("identical is rejected"),
        "identical on Int32 should name the file, span, and section 2.2, got {text}"
    );
    let Err(errors) = cleatc::check(&root) else {
        panic!("expected a rejection");
    };
    assert!(errors[0].line > 1 && errors[0].column > 1);
}
