//! The chapters against each other, and against `design/foundations.md`. Where two
//! places state the same thing, this checks that they state it the same way.

use cleatc::{analyze_sources, ast, parse};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).replace("\r\n", "\n")
}

/// Every chapter: its two digits and its text, in order.
fn chapters() -> Vec<(String, String)> {
    let mut all: Vec<(String, String)> = std::fs::read_dir(repo().join("spec"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md"))
        .map(|p| (p.file_name().unwrap().to_string_lossy()[..2].to_string(), read(&p)))
        .collect();
    all.sort();
    all
}

fn chapter(number: &str) -> String {
    chapters().into_iter().find(|(n, _)| n == number).unwrap_or_else(|| panic!("no chapter {number}")).1
}

fn foundations() -> String {
    read(&repo().join("design").join("foundations.md"))
}

/// The fenced blocks of a text. `tagged` chooses the examples, which name a language,
/// or the plain listings: the grammar and the word lists.
fn fences(text: &str, tagged: bool) -> Vec<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("```") {
            let is_tagged = lines[i].trim() != "```";
            let mut j = i + 1;
            while !lines[j].starts_with("```") {
                j += 1;
            }
            if is_tagged == tagged {
                out.push(lines[i + 1..j].iter().map(|s| s.to_string()).collect());
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

/// A text without its fenced blocks: the prose, which states rules, apart from the
/// examples, which may use names a program would declare.
fn prose(text: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("```") {
            inside = !inside;
        } else if !inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn table(text: &str, header: &str) -> Vec<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| l.starts_with(header)).unwrap_or_else(|| panic!("no table `{header}`"));
    lines[start + 2..]
        .iter()
        .take_while(|l| l.starts_with('|'))
        .map(|l| l.replace("\\|", "\u{1}").trim_matches('|').split('|').map(|c| c.trim().replace('\u{1}', "|")).collect())
        .collect()
}

fn ticks(text: &str) -> Vec<String> {
    text.split('`').skip(1).step_by(2).map(|s| s.to_string()).collect()
}

fn set<T: AsRef<str>>(items: impl IntoIterator<Item = T>) -> BTreeSet<String> {
    items.into_iter().map(|s| s.as_ref().to_string()).collect()
}

/// The words between double quotes in the grammar: its terminal symbols.
fn grammar_terminals() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for block in fences(&chapter("11"), false) {
        for line in block {
            for (i, part) in line.split('"').enumerate() {
                if i % 2 == 1 {
                    out.insert(part.to_string());
                }
            }
        }
    }
    out
}

fn is_word(s: &str) -> bool {
    s.len() >= 2 && s.chars().all(|c| c.is_ascii_lowercase())
}

fn words_after(text: &str, after: &str) -> BTreeSet<String> {
    let rest = &text[text.find(after).unwrap_or_else(|| panic!("no `{after}`"))..];
    let open = rest.find("```\n").unwrap() + 4;
    let close = rest[open..].find("```").unwrap();
    set(rest[open..open + close].split_whitespace())
}

fn prelude_names() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for entry in std::fs::read_dir(cleatc::prelude_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("cleat") {
            let unit: ast::Unit = parse::parse_unit(&path, &read(&path)).unwrap_or_else(|d| panic!("{}: {}", path.display(), d.message));
            names.extend(unit.types.into_iter().map(|t| t.name));
        }
    }
    names
}

fn messages(text: &str) -> Vec<String> {
    match analyze_sources(&[(PathBuf::from("T.cleat"), text.to_string())]) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| d.message).collect(),
    }
}

fn accepted(members: &str) {
    let m = messages(&format!("class C {{ {members} }}"));
    assert!(m.is_empty(), "rejected: {members}\n{m:?}");
}

fn rejected(members: &str) {
    assert!(!messages(&format!("class C {{ {members} }}")).is_empty(), "accepted: {members}");
}

// ---- chapter against chapter ----

