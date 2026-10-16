//! The rules of the specification, and a case for each.
//!
//! A rule is a sentence of a chapter's prose, on a line of its own, or a row of one of its
//! tables. It ends with its label, as `<!-- r[visibility.audiences.use-outside-audience] -->`.
//! The label begins with the chapter's first word and does not change when the text
//! around it does. A table that one test holds whole has one label, on its header row,
//! and its rows have none.
//!
//! A case names the rules it shows with a line `//@ rule: label`. It is one of four things:
//!
//! - a program in `tests/rules/<chapter>/`, whose header says `//@ rejected: words`, and
//!   the checker must reject it with a diagnostic that contains the words, or
//!   `//@ accepted`, and the checker must accept it. A line `//@ file: path` begins
//!   another file of the program;
//! - a line of a run program in `tests/behavior/`, above the statement whose output shows
//!   the rule;
//! - a line above a test in `tests/`;
//! - a line `label: reason` in `tests/rules/<chapter>/reasons.txt`, for a rule that no
//!   program shows.
//!
//! Every rule has a case, and every case names a rule. One checker takes all of these
//! programs, together with every example and every program of `design/programs`. If two
//! chapters asked for opposite things of one construct, a program here would be accepted
//! where it must be rejected, or rejected where it must be accepted, and its test would
//! fail.

use cleatc::analyze_sources;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// The files of a directory with this extension, in order.
fn files_in(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some(extension))
        .collect();
    out.sort();
    out
}

// ---- the rules ----

/// A rule of the specification.
struct Rule {
    label: String,
    /// The file of its chapter, and its line, counted from one.
    file: String,
    line: usize,
}

/// A label is words of lowercase letters and digits joined by `-`, and two or more of
/// them joined by `.`.
fn is_label(label: &str) -> bool {
    let parts: Vec<&str> = label.split('.').collect();
    parts.len() >= 2
        && parts.iter().all(|p| {
            p.split('-').all(|w| {
                !w.is_empty()
                    && w.chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            })
        })
}

/// The label a line ends with, and the line without it. A row ends with the label and
/// then its closing `|`.
fn label_of(line: &str) -> Result<Option<(String, String)>, String> {
    const OPEN: &str = "<!-- r[";
    const CLOSE: &str = "] -->";
    match line.matches(OPEN).count() {
        0 => return Ok(None),
        1 => {}
        n => return Err(format!("{n} labels")),
    }
    let start = line.find(OPEN).unwrap();
    let rest = &line[start + OPEN.len()..];
    let end = rest.find(CLOSE).ok_or("a label that is not closed")?;
    let label = &rest[..end];
    let after = rest[end + CLOSE.len()..].trim();
    if !(after.is_empty() || after == "|") {
        return Err("a label that does not end the line".to_string());
    }
    if !is_label(label) {
        return Err(format!("`{label}` is not a label"));
    }
    let text = format!("{} {after}", line[..start].trim_end());
    Ok(Some((label.to_string(), text.trim_end().to_string())))
}

/// The places where a line of prose ends one sentence and begins another: a `.` and a
/// space, and then a capital, a code span, a link or emphasis.
fn sentence_breaks(line: &str) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let mut breaks = 0;
    for i in 0..chars.len() {
        if chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_whitespace() {
            let next = chars[i + 1..].iter().find(|c| !c.is_whitespace());
            if next.is_some_and(|c| c.is_ascii_uppercase() || matches!(c, '`' | '*' | '[' | '(')) {
                breaks += 1;
            }
        }
    }
    breaks
}

/// A line of a list without its `-` or its number.
fn list_item(line: &str) -> &str {
    if let Some(rest) = line.strip_prefix("- ") {
        return rest;
    }
    let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
    match line[digits..].strip_prefix(". ") {
        Some(rest) if digits > 0 => rest,
        _ => line,
    }
}

/// The first word of a chapter's title, which begins each of its labels.
fn chapter_word(text: &str) -> String {
    let title = text.lines().next().unwrap_or("");
    title
        .trim_start_matches('#')
        .split_whitespace()
        .nth(1)
        .unwrap_or("")
        .to_lowercase()
}

/// Every chapter of the specification: its file, the word its labels begin with, and its
/// text.
fn chapters() -> Vec<(String, String, String)> {
    let spec = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("spec");
    files_in(&spec, "md")
        .into_iter()
        .map(|p| {
            let text = read(&p);
            (
                p.file_name().unwrap().to_string_lossy().to_string(),
                chapter_word(&text),
                text,
            )
        })
        .collect()
}

