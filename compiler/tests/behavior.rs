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
];