/// A type declaration in an example: its header, and its members with bodies left out.
fn declarations(lines: &[String]) -> Vec<(String, String, BTreeSet<String>)> {
    let text: String = lines.iter().map(|l| l.split("//").next().unwrap()).collect::<Vec<_>>().join("\n");
    let mut out = Vec::new();
    let mut rest = text.as_str();
    loop {
        let found = ["class ", "interface ", "annotation ", "enum "].iter().filter_map(|k| rest.find(k).map(|i| (i, *k))).min();
        let Some((at, keyword)) = found else { break };
        let start = rest[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let after = &rest[at + keyword.len()..];
        let name: String = after.chars().take_while(|c| c.is_alphanumeric()).collect();
        let end = after.find(|c| c == '{' || c == ';').unwrap_or(after.len());
        let header = format!("{}{}", &rest[start..at + keyword.len()], &after[..end]).split_whitespace().collect::<Vec<_>>().join(" ");
        // The body, with the bodies of its methods removed.
        let mut members = BTreeSet::new();
        let mut consumed = end;
        if after[end..].starts_with('{') {
            let (mut depth, mut body, mut close) = (0, String::new(), after.len());
            for (i, c) in after[end..].char_indices() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            close = end + i;
                            break;
                        }
                    }
                    _ if depth == 1 => body.push(c),
                    _ => {}
                }
            }
            consumed = close;
            for m in body.split(|c| c == ';' || c == '\n') {
                let m = m.split_whitespace().collect::<Vec<_>>().join(" ");
                if !m.is_empty() && !m.starts_with('@') {
                    members.insert(m);
                }
            }
        }
        if !name.is_empty() && rest[start..at].trim().chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '@' || c == '(' || c == ')' || c == '.') {
            out.push((name, header, members));
        }
        rest = &after[consumed.min(after.len())..];
    }
    out
}

#[test]
fn a_prelude_type_listed_in_two_chapters_is_the_same_in_both() {
    let prelude = prelude_names();
    let mut seen: BTreeMap<String, Vec<(String, String, BTreeSet<String>)>> = BTreeMap::new();
    for (number, text) in chapters() {
        for block in fences(&text, true) {
            for (name, header, members) in declarations(&block) {
                if prelude.contains(&name) {
                    seen.entry(name).or_default().push((number.clone(), header, members));
                }
            }
        }
    }
    let shared: Vec<&String> = seen.iter().filter(|(_, v)| v.iter().map(|x| &x.0).collect::<BTreeSet<_>>().len() > 1).map(|(k, _)| k).collect();
    assert!(shared.len() >= 3, "expected several types listed in two chapters, found {shared:?}");
    for name in shared {
        let listings = &seen[name];
        let (_, first_header, _) = &listings[0];
        let fullest = listings.iter().max_by_key(|l| l.2.len()).unwrap();
        for (number, header, members) in listings {
            assert_eq!(header, first_header, "`{name}` is declared differently in chapter {number}");
            for m in members {
                assert!(fullest.2.contains(m), "chapter {number} gives `{name}` the member `{m}`, and chapter {} does not", fullest.0);
            }
        }
    }
}

#[test]
fn a_root_method_has_one_signature_in_every_chapter() {
    // `equals` is written in sections 2.5 and 4.1, and each time in full.
    let mut found = Vec::new();
    for (number, text) in chapters() {
        for block in fences(&text, true) {
            for line in block {
                if line.contains("equals(@Nullable Object this") {
                    found.push((number.clone(), line.trim().to_string()));
                }
            }
        }
    }
    assert!(found.len() >= 2, "{found:?}");
    for (number, line) in &found {
        assert_eq!(line, &found[0].1, "chapter {number}");
    }
}

#[test]
fn the_grammar_uses_exactly_the_words_of_chapter_1() {
    let one = chapter("01");
    let keywords = words_after(&one, "**Keywords.**");
    let contextual: BTreeSet<String> = table(&one, "| Word | Is a keyword |").iter().flat_map(|r| ticks(&r[0])).collect();
    let literals = set(["true", "false", "null"]);
    let words: BTreeSet<String> = grammar_terminals().into_iter().filter(|w| is_word(w)).collect();
    for w in &words {
        assert!(keywords.contains(w) || contextual.contains(w) || literals.contains(w), "the grammar uses `{w}`, which chapter 1 does not list");
    }
    for w in keywords.iter().chain(contextual.iter()).chain(literals.iter()) {
        assert!(words.contains(w), "chapter 1 lists `{w}`, and no rule of the grammar uses it");
    }
    // A reserved word has no use.
    for w in words_after(&one, "These words are reserved.") {
        assert!(!words.contains(&w), "the grammar uses the reserved word `{w}`");
    }
}