/// Every rule of the specification, and what is wrong with its labels.
fn rules() -> (Vec<Rule>, Vec<String>) {
    let mut rules = Vec::new();
    let mut wrong = Vec::new();
    for (file, word, text) in chapters() {
        let lines: Vec<&str> = text.lines().collect();
        let mut fenced = false;
        // Whether the table this line is in has its label on its header row.
        let mut whole = false;
        for (i, raw) in lines.iter().enumerate() {
            let at = format!("{file}:{}", i + 1);
            let line = raw.trim();
            if raw.starts_with("```") {
                fenced = !fenced;
                continue;
            }
            if fenced || line.is_empty() || line.starts_with('#') || line.starts_with("| ---") {
                continue;
            }
            let header = line.starts_with('|')
                && lines
                    .get(i + 1)
                    .is_some_and(|n| n.trim().starts_with("| ---"));
            let label = match label_of(line) {
                Ok(l) => l,
                Err(e) => {
                    wrong.push(format!("{at}: {e}"));
                    continue;
                }
            };
            if header {
                whole = label.is_some();
            } else if line.starts_with('|') && whole {
                if label.is_some() {
                    wrong.push(format!(
                        "{at}: a row of a table whose header row has the label has none of its own"
                    ));
                }
                continue;
            }
            let Some((label, text)) = label else {
                if !header {
                    wrong.push(format!("{at}: a rule without a label"));
                }
                continue;
            };
            if !line.starts_with('|') && sentence_breaks(list_item(&text)) > 0 {
                wrong.push(format!(
                    "{at}: `{label}` stands after more than one sentence"
                ));
            }
            if label.split('.').next() != Some(word.as_str()) {
                wrong.push(format!("{at}: `{label}` does not begin with `{word}`"));
            }
            rules.push(Rule {
                label,
                file: file.clone(),
                line: i + 1,
            });
        }
    }
    let mut seen = BTreeMap::new();
    for r in &rules {
        if let Some(first) = seen.insert(r.label.clone(), format!("{}:{}", r.file, r.line)) {
            wrong.push(format!(
                "{}:{}: `{}` is also the label at {first}",
                r.file, r.line, r.label
            ));
        }
    }
    (rules, wrong)
}

