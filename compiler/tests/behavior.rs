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
