//! The tables of the specification against the compiler and the prelude, and the
//! documents against each other. Where a chapter lists words, methods, classes or
//! conversions, this checks that the implementation has exactly those, so the text and
//! the code cannot drift apart unnoticed.

use cleatc::{analyze_sources, ast, lex, parse};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).replace("\r\n", "\n")
}

/// The text of the chapter whose file name begins with these two digits.
fn chapter(number: &str) -> String {
    for entry in std::fs::read_dir(repo().join("spec")).unwrap() {
        let path = entry.unwrap().path();
        if path.file_name().unwrap().to_string_lossy().starts_with(number) {
            return read(&path);
        }
    }
    panic!("no chapter {number}");
}

/// The rows of the `nth` table whose header line begins with `header`. An escaped `|`
/// in a cell is kept.
fn table(text: &str, header: &str, nth: usize) -> Vec<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let starts: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| l.starts_with(header)).map(|(i, _)| i).collect();
    let start = *starts.get(nth).unwrap_or_else(|| panic!("no table `{header}` number {nth}"));
    let mut rows = Vec::new();
    for line in &lines[start + 2..] {
        if !line.starts_with('|') {
            break;
        }
        let kept = line.replace("\\|", "\u{1}");
        let cells: Vec<String> = kept.trim_matches('|').split('|').map(|c| c.trim().replace('\u{1}', "|")).collect();
        rows.push(cells);
    }
    assert!(!rows.is_empty(), "the table `{header}` has no rows");
    rows
}

/// What a cell writes between backquotes.
fn ticks(cell: &str) -> Vec<String> {
    cell.split('`').skip(1).step_by(2).map(|s| s.to_string()).collect()
}

fn messages(text: &str) -> Vec<String> {
    match analyze_sources(&[(PathBuf::from("T.cleat"), text.to_string())]) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| d.message).collect(),
    }
}

fn prelude_units() -> Vec<ast::Unit> {
    let mut units = Vec::new();
    for entry in std::fs::read_dir(cleatc::prelude_dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("cleat") {
            units.push(parse::parse_unit(&path, &read(&path)).unwrap_or_else(|d| panic!("{}: {}", path.display(), d.message)));
        }
    }
    units
}

fn prelude_type<'a>(units: &'a [ast::Unit], name: &str) -> Option<&'a ast::TypeDecl> {
    units.iter().flat_map(|u| u.types.iter()).find(|t| t.name == name)
}

fn has_method(t: &ast::TypeDecl, name: &str) -> bool {
    t.members.iter().any(|m| matches!(m, ast::Member::Method(md) if md.name == name))
}

/// The name a method listing gives: `static fromUtf8(UInt8[] bytes)` is `fromUtf8`.
fn listed_name(text: &str) -> String {
    let text = text.strip_prefix("static ").unwrap_or(text);
    text.split('(').next().unwrap().trim().to_string()
}

const NUMERIC: [&str; 11] = ["Int8", "Int16", "Int32", "Int", "UInt8", "UInt16", "UInt32", "UInt64", "Float32", "Float64", "Rational"];
const INTEGERS: [&str; 8] = ["Int8", "Int16", "Int32", "Int", "UInt8", "UInt16", "UInt32", "UInt64"];

// ---- chapter 1 ----

/// The words of the unlabeled listing that follows `after`.
fn word_list(text: &str, after: &str) -> BTreeSet<String> {
    let rest = &text[text.find(after).unwrap_or_else(|| panic!("chapter 1 has no `{after}`"))..];
    let open = rest.find("```\n").expect("a listing") + 4;
    let close = rest[open..].find("```").expect("the end of the listing");
    rest[open..open + close].split_whitespace().map(|w| w.to_string()).collect()
}