#[test]
fn the_operators_of_chapter_4_are_tokens_of_chapter_11() {
    let terminals = grammar_terminals();
    let four = chapter("04");
    for row in table(&four, "| Spelling | Call |") {
        let spelling = &ticks(&row[0])[0];
        for symbol in spelling.split(|c: char| c.is_alphanumeric() || c == ' ').filter(|s| !s.is_empty()) {
            assert!(terminals.contains(symbol), "section 4.6 spells `{spelling}`, and the grammar has no token `{symbol}`");
        }
    }
    // The compound assignments, which section 4.6 lists in prose.
    let para = four.lines().find(|l| l.starts_with("**Compound assignment.**")).expect("the paragraph");
    let listed: BTreeSet<String> = ticks(para).iter().flat_map(|t| t.split(' ').map(|w| w.to_string()).collect::<Vec<_>>()).filter(|w| w.len() >= 2 && w.ends_with('=')).collect();
    let grammar: BTreeSet<String> = terminals.iter().filter(|t| t.len() >= 2 && t.ends_with('=') && !["==", "!=", "<=", ">="].contains(&t.as_str())).cloned().collect();
    assert_eq!(listed, grammar);
    // What chapter 10 says is absent is absent from the grammar.
    for gone in [">>>", "synchronized", "throws", "volatile", "goto"] {
        assert!(!terminals.contains(gone), "the grammar has `{gone}`");
    }
}

#[test]
fn every_exception_the_prose_names_is_declared() {
    let table_names: BTreeSet<String> = table(&chapter("09"), "| Class | Raised when |").iter().map(|r| ticks(&r[0])[0].clone()).collect();
    let prelude = prelude_names();
    for name in &table_names {
        assert!(prelude.contains(name), "the prelude declares no `{name}`");
    }
    // Section 9.2 lists the exceptions the language raises. The prose of every chapter
    // names only those, or one that an example of the specification declares.
    let mut declared = table_names.clone();
    for (_, text) in chapters() {
        for block in fences(&text, true) {
            declared.extend(declarations(&block).into_iter().map(|d| d.0));
        }
    }
    let mut seen = BTreeSet::new();
    let mut unknown = Vec::new();
    for (number, text) in chapters() {
        for name in ticks(&prose(&text)) {
            if name.ends_with("Exception") && name.chars().all(|c| c.is_alphanumeric()) {
                seen.insert(name.clone());
                if !declared.contains(&name) {
                    unknown.push(format!("chapter {number}: `{name}`"));
                }
            }
        }
    }
    assert!(unknown.is_empty(), "the prose names exceptions that section 9.2 does not list: {unknown:?}");
    // And every exception of the table is raised by a rule somewhere outside the table.
    for name in &table_names {
        let uses = chapters().iter().map(|(_, t)| prose(t).matches(&format!("`{name}`")).count()).sum::<usize>();
        assert!(uses >= 2, "`{name}` is listed in section 9.2 and no rule raises it");
    }
    assert!(seen.len() >= table_names.len());
}

#[test]
fn every_annotation_the_prose_writes_is_declared() {
    let listed: BTreeSet<String> = table(&chapter("08"), "| Annotation | Written on | Rule |").iter().map(|r| ticks(&r[0])[0].trim_start_matches('@').to_string()).collect();
    let prelude = prelude_names();
    for name in &listed {
        assert!(prelude.contains(name), "the prelude declares no `{name}`");
    }
    let mut declared = listed.clone();
    for (_, text) in chapters() {
        for block in fences(&text, true) {
            declared.extend(declarations(&block).into_iter().map(|d| d.0));
        }
    }
    let mut unknown = Vec::new();
    for (number, text) in chapters() {
        let text = prose(&text);
        let mut rest = text.as_str();
        while let Some(at) = rest.find('@') {
            let name: String = rest[at + 1..].chars().take_while(|c| c.is_alphanumeric()).collect();
            rest = &rest[at + 1..];
            // A single capital letter stands for any qualifier, as `T` does for any type.
            if name.len() > 1 && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && !declared.contains(&name) {
                unknown.push(format!("chapter {number}: `@{name}`"));
            }
        }
    }
    assert!(unknown.is_empty(), "the prose writes annotations that section 8.10 does not list and no example declares: {unknown:?}");
}

