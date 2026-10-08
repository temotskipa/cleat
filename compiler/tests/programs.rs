//! Builds and runs each program in `design/programs/`, and compares what it prints with
//! what its header promises. Each runs twice: plainly, and with the collector forced at
//! every allocation.

mod common;

use common::*;
use std::path::Path;

fn built(test: &str, files: &[&str], entry: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = scratch(test);
    let roots: Vec<std::path::PathBuf> = files.iter().map(|f| program(f)).collect();
    let exe = build(test, &dir, &roots, entry);
    (dir, exe)
}

#[test]
fn calc() {
    let (_, exe) = built("calc", &["Calc"], "demo.Calc");
    expect(&exe, &["1/3 + 1/6"], 0, "1/3 + 1/6 = 1/2\n");
    expect(&exe, &["2 * (3 + 4) - 5", "-(1.5 - 2)", "0.1 + 0.2"], 0, "2 * (3 + 4) - 5 = 9\n-(1.5 - 2) = 1/2\n0.1 + 0.2 = 3/10\n");
    // A mistake in the text and a division by zero are reported, and the rest still run.
    let r = expect(&exe, &["2 +", "1 / 0", "7"], 0, "7 = 7\n");
    assert_eq!(r.err, "2 +: expected a number at 3\n1 / 0: division by zero\n");
}

#[test]
fn checks() {
    let (_, exe) = built("checks", &["Checks"], "demo.Checks");
    expect(
        &exe,
        &[],
        0,
        "pass: a new list is empty\nFAIL: add grows the list\npassed 1 of 2\nquantity is 12, above the limit 10\n",
    );
}

#[test]
fn ledger() {
    let (_, exe) = built("ledger", &["Ledger"], "demo.Ledger");
    // 3 x 4.35, 10 x 0.89 and 27.50 make 49.45. Ten percent is 4.945, which rounds to
    // the even cent, 4.94. The tax on 44.51 at 8.25% is 3.672075, so 3.67.
    expect(
        &exe,
        &[],
        0,
        "notebook x3  13.05\npen x10  8.90\ndesk lamp x1  27.50\nsubtotal  49.45\ndiscount  4.94\ntax       3.67\ndue       48.18\n",
    );
}

#[test]
fn list() {
    // The program is the prelude's own list: the two files are the same text.
    let ours = std::fs::read_to_string(program("List")).unwrap().replace("\r\n", "\n");
    let preludes = std::fs::read_to_string(repo().join("prelude").join("List.cleat")).unwrap().replace("\r\n", "\n");
    assert_eq!(ours, preludes, "design/programs/List.cleat and prelude/List.cleat differ");
    let dir = scratch("list");
    let driver = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data").join("ListDriver.cleat");
    let exe = build("list", &dir, &[program("List"), driver], "demo.ListDriver");
    expect(
        &exe,
        &[],
        0,
        "empty: true\nsize: 9\nthird: apple\nlast removed: grape\napricot banana cherry date elderberry fig kiwi pear\ntrue\nfalse\ncleat.List<cleat.String>\ngap: null\nindex 40, size 8\n",
    );
}

#[test]
fn nbody() {
    let (_, exe) = built("nbody", &["NBody"], "demo.NBody");
    for stress in [false, true] {
        let r = run(&exe, &[], stress);
        assert_eq!(r.status, 0, "{}", r.err);
        let lines: Vec<&str> = r.out.lines().collect();
        assert_eq!(lines.len(), 2, "{}", r.out);
        // Kinetic 74.5, potential -100 - 50 - 1/30.
        assert_eq!(lines[0], "energy before: -75.533333");
        let after: f64 = lines[1].strip_prefix("energy after:  ").expect("the second line").parse().expect("a number");
        assert!((after - -75.533333).abs() < 0.01, "energy is not conserved: {after}");
    }
}

#[test]
fn parallel_count() {
    let (dir, exe) = built("parallel_count", &["ParallelCount", "WordCount"], "demo.ParallelCount");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    std::fs::write(&a, "the quick brown fox jumps over the lazy dog\n").unwrap();
    std::fs::write(&b, "Extraordinary claims require extraordinary evidence, the fox said.\n").unwrap();
    let (a, b) = (a.to_string_lossy().to_string(), b.to_string_lossy().to_string());
    for stress in [false, true] {
        let r = run(&exe, &[&a, &b], stress);
        assert_eq!(r.status, 0, "{}", r.err);
        let mut lines: Vec<&str> = r.out.lines().collect();
        assert_eq!(lines.len(), 4, "{}", r.out);
        // The two tasks finish in either order.
        let mut counted: Vec<String> = lines.drain(..2).map(|l| l.to_string()).collect();
        counted.sort();
        let mut want = vec![format!("counted {a} (1 of 2)"), format!("counted {b} (2 of 2)")];
        let mut other = vec![format!("counted {a} (2 of 2)"), format!("counted {b} (1 of 2)")];
        want.sort();
        other.sort();
        assert!(counted == want || counted == other, "{}", r.out);
        // the quick brown fox jumps over lazy dog + extraordinary claims require evidence said
        assert_eq!(lines, ["distinct words: 13", "longest word: extraordinary"]);
    }
    // One unreadable file fails its task, and the scope reports it.
    let missing = dir.join("missing.txt").to_string_lossy().to_string();
    let r = run(&exe, &[&a, &missing, &b], false);
    assert_eq!(r.status, 1, "{}", r.out);
    assert!(r.err.starts_with("cannot read input: "), "{}", r.err);
}

#[test]
fn shapes() {
    let (_, exe) = built("shapes", &["Shapes"], "demo.Shapes");
    expect(
        &exe,
        &[],
        0,
        "circles: 2, area 15.71\nshapes: 3, area 21.71\nreversed: 12.57 6.00 3.14\na list of shapes, area 15.71\nsome other list\nnot a list\n",
    );
}

#[test]
fn shares() {
    let (_, exe) = built("shares", &["Shares"], "demo.Shares");
    // 120.50 among 4 is 30.125 each, which rounds to the even cent.
    expect(&exe, &["120.50", "4"], 0, "splitting 120.50 among 4\neach pays 30.12\n");
    let r = expect(&exe, &["10", "0"], 2, "");
    assert_eq!(r.err, "shares: the total must be a number and the people a positive integer\n");
    let r = expect(&exe, &["10"], 2, "");
    assert_eq!(r.err, "usage: shares <total> <people>\n");
}

#[test]
fn word_count() {
    let (dir, exe) = built("word_count", &["WordCount"], "demo.WordCount");
    let text = dir.join("text.txt");
    std::fs::write(&text, "the quick brown fox jumps over the lazy dog. The dog barks; the fox runs.\nA quick fox, a quick dog.\n").unwrap();
    let path = text.to_string_lossy().to_string();
    // Ties keep the order of first appearance.
    expect(&exe, &[&path], 0, "the 4\nquick 3\nfox 3\ndog 3\na 2\nbrown 1\njumps 1\nover 1\nlazy 1\nbarks 1\n");
    let r = expect(&exe, &[], 2, "");
    assert_eq!(r.err, "usage: wordcount <file>\n");
}

/// Every program in the directory has a test above.
#[test]
fn every_program_has_a_test() {
    let tested = ["Calc", "Checks", "Ledger", "List", "NBody", "ParallelCount", "Shapes", "Shares", "WordCount"];
    let mut found: Vec<String> = std::fs::read_dir(repo().join("design").join("programs"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".cleat"))
        .map(|n| n.trim_end_matches(".cleat").to_string())
        .collect();
    found.sort();
    assert_eq!(found, tested);
}