#[test]
fn the_keywords_of_section_1_3_are_the_lexers() {
    let text = chapter("01");
    let keywords: BTreeSet<String> = lex::KEYWORDS.iter().map(|w| w.to_string()).collect();
    let reserved: BTreeSet<String> = lex::RESERVED.iter().map(|w| w.to_string()).collect();
    assert_eq!(word_list(&text, "**Keywords.**"), keywords);
    assert_eq!(word_list(&text, "These words are reserved."), reserved);
}

#[test]
fn a_contextual_word_is_an_identifier_elsewhere() {
    let text = chapter("01");
    let mut seen = 0;
    for row in table(&text, "| Word | Is a keyword |", 0) {
        for word in ticks(&row[0]) {
            seen += 1;
            let m = messages(&format!("class A {{ Int {word} = 1; Int f(Int n) {{ var {word}2 = {word} + n; return {word}2; }} }}"));
            assert!(m.is_empty(), "`{word}` cannot name a field: {m:?}");
        }
    }
    assert_eq!(seen, 10);
    // A keyword and a reserved word cannot.
    for word in lex::KEYWORDS.iter().chain(lex::RESERVED.iter()) {
        assert!(!messages(&format!("class A {{ Int {word} = 1; }}")).is_empty(), "`{word}` names a field");
    }
}

// ---- chapter 2 ----

#[test]
fn every_enum_has_the_methods_of_section_2_9() {
    let rows = table(&chapter("02"), "| Method | Result |", 0);
    let names: Vec<String> = rows.iter().flat_map(|r| ticks(&r[0])).map(|t| listed_name(&t)).collect();
    assert_eq!(names, ["name", "ordinal", "values", "valueOf"]);
    let m = messages("enum E { A, B } class C { static String f() { E[] all = E.values(); E one = E.valueOf(\"A\"); Int at = E.B.ordinal(); return all[at].name() + one.name(); } }");
    assert!(m.is_empty(), "{m:?}");
}

// ---- chapter 4 ----

#[test]
fn each_operator_of_section_4_6_is_its_method() {
    let rows = table(&chapter("04"), "| Spelling | Call |", 0);
    assert_eq!(rows.len(), 20);
    for row in rows {
        let (spelling, call) = (ticks(&row[0])[0].clone(), ticks(&row[1])[0].clone());
        let method = call.trim_start_matches("a.").split('(').next().unwrap().to_string();
        let args = call.split('(').nth(1).unwrap().trim_end_matches(')');
        let arity = if args.trim().is_empty() { 0 } else { args.split(',').count() };
        let params: Vec<String> = (0..arity).map(|i| format!("K p{i}")).collect();
        let is_store = spelling.contains("] =");
        let declaration = if is_store {
            format!("public void {method}({}) {{ }}", params.join(", "))
        } else {
            format!("public K {method}({}) {{ return this; }}", params.join(", "))
        };
        let statement = if is_store { format!("{spelling};") } else { format!("var r = {spelling};") };
        let program = |with: bool| format!("class K {{ {} static void use(K a, K b, K i, K e) {{ {statement} }} }}", if with { declaration.as_str() } else { "" });
        if method == "equals" {
            // Every class has `equals`, from `Object`.
            let m = messages(&program(false));
            assert!(m.is_empty(), "`{spelling}`: {m:?}");
            continue;
        }
        let m = messages(&program(true));
        assert!(m.is_empty(), "`{spelling}` with `{method}` declared: {m:?}");
        let m = messages(&program(false));
        assert!(m.iter().any(|x| x.contains(&format!("`{method}`"))), "`{spelling}` without `{method}`: {m:?}");
    }
}