#[test]
fn every_chapter_reference_names_the_chapter_it_links() {
    // `[chapter 6](06-numbers.md)` must point at chapter 6.
    let mut checked = 0;
    for (number, text) in chapters() {
        let mut rest = text.as_str();
        while let Some(at) = rest.to_lowercase().find("[chapter ") {
            let after = &rest[at + 9..];
            let said: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let target = after.split("](").nth(1).unwrap_or("");
            let linked: String = target.chars().take(2).collect();
            if !said.is_empty() && after[said.len()..].starts_with("](") {
                checked += 1;
                assert_eq!(format!("{:0>2}", said), linked, "chapter {number} writes `[chapter {said}]({}`", target.split(')').next().unwrap_or(""));
            }
            rest = &after[1..];
        }
        // `[section 4.5](#45-...)` inside a chapter must point at that section.
        let mut rest = text.as_str();
        while let Some(at) = rest.find("[Section ").or_else(|| rest.find("[section ")) {
            let after = &rest[at + 9..];
            let said: String = after.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
            if after[said.len()..].starts_with("](#") {
                checked += 1;
                let anchor: String = after[said.len() + 3..].chars().take_while(|c| *c != ')').collect();
                let digits = said.replace('.', "");
                assert!(anchor.starts_with(&format!("{digits}-")), "chapter {number} writes `[section {said}](#{anchor})`");
                assert!(said.starts_with(&format!("{}.", number.trim_start_matches('0'))), "chapter {number} links section {said} as its own");
            }
            rest = &after[1..];
        }
    }
    assert!(checked > 150, "only {checked} references were found");
}

// ---- foundations.md against the chapters ----

/// The line of the decision record that begins this way.
fn record_line(start: &str) -> String {
    foundations().lines().find(|l| l.starts_with(start)).unwrap_or_else(|| panic!("foundations.md has no line `{start}`")).to_string()
}

#[test]
fn the_record_and_chapter_8_name_the_same_built_in_annotations() {
    let record: BTreeSet<String> = ticks(&record_line("6. The prelude's annotations have rules built into the compiler")).into_iter().map(|t| t.trim_start_matches('@').to_string()).collect();
    let mut chapter8: BTreeSet<String> = table(&chapter("08"), "| Annotation | Written on | Rule |").iter().map(|r| ticks(&r[0])[0].trim_start_matches('@').to_string()).collect();
    // `@Nullable` is the record's second admitted exception, on its own line.
    assert!(chapter8.remove("Nullable"));
    assert!(record_line("2. `null` is built in.").contains("`@Nullable`"));
    assert_eq!(record, chapter8);
}

#[test]
fn the_record_and_chapter_6_name_the_same_numeric_classes() {
    let text = foundations();
    let at = text.find("**Recommendation.** The numeric classes are unrelated final value classes:").expect("the recommendation");
    let block = &text[at..];
    let open = block.find("```\n").unwrap() + 4;
    let close = block[open..].find("```").unwrap();
    let record: BTreeSet<String> = block[open..open + close].split_whitespace().filter(|w| w.chars().next().unwrap().is_uppercase() && w.chars().all(|c| c.is_alphanumeric())).map(|w| w.to_string()).collect();
    let six: BTreeSet<String> = table(&chapter("06"), "| Classes | Values |").iter().flat_map(|r| ticks(&r[0])).collect();
    assert_eq!(record.iter().filter(|w| *w != "IEEE").cloned().collect::<BTreeSet<_>>(), six);
    assert_eq!(six.len(), 11);
    // The interface they share, member for member.
    let listing = |text: &str, which: usize| -> BTreeSet<String> {
        let all: Vec<Vec<String>> = fences(text, true).into_iter().filter(|b| b.iter().any(|l| l.contains("interface Numeric<T>"))).collect();
        declarations(&all[which]).into_iter().find(|d| d.0 == "Numeric").unwrap().2
    };
    assert_eq!(listing(&text, 0), listing(&chapter("06"), 0));
}

