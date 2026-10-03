use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("cleat-m1-{name}-{}", std::process::id()));
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
            .map(|err| err.message.clone())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

#[test]
fn package_type_is_visible_across_files_and_private_type_is_not() {
    let root = project(
        "visibility",
        &[
            (
                "Main.cleat",
                r#"
                package demo;
                public class Main {
                    public static void main() {
                        Int32 n = Util.answer();
                    }
                }
                "#,
            ),
            (
                "Util.cleat",
                r#"
                package demo;
                package class Util {
                    public static Int32 answer() {
                        return Secret.value();
                    }
                }
                "#,
            ),
            (
                "Secret.cleat",
                r#"
                package demo;
                class Secret {
                    public static Int32 value() {
                        return 7;
                    }
                }
                "#,
            ),
        ],
    );
    let text = messages(&root);
    assert!(
        text.contains("not visible"),
        "file-private Secret should be rejected from Util, got {text}"
    );

    fs::write(
        root.join("Util.cleat"),
        r#"
        package demo;
        package class Util {
            public static Int32 answer() {
                return 4 / 2;
            }
        }
        "#,
    )
    .unwrap();
    assert_eq!(messages(&root), "");
}

#[test]
fn public_type_must_match_its_file_name() {
    let root = project(
        "filename",
        &[(
            "Wrong.cleat",
            r#"
            package demo;
            public class Main {
                public static void main() {}
            }
            "#,
        )],
    );
    let text = messages(&root);
    assert!(
        text.contains("Main.cleat"),
        "filename rule should be named, got {text}"
    );
}

#[test]
fn constant_overflow_is_rejected_and_runtime_overflow_exits_1() {
    let root = project(
        "overflow",
        &[(
            "Main.cleat",
            r#"
            package demo;
            public class Main {
                public static void main() {
                    Int32 n = 2000000000 + 2000000000;
                }
            }
            "#,
        )],
    );
    let text = messages(&root);
    assert!(
        text.contains("not representable"),
        "constant overflow should be rejected, got {text}"
    );

    fs::write(
        root.join("Main.cleat"),
        r#"
        package demo;
        public class Main {
            public static void main() {
                Int32 n = 2147483647;
                Int32 m = n + 1;
            }
        }
        "#,
    )
    .unwrap();
    let exe = root.join("overflow.exe");
    cleatc::build(&root, "demo.Main", &exe).unwrap();
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn exact_div_runs_and_inexact_div_exits_1() {
    let root = project(
        "div",
        &[
            (
                "Main.cleat",
                r#"
                package demo;
                public class Main {
                    public static void main() {
                        Int32 n = Util.half(4);
                    }
                }
                "#,
            ),
            (
                "Util.cleat",
                r#"
                package demo;
                package class Util {
                    public static Int32 half(Int32 n) {
                        return n / 2;
                    }
                }
                "#,
            ),
        ],
    );
    let exe = root.join("div.exe");
    cleatc::build(&root, "demo.Main", &exe).unwrap_or_else(|err| {
        panic!(
            "{}",
            err.iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join("\n")
        )
    });
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(0));

    fs::write(
        root.join("Main.cleat"),
        r#"
        package demo;
        public class Main {
            public static void main() {
                Int32 n = Util.half(5);
            }
        }
        "#,
    )
    .unwrap();
    cleatc::build(&root, "demo.Main", &exe).unwrap();
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn open_static_method_and_missing_import_are_rejected() {
    let root = project(
        "open-static",
        &[(
            "Main.cleat",
            r#"
            package demo;
            import demo.Missing;
            public class Main {
                public static open void main() {}
            }
            "#,
        )],
    );
    let text = messages(&root);
    assert!(
        text.contains("open is rejected"),
        "open static should be rejected, got {text}"
    );
    assert!(
        text.contains("section 4.2"),
        "the diagnostic should name section 4.2, got {text}"
    );
    assert!(
        text.contains("demo.Missing"),
        "the import span should be reported, got {text}"
    );
}

#[test]
fn final_class_in_another_file_cannot_be_extended() {
    let root = project(
        "extend",
        &[
            (
                "Base.cleat",
                r#"
                package demo;
                public class Base {
                    public static void main() {}
                }
                "#,
            ),
            (
                "Child.cleat",
                r#"
                package demo;
                package class Child extends Base {
                    public static Int32 n() { return 1; }
                }
                "#,
            ),
        ],
    );
    let text = messages(&root);
    assert!(
        text.contains("final"),
        "unmarked class is final, got {text}"
    );
    assert!(
        text.contains("section 4.1"),
        "the diagnostic should name section 4.1, got {text}"
    );
}

#[test]
fn static_int32_field_stays_unboxed_and_is_readable() {
    let root = project(
        "field",
        &[(
            "Main.cleat",
            r#"
            package demo;
            public class Main {
                public static Int32 base = 41;
                public static void main() {
                    Int32 n = base + 1;
                    Int32 one = 1;
                    Int32 big = 2147483647;
                    if (n != 42) {
                        Int32 bad = big + one;
                    }
                }
            }
            "#,
        )],
    );
    let exe = root.join("field.exe");
    cleatc::build(&root, "demo.Main", &exe).unwrap();
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(0));
    let ir = fs::read_to_string(root.join("field.ll")).unwrap();
    assert!(
        ir.contains("@g_demo_Main_base = global i32 41"),
        "static Int32 is a global, not a heap object: {ir}"
    );
    assert!(
        ir.contains("alloca i32"),
        "Int32 locals stay in i32 slots: {ir}"
    );
}

