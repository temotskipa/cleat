//! Every fenced example in `spec/` as a test. A fragment is wrapped in the smallest
//! program that gives it its names, and `cleatc check` must accept it. A line whose
//! comment begins with `rejected` must be the one line the checker rejects. A listing
//! of prelude declarations is compared with the prelude's source.

use cleatc::{analyze_sources, ast, parse};
use std::path::{Path, PathBuf};

struct Fence {
    lines: Vec<String>,
    /// The line of the chapter the example starts on, for messages.
    at: usize,
}

fn spec_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("spec")
}

/// The fenced examples of one chapter, named by the two digits its file begins with.
fn fences(chapter: &str) -> Vec<Fence> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir(spec_dir()).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.starts_with(chapter) || !name.ends_with(".md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("```") && lines[i].trim() != "```" {
                let mut j = i + 1;
                while !lines[j].starts_with("```") {
                    j += 1;
                }
                found.push(Fence { lines: lines[i + 1..j].iter().map(|s| s.to_string()).collect(), at: i + 2 });
                i = j + 1;
            } else {
                i += 1;
            }
        }
    }
    found
}

enum Kind {
    /// Type declarations: a compilation unit.
    Unit,
    /// Members of the class whose header is given.
    Members(&'static str),
    /// Statements of a static method with this result and these parameters.
    Stmts(&'static str, &'static str),
    /// Method signatures, each given a body that raises, in the class whose header is given.
    Sigs(&'static str),
    /// Declarations of the prelude. With a name, the signatures are members of that class.
    Prelude(Option<&'static str>),
}

struct Plan {
    kind: Kind,
    /// More members of the wrapping class.
    members: &'static str,
    /// More types in the same file.
    support: &'static str,
    /// More files of the same package, each with its name.
    files: &'static [(&'static str, &'static str)],
}

fn p(kind: Kind) -> Plan {
    Plan { kind, members: "", support: "", files: &[] }
}

fn pm(kind: Kind, members: &'static str, support: &'static str) -> Plan {
    Plan { kind, members, support, files: &[] }
}

const SHAPES: &str = "interface Shape { } value class Circle implements Shape { public Rational radius; }";
const POSITIVE: &str = "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } }";
const CALC: &str = "sealed interface Expr permits Num, Neg, Binary { } value class Num implements Expr { public Rational value; } \
    value class Neg implements Expr { public Expr operand; } value class Binary implements Expr { public Op op; public Expr left; public Expr right; } \
    enum Op { ADD, SUBTRACT, MULTIPLY, DIVIDE } \
    class ParseException extends Throwable { public ParseException(String message) { super(message); } } \
    class Parser { public Parser(String text) { } public Expr parse() { throw new ParseException(\"nothing\"); } }";
const GENERIC_SIGS: &str = "static <T> T first(List<T> items) { throw new IllegalStateException(); } \
    static <T> List<T> pair(T a, T b) { throw new IllegalStateException(); } \
    static <A, B> List<B> map(List<A> items, Function1<A, B> f) { throw new IllegalStateException(); } \
    static <T> void addAll(List<? super T> sink, Iterable<T> items) { throw new IllegalStateException(); }";

fn plan(chapter: &str, index: usize) -> Plan {
    use Kind::*;
    match (chapter, index) {
        ("01", 0) => p(Stmts("void", "")),
        ("02", 0) => p(Prelude(Some("Object"))),
        ("02", 1) => p(Prelude(None)),
        ("02", 2) => p(Prelude(None)),
        ("02", 3) => p(Unit),
        ("03", 0) => p(Unit),
        ("03", 1) => pm(Unit, "", "class SubList<T> extends ArrayList<T> { } class ArrayListItr { }"),
        ("04", 0) => p(Prelude(Some("Object"))),
        ("04", 1) => p(Prelude(None)),
        ("04", 2) => p(Prelude(None)),
        ("04", 3) => pm(
            Members("class Fence"),
            "static Rational apply(Op op, Rational left, Rational right) { return left; }",
            CALC,
        ),
        ("05", 0) => p(Sigs("value class String")),
        ("05", 1) => p(Prelude(None)),
        ("05", 2) => p(Stmts("void", "Map<String, Int> counts, String word")),
        ("05", 3) => pm(Stmts("Num", "@Nullable Rational value"), "", CALC),
        ("05", 4) => p(Stmts("void", "Map<String, Int> counts, String word")),
        ("05", 5) => p(Members("class Fence")),
        ("05", 6) => pm(Stmts("void", ""), "static void log(String message) { }", ""),
        ("06", 0) => p(Prelude(None)),
        ("06", 1) => p(Stmts("void", "Float64 total, Float64 mass")),
        ("06", 2) => p(Stmts("void", "")),
        ("07", 0) => p(Unit),
        ("07", 1) => p(Prelude(Some("Array"))),
        ("07", 2) => p(Sigs("class Fence")),
        ("07", 3) => pm(Sigs("class Fence"), "", SHAPES),
        ("07", 4) => pm(Stmts("void", "List<Circle> circles, List<Shape> shapes2"), "", SHAPES),
        ("07", 5) => p(Members("class Fence")),
        ("07", 6) => p(Sigs("class Fence")),
        ("07", 7) => pm(Stmts("void", "List<String> names, List<Shape> shapes, List<Circle> circles"), GENERIC_SIGS, SHAPES),
        ("07", 8) => p(Members("class Fence")),
        ("08", 0) => p(Unit),
        ("08", 1) => pm(Sigs("class Order"), "", "@Refines annotation Priced;"),
        ("08", 2) => p(Unit),
        ("08", 3) => pm(Members("class Fence"), "", "@Refines annotation Positive;"),
        ("08", 4) => pm(
            Stmts("void", "Int n, Rational total"),
            "static Rational share(Rational total, @Positive Int people) { return total / people; }",
            POSITIVE,
        ),
        ("08", 5) => p(Prelude(None)),
        ("08", 6) => p(Unit),
        ("08", 7) => pm(Unit, "", "@Target(Site.TYPE) annotation Frozen;"),
        ("08", 8) => p(Unit),
        ("08", 9) => pm(
            Stmts("void", ""),
            "",
            "value class Meters { public Rational length; @Implicit public static Meters from(Int whole) { return new Meters(whole); } }",
        ),
        ("08", 10) => p(Prelude(None)),
        ("08", 11) => p(Unit),
        ("08", 12) => pm(Members("class Fence"), "", "@Target(Site.METHOD) annotation Check(String name);"),
        ("09", 0) => p(Prelude(None)),
        ("09", 1) => pm(
            Stmts("void", "String text, Int attempts"),
            "static Rational eval(Expr expr) { throw new IllegalStateException(); }",
            CALC,
        ),
        ("09", 2) => pm(Stmts("void", "String path"), "static void process(File input) { }", ""),
        // A public constructor names `Ledger`, so `Ledger` is public too, in its own file.
        ("09", 3) => Plan {
            kind: Unit,
            members: "",
            support: "",
            files: &[("Ledger.cleat", "package spec; public class Ledger { public void register(Account account) { } }")],
        },
        ("09", 4) => p(Unit),
        ("09", 5) => p(Sigs("class Fence")),
        ("12", 0) => p(Members("class Fence")),
        ("12", 1) => pm(Stmts("void", ""), "static void step() { } static void cleanup() { }", ""),
        ("12", 2) => pm(
            Stmts("void", "Boolean strict"),
            "static void use(Int n) { } static Boolean more() { return false; }",
            "",
        ),
        ("12", 3) => p(Unit),
        ("12", 4) => p(Members("class Fence")),
        ("13", 0) => p(Prelude(None)),
        ("13", 1) => p(Stmts("void", "")),
        ("13", 2) => p(Prelude(None)),
        ("13", 3) => p(Unit),
        ("13", 4) => p(Unit),
        ("13", 5) => p(Prelude(None)),
        ("13", 6) => p(Prelude(None)),
        ("13", 7) => pm(
            Stmts("void", "String firstHalf, String secondHalf"),
            "static Int count(String text) { return text.length(); }",
            "",
        ),
        ("13", 8) => pm(Stmts("void", "List<String> paths"), "static void index(String line) { }", ""),
        _ => panic!("the example {chapter}#{index} has no plan in tests/spec_examples.rs"),
    }
}

fn comment_of(line: &str) -> Option<&str> {
    line.find("//").map(|i| line[i + 2..].trim())
}

fn code_of(line: &str) -> &str {
    match line.find("//") {
        Some(i) => line[..i].trim_end(),
        None => line.trim_end(),
    }
}

/// The name of a type that a line declares at the top level.
fn declared_type(line: &str) -> Option<(String, bool)> {
    if line.starts_with(' ') || line.starts_with('}') {
        return None;
    }
    let words: Vec<&str> = code_of(line).split(|c: char| !c.is_alphanumeric() && c != '_' && c != '@').filter(|w| !w.is_empty()).collect();
    for (i, w) in words.iter().enumerate() {
        if matches!(*w, "class" | "interface" | "enum" | "annotation") && i + 1 < words.len() {
            return Some((words[i + 1].to_string(), words.contains(&"public")));
        }
    }
    None
}

/// The files that hold a wrapped example. In each, line `n + 2` is line `n` of the example.
fn wrap(plan: &Plan, lines: &[String]) -> Vec<(PathBuf, String)> {
    let body = lines.join("\n");
    let one = |name: &str, text: String| vec![(PathBuf::from(format!("{name}.cleat")), text)];
    match &plan.kind {
        Kind::Unit => {
            let decls: Vec<(usize, String, bool)> = lines
                .iter()
                .enumerate()
                .filter_map(|(i, l)| declared_type(l).map(|(n, public)| (i, n, public)))
                .collect();
            let publics: Vec<&(usize, String, bool)> = decls.iter().filter(|d| d.2).collect();
            if publics.len() <= 1 {
                let name = publics.first().map(|d| d.1.clone()).unwrap_or_else(|| "Fence".to_string());
                return one(&name, format!("package spec;\n{body}\n{}\n", plan.support));
            }
            // One file for each type: a public type is in a file of its name.
            assert!(plan.support.is_empty(), "an example split into files has no support");
            let mut files = Vec::new();
            let mut start = 0;
            let mut depth = 0i32;
            let mut name: Option<String> = None;
            for (i, l) in lines.iter().enumerate() {
                if name.is_none() {
                    name = declared_type(l).map(|d| d.0);
                }
                let code = code_of(l);
                depth += code.matches('{').count() as i32 - code.matches('}').count() as i32;
                if depth == 0 && (code.ends_with('}') || code.ends_with(';')) && name.is_some() {
                    let mut text = String::from("package spec;\n");
                    for (k, line) in lines.iter().enumerate() {
                        if k >= start && k <= i {
                            text.push_str(line);
                        }
                        text.push('\n');
                    }
                    files.push((PathBuf::from(format!("{}.cleat", name.take().unwrap())), text));
                    start = i + 1;
                }
            }
            files
        }
        Kind::Members(head) => one("Fence", format!("package spec; {head} {{\n{body}\n{}\n}}\n{}\n", plan.members, plan.support)),
        Kind::Stmts(ret, params) => one(
            "Fence",
            format!("package spec; class Fence {{ static {ret} run({params}) {{\n{body}\n}}\n{}\n}}\n{}\n", plan.members, plan.support),
        ),
        Kind::Sigs(head) => {
            let with_bodies: Vec<String> = lines
                .iter()
                .map(|l| if code_of(l).ends_with(')') { format!("{} {{ throw new IllegalStateException(); }}", code_of(l)) } else { l.clone() })
                .collect();
            one("Fence", format!("package spec; {head} {{\n{}\n{}\n}}\n{}\n", with_bodies.join("\n"), plan.members, plan.support))
        }
        Kind::Prelude(_) => unreachable!(),
    }
}

/// Checks the example and returns, for each diagnostic, the line of the example it is
/// on and its text. A diagnostic outside the example fails the test.
fn check(plan: &Plan, lines: &[String], what: &str) -> Vec<(usize, String)> {
    let mut files = wrap(plan, lines);
    let own = files.len();
    files.extend(plan.files.iter().map(|(n, t)| (PathBuf::from(n), t.to_string())));
    let diags = match analyze_sources(&files) {
        Ok(_) => Vec::new(),
        Err(d) => d,
    };
    let mut out = Vec::new();
    for d in diags {
        let in_example = files[..own].iter().any(|(f, _)| *f == d.file) && d.line >= 2 && (d.line as usize - 2) < lines.len();
        assert!(in_example, "{what}: a diagnostic outside the example: {}:{}:{}: {}", d.file.display(), d.line, d.column, d.message);
        out.push((d.line as usize - 2, d.message));
    }
    out
}

fn run(chapter: &str, index: usize) {
    let all = fences(chapter);
    let fence = all.get(index).unwrap_or_else(|| panic!("chapter {chapter} has no example {index}"));
    let plan = plan(chapter, index);
    let what = format!("chapter {chapter}, the example at line {}", fence.at);
    if let Kind::Prelude(class) = &plan.kind {
        compare_with_prelude(&fence.lines, *class, &what);
        return;
    }
    let rejected: Vec<usize> = fence
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| comment_of(l).map(|c| c.starts_with("rejected")).unwrap_or(false))
        .map(|(i, _)| i)
        .collect();
    let needed: Vec<usize> = fence
        .lines
        .iter()
        .enumerate()
        .filter(|(_, l)| comment_of(l).map(|c| c.starts_with("without this line")).unwrap_or(false))
        .map(|(i, _)| i)
        .collect();
    let expect = |lines: &[String], rejected: &[usize], what: &str| {
        let diags = check(&plan, lines, what);
        for (line, message) in &diags {
            assert!(
                rejected.contains(line),
                "{what}: line {} is not marked rejected, and the checker says: {message}\n    {}",
                line + 1,
                lines[*line]
            );
        }
        for r in rejected {
            assert!(diags.iter().any(|(l, _)| l == r), "{what}: line {} is marked rejected, and the checker accepts it\n    {}", r + 1, lines[*r]);
        }
    };
    expect(&fence.lines, &rejected, &what);
    // Each rejected line is the whole reason: without it, the rest stands as marked.
    for r in &rejected {
        let mut lines = fence.lines.clone();
        lines[*r] = String::new();
        let rest: Vec<usize> = rejected.iter().copied().filter(|x| x != r).collect();
        expect(&lines, &rest, &format!("{what}, without its line {}", r + 1));
    }
    // A line the text calls necessary: without it the example is rejected.
    for n in &needed {
        let mut lines = fence.lines.clone();
        lines[*n] = String::new();
        let mut files = wrap(&plan, &lines);
        files.extend(plan.files.iter().map(|(n, t)| (PathBuf::from(n), t.to_string())));
        assert!(analyze_sources(&files).is_err(), "{what}: without line {} the example should be rejected, and the checker accepts it", n + 1);
    }
}

// ---- listings of prelude declarations ----

fn show_anns(anns: &[ast::AnnotationUse]) -> String {
    let mut names: Vec<String> = anns.iter().map(|a| format!("@{} ", a.name.join("."))).collect();
    names.sort();
    names.concat()
}

fn show_type(t: &ast::TypeRef) -> String {
    let mut s = show_anns(&t.annotations);
    s.push_str(&t.name.join("."));
    if let Some(args) = &t.args {
        let parts: Vec<String> = args
            .iter()
            .map(|a| match a {
                ast::TypeArg::Type(t) => show_type(t),
                ast::TypeArg::Wildcard(None, _) => "?".to_string(),
                ast::TypeArg::Wildcard(Some((true, t)), _) => format!("? super {}", show_type(t)),
                ast::TypeArg::Wildcard(Some((false, t)), _) => format!("? extends {}", show_type(t)),
            })
            .collect();
        s.push_str(&format!("<{}>", parts.join(", ")));
    }
    for d in &t.dims {
        s.push_str(&format!(" {}[]", show_anns(d)));
    }
    s
}

fn show_tparams(ps: &[ast::TypeParam]) -> String {
    let parts: Vec<String> = ps
        .iter()
        .map(|tp| {
            let v = match tp.variance {
                ast::Variance::In => "in ",
                ast::Variance::Out => "out ",
                ast::Variance::Invariant => "",
            };
            let bounds: Vec<String> = tp.bounds.iter().map(show_type).collect();
            format!("{}{v}{} extends {}", show_anns(&tp.annotations), tp.name, bounds.join(" & "))
        })
        .collect();
    parts.join(", ")
}

/// The modifiers that are not annotations, and the qualifiers among the annotations.
/// A qualifier written among the modifiers of a method belongs to its result.
fn show_mods(m: &ast::Mods) -> String {
    let quals: Vec<&ast::AnnotationUse> = m.annotations.iter().filter(|a| a.name.last().map(|n| n == "Nullable").unwrap_or(false)).collect();
    format!(
        "{:?} static={} final={} open={} abstract={} sealed={} foreign={} {}",
        m.audience,
        m.is_static,
        m.is_final,
        m.is_open,
        m.is_abstract,
        m.is_sealed,
        m.is_foreign,
        quals.iter().map(|a| format!("@{}", a.name.join("."))).collect::<String>()
    )
}

fn listed_anns(m: &ast::Mods) -> Vec<String> {
    m.annotations.iter().map(|a| a.name.join(".")).collect()
}

fn show_params(ps: &[ast::Param]) -> String {
    let parts: Vec<String> =
        ps.iter().map(|p| format!("{}{}{} {}", show_anns(&p.mods.annotations), show_type(&p.ty), if p.varargs { "..." } else { "" }, p.name)).collect();
    parts.join(", ")
}

fn prelude_type(name: &str) -> ast::TypeDecl {
    let dir = cleatc::prelude_dir();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("cleat") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let unit = parse::parse_unit(&path, &text).unwrap_or_else(|d| panic!("{}: {}", path.display(), d.message));
        if let Some(t) = unit.types.into_iter().find(|t| t.name == name) {
            return t;
        }
    }
    panic!("the prelude declares no type named `{name}`");
}

fn compare_with_prelude(lines: &[String], class: Option<&str>, what: &str) {
    // A listing ends a signature where its body would be.
    let listed: Vec<String> = lines.iter().map(|l| if code_of(l).ends_with(')') { format!("{};", code_of(l)) } else { l.clone() }).collect();
    let text = match class {
        Some(name) => format!("class {name} {{\n{}\n}}\n", listed.join("\n")),
        None => listed.join("\n"),
    };
    let unit = parse::parse_unit(Path::new("listing.cleat"), &text).unwrap_or_else(|d| panic!("{what}: the listing does not parse: {}:{}: {}", d.line, d.column, d.message));
    assert!(!unit.types.is_empty(), "{what}: the listing declares nothing");
    for listing in &unit.types {
        let real = prelude_type(&listing.name);
        let name = &listing.name;
        if class.is_none() {
            assert_eq!(format!("{:?}", listing.kind), format!("{:?}", real.kind), "{what}: the kind of `{name}`");
            assert_eq!(show_mods(&listing.mods), show_mods(&real.mods), "{what}: the modifiers of `{name}`");
            assert_eq!(show_tparams(&listing.type_params), show_tparams(&real.type_params), "{what}: the type parameters of `{name}`");
            for a in listed_anns(&listing.mods) {
                assert!(listed_anns(&real.mods).contains(&a), "{what}: the prelude's `{name}` does not carry `@{a}`");
            }
            if !listing.extends.is_empty() {
                let (a, b): (Vec<String>, Vec<String>) = (listing.extends.iter().map(show_type).collect(), real.extends.iter().map(show_type).collect());
                assert_eq!(a, b, "{what}: what `{name}` extends");
            }
            assert_eq!(listing.elements.len(), real.elements.len(), "{what}: the elements of `{name}`");
            for (x, y) in listing.elements.iter().zip(real.elements.iter()) {
                assert_eq!((show_type(&x.ty), &x.name, x.default.is_some()), (show_type(&y.ty), &y.name, y.default.is_some()), "{what}: an element of `{name}`");
            }
        }
        for member in &listing.members {
            match member {
                ast::Member::Method(m) => {
                    let sig = |m: &ast::MethodDecl| {
                        format!("<{}> {}({}) this={}", show_tparams(&m.type_params), m.name, show_params(&m.params), m.receiver.as_ref().map(show_type).unwrap_or_default())
                    };
                    let found = real.members.iter().find_map(|r| match r {
                        ast::Member::Method(r) if sig(r) == sig(m) => Some(r),
                        _ => None,
                    });
                    let Some(r) = found else {
                        panic!("{what}: the prelude's `{name}` declares no method `{}`", sig(m));
                    };
                    assert_eq!(show_mods(&m.mods), show_mods(&r.mods), "{what}: the modifiers of `{name}.{}`", m.name);
                    assert_eq!(m.result.as_ref().map(show_type), r.result.as_ref().map(show_type), "{what}: the result of `{name}.{}`", m.name);
                    for a in listed_anns(&m.mods) {
                        assert!(listed_anns(&r.mods).contains(&a), "{what}: the prelude's `{name}.{}` does not carry `@{a}`", m.name);
                    }
                }
                ast::Member::Ctor(c) => {
                    let sig = |c: &ast::CtorDecl| c.params.as_ref().map(|p| show_params(p));
                    let found = real.members.iter().find_map(|r| match r {
                        ast::Member::Ctor(r) if sig(r) == sig(c) => Some(r),
                        _ => None,
                    });
                    let Some(r) = found else {
                        panic!("{what}: the prelude's `{name}` declares no constructor ({})", sig(c).unwrap_or_default());
                    };
                    assert_eq!(show_mods(&c.mods), show_mods(&r.mods), "{what}: the modifiers of a constructor of `{name}`");
                }
                ast::Member::Field(f) => {
                    let found = real.members.iter().any(|r| matches!(r, ast::Member::Field(r) if r.name == f.name && show_type(&r.ty) == show_type(&f.ty)));
                    assert!(found, "{what}: the prelude's `{name}` declares no field `{}`", f.name);
                }
                ast::Member::StaticInit(_) => {}
            }
        }
    }
}

// ---- one test for each example ----

macro_rules! examples {
    ($(($name:ident, $chapter:literal, $index:literal)),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                run($chapter, $index);
            }
        )*
        const TESTED: &[(&str, usize)] = &[$(($chapter, $index)),*];
    };
}