#[test]
fn every_rule_of_the_specification_has_a_label() {
    let (rules, wrong) = rules();
    assert!(
        wrong.is_empty(),
        "{} lines of the specification are wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    println!("{} rules, each with its own label", rules.len());
}

// ---- the cases ----

/// The labels a line `//@ rule: ...` names.
fn named(line: &str) -> Option<Vec<String>> {
    let rest = line.trim().strip_prefix("//@ rule:")?;
    Some(rest.split_whitespace().map(str::to_string).collect())
}

enum Verdict {
    Rejected(String),
    Accepted,
}

/// A program of `tests/rules`.
struct Case {
    rules: Vec<String>,
    verdict: Verdict,
    files: Vec<(PathBuf, String)>,
}

/// The program of a case file: its header, and then one file, `T.cleat`, or several,
/// each after a line `//@ file: path`.
fn case(path: &Path) -> Result<Case, String> {
    let text = read(path);
    let mut rules = Vec::new();
    let mut verdict = None;
    let mut files = Vec::new();
    let mut name = "T.cleat".to_string();
    let mut body = String::new();
    let mut header = true;
    for line in text.split('\n') {
        if header {
            if let Some(labels) = named(line) {
                rules.extend(labels);
                continue;
            }
            if let Some(words) = line.strip_prefix("//@ rejected: ") {
                verdict = Some(Verdict::Rejected(words.to_string()));
                continue;
            }
            if line == "//@ accepted" {
                verdict = Some(Verdict::Accepted);
                continue;
            }
            header = false;
        }
        if let Some(n) = line.strip_prefix("//@ file: ") {
            if !body.trim().is_empty() {
                files.push((PathBuf::from(&name), std::mem::take(&mut body)));
            }
            name = n.trim().to_string();
            body.clear();
        } else if line.starts_with("//@") {
            return Err(format!("`{line}` is not a line of a case"));
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    if !body.trim().is_empty() {
        files.push((PathBuf::from(&name), body));
    }
    if rules.is_empty() {
        return Err("it names no rule".to_string());
    }
    let verdict = verdict.ok_or("it says neither `//@ rejected:` nor `//@ accepted`")?;
    Ok(Case {
        rules,
        verdict,
        files,
    })
}

/// The directories of `tests/rules`, one for each chapter.
fn case_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(tests_dir().join("rules"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// A line `//@ rule:` in a run program or a test, and the labels it names.
struct Marker {
    at: String,
    rules: Vec<String>,
}

/// The lines `//@ rule:` of the run programs, and whether `tests/behavior.rs` runs each
/// program.
fn program_markers(wrong: &mut Vec<String>) -> Vec<Marker> {
    let dir = tests_dir();
    let driver = read(&dir.join("behavior.rs"));
    let mut out = Vec::new();
    for path in files_in(&dir.join("behavior"), "cleat") {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let text = read(&path);
        let mut marked = false;
        for (i, line) in text.lines().enumerate() {
            if let Some(rules) = named(line) {
                marked = true;
                out.push(Marker {
                    at: format!("tests/behavior/{name}.cleat:{}", i + 1),
                    rules,
                });
            }
        }
        if marked && !driver.contains(&format!("\"{name}\"")) {
            wrong.push(format!(
                "tests/behavior/{name}.cleat names rules, and tests/behavior.rs does not run it"
            ));
        }
    }
    out
}

/// The test a line `//@ rule:` stands above: a function marked `#[test]`, or a row of a
/// macro that makes one.
fn test_below(lines: &[&str], from: usize) -> Option<String> {
    let mut test = false;
    for line in &lines[from..] {
        let l = line.trim();
        if l.starts_with("//") {
            continue;
        }
        if l.starts_with("#[") {
            test |= l == "#[test]";
            continue;
        }
        let name: String = l
            .trim_start_matches("fn ")
            .trim_start_matches('(')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let rest = &l[l.find(&name).unwrap_or(0) + name.len()..];
        let ok = if l.starts_with("fn ") {
            test && rest.starts_with('(')
        } else {
            rest.starts_with(" => ") || (l.starts_with('(') && rest.starts_with(", "))
        };
        return (ok && !name.is_empty()).then_some(name);
    }
    None
}

/// The lines `//@ rule:` of the tests, each above the test that shows its rules.
fn test_markers(wrong: &mut Vec<String>) -> Vec<Marker> {
    let mut out = Vec::new();
    for path in files_in(&tests_dir(), "rs") {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = read(&path);
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if let Some(rules) = named(line) {
                let at = format!("tests/{name}:{}", i + 1);
                if test_below(&lines, i + 1).is_none() {
                    wrong.push(format!("{at}: the line does not stand above a test"));
                }
                out.push(Marker { at, rules });
            }
        }
    }
    out
}

/// The lines of the `reasons.txt` files: a label, and why no program shows its rule.
fn reasons(wrong: &mut Vec<String>) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for dir in case_dirs() {
        let path = dir.join("reasons.txt");
        if !path.exists() {
            continue;
        }
        let chapter = dir.file_name().unwrap().to_string_lossy().to_string();
        for (i, line) in read(&path).lines().enumerate() {
            let at = format!("tests/rules/{chapter}/reasons.txt:{}", i + 1);
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            match line.split_once(": ") {
                Some((label, why)) if is_label(label) && !why.trim().is_empty() => {
                    out.push((at, label.to_string(), why.to_string()))
                }
                _ => wrong.push(format!("{at}: a line of reasons is `label: reason`")),
            }
        }
    }
    out
}

#[test]
fn every_rule_has_a_case_and_every_case_names_a_rule() {
    let (rules, _) = rules();
    let labels: BTreeSet<&str> = rules.iter().map(|r| r.label.as_str()).collect();
    let words: BTreeSet<String> = chapters().into_iter().map(|c| c.1).collect();
    let mut wrong = Vec::new();
    // Each label, and where a case names it.
    let mut shown: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut programs = 0;
    for dir in case_dirs() {
        let chapter = dir.file_name().unwrap().to_string_lossy().to_string();
        if !words.contains(&chapter) {
            wrong.push(format!(
                "tests/rules/{chapter} is not the word of a chapter"
            ));
        }
        for path in files_in(&dir, "cleat") {
            let at = format!(
                "tests/rules/{chapter}/{}",
                path.file_name().unwrap().to_string_lossy()
            );
            match case(&path) {
                Ok(c) => {
                    programs += 1;
                    for label in &c.rules {
                        if label.split('.').next() != Some(chapter.as_str()) {
                            wrong.push(format!("{at}: `{label}` is not a rule of this chapter"));
                        }
                        shown.entry(label.clone()).or_default().push(at.clone());
                    }
                }
                Err(e) => wrong.push(format!("{at}: {e}")),
            }
        }
    }
    let in_programs = program_markers(&mut wrong);
    let in_tests = test_markers(&mut wrong);
    for m in in_programs.iter().chain(&in_tests) {
        for label in &m.rules {
            shown.entry(label.clone()).or_default().push(m.at.clone());
        }
    }
    let reasons = reasons(&mut wrong);
    let mut explained = BTreeMap::new();
    for (at, label, _) in &reasons {
        if let Some(first) = explained.insert(label.clone(), at.clone()) {
            wrong.push(format!("{at}: `{label}` has a reason at {first} already"));
        }
        if let Some(cases) = shown.get(label) {
            wrong.push(format!(
                "{at}: `{label}` has a case, at {}, so it needs no reason",
                cases[0]
            ));
        }
    }
    for (label, places) in &shown {
        if !labels.contains(label.as_str()) {
            wrong.push(format!("{}: there is no rule `{label}`", places[0]));
        }
    }
    for (at, label, _) in &reasons {
        if !labels.contains(label.as_str()) {
            wrong.push(format!("{at}: there is no rule `{label}`"));
        }
    }
    for r in &rules {
        if !shown.contains_key(&r.label) && !explained.contains_key(&r.label) {
            wrong.push(format!(
                "spec/{}:{}: `{}` has no case",
                r.file, r.line, r.label
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} things are wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    let count = |places: &[String], from: &str| places.iter().any(|p| p.starts_with(from));
    let by = |from: &str| shown.values().filter(|p| count(p, from)).count();
    println!(
        "{} rules: {} have programs ({programs} in all), {} a line of a run program, {} a test, {} a reason",
        rules.len(),
        by("tests/rules/"),
        by("tests/behavior/"),
        shown
            .values()
            .filter(|p| p
                .iter()
                .any(|x| x.starts_with("tests/") && x.contains(".rs:")))
            .count(),
        reasons.len()
    );
    for (_, label, why) in &reasons {
        println!("  {label}: {why}");
    }
}

// ---- the programs ----

fn diagnostics(files: &[(PathBuf, String)]) -> Vec<String> {
    match analyze_sources(files) {
        Ok(_) => Vec::new(),
        Err(d) => d
            .into_iter()
            .map(|d| {
                format!(
                    "{}:{}:{}: {}",
                    d.file.display(),
                    d.line,
                    d.column,
                    d.message
                )
            })
            .collect(),
    }
}

/// Every program of the chapter whose labels begin with this word.
fn run(word: &str) {
    let dir = tests_dir().join("rules").join(word);
    let mut failures = Vec::new();
    let mut rejected = 0;
    let mut accepted = 0;
    for path in files_in(&dir, "cleat") {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let c = match case(&path) {
            Ok(c) => c,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        let d = diagnostics(&c.files);
        let before = failures.len();
        match &c.verdict {
            Verdict::Rejected(words) => {
                rejected += 1;
                if d.is_empty() {
                    failures.push(format!(
                        "{name}\n  must be rejected, and the checker accepts it"
                    ));
                } else if !d.iter().any(|m| m.contains(words.as_str())) {
                    failures.push(format!(
                        "{name}\n  is rejected, and not for its rule\n  expected: {words}\n  the checker says:\n  {}",
                        d.join("\n  ")
                    ));
                }
            }
            Verdict::Accepted => {
                accepted += 1;
                if !d.is_empty() {
                    failures.push(format!(
                        "{name}\n  must be accepted, and the checker says:\n  {}",
                        d.join("\n  ")
                    ));
                }
            }
        }
        let verdict = if failures.len() == before {
            "ok  "
        } else {
            "FAIL"
        };
        println!("{verdict} {name}: {}", c.rules.join(" "));
    }
    println!(
        "{word}: {rejected} programs rejected each for its rule, {accepted} programs accepted"
    );
    assert!(
        failures.is_empty(),
        "{} programs of tests/rules/{word} went wrong:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

macro_rules! chapters {
    ($($name:ident => $word:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                run($word);
            }
        )*
    };
}

chapters![
    chapter_01_source => "source",
    chapter_02_objects => "objects",
    chapter_03_visibility => "visibility",
    chapter_04_methods => "methods",
    chapter_05_null_and_unit => "null",
    chapter_06_numbers => "numbers",
    chapter_07_generics => "generics",
    chapter_08_annotations => "annotations",
    chapter_09_execution => "execution",
    chapter_10_omissions => "omissions",
    chapter_11_syntax => "syntax",
    chapter_12_flow => "flow",
    chapter_13_concurrency => "concurrency",
];

// ---- the rules a checker's verdict cannot show ----

//@ rule: source.text.byte-sequence-not
#[test]
fn a_file_that_is_not_utf8_is_rejected() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("not_utf8");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Bad.cleat");
    std::fs::write(
        &file,
        [
            b'c', b'l', b'a', b's', b's', b' ', 0xff, 0xfe, b' ', b'{', b'}',
        ],
    )
    .unwrap();
    let err = cleatc::analyze(&[file.as_path()])
        .err()
        .expect("bytes that are not UTF-8");
    assert!(
        err.iter().any(|d| d.message.contains("UTF-8")),
        "{}",
        err[0].message
    );
    std::fs::write(&file, "class Bad { }").unwrap();
    assert!(cleatc::analyze(&[file.as_path()]).is_ok());
}

//@ rule: visibility.fixed.main-entry-class
//@ rule: execution.startup.class-declares-exactly
//@ rule: execution.startup.class-declares-rejected
#[test]
fn an_entry_class_declares_exactly_one_main() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("entry_rule");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [
        (
            "Both",
            "public class Both { public static void main() { } public static void main(String[] args) { } }",
        ),
        (
            "Neither",
            "public class Neither { public static void start() { } }",
        ),
        // Section 3.5: `main` of the entry class is `public`.
        ("Private", "public class Private { static void main() { } }"),
    ] {
        let file = dir.join(format!("{name}.cleat"));
        std::fs::write(&file, text).unwrap();
        let err = cleatc::build_roots(&[file.as_path()], name, &dir.join(format!("{name}.exe")))
            .expect_err(name);
        assert!(
            err.iter().any(|d| d.message.contains("not an entry class")),
            "{name}: {}",
            err[0].message
        );
    }
}

//@ rule: annotations.declaration.implementation-report-use
//@ rule: annotations.declaration.report-not-reject
#[test]
fn a_deprecated_use_is_reported_from_another_unit() {
    let old = (
        PathBuf::from("Old.cleat"),
        "public class Old { @Deprecated(\"use fresh\") public static Int stale() { return 1; } public static Int own() { return stale(); } }\n".to_string(),
    );
    let alone = analyze_sources(std::slice::from_ref(&old)).expect("the declaring unit");
    assert!(
        alone.warnings.is_empty(),
        "a use inside the declaring unit is not reported: {}",
        alone.warnings[0].message
    );
    let used = [
        old,
        (
            PathBuf::from("User.cleat"),
            "public class User { public static Int f() { return Old.stale(); } }\n".to_string(),
        ),
    ];
    let program = analyze_sources(&used).expect("a deprecated use does not reject the program");
    assert_eq!(program.warnings.len(), 1);
    let w = &program.warnings[0];
    assert!(
        w.file.ends_with("User.cleat")
            && w.message.contains("deprecated")
            && w.message.contains("use fresh"),
        "{}",
        w.message
    );
}

/// Section 13.6: a cancelled task raises `CancellationException` at a call of
/// `Thread.checkCancelled`, `sleep`, `join`, `Task.result` or `Condition.await`, and at
/// no others. One method of the prelude raises it, and only those call that method.
//@ rule: concurrency.cancellation.cancelled-task-raises
//@ rule: concurrency.cancellation.task-inside-foreign
#[test]
fn cancellation_is_raised_only_where_section_13_6_says() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut raised = Vec::new();
    let mut callers = Vec::new();
    for entry in std::fs::read_dir(repo.join("prelude")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        if text.contains("new CancellationException") {
            raised.push(name.clone());
        }
        if text.contains("checkCancelled()") {
            callers.push(name);
        }
    }
    raised.sort();
    callers.sort();
    assert_eq!(raised, ["Thread.cleat"]);
    assert_eq!(callers, ["Condition.cleat", "Task.cleat", "Thread.cleat"]);
    // The runtime names the exception's kind and never raises it.
    let mut uses = 0;
    for entry in std::fs::read_dir(repo.join("compiler").join("rt").join("src")).unwrap() {
        uses += std::fs::read_to_string(entry.unwrap().path())
            .unwrap()
            .matches("X_CANCELLATION")
            .count();
    }
    assert_eq!(uses, 1);
}

/// Section 2.2: the prelude is written in the language. The checker takes its source
/// as it takes a program's.
//@ rule: objects.built-in.prelude-written-cleat
//@ rule: annotations.declaration.declaration-ends-body
#[test]
fn the_prelude_is_source_the_checker_accepts() {
    let prelude = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("prelude");
    let files = std::fs::read_dir(&prelude)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("cleat")
        })
        .count();
    assert!(files > 50, "{files} files");
    if let Err(errors) = cleatc::analyze(&[prelude.as_path()]) {
        panic!(
            "{}:{}: {}",
            errors[0].file.display(),
            errors[0].line,
            errors[0].message
        );
    }
}