#[test]
fn only_names_a_project_type_and_narrows_the_caller() {
    let root = project(
        "only",
        &[
            (
                "Main.cleat",
                r#"
                package demo;
                public class Main {
                    public static void main() {
                        Int32 n = Gate.through();
                    }
                }
                "#,
            ),
            (
                "Gate.cleat",
                r#"
                package demo;
                public class Gate {
                    public static Int32 through() only Main {
                        return 1;
                    }
                }
                "#,
            ),
            (
                "Other.cleat",
                r#"
                package demo;
                public class Other {
                    public static Int32 peek() {
                        return Gate.through();
                    }
                }
                "#,
            ),
        ],
    );
    let text = messages(&root);
    assert!(
        text.contains("not visible") && text.contains("section 3.2"),
        "Other cannot call a method narrowed by only, got {text}"
    );

    fs::write(
        root.join("Gate.cleat"),
        r#"
        package demo;
        public class Gate {
            public static Int32 through() only Missing {
                return 1;
            }
        }
        "#,
    )
    .unwrap();
    let text = messages(&root);
    assert!(
        text.contains("Missing") && text.contains("section 3.2"),
        "only must name a project type, got {text}"
    );
}

#[test]
fn null_requires_nullable_and_min_div_is_rejected() {
    let root = project(
        "nulls",
        &[(
            "Main.cleat",
            r#"
            package demo;
            public class Main {
                public static void main() {
                    Int32 n = null;
                }
            }
            "#,
        )],
    );
    let Err(errors) = cleatc::check(&root) else {
        panic!("null assigned to Int32 should be rejected");
    };
    let shown = format!(
        "{}:{}:{}: {}",
        errors[0].file.display(),
        errors[0].line,
        errors[0].column,
        errors[0].message
    );
    assert!(
        shown.contains("section 5.2")
            && shown.contains("Main.cleat")
            && errors[0].line > 1
            && errors[0].column > 1,
        "null into Int32 should name the file, span, and section 5.2, got {shown}"
    );

    fs::write(
        root.join("Main.cleat"),
        r#"
        package demo;
        public class Main {
            public static @Nullable Int32 slot = null;
            public static void main() {}
        }
        "#,
    )
    .unwrap();
    cleatc::check(&root).unwrap_or_else(|err| {
        panic!(
            "{}",
            err.iter().map(|e| e.message.clone()).collect::<Vec<_>>().join("\n")
        )
    });

    fs::write(
        root.join("Main.cleat"),
        r#"
        package demo;
        public class Main {
            public static void main() {
                Int32 n = -2147483648 / -1;
            }
        }
        "#,
    )
    .unwrap();
    let text = messages(&root);
    assert!(
        text.contains("ArithmeticException") && text.contains("section 6.3"),
        "MIN / -1 is a constant ArithmeticException, got {text}"
    );

    fs::write(
        root.join("Main.cleat"),
        r#"
        package demo;
        public class Main {
            public static void main() {
                Int32 n = -2147483648;
                Int32 m = n / -1;
            }
        }
        "#,
    )
    .unwrap();
    let exe = root.join("min.exe");
    cleatc::build(&root, "demo.Main", &exe).unwrap();
    let status = Command::new(&exe).status().unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn permits_must_name_a_project_type() {
    let root = project(
        "permits",
        &[(
            "Base.cleat",
            r#"
            package demo;
            public sealed class Base permits Missing {
                public static void main() {}
            }
            "#,
        )],
    );
    let text = messages(&root);
    assert!(
        text.contains("Missing") && text.contains("section 3.4"),
        "permits must name a project type, got {text}"
    );
}