#[test]
fn the_record_and_chapter_1_name_the_same_contextual_words() {
    let line = record_line("- The words Cleat adds to Java's are keywords in one position each");
    let named = ticks(&line);
    let one = chapter("01");
    let contextual: BTreeSet<String> = table(&one, "| Word | Is a keyword |").iter().flat_map(|r| ticks(&r[0])).collect();
    let reserved = words_after(&one, "These words are reserved.");
    // The line names the contextual words, then a method name as an example, then two reserved words.
    let first: BTreeSet<String> = named.iter().take(10).cloned().collect();
    assert_eq!(first, contextual);
    for w in ["inline", "when"] {
        assert!(named.iter().any(|n| n == w) && reserved.contains(w), "`{w}`");
    }
}

#[test]
fn the_record_and_chapter_13_name_the_same_cancellation_points() {
    let line = record_line("- Cancellation is checked at");
    let last = |s: &String| s.rsplit('.').next().unwrap().to_string();
    let record: BTreeSet<String> = ticks(&line).iter().map(last).collect();
    let thirteen = chapter("13");
    let at = thirteen.find("A cancelled task raises `CancellationException` at these points").expect("section 13.6");
    let bullets: Vec<&str> = thirteen[at..].lines().skip(1).skip_while(|l| l.is_empty()).take_while(|l| l.starts_with("- ")).collect();
    let chapter_points: BTreeSet<String> = bullets.iter().flat_map(|b| ticks(b)).map(|t| last(&t)).collect();
    // The record also names `Lock.lock`, to say it is not one, as the chapter does next.
    let mut expected = chapter_points.clone();
    expected.insert("lock".into());
    assert_eq!(record, expected);
    assert_eq!(chapter_points, set(["checkCancelled", "sleep", "join", "result", "await"]));
    assert!(thirteen.contains("`Lock.lock` is not such a point"));
}

#[test]
fn the_record_and_chapter_2_admit_the_same_exceptions() {
    // Section 2.2 lists what is built in. The record's list has one more line, because
    // it counts the mirrors apart from the annotations.
    let two = chapter("02");
    let titles: Vec<String> = two.lines().filter(|l| l.len() > 3 && l.as_bytes()[0].is_ascii_digit() && l[1..].starts_with(". **")).map(|l| l.split("**").nth(1).unwrap().to_string()).collect();
    assert_eq!(titles, ["Literals.", "`null`.", "Arrays.", "Method bodies.", "Roots.", "Annotations."]);
    let text = foundations();
    let at = text.find("## The consistency test").unwrap();
    let items: Vec<&str> = text[at..].lines().filter(|l| l.len() > 2 && l.as_bytes()[0].is_ascii_digit() && l[1..].starts_with(". ")).take(7).collect();
    assert_eq!(items.len(), 7);
    for (item, word) in items.iter().zip(["Literals", "`null`", "Array", "method bodies", "roots", "annotations", "mirrors"]) {
        assert!(item.to_lowercase().contains(&word.to_lowercase()), "`{item}` should be about {word}");
    }
}

