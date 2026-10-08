//! What compiled programs do, slice by slice. Each case is a program in
//! `tests/behavior/` with the output it must print beside it. It runs twice: plainly,
//! and with the collector forced at every allocation.

mod common;

use common::*;
use std::path::Path;

fn case(name: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("behavior");
    let source = dir.join(format!("{name}.cleat"));
    let want = std::fs::read_to_string(dir.join(format!("{name}.out"))).unwrap().replace("\r\n", "\n");
    let test = format!("behavior_{name}");
    let scratch = scratch(&test);
    let exe = build(&test, &scratch, &[source], &format!("t.{name}"));
    expect(&exe, &[], 0, &want);
}

macro_rules! cases {
    ($($test:ident => $name:literal),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                case($name);
            }
        )*
    };
}

cases![
    numbers => "Numbers",
    objects => "Objects",
    nulls => "Nulls",
    generics => "Generics",
    control => "Control",
    annotations => "Annotations",
    text => "Text",
    threads => "Threads",
    foreign => "Foreign",
    statics => "Statics",
    members => "Members",
    widths => "Widths",
    rules => "Rules",
    sentences_01 => "Sentences01",
    sentences_02 => "Sentences02",
    sentences_05 => "Sentences05",
    sentences_11 => "Sentences11",
    sentences_13 => "Sentences13",
];

/// Section 3.4: a method that a subclass may not name is not overridden. The subclass
/// declares a new method, and the superclass's own calls still reach its own.
#[test]
fn a_method_a_class_cannot_name_is_not_overridden() {
    let dir = scratch("behavior_not_overridden");
    let files = [
        (
            "a/Base.cleat",
            "package a;\n\npublic open class Base {\n    package open Int rank() { return 1; }\n    private Int own() { return 10; }\n    public Int viaBase() { return rank() + own(); }\n    public static Int through(Base b) { return b.rank(); }\n    public static Int mine(b.Derived d) { return d.own() + d.rank(); }\n}\n",
        ),
        ("b/Derived.cleat", "package b;\n\nimport a.Base;\n\npublic class Derived extends Base {\n    public Int rank() { return 2; }\n    public Int own() { return 20; }\n}\n"),
        (
            "Main.cleat",
            "import a.Base;\nimport b.Derived;\n\npublic class Main {\n    public static void main() {\n        var d = new Derived();\n        Console.println(d.rank());\n        Console.println(d.own());\n        Console.println(d.viaBase());\n        Console.println(Base.through(d));\n        Console.println(Base.mine(d));\n    }\n}\n",
        ),
    ];
    let mut roots = Vec::new();
    for (name, text) in files {
        let file = dir.join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, text).unwrap();
        roots.push(file);
    }
    let exe = build("behavior_not_overridden", &dir, &roots, "Main");
    expect(&exe, &[], 0, "2\n20\n11\n1\n22\n");
}

/// Section 3.6: an annotated public member of a type that is not public can be used
/// through its mirror by code that could not name the type, and a member that is not
/// public cannot.
#[test]
fn a_mirror_reaches_a_public_member_of_a_type_the_code_cannot_name() {
    let dir = scratch("behavior_mirror_reach");
    let files = [
        ("Mark.cleat", "public annotation Mark;\n"),
        (
            "Hidden.cleat",
            "class Secret {\n    @Mark\n    public Int code = 42;\n    @Mark\n    Int kept = 7;\n\n    @Mark\n    public Int twice() {\n        return code * 2;\n    }\n}\n\npublic class Hidden {\n    public static Object make() {\n        return new Secret();\n    }\n}\n",
        ),
        (
            "Main.cleat",
            "public class Main {\n    public static void main() {\n        Object o = Hidden.make();\n        Class c = o.getClass();\n        Console.println(c.getName());\n        Field[] fields = c.<Mark>getAnnotatedFields();\n        Console.println(fields[0].get(o));\n        try {\n            Console.println(fields[1].get(o));\n        } catch (IllegalAccessException e) {\n            Console.println(\"not public\");\n        }\n        Console.println(c.<Mark>getAnnotatedMethods()[0].invoke(o));\n    }\n}\n",
        ),
    ];
    let mut roots = Vec::new();
    for (name, text) in files {
        let file = dir.join(name);
        std::fs::write(&file, text).unwrap();
        roots.push(file);
    }
    let exe = build("behavior_mirror_reach", &dir, &roots, "Main");
    expect(&exe, &[], 0, "Secret\n42\nnot public\n84\n");
}

/// Section 13.1: an exception that leaves a thread's body ends that thread and no
/// other, and its `toString` is written to the host's error stream.
#[test]
fn an_exception_that_leaves_a_thread_is_written_to_the_error_stream() {
    let text = "public class Loud {\n    public static void main() {\n        var t = Thread.start(() -> {\n            throw new IllegalStateException(\"the body failed\");\n        });\n        t.join();\n        Console.println(\"main goes on\");\n    }\n}\n";
    let (_, exe) = build_text("behavior_thread_error", "Loud", text);
    let run = expect(&exe, &[], 0, "main goes on\n");
    assert!(run.err.contains("IllegalStateException: the body failed"), "{}", run.err);
}