#[test]
fn the_machine_types_of_section_4_11_are_accepted_on_a_foreign_method() {
    let rows = table(&chapter("04"), "| Cleat type | C type |", 0);
    let mut seen = 0;
    for row in &rows {
        for ty in ticks(&row[0]) {
            seen += 1;
            let m = messages(&format!("class C {{ foreign static {ty} same({ty} x); foreign static void fill({ty}[] items); }}"));
            assert!(m.is_empty(), "`{ty}`: {m:?}");
        }
    }
    // Boolean, Char, eight integers, two floats and Pointer.
    assert_eq!(seen, 13);
    // The last row: a value class of machine types.
    let m = messages("value class V { public Float64 x; public Int32 n; } class C { foreign static V same(V v); }");
    assert!(m.is_empty(), "{m:?}");
    for other in ["String", "Rational", "Object", "@Nullable Int"] {
        assert!(!messages(&format!("class C {{ foreign static void take({other} x); }}")).is_empty(), "`{other}` is accepted");
    }
}

// ---- chapter 6 ----

#[test]
fn the_implicit_conversions_are_exactly_the_table_of_section_6_7() {
    let rows = table(&chapter("06"), "| From | Converts implicitly to |", 0);
    let listed = |from: &str, to: &str| rows.iter().any(|r| ticks(&r[0]) == [from] && ticks(&r[1]).iter().any(|t| t == to));
    assert_eq!(rows.len(), 9);
    for from in NUMERIC {
        for to in NUMERIC {
            let accepted = messages(&format!("class C {{ static {to} f({from} x) {{ return x; }} }}")).is_empty();
            assert_eq!(accepted, from == to || listed(from, to), "a `{from}` where a `{to}` is expected");
        }
    }
}

#[test]
fn the_numeric_classes_declare_the_methods_chapter_6_lists() {
    let text = chapter("06");
    let units = prelude_units();
    let names = |nth: usize| -> Vec<String> { table(&text, "| Method | Result |", nth).iter().flat_map(|r| ticks(&r[0])).map(|t| listed_name(&t)).collect() };
    // Section 6.4, on every integer class.
    let division = names(0);
    assert_eq!(division, ["floorDiv", "mod", "truncatingDiv", "truncatingRem"]);
    for class in INTEGERS {
        let t = prelude_type(&units, class).unwrap();
        for m in division.iter().map(|s| s.as_str()).chain(["wrappingPlus", "wrappingMinus", "wrappingTimes", "and", "or", "xor", "complement", "shiftLeft", "shiftRight"]) {
            assert!(has_method(t, m), "`{class}` declares no `{m}`");
        }
        assert!(!has_method(t, "div"), "`{class}` declares `div`");
    }
    // Section 6.5.
    for class in ["Float32", "Float64"] {
        let t = prelude_type(&units, class).unwrap();
        for m in ["div", "floor", "ceil", "truncate", "round", "sqrt", "totalOrder", "nearest"] {
            assert!(has_method(t, m), "`{class}` declares no `{m}`");
        }
        assert!(!has_method(t, "mod"), "`{class}` declares `mod`");
    }
    // Section 6.6.
    let rational = prelude_type(&units, "Rational").unwrap();
    for m in names(1) {
        assert!(has_method(rational, &m), "`Rational` declares no `{m}`");
    }
    // Section 6.9.
    let string = prelude_type(&units, "String").unwrap();
    for m in names(2) {
        assert!(has_method(string, &m), "`String` declares no `{m}`");
    }
    // Every numeric class converts from every numeric class by name (section 6.7).
    for class in NUMERIC {
        let t = prelude_type(&units, class).unwrap();
        let from: Vec<String> = t
            .members
            .iter()
            .filter_map(|m| match m {
                ast::Member::Method(md) if md.name == "from" && md.params.len() == 1 => Some(md.params[0].ty.name.join(".")),
                _ => None,
            })
            .collect();
        for source in NUMERIC {
            assert!(from.iter().any(|f| f == source), "`{class}.from` takes no `{source}`");
        }
    }
}

// ---- chapter 8 ----

#[test]
fn the_sites_of_section_8_2_are_the_constants_of_site() {
    let rows = table(&chapter("08"), "| Site | Declaration |", 0);
    let listed: Vec<String> = rows.iter().map(|r| ticks(&r[0])[0].clone()).collect();
    let units = prelude_units();
    let site = prelude_type(&units, "Site").unwrap();
    let declared: Vec<String> = site.constants.iter().map(|c| c.name.clone()).collect();
    assert_eq!(listed, declared);
}