#[test]
fn every_decided_row_of_the_record_holds_in_the_language() {
    let text = foundations();
    let rows = table(&text, "| # | Question |");
    assert_eq!(rows.len(), 19);
    let status = |n: usize| rows[n - 1][4].clone();
    for n in [3, 4, 5, 6, 8, 11, 12, 13, 14, 15, 16, 17, 18, 19] {
        assert!(status(n).starts_with("Decided"), "row {n}: {}", status(n));
    }
    // 3: the everyday integer is `Int`, 64 bits, for literals, lengths and indexes.
    accepted("static Int f(String[] a) { var i = 0; Int n = a.length() + i; return 9223372036854775807 - n; }");
    rejected("static Int32 f() { var i = 0; return i; }");
    // 4: with no context a decimal literal is an exact `Rational`.
    accepted("static Rational f() { var price = 19.99; return price; }");
    rejected("static Float64 f() { var price = 19.99; return price; }");
    // 5 and 15: no `/` on integers; `floorDiv` and a floored `%`.
    rejected("static Int f(Int a, Int b) { return a / b; }");
    accepted("static Int f(Int a, Int b) { return a.floorDiv(b) + a % b; }");
    accepted("static Int f(Int n) { return switch (n) { case -7 % 3 -> 1; case 2 + 1 -> 3; default -> 0; }; }");
    rejected("static Int f(Int n) { return switch (n) { case -7 % 3 -> 1; case 2 -> 3; default -> 0; }; }");
    // 7: wildcards are kept, beside `in` and `out`.
    accepted("static Int f(List<? extends Object> a, List<? super Int> b, List<?> c, Iterable<Object> d) { return a.size() + b.size() + c.size(); }");
    // 8: a lambda is an instance of a single-method interface, and needs one to stand for.
    accepted("static Int f() { Function1<Int, Int> g = (n) -> n + 1; Comparator<Int> c = (a, b) -> a.compare(b); return g.invoke(c.compare(1, 2)); }");
    rejected("static void f() { var g = (n) -> n; }");
    // 10: a program declares its own annotations and reads them.
    let m = messages("@Target(Site.METHOD) annotation Mark(String note); class C { @Mark(\"x\") public void f() { } static String g(C c) { for (Method m : c.getClass().<Mark>getAnnotatedMethods()) { Mark k = (Mark) m.<Mark>getAnnotation(); return k.note; } return \"\"; } }");
    assert!(m.is_empty(), "{m:?}");
    // 11: one superclass, and an interface has no instance fields.
    assert!(!messages("open class A { } open class B { } class C extends A, B { }").is_empty());
    assert!(!messages("interface I { Int count; }").is_empty());
    // 12: a constructor assigns its fields, then calls `super`.
    let m = messages("open class A { public A() { } } class B extends A { final Int n; public B(Int n) { this.n = n; super(); } }");
    assert!(m.is_empty(), "{m:?}");
    assert!(!messages("open class A { public A() { } } class B extends A { final Int n; public B(Int n) { super(); this.n = n; } }").is_empty());
    // 13 and 14: `@Override` is required, and an override may return a subtype.
    let shapes = "open class Shape { public open Shape copy() { return this; } }";
    assert!(messages(&format!("{shapes} class Circle extends Shape {{ @Override public Circle copy() {{ return this; }} }}")).is_empty());
    assert!(!messages(&format!("{shapes} class Circle extends Shape {{ public Circle copy() {{ return this; }} }}")).is_empty());
    // 16: a result must be used unless the method is `@Discardable`.
    rejected("static Int g() { return 1; } static void f() { g(); }");
    accepted("@Discardable static Int g() { return 1; } static void f() { g(); }");
    // 17: a lossless conversion is implicit, and a program's value class may declare one.
    accepted("static Rational f(Int32 n) { Int wide = n; return wide; }");
    rejected("static Int32 f(Int n) { return n; }");
    let m = messages("value class Meters { public Rational length; @Implicit public static Meters from(Int whole) { return new Meters(whole); } } class C { static Meters f() { Meters m = 3; return m; } }");
    assert!(m.is_empty(), "{m:?}");
    // 18: `??`, and no `?.`.
    accepted("static Int f(@Nullable Int n) { return n ?? 0; }");
    rejected("static Int f(@Nullable String s) { return s?.length() ?? 0; }");
    // 1: `@Nullable T` is `T` plus `null`, and `Null` is outside `Object`.
    accepted("static @Nullable Object f(String s) { @Nullable String t = s; return t; }");
    rejected("static Object f() { return null; }");
    // 2: the numeric classes are unrelated, and share `Numeric<T>`.
    rejected("static Int f(Object o, Int32 n) { return (Int) n; }");
    accepted("static <T extends Numeric<T>> T twice(T x) { return x + x; } static Int f(Int n, Float64 x, Rational r) { var a = twice(n); var b = twice(x); var c = twice(r); return a; }");
}