examples![
    (source_text_block, "01", 0),
    (objects_root_methods, "02", 0),
    (objects_static_requirement, "02", 1),
    (objects_class_object, "02", 2),
    (objects_enums, "02", 3),
    (visibility_audiences, "03", 0),
    (visibility_only, "03", 1),
    (methods_receiver_parameter, "04", 0),
    (methods_function_interfaces, "04", 1),
    (methods_iterable, "04", 2),
    (methods_switch_over_types, "04", 3),
    (null_receiver, "05", 0),
    (null_class, "05", 1),
    (null_narrowing, "05", 2),
    (null_branch_that_raises, "05", 3),
    (null_coalescing, "05", 4),
    (unit_void_method, "05", 5),
    (unit_as_type_argument, "05", 6),
    (numbers_interfaces, "06", 0),
    (numbers_literals, "06", 1),
    (numbers_conversion, "06", 2),
    (generics_type_parameters, "07", 0),
    (generics_method, "07", 1),
    (generics_bounds, "07", 2),
    (generics_wildcard_signatures, "07", 3),
    (generics_wildcard_members, "07", 4),
    (generics_naming_the_unknown, "07", 5),
    (generics_inference_signatures, "07", 6),
    (generics_inference, "07", 7),
    (generics_static_requirement, "07", 8),
    (annotations_declaring, "08", 0),
    (annotations_receiver, "08", 1),
    (annotations_narrows, "08", 2),
    (annotations_qualified_parameter, "08", 3),
    (annotations_narrowing, "08", 4),
    (annotations_nullable, "08", 5),
    (annotations_tag, "08", 6),
    (annotations_required_tag, "08", 7),
    (annotations_implicit, "08", 8),
    (annotations_implicit_use, "08", 9),
    (annotations_mirrors, "08", 10),
    (annotations_check, "08", 11),
    (annotations_reading, "08", 12),
    (execution_throwable, "09", 0),
    (execution_try, "09", 1),
    (execution_using, "09", 2),
    (execution_constructors, "09", 3),
    (execution_value_constructor, "09", 4),
    (execution_main, "09", 5),
    (flow_end_of_body, "12", 0),
    (flow_unreachable, "12", 1),
    (flow_definite_assignment, "12", 2),
    (flow_fields_in_constructor, "12", 3),
    (flow_narrowing, "12", 4),
    (concurrency_thread, "13", 0),
    (concurrency_start_and_join, "13", 1),
    (concurrency_lock, "13", 2),
    (concurrency_exclusive, "13", 3),
    (concurrency_condition, "13", 4),
    (concurrency_atomics, "13", 5),
    (concurrency_scope, "13", 6),
    (concurrency_fork, "13", 7),
    (concurrency_cancellation, "13", 8),
];

/// Every fenced example of every chapter has a test above, and no test names an example
/// that is gone.
#[test]
fn every_example_has_a_test() {
    let mut chapters: Vec<String> = std::fs::read_dir(spec_dir())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".md"))
        .map(|n| n[..2].to_string())
        .collect();
    chapters.sort();
    let mut total = 0;
    for chapter in &chapters {
        let count = fences(chapter).len();
        total += count;
        for index in 0..count {
            assert!(TESTED.contains(&(chapter.as_str(), index)), "the example {chapter}#{index} has no test");
        }
    }
    assert_eq!(total, TESTED.len(), "a test names an example that the specification no longer has");
}