#[test]
fn the_prelude_declares_the_annotations_of_section_8_10() {
    let rows = table(&chapter("08"), "| Annotation | Written on | Rule |", 0);
    let units = prelude_units();
    assert_eq!(rows.len(), 12);
    for row in rows {
        let name = ticks(&row[0])[0].trim_start_matches('@').to_string();
        let t = prelude_type(&units, &name).unwrap_or_else(|| panic!("the prelude declares no `{name}`"));
        assert_eq!(t.kind, ast::TypeKind::Annotation, "`{name}`");
    }
}

// ---- chapter 9 ----

#[test]
fn the_prelude_declares_the_exceptions_of_section_9_2() {
    let rows = table(&chapter("09"), "| Class | Raised when |", 0);
    let units = prelude_units();
    assert_eq!(rows.len(), 10);
    for row in rows {
        let name = ticks(&row[0])[0].clone();
        let t = prelude_type(&units, &name).unwrap_or_else(|| panic!("the prelude declares no `{name}`"));
        assert!(t.mods.is_open && t.kind == ast::TypeKind::Class, "`{name}` is an open class");
        assert_eq!(t.extends.iter().map(|e| e.name.join(".")).collect::<Vec<_>>(), ["Throwable"], "`{name}`");
    }
}

// ---- the documents against each other ----

#[test]
fn the_readme_names_only_what_the_prelude_declares() {
    let rows = table(&read(&repo().join("README.md")), "| Name | Role |", 0);
    let units = prelude_units();
    for row in rows {
        for name in ticks(&row[0]) {
            let found = match name.split_once('.') {
                Some((class, method)) => prelude_type(&units, class).map(|t| has_method(t, method)).unwrap_or(false),
                None => prelude_type(&units, &name).is_some() || units.iter().flat_map(|u| u.types.iter()).any(|t| has_method(t, &name)),
            };
            assert!(found, "the README names `{name}`, and the prelude declares no such type or method");
        }
    }
}

#[test]
fn the_readme_lists_every_chapter_as_revised() {
    let readme = read(&repo().join("README.md"));
    let rows = table(&readme, "| Chapter | Status |", 0);
    let mut files: Vec<String> = std::fs::read_dir(repo().join("spec")).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).filter(|n| n.ends_with(".md")).collect();
    files.sort();
    assert_eq!(rows.len(), files.len(), "one row for each chapter");
    for (row, file) in rows.iter().zip(files.iter()) {
        let number: u32 = file[..2].parse().unwrap();
        assert!(row[0].starts_with(&format!("{number}. ")), "the row `{}` for {file}", row[0]);
        assert_eq!(row[1], "Revised", "{file}");
        // The chapter's own title carries the same number.
        let title = read(&repo().join("spec").join(file));
        assert!(title.starts_with(&format!("# {number}. ")), "{file} begins `{}`", title.lines().next().unwrap_or(""));
    }
}

/// The anchor GitHub gives a heading.
fn anchor(heading: &str) -> String {
    heading.trim().to_lowercase().chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').collect::<String>().replace(' ', "-")
}

#[test]
fn every_reference_between_the_documents_resolves() {
    let mut documents: Vec<PathBuf> = std::fs::read_dir(repo().join("spec")).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md")).collect();
    for extra in ["README.md", "design/foundations.md", "design/goal.md", "compiler/DESIGN.md"] {
        documents.push(repo().join(extra));
    }
    let mut checked = 0;
    for doc in &documents {
        let text = read(doc);
        let mut rest = text.as_str();
        while let Some(at) = rest.find("](") {
            let after = &rest[at + 2..];
            let end = after.find(')').unwrap_or(after.len());
            let target = &after[..end];
            rest = &after[end..];
            if target.starts_with("http") || target.contains(' ') {
                continue;
            }
            checked += 1;
            let (path, fragment) = target.split_once('#').unwrap_or((target, ""));
            let file = if path.is_empty() { doc.clone() } else { doc.parent().unwrap().join(path) };
            assert!(file.exists(), "{}: `{target}` names a file that is not there", doc.display());
            if !fragment.is_empty() && file.extension().and_then(|e| e.to_str()) == Some("md") {
                let anchors: Vec<String> = read(&file).lines().filter(|l| l.starts_with('#')).map(|l| anchor(l.trim_start_matches('#'))).collect();
                assert!(anchors.iter().any(|a| a == fragment), "{}: `{target}` names a section that is not there", doc.display());
            }
        }
    }
    assert!(checked > 100, "only {checked} references were found");
}

#[test]
fn every_program_the_grammar_must_derive_is_parsed() {
    // Chapter 11 says a program the grammar does not derive is rejected. Each form the
    // other chapters define is written here once, and the parser takes all of them.
    let forms = "package t; import cleat.List; \
        @Refines annotation Q; annotation A(String name, Int n = 0); \
        sealed interface I<out T> permits C { T get(); static Int zero(); } \
        enum E implements Ordered<E> { X(1), Y(2); final Int n; E(Int n) { this.n = n; } \
            @Override public Boolean lessThan(E o) { return n < o.n; } @Override public Boolean atMost(E o) { return n <= o.n; } \
            @Override public Boolean greaterThan(E o) { return n > o.n; } @Override public Boolean atLeast(E o) { return n >= o.n; } \
            @Override public Int compare(E o) { return n.compare(o.n); } } \
        value class V { public Int a; public V { assert a >= 0 : \"negative\"; } } \
        class C implements I<Int> { \
            static Int count = 0; static { count = 1; } \
            package only(D) @Nullable String[] names = new @Nullable String[2]; \
            public C() { super(); } \
            @Override public Int get() { return count; } \
            @Override public static Int zero() { return 0; } \
            @A(name = \"m\") <T extends Object & Ordered<T>> T pick(T a, T... rest) { \
                T best = a; \
                outer: for (T x : rest) { for (var i = 0; i < 2; i++) { if (x > best) { best = x; continue outer; } else { break; } } } \
                return best; } \
            Int all(List<? extends Object> xs, List<? super Int> ys, List<?> zs) { \
                Int[] sizes = new Int[] { xs.size(), ys.size(), zs.size() }; \
                Int[] @Nullable [] grid = new Int[2] @Nullable []; \
                var total = 0; var i = 0; \
                while (i < sizes.length()) { total += sizes[i]; i++; } \
                total = total > 9 ? total % 10 : -total; total <<= 1; total >>= 1; \
                Function1<Int, Int> twice = (n) -> n * 2; Function2<Int, Int, Int> add = (Int x, Int y) -> { return x + y; }; \
                Function1<String, Int> length = String::length; Function0<List<Int>> make = List<Int>::new; \
                Object o = (Object) total; Class k = List<String>.class; \
                @Nullable String first = names[0]; String shown = first ?? \"none\"; \
                switch (total) { case 1, 2 -> { total = 3; } default -> { } } \
                String word = switch (o) { case Int n -> \"int\"; case String s -> s; default -> throw new IllegalStateException(shown); }; \
                try { using (var d = new D(), D e = new D()) { d.close(); } } catch (IllegalStateException x) { throw x; } finally { count++; } \
                return twice.invoke(add.invoke(total, length.invoke(word))) + make.invoke().size() + this.get() + C.zero() + super.hashCode() + grid.length() + k.hashCode(); } \
        } \
        class D { public D() { } public void close() { } }";
    let m = match analyze_sources(&[(PathBuf::from("Forms.cleat"), forms.to_string())]) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| format!("{}:{}: {}", d.line, d.column, d.message)).collect::<Vec<_>>(),
    };
    assert!(m.is_empty(), "{}", m.join("\n"));
}
