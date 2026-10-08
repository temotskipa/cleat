//! Every sentence of the specification that states a requirement, with a program that
//! breaks it and, where it helps, one that keeps it. A requirement is a sentence that
//! uses one of the words chapter 1 gives that meaning ("must", "is rejected"), or says
//! that a program "cannot" or "may not" do something, or that something is "legal".
//!
//! One checker takes all of these programs, together with every example and every
//! program of `design/programs`. If two chapters asked for opposite things of one
//! construct, a program here would be accepted where it must be rejected, or rejected
//! where it must be accepted, and its test would fail.

use cleatc::analyze_sources;
use std::path::{Path, PathBuf};

struct Rule {
    chapter: &'static str,
    /// Words of the sentence, exactly as the chapter writes them.
    phrase: &'static str,
    /// A program the rule rejects, and words of the diagnostic that says so.
    reject: &'static [(&'static str, &'static str)],
    accept: &'static [&'static str],
    /// For a sentence no program can break: why it has no case.
    note: &'static str,
    /// For a sentence about a running program: a program of `tests/behavior`, and the
    /// start of the line of its output that shows the sentence.
    ran: (&'static str, &'static str),
    /// For a sentence another test holds: the file under `tests`, and the test's name.
    held: (&'static str, &'static str),
}

const fn rule(chapter: &'static str, phrase: &'static str, reject: &'static [(&'static str, &'static str)], accept: &'static [&'static str]) -> Rule {
    Rule { chapter, phrase, reject, accept, note: "", ran: ("", ""), held: ("", "") }
}

const fn stated(chapter: &'static str, phrase: &'static str, note: &'static str) -> Rule {
    Rule { chapter, phrase, reject: &[], accept: &[], note, ran: ("", ""), held: ("", "") }
}

const fn ran(chapter: &'static str, phrase: &'static str, program: &'static str, line: &'static str) -> Rule {
    Rule { chapter, phrase, reject: &[], accept: &[], note: "", ran: (program, line), held: ("", "") }
}

const fn held(chapter: &'static str, phrase: &'static str, file: &'static str, test: &'static str) -> Rule {
    Rule { chapter, phrase, reject: &[], accept: &[], note: "", ran: ("", ""), held: (file, test) }
}

impl Rule {
    fn has_programs(&self) -> bool {
        !self.reject.is_empty() || !self.accept.is_empty()
    }

    /// How many of the four ways a case can stand it uses. One is right.
    fn ways(&self) -> usize {
        [self.has_programs(), !self.note.is_empty(), !self.ran.0.is_empty(), !self.held.0.is_empty()].iter().filter(|x| **x).count()
    }
}

// The sentences that define and describe, one file for each chapter.
#[path = "spec_rules/described_01.rs"]
mod described_01;
#[path = "spec_rules/described_02.rs"]
mod described_02;

/// Every case for a sentence that neither requires nor restricts.
fn described() -> Vec<&'static Rule> {
    let chapters: &[&'static [Rule]] = &[described_01::SENTENCES, described_02::SENTENCES];
    chapters.iter().flat_map(|c| c.iter()).collect()
}

/// The chapters whose every sentence has a case.
const DESCRIBED: &[&str] = &["01", "02"];

/// A table that one test holds whole: the chapter, words of its header row, the file
/// under `tests` and the test.
const TABLES: &[(&str, &str, &str, &str)] = &[
    ("04", "| Spelling | Call |", "spec_tables", "each_operator_of_section_4_6_is_its_method"),
    ("04", "| Cleat type | C type |", "spec_tables", "the_machine_types_of_section_4_11_are_accepted_on_a_foreign_method"),
    ("06", "| From | Converts implicitly to |", "spec_tables", "the_implicit_conversions_are_exactly_the_table_of_section_6_7"),
    ("08", "| Site | Declaration |", "spec_tables", "the_sites_of_section_8_2_are_the_constants_of_site"),
    ("08", "| Annotation | Written on | Rule |", "spec_tables", "the_prelude_declares_the_annotations_of_section_8_10"),
    ("09", "| Class | Raised when |", "spec_tables", "the_prelude_declares_the_exceptions_of_section_9_2"),
];

const UNREACHABLE: &str = "this statement is unreachable";
const UNASSIGNED: &str = "is used where it may not have been assigned";
const DISCARDED: &str = "the result of this call must be used";
const NOT_A_STATEMENT: &str = "this expression is not a statement";
const HIDES: &str = "a variable does not hide another";
const NO_DEFAULT: &str = "has no default value";
const SITE: &str = "its `@Target` does not list this site";
const CONSTANT_RAISES: &str = "this constant expression would raise `ArithmeticException`";
const FALSE_LOOP: &str = "a loop whose condition is the constant `false` is rejected";
const FALLS_OFF: &str = "can reach the end of its body";
const FINAL_LOCAL: &str = "it is final and may already have a value";
const FOREIGN_PARAM: &str = "a parameter of a `foreign` method is a machine type or an array of one";

const RULES: &[Rule] = &[
    // ---- chapter 1 ----
    stated("01", "\"Must\" and \"is rejected\" are requirements", "It defines the words the other sentences use."),
    stated("01", "A program that this specification rejects must be diagnosed", "It binds an implementation: every rejected program of this file is one the checker diagnoses."),
    stated("01", "may be rejected, and an implementation must not give it", "It is about programs the specification does not define."),
    stated("01", "An example is normative when it says", "tests/spec_examples.rs holds every example to what it says."),
    stated("01", "not well-formed UTF-8 is rejected", "Checked by `a_file_that_is_not_utf8_is_rejected`, which needs a file of bytes."),
    rule(
        "01",
        "an unclosed one is rejected",
        &[("class A { } /* never closed", "this comment is never closed"), ("/* a /* b */ c */ class A { }", "expected a class, an interface, an enum or an annotation, found `c`")],
        &["/* a /* b */ class A { }"],
    ),
    rule(
        "01",
        "A reserved word is rejected wherever an identifier",
        &[
            ("class A { Int long = 1; }", "`long` is a reserved word"),
            ("class when { }", "`when` is a reserved word"),
            ("class A { void f(Int inline) { } }", "`inline` is a reserved word"),
        ],
        &["class A { Int longer = 1; }"],
    ),
    rule(
        "01",
        "`1L`, `1.0f` and `07` are rejected",
        &[
            ("class A { Int f() { return 1L; } }", "a numeric literal has no suffix"),
            ("class A { Float64 f() { return 1.0f; } }", "a numeric literal has no suffix"),
            ("class A { Int f() { return 07; } }", "a decimal numeral has no leading zero"),
        ],
        &["class A { Int f() { return 0 + 1_000 + 0x1F + 0b101; } Float64 g() { return 1.0e3; } }"],
    ),
    rule(
        "01",
        "An escape that names a surrogate",
        &[(r#"class A { String s = "\uD800"; }"#, "this escape names a surrogate"), (r#"class A { String s = "\u{110000}"; }"#, "a number above 10FFFF")],
        &[r#"class A { String s = "é\u{1F600}"; }"#],
    ),
    rule("01", "Any other character after a backslash is rejected", &[(r#"class A { String s = "\q"; }"#, "`\\q` is not an escape")], &[r#"class A { String s = "\n\t\r\\\"\'"; }"#]),
    rule(
        "01",
        "a line that does not is rejected",
        &[("class A { String s = \"\"\"\n        ok\n    short\n        \"\"\"; }", "every line of a text block begins with its indent")],
        &["class A { String s = \"\"\"\n        ok\n          deeper\n        \"\"\"; }"],
    ),
    rule("01", "`>>>` is rejected", &[("class A { Int f(Int a) { return a >>> 1; } }", "`>>>` is not an operator")], &["class A { Int f(Int a) { return a >> 1; } }"]),
    rule(
        "01",
        "whose types cannot be named from another package",
        &[
            ("==== U.cleat\npublic class U { }\n==== a/B.cleat\npackage a; class B { void f(U u) { } }", "there is no type named `U` here"),
            ("==== U.cleat\npublic class U { }\n==== a/B.cleat\npackage a; import U; class B { }", "the unnamed package"),
        ],
        &["==== U.cleat\npublic class U { }\n==== V.cleat\nclass V { void f(U u) { } }"],
    ),
    rule(
        "01",
        "An import of a type that the file may not name",
        &[
            ("==== a/Hidden.cleat\npackage a; class Hidden { }\n==== b/B.cleat\npackage b; import a.Hidden; class B { }", "`a.Hidden` is visible to its file only, so it is not imported"),
            ("==== a/Inner.cleat\npackage a; package class Inner { }\n==== b/B.cleat\npackage b; import a.Inner; class B { }", "`a.Inner` is outside its audience here, so it is not imported"),
            ("==== b/B.cleat\npackage b; import a.Missing; class B { }", "there is no type named `a.Missing`"),
        ],
        &["==== a/Shown.cleat\npackage a; public class Shown { }\n==== b/B.cleat\npackage b; import a.Shown; class B { void f(Shown s) { } }"],
    ),
    rule(
        "01",
        "When two `*` imports provide the name",
        &[(
            "==== a/X.cleat\npackage a; public class X { public X() { } }\n==== b/X.cleat\npackage b; public class X { public X() { } }\n==== C.cleat\nimport a.*; import b.*; class C { X x = new X(); }",
            "`X` is imported from two packages; write its qualified name",
        )],
        &["==== a/X.cleat\npackage a; public class X { public X() { } }\n==== b/X.cleat\npackage b; public class X { public X() { } }\n==== C.cleat\nimport a.*; import b.*; class C { a.X x = new a.X(); }"],
    ),
    rule(
        "01",
        "Two imports by name of the same simple name are rejected",
        &[(
            "==== a/X.cleat\npackage a; public class X { public X() { } }\n==== b/X.cleat\npackage b; public class X { public X() { } }\n==== C.cleat\nimport a.X; import b.X; class C { }",
            "two imports give the name `X`",
        )],
        &["==== a/X.cleat\npackage a; public class X { public X() { } }\n==== b/X.cleat\npackage b; public class X { public X() { } }\n==== C.cleat\nimport a.X; class C { X x = new X(); }"],
    ),
    rule(
        "01",
        "A variable does not hide another variable",
        &[
            ("class A { void f(Int n) { Int n = 1; } }", HIDES),
            ("class A { void f() { Int x = 1; { Int x = 2; } } }", HIDES),
            ("class A { void f(Int n) { Function1<Int, Int> g = (n) -> n; } }", HIDES),
            ("class A { void f(List<Int> xs) { Int x = 0; for (Int x : xs) { } } }", HIDES),
        ],
        &["class A { Int n = 1; void f() { Int n = 2; this.n = n; } void g() { { Int x = 1; } { Int x = 2; } } }"],
    ),
    rule(
        "01",
        "A label does not hide another label",
        &[("class A { void f() { a: while (true) { a: while (true) { break a; } } } }", "the label `a` is already on an enclosing statement")],
        &["class A { void f() { a: while (true) { b: while (true) { break a; } } } }"],
    ),
    // ---- chapter 2 ----
    stated("02", "and a program cannot tell which", "It says what no program can observe."),
    rule(
        "02",
        "`open`, `sealed` and `abstract` are rejected on a value class",
        &[
            ("open value class V { public Int a; }", "a value class is final"),
            ("abstract value class V { public Int a; }", "a value class is final"),
            ("sealed value class V permits W { public Int a; } class W { }", "a value class is final"),
        ],
        &["value class V { public Int a; }"],
    ),
    rule(
        "02",
        "Each named class must be able to see the sealed class and must extend it",
        &[
            ("sealed class A permits B { } class B { }", "`B` is permitted and does not extend or implement this type"),
            ("==== A.cleat\nsealed class A permits B { }\n==== B.cleat\npublic class B { }", "`B` is permitted and does not extend or implement this type"),
        ],
        &["sealed class A permits B, C { } class B extends A { } class C extends A { }"],
    ),
    rule(
        "02",
        "A `sealed` class with no `permits` clause",
        &[("sealed class A { }", "a sealed type names the types it permits"), ("sealed class A permits { }", "expected a name, found `{`")],
        &["sealed class A permits B { } class B extends A { }"],
    ),
    rule(
        "02",
        "`new` on it is rejected",
        &[("abstract class A { } class B { A f() { return new A(); } }", "`new A` is rejected: an abstract class has no instances of its own")],
        &["abstract class A { } class B extends A { A f() { return new B(); } }"],
    ),
    rule(
        "02",
        "`abstract final`, `open final` and `sealed open` are rejected",
        &[
            ("abstract final class A { }", "`final` contradicts `open` and `abstract`"),
            ("open final class A { }", "`final` contradicts `open` and `abstract`"),
            ("sealed open class A permits B { } class B extends A { }", "`sealed open` is rejected"),
        ],
        &["final class A { } open class B { } abstract class C { }"],
    ),
    stated("02", "a program must not rely on its characters", "It limits what a program may assume, and no program can be shown to break it."),
    stated("02", "An override of `equals` must be an equivalence relation", "It binds the method a program writes, and no checker decides it."),
    rule(
        "02",
        "A program cannot list the members of a class that carry no annotation",
        &[
            ("class A { void f(Class c) { var m = c.getMethods(); } }", "`Class` has no method named `getMethods`"),
            ("class A { void f(Class c) { var m = c.getMethod(\"f\"); } }", "`Class` has no method named `getMethod`"),
            ("class A { void f(Class c) { var o = c.newInstance(); } }", "`Class` has no method named `newInstance`"),
        ],
        &["class A { String f(Class c) { return c.getName(); } }"],
    ),
    rule("02", "`final` on a method without a body is rejected", &[("interface I { final void f(); }", "`final` on an interface method without a body is rejected")], &["interface I { final Int f() { return 1; } }"]),
    rule("02", "The word `default` is not written", &[("interface I { default Int f() { return 1; } }", "`default` is not written on an interface method")], &["interface I { Int f() { return 1; } }"]),
    rule("02", "A contradictory modifier is rejected", &[("interface I { private Int X = 1; }", "a field of an interface is public")], &["interface I { public static final Int X = 1; Int Y = 2; }"]),
    rule(
        "02",
        "If two superinterfaces provide a body for the same signature",
        &[("interface P { Int f() { return 1; } } interface Q { Int f() { return 2; } } class C implements P, Q { }", "two interfaces provide a body for `f`; the class must override it")],
        &["interface P { Int f() { return 1; } } interface Q { Int f() { return 2; } } class C implements P, Q { @Override public Int f() { return 3; } }"],
    ),
    rule("02", "`final interface` is rejected", &[("final interface I { }", "`final interface` is rejected")], &["interface I { } open interface J { }"]),
    rule("02", "`value enum` is rejected", &[("value enum E { A }", "`value enum` is rejected")], &["enum E { A }"]),
    rule(
        "02",
        "a class that writes `extends Enum` is rejected",
        &[("class A extends Enum { }", "a class does not extend `Enum`; an enum is declared with `enum`")],
        &["enum E { A } class A { String f(Enum e) { return e.name(); } }"],
    ),
    // One program for each feature that section 10.1 lists and a program can write.
    rule(
        "02",
        "Each is rejected if a program writes it",
        &[
            ("class A { int x = 1; }", "`int` is a reserved word"),
            ("class A { void f(List xs) { } }", "there are no raw types"),
            ("class A { void f() throws Throwable { } }", "expected `{`, found `throws`"),
            ("class A { class B { } }", "a type is not declared inside another type"),
            ("class A { void f() { class B { } } }", "a type is not declared inside a method"),
            ("class A { Object f() { return new Object() { }; } }", "expected `;`, found `{`"),
            ("class A { synchronized void f() { } }", "there is no modifier `synchronized`"),
            ("class A { volatile Int x = 1; }", "there is no modifier `volatile`"),
            ("class A { void f(Object o) { o.wait(); } }", "`Object` has no method named `wait`"),
            ("class A { Int f(Float64 x) { return x; } }", "`Float64` is not assignable to `Int`"),
            ("class A { String f(Int n) { return n; } }", "`Int` is not assignable to `String`"),
            ("class A { Int f(Int a, Int b) { return a / b; } }", "`Int` has no method `div`"),
            ("class A { Int f(Int a) { return a >>> 1; } }", "`>>>` is not an operator"),
            ("class A { Int f(Int n) { return +n; } }", "there is no unary `+`"),
            ("class A { Int f(Int a, Int b) { return a ** b; } }", "expected an expression, found `*`"),
            ("class A { void f(Int n) { switch (n) { case 1: break; } } }", "expected `->`, found `:`"),
            ("class A { Int f() { return 017; } }", "a decimal numeral has no leading zero"),
            ("class A { Int f() { return 1L; } }", "a numeric literal has no suffix"),
            ("class A { Object f(Object o) { return o.clone(); } }", "`Object` has no method named `clone`"),
            ("interface I { default Int f() { return 1; } }", "`default` is not written on an interface method"),
            ("class A { void f(Class c) { var m = c.getMethods(); } }", "`Class` has no method named `getMethods`"),
        ],
        &[],
    ),
    // ---- chapter 3 ----
    rule(
        "03",
        "A use from outside the audience is rejected",
        &[
            ("class A { Int secret = 1; } class B { Int f(A a) { return a.secret; } }", "the field `secret` of `A` is outside its audience here"),
            ("class A { void hidden() { } } class B { void f(A a) { a.hidden(); } }", "the method `hidden` of `A` is outside its audience here"),
        ],
        &["class A { public Int shown = 1; public void open() { } } class B { Int f(A a) { a.open(); return a.shown; } }"],
    ),
    rule(
        "03",
        "`private` and `protected` are rejected on a type",
        &[("private class A { }", "a type is `public`, `package`, or visible to its file only"), ("protected class A { }", "a type is `public`, `package`, or visible to its file only")],
        &["package class A { }"],
    ),
    rule(
        "03",
        "may not name a type in its signature",
        &[
            ("==== A.cleat\npublic class A { public void f(Hidden h) { } } class Hidden { }", "the method `f` can be seen where `Hidden` cannot be named, and its signature names it"),
            ("==== A.cleat\npublic class A { public Hidden h = new Hidden(); } class Hidden { }", "the field `h` can be seen where `Hidden` cannot be named, and its signature names it"),
        ],
        &["==== A.cleat\npublic class A { Hidden h = new Hidden(); void f(Hidden other) { } } class Hidden { }"],
    ),
    rule(
        "03",
        "Each listed type must lie inside the audience that `only` narrows",
        &[
            ("==== a/A.cleat\npackage a; import b.B; public class A { package only(B) Int x = 1; }\n==== b/B.cleat\npackage b; public class B { }", "after `package`, `only` lists types of the same package"),
            ("open class A { protected only(B) Int x = 1; } class B { }", "after `protected`, `only` lists subclasses of the declaring type"),
        ],
        &["open class A { package only(B) Int w = 0; protected only(B) Int x = 1; public only(C) Int y = 2; } class B extends A { } class C { }"],
    ),
    rule("03", "A type listed twice is rejected", &[("class A { package only(B, B) Int x = 1; } class B { }", "a type is listed once in `only`")], &["class A { package only(B, C) Int x = 1; } class B { } class C { }"]),
    rule(
        "03",
        "`only` is not written on a type or after `private`",
        &[("class A { private only(B) Int x = 1; } class B { }", "`only` is not written after `private`"), ("package only(B) class A { } class B { }", "`only` is not written on a type")],
        &[],
    ),
    rule(
        "03",
        "An audience written on one of them is rejected unless it is `public`",
        &[("interface I { private void f(); }", "a method of an interface is public"), ("interface I { package Int X = 1; }", "a field of an interface is public")],
        &["interface I { public void f(); public Int X = 1; }"],
    ),
    // ---- chapter 4 ----
    rule(
        "04",
        "`var` is rejected in those positions",
        &[("class A { void f(var x) { } }", "`var` is a keyword and cannot be a name"), ("class A { var f() { return 1; } }", "`var` is a keyword and cannot be a name")],
        &["class A { Int f(Int x) { var y = x; return y; } }"],
    ),
    rule(
        "04",
        "`open final`, `abstract final` and `abstract static` are rejected",
        &[
            ("class A { open final void f() { } }", "`open final` is rejected"),
            ("abstract class A { abstract final void f(); }", "`abstract final` and `abstract static` are rejected"),
            ("abstract class A { abstract static void f(); }", "`abstract final` and `abstract static` are rejected"),
        ],
        &["abstract class A { abstract void f(); final void g() { } static void h() { } }"],
    ),
    rule(
        "04",
        "`this` and `super` are rejected in it",
        &[
            ("class A { Int n = 1; static Int f() { return this.n; } }", "`this` is not available in a static context"),
            ("class A { static String f() { return super.toString(); } }", "is not available in a static context"),
        ],
        &["class A { Int n = 1; Int f() { return this.n; } String g() { return super.toString(); } }"],
    ),
    rule(
        "04",
        "Declaring a method with the name and signature of a visible final method",
        &[("open class A { public Int f() { return 1; } } class B extends A { public Int f() { return 2; } }", "`f` is final in `A` and cannot be overridden")],
        &["open class A { Int f() { return 1; } } class B extends A { public Int f() { return 2; } }"],
    ),
    rule(
        "04",
        "The static type of the receiver must be a subtype of the method's receiver type",
        &[
            ("@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } } class C { Int f(Order o) { return o.unitPrice(); } }", "`unitPrice` is sent to a `Order`"),
            ("class A { Int f(@Nullable String s) { return s.length(); } }", "`length` is sent to a `@Nullable String`"),
        ],
        &["@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } public Int id() { return 2; } } class C { Int f(@Priced Order o) { return o.unitPrice() + o.id(); } }"],
    ),
    rule(
        "04",
        "Calling an instance method that way from a static method is rejected",
        &[("class A { void g() { } static void f() { g(); } }", "`g` is an instance method, and a static method has no receiver to call it on")],
        &["class A { static void g() { } static void f() { g(); } void h() { g(); h(); } }"],
    ),
    rule(
        "04",
        "A call of any other method must be used",
        &[("class A { static Int g() { return 1; } static void f() { g(); } }", DISCARDED)],
        &["class A { static Int g() { return 1; } static void h(Int n) { } static void f() { var a = g(); a = g(); h(g()); var b = g() + 1; } }"],
    ),
    rule("04", "A statement that would discard its result is rejected", &[("class A { static Int g() { return 1; } static void f() { g(); } }", DISCARDED)], &["class A { static Int g() { return 1; } static void f() { var used = g(); } }"]),
    rule(
        "04",
        "Otherwise the call is rejected, and the program writes a cast",
        &[(
            "interface P { } interface Q { } class B implements P, Q { } class A { static void g(P p) { } static void g(Q q) { } static void f(B b) { g(b); } }",
            "the call of `g` is ambiguous: 2 methods accept these arguments and none is more specific than the rest",
        )],
        &["interface P { } interface Q { } class B implements P, Q { } class A { static void g(P p) { } static void g(Q q) { } static void f(B b) { g((P) b); } }"],
    ),
    rule(
        "04",
        "`==`, casts and `instanceof` are rejected between disjoint types",
        &[
            ("class P { } class Q { } class C { Boolean f(P a, Q b) { return a == b; } }", "`==` between `P` and `Q` is rejected: the two types are disjoint"),
            ("class P { } class Q { } class C { Boolean f(P a) { return a instanceof Q; } }", "`P` and `Q` are disjoint, so this test could never be `true`"),
            ("class P { } class Q { } class C { Q f(P a) { return (Q) a; } }", "a cast from `P` to `Q` is rejected: the types are disjoint"),
            ("class Q { } interface I { } class C { Boolean f(Q a, I i) { return a == i; } }", "`==` between `Q` and `I` is rejected: the two types are disjoint"),
        ],
        &["open class P { } class Q extends P { } interface I { } class C { Boolean f(P a, Q b, I i) { return a == i && a == b && a instanceof Q; } P g(Q b) { return (P) b; } <T> Boolean h(T t, Q q, I i, Iterable<Int> j) { return t == q && t == i && i == j; } }"],
    ),
    rule(
        "04",
        "The expression is legal exactly when that call is legal",
        &[("class P { } class A { P f(P a, P b) { return a + b; } }", "`+` is the method `plus`, and `P` has no method `plus`")],
        &["class P { public P plus(P other) { return this; } public Boolean not() { return true; } } class A { P f(P a, P b) { return !a ? a + b : b; } }"],
    ),
    rule(
        "04",
        "When the static types of the operands are disjoint, the expression is rejected",
        &[("class C { Boolean f(String s, Int n) { return s == n; } }", "`==` between `String` and `Int` is rejected: the two types are disjoint")],
        &["class C { Boolean f(@Nullable String s, @Nullable Int n) { return s == n; } }"],
    ),
    rule("04", "Two classes are not rejected when one converts implicitly", &[], &["class C { Boolean f(Int32 i, Int n, Rational r) { return i == n && n == r; } }"]),
    rule(
        "04",
        "the other arm must be assignable to it",
        &[("class A { void f(Boolean b, String s, Int n) { var x = b ? s : n; } }", "the arms have the types `String` and `Int`, and neither is assignable to the other")],
        &["open class P { } class Q extends P { } class A { P f(Boolean b, P p, Q q) { var x = b ? p : q; var y = b ? q : p; Int n = b ? 1 : 2; return b ? x : y; } }"],
    ),
    rule("04", "A cast is rejected when `T` and the static type of `e` are disjoint", &[("class C { Int f(String s) { return (Int) s; } }", "a cast from `String` to `Int` is rejected: the types are disjoint")], &["class C { Int f(Object o) { return (Int) o; } }"]),
    rule("04", "It is rejected when `T` and the static type", &[("class C { Boolean f(String s) { return s instanceof Int; } }", "`String` and `Int` are disjoint, so this test could never be `true`")], &["class C { Boolean f(Object o) { return o instanceof Int; } }"]),
    rule(
        "04",
        "The value of `e` must be assignable to the location",
        &[("class A { void f(String s) { Int n = 1; n = s; } }", "`String` is not assignable to `Int`"), ("class A { Int[] a = new Int[2]; void f(String s) { a[0] = s; } }", "`String` is not assignable to `Int`")],
        &["class A { Int[] a = new Int[2]; Object o = 1; void f(String s, Int32 small) { Int n = 1; n = 2; n = small; o = s; a[0] = n; } }"],
    ),
    rule("04", "`var f = () -> 1;` is rejected", &[("class A { void f() { var g = () -> 1; } }", "a lambda has no type of its own")], &["class A { void f() { Function0<Int> g = () -> 1; } }"]),
    rule(
        "04",
        "A written type must be that type",
        &[("class A { Function1<Int, Int> f() { return (String x) -> 1; } }", "the parameter is written `String`, and the interface's method takes `Int`")],
        &["class A { Function1<Int, Int> f() { return (Int x) -> x; } Function1<Int, Int> g() { return (x) -> x; } }"],
    ),
    rule(
        "04",
        "The value of an expression body is the result, and must be assignable",
        &[("class A { Function0<Int> f() { return () -> \"s\"; } }", "`String` is not assignable to `Int`")],
        &["class A { Int n = 0; Function0<Int> f() { return () -> 1; } Function0<Unit> g() { return () -> n++; } }"],
    ),
    rule(
        "04",
        "Using any other local in a lambda is rejected",
        &[("class A { Function0<Int> f() { var n = 1; n = 2; return () -> n; } }", "a lambda may use `n` only if it is never assigned after it is initialized")],
        &["class A { Function0<Int> f(Int m) { var n = m + 1; return () -> n + m; } }"],
    ),
    rule(
        "04",
        "`var name = null;` is rejected",
        &[("class A { void f() { var x = null; } }", "`null` alone gives no type"), ("class A { void f() { var x; } }", "`var` needs an initializer")],
        &["class A { void f() { @Nullable String x = null; var y = x; } }"],
    ),
    rule(
        "04",
        "Every other expression is rejected as a statement",
        &[("class A { void f(Int n) { n + 1; } }", NOT_A_STATEMENT), ("class A { public A() { } void f() { new A(); } }", NOT_A_STATEMENT), ("class A { void f(Int n) { n; } }", NOT_A_STATEMENT)],
        &["class A { Int n = 0; void g() { } void f(StringBuilder b) { n = 1; n++; --n; n += 2; g(); b.append(n); } }"],
    ),
    rule(
        "04",
        "Writing one constant twice in a switch is rejected",
        &[
            ("class A { Int f(Int n) { return switch (n) { case 1 -> 1; case 1 -> 2; default -> 3; }; } }", "this constant is written twice in the switch"),
            ("class A { Int f(Int n) { return switch (n) { case 1, 2 -> 1; case 1 + 1 -> 2; default -> 3; }; } }", "this constant is written twice in the switch"),
        ],
        &["class A { Int f(Int n) { return switch (n) { case 1 -> 1; case 2 -> 2; default -> 3; }; } }"],
    ),
    rule(
        "04",
        "`T` must not be disjoint from the selector's type",
        &[("class A { Int f(String s) { switch (s) { case Int n -> { return 1; } default -> { return 2; } } } }", "`String` and `Int` are disjoint, so this arm could never match")],
        &["class A { Int f(Object s) { switch (s) { case Int n -> { return 1; } default -> { return 2; } } } }"],
    ),
    rule(
        "04",
        "An arm that cannot match because an earlier arm names",
        &[("class A { Int f(Object o) { return switch (o) { case Object x -> 1; case String s -> 2; }; } }", "this arm cannot match: an earlier arm names its type or a supertype of it")],
        &["class A { Int f(Object o) { return switch (o) { case String s -> 2; case Object x -> 1; }; } }"],
    ),
    rule(
        "04",
        "and the switch must be exhaustive",
        &[
            ("class A { Int f(Int n) { return switch (n) { case 1 -> 1; }; } }", "a switch expression is exhaustive: add the missing arms or a `default` arm"),
            ("enum E { A, B } class C { Int f(E e) { return switch (e) { case E.A -> 1; }; } }", "a switch expression is exhaustive: add the missing arms or a `default` arm"),
            ("sealed interface S permits P, Q { } class P implements S { } class Q implements S { } class C { Int f(S s) { return switch (s) { case P p -> 1; }; } }", "a switch expression is exhaustive: add the missing arms or a `default` arm"),
        ],
        &[
            "enum E { A, B } class C { Int f(E e) { return switch (e) { case E.A -> 1; case E.B -> 2; }; } Int g(Boolean b) { return switch (b) { case true -> 1; case false -> 2; }; } Int h(Int n) { return switch (n) { case 1 -> 1; default -> throw new IllegalStateException(); }; } void k(Int n) { switch (n) { case 1 -> { } } } }",
            "sealed interface S permits P, Q { } class P implements S { } class Q implements S { } class C { Int f(S s) { return switch (s) { case P p -> 1; case Q q -> 2; }; } }",
        ],
    ),
    rule(
        "04",
        "A label that does not enclose the `break` or `continue` is rejected",
        &[
            ("class A { void f() { a: while (true) { break a; } while (true) { break a; } } }", "no statement labeled `a` encloses this `break`"),
            ("class A { void f() { break; } }", "`break` is not inside a loop or a switch statement"),
            ("class A { void f() { a: { continue a; } } }", "`continue` with a label continues a loop, and this label is not on a loop"),
        ],
        &["class A { void f() { a: { break a; } b: while (true) { continue b; } } }"],
    ),
    rule(
        "04",
        "returns the value of `e`, which must be assignable to the method's result type",
        &[("class A { Int f(String s) { return s; } }", "`String` is not assignable to `Int`")],
        &["class A { Int f(Int32 small) { return small; } Object g(String s) { return s; } @Nullable String h() { return null; } }"],
    ),
    stated("04", "For an empty array the pointer must not be used", "It binds the C function, which is outside the language."),
    rule(
        "04",
        "Every other type is rejected on a foreign method",
        &[
            ("class A { foreign static void f(String s); }", "a parameter of a `foreign` method is a machine type or an array of one, and `String` is neither"),
            ("class A { foreign static Rational f(); }", "the result of a `foreign` method is a machine type or `void`, and `Rational` is neither"),
            ("class A { foreign static void f(@Nullable Int n); }", FOREIGN_PARAM),
            ("class A<T> { foreign static <T> void f(T x); }", FOREIGN_PARAM),
        ],
        &["class A { foreign static Int32 f(Int32 a, Float64[] b, Pointer p); }"],
    ),
    rule("04", "A lambda cannot be passed, so a C function cannot call back", &[("class A { foreign static void f(Function0<Int> g); }", FOREIGN_PARAM)], &[]),
    // ---- chapter 5 ----
    rule("05", "`new Null()` is rejected", &[("class A { @Nullable Object f() { return new Null(); } }", "`new Null` is rejected: `null` is the one instance")], &["class A { @Nullable Object f() { return null; } }"]),
    rule("05", "A variable of type `Object` therefore cannot hold `null`", &[("class A { void f() { Object o = null; } }", "`null` is not a value of `Object`")], &["class A { void f() { @Nullable Object o = null; Object p = 1; o = p; } }"]),
    rule("05", "`String s = null` is rejected", &[("class A { void f() { String s = null; } }", "`null` is not a value of `String`")], &["class A { void f() { @Nullable String s = null; String t = \"x\"; s = t; } }"]),
    rule("05", "`@Nullable String s = null` is legal", &[], &["class A { void f() { @Nullable String s = null; } }"]),
    rule(
        "05",
        "Every other send on a `@Nullable` receiver is rejected",
        &[("class A { Int f(@Nullable String s) { return s.length(); } }", "`length` is sent to a `@Nullable String`, which may be `null`")],
        &["class A { Int f(@Nullable String s) { if (s != null) { return s.length(); } return s.toString().length() + s.hashCode(); } }"],
    ),
    rule(
        "05",
        "An `open` or `abstract` method other than those three must not declare one",
        &[
            ("open class A { public open Int f(@Nullable A this) { return 1; } }", "an `open` or `abstract` method does not declare a `@Nullable` receiver"),
            ("abstract class A { public abstract Int f(@Nullable A this); }", "an `open` or `abstract` method does not declare a `@Nullable` receiver"),
        ],
        &["class A { public Int f(@Nullable A this) { return this == null ? 0 : 1; } }"],
    ),
    rule(
        "05",
        "A null test of an expression whose static type cannot contain",
        &[("class A { Boolean f(String s) { return s == null; } }", "a null test of a `String` is rejected, because it cannot be `null`"), ("class A { Boolean f(Int n) { return null != n; } }", "is rejected, because it cannot be `null`")],
        &["class A { Boolean f(@Nullable String s) { return s == null; } }"],
    ),
    rule(
        "05",
        "A test whose `null` branch cannot complete normally narrows the code after it",
        &[("class A { Int f(@Nullable String s) { if (s == null) { } return s.length(); } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s) { if (s == null) { return 0; } return s.length(); } Int g(@Nullable String s) { if (s == null) { throw new IllegalStateException(); } return s.length(); } }"],
    ),
    rule(
        "05",
        "the right operand runs only when `x` is not `null`, so the send is legal",
        &[("class A { Boolean f(@Nullable String x) { return x != null || x.isEmpty(); } }", "`isEmpty` is sent to")],
        &["class A { Boolean f(@Nullable String x) { return x != null && x.isEmpty(); } Boolean g(@Nullable String x) { return x == null || x.isEmpty(); } }"],
    ),
    rule(
        "05",
        "The static type of `a` must be able to contain `null`",
        &[("class A { String f(String s) { return s ?? \"x\"; } }", "the left operand of `??` is a `String`, which cannot be `null`")],
        &["class A { String f(@Nullable String s) { return s ?? \"x\"; } }"],
    ),
    rule("05", "Writing `@Nullable` on the type after `instanceof` is rejected", &[("class A { Boolean f(Object o) { return o instanceof @Nullable String; } }", "`@Nullable` is rejected after `instanceof`")], &["class A { Boolean f(@Nullable Object o) { return o instanceof String; } }"]),
    rule("05", "`return` with an operand is rejected in it", &[("class A { void f() { return 1; } }", "`return` with an operand is rejected in a method whose result is `Unit`")], &["class A { void f() { return; } Unit g() { } }"]),
    rule(
        "05",
        "A call with any other result must be used, unless its method is marked `@Discardable`",
        &[("class A { static Int g() { return 1; } static void f() { g(); } }", DISCARDED)],
        &["class A { @Discardable static Int g() { return 1; } static void h() { } static void f() { g(); h(); } }"],
    ),
    // ---- chapter 6 ----
    rule(
        "06",
        "`i + f` is rejected",
        &[("class A { Float64 g(Int32 i, Float64 f) { return i + f; } }", "`Float64` is not assignable to `Int32`")],
        &["class A { Float64 g(Int32 i, Float64 f) { return Float64.nearest(i) + f; } Int h(Int32 i, Int n) { return i + n; } }"],
    ),
    rule(
        "06",
        "A literal of class `C` must denote a value of `C`",
        &[("class A { Int8 x = 128; }", "the literal is outside the range of `Int8`"), ("class A { UInt8 x = 0x100; }", "the literal is outside the range of `UInt8`"), ("class A { UInt16 x = -1; }", "the literal is outside the range of `UInt16`"), ("class A { Int8 x = -129; }", "the literal is outside the range of `Int8`")],
        &["class A { Int8 x = 127; Int8 y = -128; UInt8 z = 0xFF; UInt64 w = 18446744073709551615; }"],
    ),
    rule(
        "06",
        "A decimal literal is rejected there, including one such as `1.0`",
        &[("class A { Int n = 1.0; }", "a decimal literal is not a value of `Int`"), ("class A { UInt8 n = 0.5; }", "a decimal literal is not a value of `UInt8`")],
        &["class A { Float64 x = 1; Rational r = 1.0; Float32 y = 0.5; }"],
    ),
    rule(
        "06",
        "A literal whose nearest value is an infinity is rejected",
        &[("class A { Float64 x = 1e400; }", "the literal is too large for `Float64`"), ("class A { Float32 x = 1e39; }", "the literal is too large for `Float32`")],
        &["class A { Float64 x = 1e300; Float32 y = 1e38; Rational r = 1e400; }"],
    ),
    rule(
        "06",
        "the declaration `Float64 y = x;` is rejected",
        &[("class A { void f() { var x = 0.1; Float64 y = x; } }", "`Rational` is not assignable to `Float64`")],
        &["class A { void f() { var x = 0.1; Rational y = x; Float64 z = 0.1; } }"],
    ),
    rule(
        "06",
        "If there is none, the call is rejected.",
        &[("class A { static void g(Int8 a) { } static void g(Int16 a) { } static void f() { g(1); } }", "the call of `g` is ambiguous")],
        &["class A { static void g(Int8 a) { } static void g(Int16 a) { } static void g(Int a) { } static void f() { g(1); } }"],
    ),
    stated("06", "An operation whose result cannot be stored raises", "It says what a running program does when storage runs out."),
    rule(
        "06",
        "`a / b` on two integers is rejected",
        &[("class A { Int f(Int a, Int b) { return a / b; } }", "`/` is the method `div`, and `Int` has no method `div`"), ("class A { UInt8 f(UInt8 a, UInt8 b) { return a / b; } }", "`/` is the method `div`, and `UInt8` has no method `div`")],
        &["class A { Int f(Int a, Int b) { return a.floorDiv(b); } Rational g(Rational a, Rational b) { return a / b; } Float64 h(Float64 a) { return a / 2; } }"],
    ),
    rule(
        "06",
        "Each of these conversions is the method `T.from`, which cannot fail for these pairs",
        &[("class A { void f(Int n) { Int32 a = n; } }", "`Int` is not assignable to `Int32`"), ("class A { void f(Int n) { Float64 x = n; } }", "`Int` is not assignable to `Float64`"), ("class A { void f(Int8 n) { UInt8 x = n; } }", "`Int8` is not assignable to `UInt8`"), ("class A { void f(Float32 x) { Rational r = x; } }", "`Float32` is not assignable to `Rational`")],
        &["class A { void f(Int8 a, UInt8 b, Int32 c, Float32 x) { Int16 p = a; Int q = b; Int r = c; Float64 y = x; Rational z = c; UInt64 w = b; } }"],
    ),
    rule(
        "06",
        "`(Int) x` is rejected when the static type of `x` is a different numeric class",
        &[("class A { Int f(Int32 x) { return (Int) x; } }", "a cast from `Int32` to `Int` is rejected"), ("class A { Int f(Float64 x) { return (Int) x; } }", "a cast from `Float64` to `Int` is rejected")],
        &["class A { Int f(Int32 x) { return Int.from(x); } }"],
    ),
    rule(
        "06",
        "Otherwise `a == b` is rejected",
        &[("class A { Boolean f(Int32 i, Float64 x) { return i == x; } }", "`==` between `Int32` and `Float64` is rejected"), ("class A { Boolean f(Int n, UInt64 u) { return n == u; } }", "`==` between `Int` and `UInt64` is rejected")],
        &["class A { Boolean f(Int32 i, Int n, UInt8 u) { return i == n && u == n; } }"],
    ),
    rule("06", "`'a' + 1` is rejected", &[("class A { Char f() { return 'a' + 1; } }", "`+` is the method `plus`, and `Char` has no method `plus`")], &["class A { Char f() { return Char.from('a'.code() + 1); } }"]),
    rule(
        "06",
        "is rejected when `n` is an `Int`, because `Int.plus` has no `String` parameter",
        &[("class A { String f(Int n) { return n + \" items\"; } }", "`String` is not assignable to `Int`")],
        &["class A { String f(Int n) { return \"count: \" + n; } }"],
    ),
    rule(
        "06",
        "Every other `new T[n]` is rejected",
        &[
            ("class A { String[] f() { return new String[3]; } }", NO_DEFAULT),
            ("class B { public B() { } } class A { B[] f() { return new B[2]; } }", NO_DEFAULT),
            ("value class V { public String s; } class A { V[] f() { return new V[2]; } }", NO_DEFAULT),
        ],
        &["value class V { public Int a; public Rational r; public @Nullable String s; } class A { void f() { var a = new @Nullable String[3]; var b = new Int[3]; var c = new Boolean[1]; var d = new Char[1]; var e = new V[2]; var g = new Rational[2]; } }"],
    ),
    rule("06", "When `T` is a type parameter, `new T[n]` is rejected", &[("class A<T> { T[] f(Int n) { return new T[n]; } }", NO_DEFAULT)], &["class A<T> { @Nullable T[] f(Int n) { return new @Nullable T[n]; } }"]),
    rule("06", "It is legal for every `T`", &[], &["class B { public B() { } } class A<T> { T[] f(T a, T b) { return new T[] { a, b }; } String[] g() { return new String[] { \"a\" }; } B[] h() { return new B[] { new B() }; } }"]),
    rule("06", "and is legal for every `T`", &[], &["class B { public B() { } } class A<T> { T[] f(Int n, T x) { return Array.build(n, (i) -> x); } String[] g() { return Array.build(2, (i) -> \"s\"); } B[] h() { return Array.build(2, (i) -> new B()); } }"]),
    // ---- chapter 7 ----
    rule("07", "An enum and an annotation may not", &[("enum E<T> { A }", "an enum declares no type parameters"), ("annotation N<T>;", "an annotation declares no type parameters")], &["class B<T> { } interface I<T> { } class A { <T> void f(T x) { } }"]),
    rule(
        "07",
        "A use with no arguments is rejected",
        &[
            ("class A { void f() { List xs = new List<Int>(); } }", "`List` is generic and is written with its type arguments; there are no raw types"),
            ("class A { void f(Map m) { } }", "`Map` is generic and is written with its type arguments"),
            ("class A { void f() { List<Int> xs = new List<>(); } }", "expected a name, found `>`"),
        ],
        &["class A { void f(Int n) { var a = Array.build(n, (i) -> i); } }"],
    ),
    rule(
        "07",
        "It must satisfy the bound of the parameter it is given for",
        &[("class Box<T extends Throwable> { } class A { void f(Box<String> b) { } }", "`String` is not within the bound `Throwable`")],
        &["class Box<T extends Throwable> { } class A<E extends IllegalStateException> { void f(Box<IllegalStateException> b, Box<E> c, Box<? extends ArithmeticException> d) { } }"],
    ),
    rule(
        "07",
        "A type parameter must not have the name of another type parameter",
        &[("class A<T> { <T> void f(T x) { } }", "the type parameter `T` has the name of another one in scope"), ("class A<T, T> { }", "the type parameter `T` is declared twice")],
        &["class A<T> { <U> void f(T x, U y) { } static <T> void g(T x) { } }"],
    ),
    rule(
        "07",
        "A type argument must be a subtype of each bound, after the argument is substituted",
        &[("class Top<T extends Ordered<T>> { } class P { } class A { void f(Top<P> t) { } }", "`P` is not within the bound `Ordered<P>`")],
        &["class Top<T extends Ordered<T>> { } class A { void f(Top<Int> a, Top<String> b) { } }"],
    ),
    rule(
        "07",
        "A declaration that breaks these rules is rejected",
        &[
            ("interface S<out T> { void put(T x); }", "`T` is declared `out` and is written where a value is consumed"),
            ("interface S<in T> { T take(); }", "`T` is declared `in` and is written where a value is produced"),
            ("class S<out T> { T item; public S(T item) { this.item = item; } }", "`T` is declared `out`"),
            ("interface S<out T> { List<T> all(); }", "`T` is declared `out` and is written where a value is both produced and consumed"),
            ("interface K<in T> { void each(Comparator<T> c); }", "`T` is declared `in` and is written where a value is produced"),
        ],
        &["interface S<out T> { T take(); Iterable<T> all(); } interface K<in T> { void put(T x); void all(Iterable<T> xs); Comparator<T> order(); } class B<out T> { final T item; public B(T item) { this.item = item; } }"],
    ),
    rule("07", "`new List<?>()` is rejected", &[("class A { void f() { var x = new List<?>(); } }", "a wildcard is not written among the arguments of the class in `new`")], &["class A { void f() { List<?> y = new List<Int>(); } }"]),
    rule("07", "A wildcard inside one of those arguments is legal", &[], &["class A { void f() { var x = new List<List<?>>(); x.add(new List<Int>()); } }"]),
    rule("07", "and `? super` is rejected", &[("class A { void f(Iterable<? super Int> x) { } }", "`? super` is rejected for a parameter declared `out`")], &["class A { void f(Iterable<? extends Object> x, Iterable<?> y) { } }"]),
    rule("07", "and `? extends` is rejected", &[("class A { void f(Comparator<? extends Int> x) { } }", "`? extends` is rejected for a parameter declared `in`")], &["class A { void f(Comparator<? super Int> x, Comparator<?> y) { } }"]),
    rule(
        "07",
        "An argument must be assignable to its parameter as read",
        &[
            ("class A { void f(List<? extends Object> xs, String s) { xs.add(s); } }", "`String` is not assignable to `an unknown type`"),
            ("class A { void f(List<?> xs, String s) { xs.add(s); } }", "`String` is not assignable to `an unknown type`"),
            ("class A { void f(List<? super String> xs, Object o) { xs.add(o); } }", "`Object` is not assignable to"),
        ],
        &["class A { void f(List<? super String> xs, String s) { xs.add(s); } }"],
    ),
    rule(
        "07",
        "`items[0] = items[1]` is rejected",
        &[("class A { void f(List<?> items) { items[0] = items[1]; } }", "is not assignable to `an unknown type`")],
        &["class A { static <T> void swap(List<T> items) { T held = items[0]; items[0] = items[1]; items[1] = held; } void f(List<?> items) { swap(items); } }"],
    ),
    rule(
        "07",
        "Its parameter types must be known by then",
        &[("class A { static <T> void g(Function1<T, Int> f) { } static void h() { g((x) -> 1); } }", "the type arguments of `g` cannot be inferred from this call")],
        &["class A { static <T> void g(Function1<T, Int> f) { } static <T> void k(T seed, Function1<T, Int> f) { } static void h() { g((Int x) -> 1); k(\"s\", (x) -> x.length()); } }"],
    ),
    rule(
        "07",
        "All of them must fix it to the same type",
        &[("class A { static <T> void g(List<T> a, List<T> b) { } static void h(List<Int> a, List<String> b) { g(a, b); } }", "the type arguments of `g` cannot be inferred from this call")],
        &["class A { static <T> void g(List<T> a, List<T> b) { } static void h(List<Int> a, List<Int> b) { g(a, b); } }"],
    ),
    rule(
        "07",
        "Once `P` is determined, each argument must be assignable to its parameter",
        &[("class A { static <T> void g(T a, List<T> b) { } static void h(String s, List<Int> b) { g(s, b); } }", "`String` is not assignable to `Int`")],
        &["class A { static <T> void g(T a, List<T> b) { } static void h(Int n, Int32 small, List<Int> b) { g(n, b); g(small, b); } }"],
    ),
    rule(
        "07",
        "rejects two arguments of type `List<?>`",
        &[("class A { static <T> void copy(List<T> from, List<T> to) { } void f(List<?> a, List<?> b) { copy(a, b); } }", "the type arguments of `copy` cannot be inferred from this call")],
        &["class A { static <T> void copy(List<T> from, List<T> to) { } static <T> Int count(List<T> xs) { return xs.size(); } void f(List<Int> a, List<Int> b, List<?> c) { copy(a, b); var n = count(c); } }"],
    ),
    rule("07", "`implements Ordered<A>, Ordered<B>` is rejected", &[("class C implements Iterable<Int>, Iterable<String> { }", "an interface is named once in a declaration")], &[]),
    // ---- chapter 8 ----
    stated("08", "An annotation cannot add a member to a class or change the body of a method", "It says what annotations do not do; no program can ask for it."),
    rule(
        "08",
        "`@Positive` is legal wherever the name `Positive` is visible",
        &[("==== a/Positive.cleat\npackage a; @Refines public annotation Positive;\n==== b/B.cleat\npackage b; class B { @Positive Int f(@Positive Int n) { return n; } }", "there is no type named `Positive` here")],
        &[
            "==== a/Positive.cleat\npackage a; @Refines public annotation Positive;\n==== b/B.cleat\npackage b; import a.Positive; class B { @Positive Int f(@Positive Int n) { return n; } }",
            "@Refines annotation Positive; class B { @Positive Int f(@Positive Int n) { List<@Positive Int> xs = new List<@Positive Int>(); return n; } }",
        ],
    ),
    rule(
        "08",
        "a class or an interface that names it as a supertype is rejected",
        &[("class C implements Annotation { }", "only an annotation implements `Annotation`"), ("interface I extends Annotation { }", "only an annotation implements `Annotation`")],
        &["annotation A; class C { @Nullable Annotation f(@Nullable A a) { return a; } }"],
    ),
    rule(
        "08",
        "an element with no default must be supplied",
        &[("annotation Team(String name); @Team class C { }", "the element `name` of `@Team` has no default and must be supplied"), ("annotation Team(String name, Int size); @Team(\"a\") class C { }", "the element `size` of `@Team` has no default and must be supplied")],
        &["annotation Team(String name, Int size = 1); @Team(\"a\") class C { } @Team(name = \"b\") class D { } @Team(\"c\", 3) class E { }"],
    ),
    rule(
        "08",
        "A use at any other site is rejected",
        &[
            ("@Target(Site.METHOD) annotation M; @M class C { }", SITE),
            ("@Target(Site.TYPE) annotation M; class C { @M Int x = 1; }", SITE),
            ("@Target(Site.FIELD) annotation M; class C { void f(@M Int x) { } }", SITE),
        ],
        &["@Target({Site.METHOD, Site.TYPE}) annotation M; annotation Any; @M @Any class C { @M @Any void f(@Any Int x) { } @Any Int y = 1; @Any public C() { } }"],
    ),
    rule(
        "08",
        "and a plain `T` may not be used where `@Q T` is expected",
        &[("@Refines annotation Q; class A { @Q Int f(Int n) { return n; } }", "`Int` is not assignable to `@Q Int`")],
        &["@Refines annotation Q; @Refines(Q.class) annotation R; class A { Int f(@Q Int n) { return n; } @Q Int g(@R Int n) { return n; } }"],
    ),
    rule("08", "`@Target` on its declaration is rejected", &[("@Refines @Target(Site.TYPE) annotation Q;", "a qualifier is written on types, and `@Target` on its declaration is rejected")], &["@Refines annotation Q;"]),
    rule(
        "08",
        "A cycle is rejected",
        &[("@Refines(B.class) annotation A; @Refines(A.class) annotation B;", "this refinement is placed below itself"), ("@Refines(Q.class) annotation Q;", "this refinement is placed below itself")],
        &["@Refines annotation A; @Refines(A.class) annotation B; @Refines({A.class, B.class}) annotation C;"],
    ),
    rule(
        "08",
        "a value that has not been shown to belong in `T` may not be used as one",
        &[("@Widens annotation Raw; class A { String f(@Raw String s) { return s; } }", "`@Raw String` is not assignable to `String`")],
        &["@Widens annotation Raw; class A { @Raw String f(String s) { return s; } }"],
    ),
    rule(
        "08",
        "A send is legal only when the static type of the receiver is a subtype of the method's receiver type",
        &[
            ("@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } } class C { Int f(Order o) { return o.unitPrice(); } }", "`unitPrice` is sent to a `Order`"),
            ("@Widens annotation Raw; class Order { public Int id() { return 1; } } class C { Int f(@Raw Order o) { return o.id(); } }", "`id` is sent to a `@Raw Order`"),
        ],
        &["@Refines annotation Priced; @Widens annotation Raw; class Order { public Int unitPrice(@Priced Order this) { return 1; } public Int id(@Raw Order this) { return 2; } } class C { Int f(@Priced Order o, @Raw Order r) { return o.unitPrice() + o.id() + r.id(); } }"],
    ),
    rule(
        "08",
        "whose type has a qualifier outside every type argument is therefore rejected",
        &[("@Refines annotation P; class C { Boolean f(Object o) { return o instanceof @P Int; } }", "a value does not carry its qualifiers at run time, so `instanceof` does not test one")],
        &["@Refines annotation P; class C { Boolean f(Object o) { return o instanceof List<@P Int>; } }"],
    ),
    rule(
        "08",
        "`(@Positive Int) n` is rejected, and `(List<@Positive Int>) o` is legal",
        &[("@Refines annotation Positive; class C { @Positive Int f(Int n) { return (@Positive Int) n; } }", "a value does not carry its qualifiers at run time, so a cast does not name one outside a type argument")],
        &["@Refines annotation Positive; class C { List<@Positive Int> f(Object o) { return (List<@Positive Int>) o; } @Nullable String g(Object o) { return (@Nullable String) o; } }"],
    ),
    stated("08", "`(@Nullable String) e` accepts `null`, and `(String) e` rejects it", "Checked by tests/behavior/Nulls.cleat, which runs both casts on `null`."),
    stated("08", "A qualifier must describe a fact that stays true of a value", "It binds the author of a qualifier, and no checker decides it."),
    stated("08", "cannot change, so a qualifier on those types meets the obligation", "It explains the obligation of the sentence before it."),
    rule(
        "08",
        "A type argument for `T` must be a type whose declaration carries `Frozen`",
        &[("@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class Pair<U> { void g(Snapshot<U> s) { } }", "`U` does not carry the tag `@Frozen` that the type parameter requires")],
        &["@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class Pair<@Frozen U> { void g(Snapshot<U> s) { } }"],
    ),
    rule(
        "08",
        "`Snapshot<StringBuilder>` is rejected",
        &[("@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class C { void f(Snapshot<StringBuilder> s) { } }", "`StringBuilder` does not carry the tag `@Frozen` that the type parameter requires")],
        &["@Target(Site.TYPE) annotation Frozen; @Frozen value class Bill { } class Snapshot<@Frozen T> { } class C { void f(Snapshot<Bill> s) { } }"],
    ),
    rule("08", "`Snapshot<Bill>` is legal", &[], &["@Target(Site.TYPE) annotation Frozen; @Frozen value class Bill { } class Snapshot<@Frozen T> { } class C { void f(Snapshot<Bill> s) { } }"]),
    rule(
        "08",
        "A declaration that would inherit two uses with different arguments",
        &[("@Inherited annotation Team(String name); @Team(\"a\") interface P { } @Team(\"b\") interface Q { } class C implements P, Q { }", "`C` would inherit two uses of `@Team` with different arguments")],
        &["@Inherited annotation Team(String name); @Team(\"a\") interface P { } @Team(\"b\") interface Q { } @Team(\"c\") class C implements P, Q { } @Team(\"a\") interface R { } class D implements P, R { }"],
    ),
    rule(
        "08",
        "A method that does either must carry it",
        &[
            ("open class A { public open Int f() { return 1; } } class B extends A { public Int f() { return 2; } }", "`f` overrides or implements a method and must carry `@Override`"),
            ("interface I { Int f(); } class B implements I { public Int f() { return 2; } }", "`f` overrides or implements a method and must carry `@Override`"),
        ],
        &["open class A { public open Int f() { return 1; } } interface I { Int g(); } class B extends A implements I { @Override public Int f() { return 2; } @Override public Int g() { return 3; } }"],
    ),
    rule("08", "A method that carries it must do one of them", &[("class A { @Override public Int f() { return 1; } }", "`f` carries `@Override` and overrides nothing")], &["class A { @Override public String toString() { return \"a\"; } }"]),
    rule(
        "08",
        "Both failures are rejected",
        &[
            ("open class A { public open Int f() { return 1; } } class B extends A { public Int f() { return 2; } }", "`f` overrides or implements a method and must carry `@Override`"),
            ("class A { @Override public Int f() { return 1; } }", "`f` carries `@Override` and overrides nothing"),
        ],
        &["open class A { public open Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }"],
    ),
    rule(
        "08",
        "Without it, a result other than `Unit` must be used",
        &[("class A { static Int g() { return 1; } static void f() { g(); } }", DISCARDED)],
        &["class A { @Discardable static Int g() { return 1; } static void f() { g(); } }"],
    ),
    rule("08", "`@Intrinsic` is rejected outside the package `cleat`", &[("class A { @Intrinsic Int f(); }", "`@Intrinsic` is rejected outside the package `cleat`")], &[]),
    stated("08", "An implementation must report each use of a deprecated declaration", "Checked by `a_deprecated_use_is_reported_from_another_unit`, which reads the warnings."),
    stated("08", "The report does not reject the program", "Checked by `a_deprecated_use_is_reported_from_another_unit`, which reads the warnings."),
    // ---- chapter 9 ----
    stated("09", "A method cannot accept an argument it was given", "A row of the table of exceptions, which tests/spec_tables.rs holds to the prelude."),
    stated("09", "| Storage cannot be found for an allocation", "A row of the table of exceptions, which tests/spec_tables.rs holds to the prelude."),
    stated("09", "A program cannot read it", "It is about what an implementation may record of an exception."),
    rule(
        "09",
        "A clause that can never run",
        &[
            ("class A { void f() { try { } catch (Throwable e) { } catch (ArithmeticException e) { } } }", "this `catch` clause can never run: an earlier clause names its type or a supertype of it"),
            ("class A { void f() { try { } catch (ArithmeticException e) { } catch (ArithmeticException x) { } } }", "this `catch` clause can never run"),
        ],
        &["class A { void f() { try { } catch (ArithmeticException e) { } catch (Throwable e) { } } }"],
    ),
    rule(
        "09",
        "`return` is rejected in a `finally` block, and so are a `break` and a `continue`",
        &[
            ("class A { Int f() { try { return 1; } finally { return 2; } } }", "`return` is rejected in a `finally` block"),
            ("class A { void f() { while (true) { try { } finally { break; } } } }", "a `break` does not leave a `finally` block"),
            ("class A { void f() { while (true) { try { } finally { continue; } } } }", "a `continue` does not leave a `finally` block"),
        ],
        &["class A { Int f() { try { return 1; } finally { while (true) { break; } } } }"],
    ),
    rule(
        "09",
        "must have a method `close()` with no parameters and the result `Unit`, and must not be `@Nullable`",
        &[
            ("class A { void f(String s) { using (String t = s) { } } }", "`String` has no method named `close`"),
            ("class R { public R() { } public void close() { } } class A { void f(@Nullable R r) { using (@Nullable R x = r) { } } }", "a `using` binding is not `@Nullable`"),
            ("class R { public R() { } public Int close() { return 1; } } class A { void f() { using (R x = new R()) { } } }", "`using` calls `close()`, whose result is `Unit`"),
        ],
        &["class R { public R() { } public void close() { } } class A { void f() { using (R x = new R(), var y = new R()) { } } }"],
    ),
    rule(
        "09",
        "A constructor's audience is written like a member's",
        &[("class R { R() { } } class A { R f() { return new R(); } }", "the constructor of `R` is outside its audience here")],
        &["class R { public R() { } } class S { } class A { R f() { var s = new S(); return new R(); } }"],
    ),
    rule(
        "09",
        "every field declared in the class must be assigned",
        &[
            ("class A { Int n; public A() { } }", "the field `n` is not assigned on every path to the `super(...)` call"),
            ("class A { Int n; public A(Boolean b) { if (b) { this.n = 1; } super(); } }", "the field `n` is not assigned on every path to the `super(...)` call"),
        ],
        &["class A { Int n; Int m = 2; public A(Boolean b) { if (b) { this.n = 1; } else { n = 3; } super(); } }"],
    ),
    rule(
        "09",
        "Its audience is `private` when it is not written.",
        &[("value class V { public Int a; V { } } class A { V f() { return new V(1); } }", "the constructor of `V` is outside its audience here")],
        &["value class V { public Int a; public V { } } value class W { public Int a; } class A { V f() { var w = new W(1); return new V(1); } }"],
    ),
    stated("09", "When storage cannot be found for an allocation, even after reclaiming", "It says what a running program does when storage runs out."),
    stated("09", "A program must not rely on how much storage", "It limits what a program may assume, and no program can be shown to break it."),
    stated("09", "A class that declares both, or neither, is rejected as an entry class", "Checked by `an_entry_class_declares_exactly_one_main`, which needs a build."),
    stated("09", "| A call cannot be given stack space", "Checked by `a_full_stack_ends_the_program` in tests/host.rs, which runs a program."),
    stated("09", "Running out of stack is not an exception, and a program cannot catch it", "Checked by `a_full_stack_ends_the_program` in tests/host.rs, which runs a program."),
    // ---- chapter 10 ----
    stated("10", "An implementation must not give an omitted feature a meaning", "It binds an implementation; the cases of section 2.11 show the omitted features rejected."),
    stated("10", "a program must not assume any particular form for them", "It is about features that are not in the language."),
    // ---- chapter 11 ----
    rule(
        "11",
        "A program that the grammar does not derive is rejected",
        &[
            ("class A { void f(Boolean x) { if x { } } }", "expected `(`, found `x`"),
            ("class A { Int f() { return (1 + ; } }", "expected an expression, found `;`"),
            ("class { }", "expected a name, found `{`"),
            ("class A { void f() { Int[3] a; } }", "expected `;`, found `a`"),
        ],
        &[],
    ),
    stated("11", "The grammar derives more than is legal", "It introduces the rules after it, which the other chapters state."),
    // ---- chapter 12 ----
    rule(
        "12",
        "A constant expression whose evaluation would raise an exception is rejected.",
        &[("class A { Int8 x = 100 + 100; }", CONSTANT_RAISES), ("class A { Char c = Char.from(55296); }", CONSTANT_RAISES), ("class A { Int8 x = Int8.from(200); }", CONSTANT_RAISES)],
        &["class A { Int8 x = 100 + 27; Char c = Char.from(65); Int8 y = Int8.from(27); }"],
    ),
    rule(
        "12",
        "`Int32 big = 2147483647 + 1;` is rejected, and so is `1.0 / 0`",
        &[("class A { void f() { Int32 big = 2147483647 + 1; } }", CONSTANT_RAISES), ("class A { void f() { var x = 1.0 / 0; } }", CONSTANT_RAISES)],
        &["class A { void f() { Int big = 2147483647 + 1; var x = 1.0 / 4; } }"],
    ),
    rule("12", "Every statement must be reachable", &[("class A { Int f() { return 1; return 2; } }", UNREACHABLE), ("class A { void f() { for (;;) { } f(); } }", UNREACHABLE)], &["class A { Int f(Boolean b) { if (b) { return 1; } return 2; } }"]),
    stated("12", "A statement either can complete normally or cannot", "It defines the two cases the rules after it use."),
    rule(
        "12",
        "An unreachable statement is rejected.",
        &[
            ("class A { Int f() { return 1; return 2; } }", UNREACHABLE),
            ("class A { void f() { while (true) { } f(); } }", UNREACHABLE),
            ("class A { void f() { throw new IllegalStateException(); f(); } }", UNREACHABLE),
            ("class A { void f() { while (true) { break; f(); } } }", UNREACHABLE),
        ],
        &["class A { static final Boolean DEBUG = false; void f() { if (DEBUG) { f(); } while (true) { if (DEBUG) { break; } } f(); } }"],
    ),
    rule("12", "`if (DEBUG) { ... }` is legal when `DEBUG` is the constant `false`", &[], &["class A { static final Boolean DEBUG = false; void g() { } void f() { if (DEBUG) { g(); } if (false) { g(); } g(); } }"]),
    rule(
        "12",
        "A loop whose condition is the constant `false` is rejected.",
        &[
            ("class A { void f() { while (false) { } } }", FALSE_LOOP),
            ("class A { static final Boolean OFF = false; void f() { while (OFF) { } } }", FALSE_LOOP),
            ("class A { void f() { for (var i = 0; false; i++) { } } }", FALSE_LOOP),
        ],
        &["class A { void f(Boolean off) { while (off) { } } }"],
    ),
    rule(
        "12",
        "rejects a `catch` clause that an earlier clause makes useless",
        &[
            ("class A { void f() { try { } catch (Throwable e) { } catch (ArithmeticException e) { } } }", "this `catch` clause can never run"),
            ("class A { Int f(Object o) { return switch (o) { case Object x -> 1; case String s -> 2; }; } }", "this arm cannot match"),
        ],
        &[],
    ),
    rule(
        "12",
        "The body of any other method must not be able to complete normally",
        &[
            ("class A { Int f(Boolean b) { if (b) { return 1; } } }", FALLS_OFF),
            ("class A { Int f(Int n) { while (n > 0) { return 1; } } }", FALLS_OFF),
            ("class A { Function0<Int> f(Boolean b) { return () -> { if (b) { return 1; } }; } }", FALLS_OFF),
        ],
        &["class A { Int f(Boolean b) { if (b) { return 1; } else { throw new IllegalStateException(); } } Int g() { while (true) { } } void h() { } }"],
    ),
    rule(
        "12",
        "`return;` is legal only after the `super(...)` or `this(...)` call that the body writes",
        &[
            ("class A { Int n; public A(Int n) { if (n > 0) { return; } this.n = n; super(); } }", "in a constructor, `return;` is legal only after the `super(...)` or `this(...)` call that the body writes"),
            ("class A { public A() { return; } }", "in a constructor, `return;` is legal only after the `super(...)` or `this(...)` call that the body writes"),
        ],
        &["class A { Int n; public A(Int n) { this.n = n; super(); if (n > 0) { return; } } }"],
    ),
    rule("12", "`return` with a value is rejected in a constructor", &[("class A { public A() { return 1; } }", "`return` with a value is rejected in a constructor")], &["class A { public A() { } }"]),
    rule(
        "12",
        "A statement that cannot complete normally contributes no path to what follows it",
        &[],
        &["class A { Int f(Boolean b) { Int n; if (b) { throw new IllegalStateException(); } else { n = 2; } return n; } Int g(Boolean b) { Int n; if (b) { n = 1; } else { return 0; } return n; } }"],
    ),
    rule(
        "12",
        "A use of a local that is not definitely assigned is rejected.",
        &[
            ("class A { Int f(Boolean b) { Int n; if (b) { n = 1; } return n; } }", UNASSIGNED),
            ("class A { Int f(Int k) { Int n; while (k > 0) { n = 1; k--; } return n; } }", UNASSIGNED),
            ("class A { Int f() { Int n; try { n = 1; } catch (Throwable e) { } return n; } }", UNASSIGNED),
        ],
        &["class A { Int f(Boolean b) { Int n; if (b) { n = 1; } else { n = 2; } return n; } }"],
    ),
    rule(
        "12",
        "A branch that cannot complete normally does not count against it",
        &[("class A { Int f(Boolean b) { Int n; if (b) { n = 1; } else { } return n; } }", UNASSIGNED)],
        &["class A { Int f(Int k) { Int n; switch (k) { case 1 -> { n = 1; } case 2 -> { return 0; } default -> { throw new IllegalStateException(); } } return n; } }"],
    ),
    rule(
        "12",
        "An assignment in the body does not count, because the body may not run",
        &[("class A { Int f(Int k) { Int n; while (k > 0) { n = 1; k--; } return n; } }", UNASSIGNED), ("class A { Int f(List<Int> xs) { Int n; for (Int x : xs) { n = x; } return n; } }", UNASSIGNED)],
        &["class A { Int f(Int k) { Int n; while (true) { n = 1; if (k > 0) { break; } } return n; } }"],
    ),
    rule(
        "12",
        "An assignment to it is rejected unless the local is definitely unassigned there",
        &[
            ("class A { void f(Boolean b) { final Int n; if (b) { n = 1; } n = 2; } }", FINAL_LOCAL),
            ("class A { void f() { final Int n = 1; n = 2; } }", FINAL_LOCAL),
            ("class A { void f(Int k) { final Int n; while (k > 0) { n = 1; k--; } } }", FINAL_LOCAL),
        ],
        &["class A { Int f(Boolean b) { final Int n; if (b) { n = 1; } else { n = 2; } return n; } }"],
    ),
    rule(
        "12",
        "A lambda uses only variables that are never assigned again, so the fact cannot change",
        &[("class A { Function0<Int> f(@Nullable String s) { return () -> s.length(); } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Function0<Int> f(@Nullable String s) { if (s == null) { return () -> 0; } return () -> s.length(); } }"],
    ),
    // ---- chapter 13 ----
    stated("13", "A value that must change in one step is held in an atomic cell", "It is advice to a program, and no checker decides which values those are."),
];

/// The sentences that forbid or limit in other words: "only", "never", "does not",
/// "is not a", "are not", "has no", "no ... may". Every such sentence of the chapters
/// is here too, with its programs or with the reason it has none.
const FURTHER: &[Rule] = &[
    // ---- chapter 1 ----
    rule(
        "01",
        "A program does not declare a package named `cleat`",
        &[("==== cleat/Extra.cleat\npackage cleat; public class Extra { }", "a program does not declare the package `cleat`, or a package whose name begins with `cleat.`"), ("==== cleat/util/Extra.cleat\npackage cleat.util; public class Extra { }", "a program does not declare the package `cleat`, or a package whose name begins with `cleat.`")],
        &["==== cleats/Extra.cleat\npackage cleats; public class Extra { }"],
    ),
    rule("01", "Two fields of one type do not share a name", &[("class A { Int x = 1; Int x = 2; }", "the field `x` is declared twice")], &["class A { Int x = 1; } class B { Int x = 2; }"]),
    rule(
        "01",
        "It does not name the field it initializes or one declared later",
        &[("class A { Int a = b; Int b = 1; }", "the field `b` is read before it is assigned"), ("class A { Int a = a + 1; }", "the field `a` is read before it is assigned"), ("class A { static Int a = b; static Int b = 1; }", "the static field `b` is read before it is assigned")],
        &["class A { Int a = 1; Int b = a + 1; static Int c = 2; static Int d = c; }"],
    ),
    // ---- chapter 2 ----
    rule("02", "no class may extend it", &[("class A { } class B extends A { }", "`A` is final; only an `open`, `sealed` or `abstract` class is extended")], &["open class A { } class B extends A { } abstract class C { } class D extends C { }"]),
    rule("02", "`sealed` lets only the classes named in its `permits` clause extend it", &[("sealed class A permits B { } class B extends A { } class C extends A { }", "`A` is sealed and does not permit this type")], &[]),
    rule(
        "02",
        "A class whose methods happen to match an interface does not implement it",
        &[("interface I { Int f(); } class C { public Int f() { return 1; } } class D { I g(C c) { return c; } }", "`C` is not assignable to `I`")],
        &["interface I { Int f(); } class C implements I { @Override public Int f() { return 1; } } class D { I g(C c) { return c; } }"],
    ),
    rule(
        "02",
        "names in `permits` the only types that may implement or extend it",
        &[("sealed interface S permits P { } class P implements S { } class Q implements S { }", "`S` is sealed and does not permit this type"), ("sealed interface S permits P { } class P implements S { } interface J extends S { }", "`S` is sealed and does not permit this type")],
        &["sealed interface S permits P, J { } class P implements S { } interface J extends S { }"],
    ),
    rule("02", "An interface has no instance fields", &[("interface I { Int x; }", "a field of an interface is a constant and needs an initializer")], &["interface I { Int X = 1; }"]),
    rule("02", "A constant has no class body", &[("enum E { A { } }", "an enum constant has no class body")], &["enum E { A, B }"]),
    rule(
        "02",
        "A static member does not use the type parameters of its class",
        &[("class A<T> { static void f(T x) { } }", "there is no type named `T` here"), ("class A<T> { static @Nullable T held = null; }", "there is no type named `T` here")],
        &["class A<T> { static <U> U f(U x) { return x; } static Int[] g(Int n) { return Array.build(n, (i) -> i); } }"],
    ),
    rule(
        "02",
        "A final field is assigned on every constructor path and never afterwards",
        &[
            ("class A { final Int n = 1; void f() { n = 2; } }", "the field `n` is not assigned here: it is final"),
            ("class A { final Int n; public A() { } }", "the field `n` is not assigned on every path"),
            ("value class V { public Int a; public V bump() { a = 2; return this; } }", "the field `a` is not assigned here: a field of a value class is final"),
        ],
        &["class A { final Int n; Int m = 0; public A(Int n) { this.n = n; super(); } void f() { m = n; } }"],
    ),
    rule(
        "02",
        "`Object` does not declare `wait`, `notify`, `notifyAll`, `clone` or `finalize`",
        &[
            ("class A { void f(Object o) { o.notify(); } }", "`Object` has no method named `notify`"),
            ("class A { void f(Object o) { o.notifyAll(); } }", "`Object` has no method named `notifyAll`"),
            ("class A { void f(Object o) { o.finalize(); } }", "`Object` has no method named `finalize`"),
        ],
        &["class A { String f(Object o) { return o.toString() + o.hashCode() + o.equals(o); } }"],
    ),
    rule(
        "02",
        "A method overrides another only as chapter 4 allows, and it carries `@Override`",
        &[("open class A { public Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }", "`f` is final in `A` and cannot be overridden")],
        &["open class A { public open Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }"],
    ),
    // ---- chapter 3 ----
    rule(
        "03",
        "`protected` does not include the rest of the package",
        &[("open class A { protected Int x = 1; } class B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")],
        &["open class A { protected Int x = 1; } class B extends A { Int f() { return x; } }"],
    ),
    rule(
        "03",
        "uses a protected instance member only on a receiver whose static type is `S` or a subtype of `S`",
        &[("open class A { protected Int x = 1; } class S extends A { Int f(A other) { return other.x; } }", "the field `x` of `A` is outside its audience here")],
        &["open class A { protected Int x = 1; protected open Int g() { return 1; } } open class S extends A { Int f(S other, T sub) { return other.x + this.x + x + sub.x + super.g(); } } class T extends S { }"],
    ),
    rule("03", "It does not reach into an instance of another subclass", &[("open class A { protected Int x = 1; } class S extends A { Int f(T other) { return other.x; } } class T extends A { }", "the field `x` of `A` is outside its audience here")], &[]),
    rule("03", "A member is never visible more widely than its type", &[("==== A.cleat\nclass Hidden { public Int x = 1; }\n==== B.cleat\nclass B { Int f(Hidden h) { return h.x; } }", "there is no type named `Hidden` here")], &[]),
    rule("03", "A public method does not take a parameter of a type that is private to the file", &[("==== A.cleat\npublic class A { public void f(Hidden h) { } } class Hidden { }", "its signature names it")], &[]),
    rule(
        "03",
        "A method can be overridden only by a class that may name it",
        &[(
            "==== a/A.cleat\npackage a; public open class A { package open Int f() { return 1; } }\n==== b/B.cleat\npackage b; import a.A; public class B extends A { @Override public Int f() { return 2; } }",
            "`f` carries `@Override` and overrides nothing",
        )],
        &["==== a/A.cleat\npackage a; public open class A { package open Int f() { return 1; } }\n==== b/B.cleat\npackage b; import a.A; public class B extends A { public Int f() { return 2; } }"],
    ),
    rule(
        "03",
        "It is never narrower",
        &[
            ("open class A { public open Int f() { return 1; } } class B extends A { @Override Int f() { return 2; } }", "an overriding method has the audience of the method it overrides, or a wider one"),
            ("open class A { public open Int f() { return 1; } } class B extends A { @Override protected Int f() { return 2; } }", "an overriding method has the audience of the method it overrides, or a wider one"),
        ],
        &["open class A { protected open Int f() { return 1; } package open Int g() { return 1; } } class B extends A { @Override public Int f() { return 2; } @Override protected Int g() { return 2; } }"],
    ),
    rule(
        "03",
        "It does not drop `only`",
        &[("open class A { public only(B, C) open Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } } class C { }", "the overridden method has an `only` list, so the overriding method has the same audience keyword and a list drawn from that list")],
        &["open class A { public only(B, C) open Int f() { return 1; } } class B extends A { @Override public only(C) Int f() { return 2; } } class C { }"],
    ),
    rule("03", "A written constructor with no audience is `private`", &[("class R { R() { } } class A { R f() { return new R(); } }", "the constructor of `R` is outside its audience here")], &["class R { R() { } static R make() { return new R(); } }"]),
    // ---- chapter 4 ----
    rule("04", "An `abstract` method has no body and appears only in an abstract class", &[("class A { abstract void f(); }", "an abstract method appears only in an abstract class"), ("abstract class A { abstract void f() { } }", "an abstract, `foreign` or `@Intrinsic` method has no body")], &["abstract class A { abstract void f(); }"]),
    rule("04", "A static method and a constructor do not declare a receiver", &[("class A { static void f(A this) { } }", "a static method does not declare a receiver"), ("class A { public A(A this) { } }", "a constructor does not declare a receiver")], &["class A { void f(A this) { } }"]),
    rule(
        "04",
        "Only an `open` method, an `abstract` method, or an interface method not declared `final` can be overridden",
        &[
            ("interface I { final Int f() { return 1; } } class C implements I { @Override public Int f() { return 2; } }", "`f` is final in `I` and cannot be overridden"),
            ("open class A { public static Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }", "`f` is static in one declaration and not in the other"),
        ],
        &["abstract class A { public abstract Int f(); public open Int g() { return 1; } } interface I { Int h() { return 1; } } class B extends A implements I { @Override public Int f() { return 2; } @Override public Int g() { return 2; } @Override public Int h() { return 2; } }"],
    ),
    rule(
        "04",
        "Types that differ only in qualifiers do not count as different, and neither do result types",
        &[("class A { void f(String s) { } void f(@Nullable String s) { } }", "the method `f` is declared twice with the same parameter types"), ("class A { Int f() { return 1; } String f() { return \"\"; } }", "the method `f` is declared twice with the same parameter types")],
        &["class A { void f(String s) { } void f(Int n) { } void f(String s, Int n) { } }"],
    ),
    rule(
        "04",
        "so an `Int32[]` is never an `Int[]`",
        &[("class A { Int[] f(Int32[] a) { return a; } }", "`Int32[]` is not assignable to `Int[]`"), ("class A { Int f(@Nullable Int32 n) { Int m = n; return m; } }", "is not assignable to `Int`")],
        &["class A { Int f(Int32 n) { Int m = n; return m; } }"],
    ),
    rule(
        "04",
        "`break` and `continue` in a lambda do not reach a statement outside it",
        &[
            ("class A { void f() { while (true) { Function0<Unit> g = () -> { break; }; } } }", "`break` is not inside a loop or a switch statement"),
            ("class A { void f() { a: while (true) { Function0<Unit> g = () -> { continue a; }; } } }", "no statement labeled `a` encloses this `continue`"),
        ],
        &["class A { void f() { Function0<Unit> g = () -> { while (true) { break; } }; } }"],
    ),
    rule("04", "A method declared `foreign` is also declared `static` and has no body", &[("class A { foreign void f(); }", "a `foreign` method is also declared `static`"), ("class A { foreign static void f() { } }", "an abstract, `foreign` or `@Intrinsic` method has no body")], &["class A { foreign static void f(); }"]),
    // ---- chapter 5 ----
    rule("05", "`@Nullable T` is not a subtype of `T`", &[("class A { String f(@Nullable String s) { return s; } }", "`@Nullable String` is not assignable to `String`")], &["class A { @Nullable String f(String s) { return s; } }"]),
    rule(
        "05",
        "such as `T extends Object`, does not",
        &[("class Box<T extends Object> { } class A { void f(Box<@Nullable String> b) { } }", "`@Nullable String` is not within the bound `Object` of the type parameter `T`")],
        &["class Box<T extends Object> { } class Bag<T> { } class A { void f(Box<String> b, Bag<@Nullable String> c) { } }"],
    ),
    rule(
        "05",
        "It does not apply to a field, an array element, or any other expression that is evaluated again",
        &[
            ("class A { @Nullable String s = null; Int f() { if (s != null) { return s.length(); } return 0; } }", "`length` is sent to a `@Nullable String`"),
            ("class A { Int f(@Nullable String[] a) { if (a[0] != null) { return a[0].length(); } return 0; } }", "`length` is sent to a `@Nullable String`"),
        ],
        &["class A { @Nullable String s = null; Int f() { var held = s; if (held != null) { return held.length(); } return 0; } }"],
    ),
    // ---- chapter 6 ----
    rule("06", "A value of one numeric class is never an instance of a different one", &[("class A { Boolean f(Int32 x) { return x instanceof Int; } }", "`Int32` and `Int` are disjoint")], &["class A { Boolean f(Object x) { return x instanceof Int; } }"]),
    rule(
        "06",
        "The integer classes do not",
        &[("class A { Divisible<Int> f(Int n) { return n; } }", "`Int` is not assignable to `Divisible<Int>`"), ("class A { Divisible<UInt8> f(UInt8 n) { return n; } }", "`UInt8` is not assignable to `Divisible<UInt8>`")],
        &["class A { static <T extends Divisible<T>> void g(T x) { } static void f(Float64 x, Rational r, Float32 y) { g(x); g(r); g(y); } Numeric<Int> h(Int n) { return n; } }"],
    ),
    rule("06", "A float class does not declare `mod`", &[("class A { Float64 f(Float64 a, Float64 b) { return a % b; } }", "`%` is the method `mod`, and `Float64` has no method `mod`")], &["class A { Int f(Int a, Int b) { return a % b; } }"]),
    rule("06", "No value is converted to a `String` implicitly", &[("class A { String f(Int n) { String s = n; return s; } }", "`Int` is not assignable to `String`")], &["class A { String f(Int n) { String s = n.toString(); return s + n; } }"]),
    rule("06", "`String[]` is not a subtype of `Object[]`", &[("class A { Object[] f(String[] a) { return a; } }", "`String[]` is not assignable to `Object[]`")], &["class A { Object f(String[] a) { return a; } }"]),
    rule("06", "`T[]` does not contain `null` unless `T` is `@Nullable`", &[("class A { void f(String[] a) { a[0] = null; } }", "`null` is not a value of `String`")], &["class A { void f(@Nullable String[] a) { a[0] = null; } }"]),
    rule(
        "06",
        "`Char` and `Boolean` are not numeric",
        &[("class A { Int f(Boolean b) { return b + 1; } }", "`Boolean` has no method `plus`"), ("class A { Int f(Char c) { Int n = c; return n; } }", "`Char` is not assignable to `Int`")],
        &["class A { Int f(Char c) { return c.code(); } }"],
    ),
    // ---- chapter 7 ----
    rule("07", "They are not in scope in its static members", &[("class A<T> { static void f(T x) { } }", "there is no type named `T` here")], &["class A<T> { void f(T x) { } static <T> void g(T x) { } }"]),
    rule(
        "07",
        "`List<String>` is not a subtype of `List<Object>`",
        &[("class A { List<Object> f(List<String> xs) { return xs; } }", "`List<String>` is not assignable to `List<Object>`")],
        &["class A { List<? extends Object> f(List<String> xs) { return xs; } Iterable<Object> g(List<String> xs) { return xs; } }"],
    ),
    rule(
        "07",
        "It is not a subtype of `List<Shape>`",
        &[("open class Shape { } class A { List<Shape> f(List<? extends Shape> xs) { return xs; } }", "is not assignable to `List<Shape>`")],
        &["open class Shape { } class Circle extends Shape { } class A { List<? extends Shape> f(List<Circle> xs) { return xs; } }"],
    ),
    rule(
        "07",
        "`List<List<Circle>>` is not a subtype of `List<List<?>>`",
        &[("class Circle { } class A { List<List<?>> f(List<List<Circle>> xs) { return xs; } }", "is not assignable to `List<List<?>>`")],
        &["class Circle { } class A { List<? extends List<?>> f(List<List<Circle>> xs) { return xs; } }"],
    ),
    rule(
        "07",
        "A generic method is overridden only by a generic method with the same number of type parameters and the same bounds",
        &[("open class A { public open <T> void f(T x) { } } class B extends A { @Override public <T extends Throwable> void f(T x) { } }", "`f` carries `@Override` and overrides nothing")],
        &["open class A { public open <T> void f(T x) { } public open <T extends Throwable> void g(T x) { } } class B extends A { @Override public <U> void f(U x) { } @Override public <U extends Throwable> void g(U x) { } }"],
    ),
    // ---- chapter 8 ----
    rule("08", "It has no body and no type parameters", &[("annotation N { }", "an annotation declaration has no body")], &["annotation N; annotation M(Int n);"]),
    rule("08", "A qualifier has no elements", &[("@Refines annotation Q(Int n);", "a qualifier has no elements")], &["annotation M(Int n); @Refines annotation Q;"]),
    rule("08", "A program does not construct an instance with `new`", &[("annotation M; class A { Object f() { return new M(); } }", "`new M` is rejected: an annotation value comes only from a use of the annotation")], &[]),
    rule("08", "Parameter types that differ only in qualifiers do not distinguish overloads", &[("@Refines annotation P; class A { void f(Int n) { } void f(@P Int n) { } }", "the method `f` is declared twice with the same parameter types")], &[]),
    rule(
        "08",
        "A literal has no refinement, and neither has the result of `new`",
        &[("@Refines annotation P; class A { @P Int f() { return 1; } }", "`Int` is not assignable to `@P Int`"), ("@Refines annotation P; class B { } class A { @P B f() { return new B(); } }", "`B` is not assignable to `@P B`")],
        &[],
    ),
    rule(
        "08",
        "An invariant type parameter matches two arguments only when they have the same qualifiers",
        &[("@Refines annotation P; class A { List<Int> f(List<@P Int> xs) { return xs; } }", "`List<@P Int>` is not assignable to `List<Int>`")],
        &["@Refines annotation P; class A { List<@P Int> f(List<@P Int> xs) { return xs; } Iterable<Int> g(List<@P Int> xs) { return xs; } }"],
    ),
    // ---- chapter 9 ----
    rule(
        "09",
        "Constructors are not inherited",
        &[("open class A { public A(Int n) { } } class B extends A { public B() { super(1); } } class C { B f() { return new B(1); } }", "constructor `B` takes 0 arguments, and 1 is written")],
        &["open class A { public A(Int n) { } } class B extends A { public B() { super(1); } } class C { B f() { return new B(); } }"],
    ),
    rule(
        "09",
        "`this` may be used only to assign a field declared in the class",
        &[
            ("class A { Int n; public A() { Object o = this; this.n = 1; super(); } }", "before `super(...)` is called, `this` is used only to assign a field of the class and to read one that is assigned"),
            ("open class P { public Int base = 0; } class A extends P { Int n; public A() { this.n = base; super(); } }", "the inherited field `base` is not used before `super(...)` is called"),
        ],
        &["class A { Int n; Int m; public A() { this.n = 1; m = n + 1; super(); Object o = this; } }"],
    ),
    rule(
        "09",
        "No method is called on `this`",
        &[
            ("class A { Int n; public A() { this.n = g(); super(); } Int g() { return 1; } }", "no method is called on `this` before `super(...)` is called"),
            ("class A { Int n; public A() { this.n = this.g(); super(); } Int g() { return 1; } }", "before `super(...)` is called, `this` is used only to assign a field of the class and to read one that is assigned"),
        ],
        &["class A { Int n; public A() { this.n = h(); super(); n = g(); } Int g() { return 1; } static Int h() { return 2; } }"],
    ),
    rule("09", "A field of a value class has no initializer", &[("value class V { public Int a = 1; }", "a field of a value class has no initializer")], &["value class V { public Int a; }"]),
    // ---- chapter 11 ----
    rule("11", "A constructor ends in `;` only when it is `@Intrinsic`", &[("class A { public A(); }", "a constructor needs a body")], &["class A { public A() { } }"]),
    // ---- chapter 12 ----
    rule(
        "12",
        "With no `else`, the local is assigned after the `if` only when it was assigned before it or by `c`",
        &[("class A { Int f(Boolean b) { Int n; if (b) { n = 1; } return n; } }", UNASSIGNED)],
        &["class A { Int f(Boolean b) { Int n = 0; if (b) { n = 1; } return n; } }"],
    ),
    rule(
        "12",
        "a local is assigned only when it was assigned before the `try` statement",
        &[("class A { Int f() { Int n; try { n = 1; } catch (Throwable e) { return n; } return n; } }", UNASSIGNED), ("class A { void f() { Int n; try { n = 1; } finally { var m = n; } } }", UNASSIGNED)],
        &["class A { Int f() { Int n = 0; try { n = 1; } catch (Throwable e) { return n; } finally { var m = n; } return n; } }"],
    ),
    rule(
        "12",
        "A lambda may use a local only when it is definitely assigned before the lambda expression",
        &[("class A { void f() { Int n; Function0<Int> g = () -> n; } }", UNASSIGNED)],
        &["class A { void f() { Int n; n = 1; Function0<Int> g = () -> n; } }"],
    ),
    rule(
        "12",
        "A field is read there only where it is definitely assigned",
        &[("class A { Int n; Int m; public A() { this.m = n; this.n = 1; super(); } }", "the field `n` is read before it is assigned")],
        &["class A { Int n; Int m; public A() { this.n = 1; this.m = n; super(); } }"],
    ),
    rule("12", "A static final field is assigned at most once, and only there", &[("class A { static final Int X = 1; static void f() { X = 2; } }", "the static final field `X` is assigned only by its initializer or a static initializer block")], &["class A { static final Int X = 1; static Int f() { return X; } }"]),
    rule(
        "12",
        "holds in its `catch` and `finally` blocks when the `try` block does not assign the variable",
        &[("class A { Int f(@Nullable String s, @Nullable String t) { if (s == null) { return 0; } try { s = t; } catch (Throwable e) { return s.length(); } return 0; } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s) { if (s == null) { return 0; } try { var k = f(s); } catch (Throwable e) { return s.length(); } return 0; } }"],
    ),
    rule(
        "12",
        "A fact that holds before a loop holds in the loop only when no statement of the loop assigns `x`",
        &[("class A { Int f(@Nullable String s, @Nullable String t) { if (s == null) { return 0; } var n = 0; while (n < 3) { n = n + s.length(); s = t; } return n; } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s) { if (s == null) { return 0; } var n = 0; while (n < 3) { n = n + s.length() + 1; } return n; } }"],
    ),
    // ---- the rest of the sentences that restrict, chapter by chapter ----
    rule("01", "A line break does not end a statement", &[], &["class A { Int f() {\n return 1\n + 2\n ; } }"]),
    rule("01", "These words are keywords only in one position each", &[], &["class A { Int open = 1; Int value = 2; Int only = 3; void sealed() { } Int in(Int out) { return out; } }"]),
    rule("01", "A decimal numeral has no leading `0` unless it is exactly `0`", &[("class A { Int f() { return 007; } }", "a decimal numeral has no leading zero")], &["class A { Int f() { return 0; } Float64 g() { return 0.5; } }"]),
    rule("01", "A numeric literal has no suffix", &[("class A { Int f() { return 1L; } }", "a numeric literal has no suffix"), ("class A { Float64 f() { return 2.5d; } }", "a numeric literal has no suffix")], &[]),
    stated("01", "It is not an object and has no members of its own", "It says what a package is; the import cases name types through packages."),
    rule(
        "01",
        "It does not import the packages inside `a.b`",
        &[("==== a/b/c/X.cleat\npackage a.b.c; public class X { }\n==== D.cleat\nimport a.b.*; class D { void f(X x) { } }", "there is no type named `X` here")],
        &["==== a/b/c/X.cleat\npackage a.b.c; public class X { }\n==== D.cleat\nimport a.b.c.*; class D { void f(X x) { } }"],
    ),
    rule("01", "and the language does not require it", &[], &["class lower { } class A { void f(lower x) { } }"]),
    rule(
        "02",
        "The compiler does not rewrite an operator into a different method",
        &[("class P { } class Q { public Q plus(P p) { return this; } } class A { void f(P p, Q q) { var x = p + q; } }", "`+` is the method `plus`, and `P` has no method `plus`")],
        &["class P { } class Q { public Q plus(P p) { return this; } } class A { void f(P p, Q q) { var x = q + p; } }"],
    ),
    rule("02", "and the null test is not a method", &[("class A { Object f() { return null; } }", "`null` is not a value of `Object`")], &["class A { Boolean f(@Nullable Object o) { return o == null; } }"]),
    stated("02", "Its instances have no identity", "Checked by tests/behavior/Rules.cleat, line `identical`: two value instances with equal fields are identical."),
    rule(
        "02",
        "It extends no class other than `Object`, and no class extends it",
        &[("open class P { } value class V extends P { public Int a; }", "a value class extends no class"), ("value class V { public Int a; } class W extends V { }", "`V` is not a class that can be extended")],
        &["interface I { } value class V implements I { public Int a; }"],
    ),
    stated("02", "The language does not check these obligations", "It says what is left unchecked."),
    stated("02", "For a prelude value class whose fields are not written in source, `identical` is `equals`", "Checked by tests/behavior/Rules.cleat, line `identical`: two `Int` values of 5 are identical."),
    stated("02", "so the result never depends on how the implementation stored it", "Checked by tests/behavior/Rules.cleat, line `identical`, through variables of type `Object`."),
    rule(
        "02",
        "a sealed interface with no methods is the usual root",
        &[],
        &["sealed interface S permits P, Q { } class P implements S { } class Q implements S { } class A { Int f(S s) { return switch (s) { case P p -> 1; case Q q -> 2; }; } }"],
    ),
    rule("02", "The private-by-default rule of classes does not apply to them", &[], &["interface I { Int f(); Int X = 1; } class D { Int g(I i) { return i.f() + I.X; } }"]),
    rule("02", "A static method has no receiver", &[("class A { Int n = 1; static Int f() { return n; } }", "`this` is not available in a static context")], &["class A { static Int n = 1; static Int f() { return n; } }"]),
    stated("02", "It does not override it", "Checked by tests/behavior/Rules.cleat, line `statics`: each static method answers for the type that names it."),
    stated("02", "Evaluating it does not initialize the class", "Checked by tests/behavior/Rules.cleat, line `class literal`."),
    stated("02", "No field is dispatched", "Checked by tests/behavior/Rules.cleat, line `fields`."),
    stated("02", "A final method, a static method and a field are not dispatched", "Checked by tests/behavior/Rules.cleat, lines `fields` and `statics`."),
    stated("02", "runs the superclass's method and does not dispatch", "Checked by tests/behavior/Rules.cleat, line `super`."),
    stated("02", "lists the features of Java that Cleat does not have", "It introduces the next sentence, whose case has a program for each feature."),
    rule(
        "03",
        "A field that everyone may read and only the class may assign",
        &[("class A { private Int n = 0; public Int count() { return n; } } class B { void f(A a) { a.n = 1; } }", "the field `n` of `A` is outside its audience here")],
        &["class A { private Int n = 0; public Int count() { return n; } } class B { Int f(A a) { return a.count(); } }"],
    ),
    rule(
        "03",
        "is callable from outside the file only through a supertype that declares the method",
        &[(
            "==== Shape.cleat\npublic interface Shape { Int area(); }\n==== Shapes.cleat\nclass Square implements Shape { @Override public Int area() { return 4; } public Int side() { return 2; } } public class Shapes { public static Shape make() { return new Square(); } }\n==== B.cleat\nclass B { Int f() { return Shapes.make().side(); } }",
            "`Shape` has no method named `side`",
        )],
        &["==== Shape.cleat\npublic interface Shape { Int area(); }\n==== Shapes.cleat\nclass Square implements Shape { @Override public Int area() { return 4; } } public class Shapes { public static Shape make() { return new Square(); } }\n==== B.cleat\nclass B { Int f() { return Shapes.make().area(); } }"],
    ),
    stated("03", "It does not depend on the class of an object at run time", "It says the audience rules are decided from static types, as every case of this chapter is."),
    stated("03", "use only members declared `public`", "Checked by tests/behavior/Annotations.cleat, where a mirror of a member that is not public raises `IllegalAccessException`."),
    stated("03", "They are not audiences", "It points to section 2.4, whose sentences have their own cases."),
    rule("04", "A static method has no receiver", &[("class A { void g() { } static void f() { this.g(); } }", "`this` is not available in a static context")], &["class A { static void g() { } static void f() { g(); A.g(); } }"]),
    stated("04", "Assigning a parameter does not change a variable of the caller", "Checked by tests/behavior/Rules.cleat, line `parameter`."),
    rule(
        "04",
        "therefore accepts only the methods that declare a `@Nullable` receiver, until it is narrowed",
        &[("class A { Int f(@Nullable String s) { return s.length(); } }", "`length` is sent to a `@Nullable String`")],
        &["class A { String f(@Nullable String s) { return s.toString(); } Int g(@Nullable String s) { return s == null ? 0 : s.length(); } }"],
    ),
    rule(
        "04",
        "Resolution first considers only the methods that are applicable without an implicit conversion",
        &[],
        &["class A { static Int g(Int32 a) { return 1; } static String g(Int a) { return \"\"; } static Int f(Int32 x) { return g(x); } static String h(Int x) { return g(x); } }"],
    ),
    rule(
        "04",
        "a class type and an interface type are disjoint when the class is final and is not a subtype of the interface",
        &[("class Q { } interface I { } class C { Boolean f(Q a, I i) { return a == i; } }", "`==` between `Q` and `I` is rejected: the two types are disjoint")],
        &["open class P { } interface I { } class Q implements I { } class C { Boolean f(P a, Q q, I i) { return a == i && q == i; } }"],
    ),
    rule("04", "two `@Nullable` types are never disjoint", &[], &["class C { Boolean f(@Nullable String s, @Nullable Int n) { return s == n; } }"]),
    rule("04", "two interface types are never disjoint, and a type parameter is disjoint from nothing", &[], &["interface I { } interface J { } class C { <T> Boolean f(I i, J j, T t, String s) { return i == j && t == s; } }"]),
    rule(
        "04",
        "An operator is a call of a method on its left operand, or on its only operand",
        &[("class P { } class A { P f(P a) { return -a; } }", "`-` is the method `negate`, and `P` has no method `negate`")],
        &["class P { public P plus(P o) { return this; } public P negate() { return this; } } class A { P f(P a, P b) { return -a + b; } }"],
    ),
    rule(
        "04",
        "These two are not methods",
        &[("class P { public Boolean and(P o) { return true; } } class A { Boolean f(P a, P b) { return a && b; } }", "`P` is not assignable to `Boolean`")],
        &["class P { public Boolean and(P o) { return true; } } class A { Boolean f(P a, P b) { return a & b; } }"],
    ),
    rule("04", "A cast never converts a value", &[("class A { Int f(Int32 x) { return (Int) x; } }", "a cast from `Int32` to `Int` is rejected: the types are disjoint, and a cast never converts a value")], &[]),
    rule("04", "It does not change the class of a number", &[("class A { Int f(Float64 x) { return (Int) x; } }", "a cast from `Float64` to `Int` is rejected")], &["class A { Int f(Float64 x) { return Int.from(x.round()); } }"]),
    rule("04", "It has no type of its own", &[("class A { void f() { var g = (Int x) -> x; } }", "a lambda has no type of its own")], &["class A { void f() { Function1<Int, Int> g = (Int x) -> x; } }"]),
    rule(
        "04",
        "A lambda may use a local or a parameter of an enclosing method or lambda only if",
        &[("class A { Function0<Int> f(Int n) { n = n + 1; return () -> n; } }", "a lambda may use `n` only if it is never assigned after it is initialized")],
        &["class A { Function0<Int> f(Int n) { return () -> n; } }"],
    ),
    stated("04", "A lambda therefore has no identity", "Checked by tests/behavior/Rules.cleat, line `lambdas`: two evaluations that capture equal values are equal."),
    stated("04", "It does not run after `break`, `return` or a raised exception", "Checked by tests/behavior/Rules.cleat, line `for`."),
    rule(
        "04",
        "Cleat does not read or write through a `Pointer`",
        &[("class A { Int f(Pointer p) { return p.read(); } }", "`Pointer` has no method named `read`"), ("class A { void f(Pointer p) { p.write(1); } }", "`Pointer` has no method named `write`")],
        &["class A { Pointer f() { return Pointer.zero(); } }"],
    ),
    stated("04", "has no defined behavior", "It is about the C function, which is outside the language."),
    rule("05", "`Null` is not a subclass of `Object`, and no class extends it", &[("class A { Object f(Null n) { return n; } }", "`Null` is not assignable to `Object`"), ("class B extends Null { }", "`Null` is not a class that can be extended")], &["class A { @Nullable Object f(Null n) { return n; } Null g() { return null; } @Nullable String h(Null n) { return n; } }"]),
    rule(
        "05",
        "A type written without `@Nullable` does not contain `null`",
        &[("class A { Int n = null; }", "`null` is not a value of `Int`"), ("class A { List<String> f() { return null; } }", "`null` is not a value of `List<String>`")],
        &["class A { @Nullable Int n = null; }"],
    ),
    rule(
        "05",
        "Every other field has no default and is definitely assigned on every constructor path",
        &[("class A { String s; public A() { } }", "the field `s` is not assigned on every path")],
        &["class A { @Nullable String s; String t = \"t\"; public A() { } }"],
    ),
    rule(
        "05",
        "only when the method declares its receiver `@Nullable`",
        &[("class P { public Int size() { return 1; } } class A { Int f(@Nullable P p) { return p.size(); } }", "`size` is sent to a `@Nullable P`")],
        &["class P { public Int size(@Nullable P this) { return this == null ? 0 : 1; } } class A { Int f(@Nullable P p) { return p.size(); } }"],
    ),
    stated("05", "A null test is not a send of `equals`", "Checked by tests/behavior/Rules.cleat, line `null test`, on a class whose `equals` answers `true` to everything."),
    stated("05", "Its result depends only on whether the value of `e` is `null`", "Checked by tests/behavior/Rules.cleat, line `null test`, on a class whose `equals` answers `true` to everything."),
    rule(
        "05",
        "and does not assign `x` after that test",
        &[("class A { Int f(@Nullable String s, @Nullable String t) { if (s != null) { s = t; return s.length(); } return 0; } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s) { if (s != null) { return s.length(); } return 0; } }"],
    ),
    rule(
        "06",
        "No numeric class extends another, and their only common superclass is `Object`",
        &[("class A { Numeric<Int> f(Int32 x) { return x; } }", "`Int32` is not assignable to `Numeric<Int>`")],
        &["class A { Object f(Int32 x) { return x; } Numeric<Int32> g(Int32 x) { return x; } }"],
    ),
    rule("06", "A numeric literal has no class of its own", &[], &["class A { Int8 a = 1; Float64 b = 1; Rational c = 1; UInt64 d = 1; Float32 e = 1.5; Rational g = 1.5; }"]),
    rule(
        "06",
        "the other operand or arm is not a literal expression and has a numeric static type `C`",
        &[("class A { UInt8 g(UInt8 u) { return u + 256; } }", "the literal is outside the range of `UInt8`")],
        &["class A { Float64 f(Float64 x) { return x + 1; } UInt8 g(UInt8 u) { return u + 1; } Float32 h(Boolean b, Float32 y) { return b ? y : 2; } }"],
    ),
    rule(
        "06",
        "Rounding a decimal literal to a float is the only implicit rounding in the language",
        &[("class A { Float64 f(Rational r) { return r; } }", "`Rational` is not assignable to `Float64`"), ("class A { Float32 f(Float64 x) { return x; } }", "`Float64` is not assignable to `Float32`")],
        &["class A { Float64 f() { return 0.1; } }"],
    ),
    rule(
        "06",
        "It happens only where the program wrote a float type",
        &[("class A { Float64 f() { var x = 0.1; return x; } }", "`Rational` is not assignable to `Float64`")],
        &["class A { Rational f() { var x = 0.1; return x; } Float64 g() { Float64 x = 0.1; return x; } }"],
    ),
    stated("06", "on an integer class raise `ArithmeticException` when the mathematical result is not a value of the class", "Checked by tests/behavior/Rules.cleat, lines `plus raises` and `negate raises`."),
    stated("06", "also raise when the quotient is not a value of the class", "Checked by tests/behavior/Rules.cleat, lines `floorDiv raises` and `truncatingDiv raises`."),
    stated("06", "They do not raise", "Two sentences, of the wrapping methods and of float arithmetic. Checked by tests/behavior/Rules.cleat, lines `wrapping` and `floats`."),
    rule("06", "`>>>` is not a spelling", &[("class A { Int f(Int a) { return a >>> 1; } }", "`>>>` is not an operator")], &["class A { Int f(Int a) { return a.shiftRight(1) + (a >> 1); } }"]),
    rule(
        "06",
        "A conversion that could lose information is never implicit",
        &[("class A { Int32 f(Int n) { return n; } }", "`Int` is not assignable to `Int32`"), ("class A { Float64 f(Int n) { return n; } }", "`Int` is not assignable to `Float64`")],
        &["class A { Int32 f(Int n) { return Int32.from(n); } Float64 g(Int n) { return Float64.nearest(n); } }"],
    ),
    stated("06", "its plain form raises an exception when the value does not fit", "Checked by tests/behavior/Rules.cleat, line `from raises`."),
    stated("06", "and raises `ArithmeticException` when `C` has no such value", "Checked by tests/behavior/Rules.cleat, line `from raises`."),
    stated("06", "A NaN or an infinity has no value in an integer class or in `Rational`", "Checked by tests/behavior/Rules.cleat, lines `from NaN raises` and `from infinity raises`."),
    rule("06", "A cast is a class test and never converts", &[("class A { Float64 f(Int n) { return (Float64) n; } }", "a cast from `Int` to `Float64` is rejected")], &["class A { Float64 f(Object o) { return (Float64) o; } }"]),
    stated("06", "raises `IllegalArgumentException` when the bytes are not well-formed UTF-8", "Checked by tests/behavior/Rules.cleat, line `fromUtf8 raises`."),
    rule("07", "Type arguments are not erased", &[], &["class A { Boolean f(Object o) { return o instanceof List<Int>; } List<String> g(Object o) { return (List<String>) o; } }"]),
    rule(
        "07",
        "`T extends Object` admits exactly the types that do not contain `null`",
        &[("class Box<T extends Object> { } class A { void f(Box<@Nullable String> b) { } }", "`@Nullable String` is not within the bound `Object`")],
        &["class Box<T extends Object> { } class A { void f(Box<String> b, Box<Int> c) { } }"],
    ),
    rule(
        "07",
        "accepts only a type argument whose declaration carries `Frozen`",
        &[("@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class C { void f(Snapshot<String> s) { } }", "`String` does not carry the tag `@Frozen`")],
        &["@Target(Site.TYPE) annotation Frozen; @Frozen class Bill { } class Snapshot<@Frozen T> { } class Pair<@Frozen U> { void f(Snapshot<Bill> s, Snapshot<U> t) { } }"],
    ),
    rule("07", "The type parameters of a method have no variance", &[("class A { <out T> void f() { } }", "the type parameters of a method have no variance")], &["class A { <T> void f(T x) { } }"]),
    rule("07", "`out T` promises that the type only produces values of `T`", &[("interface S<out T> { void put(T x); }", "`T` is declared `out` and is written where a value is consumed")], &["interface S<out T> { T take(); }"]),
    rule("07", "`in T` promises that the type only consumes values of `T`", &[("interface S<in T> { T take(); }", "`T` is declared `in` and is written where a value is produced")], &["interface S<in T> { void put(T x); }"]),
    rule(
        "07",
        "so only an invariant `T` may be written there",
        &[("interface S<out T> { List<T> all(); }", "is written where a value is both produced and consumed"), ("interface S<in T> { void all(List<T> xs); }", "is written where a value is both produced and consumed")],
        &["interface S<T> { List<T> all(); void put(List<T> xs); }"],
    ),
    rule("07", "The parameters of a constructor are not restricted", &[], &["class B<out T> { final T item; public B(T item, List<T> others) { this.item = item; } }"]),
    rule("07", "A wildcard stands for a type that the program does not name", &[], &["class A { Int f(List<?> xs, Map<?, ? extends Throwable> m) { return xs.size(); } }"]),
    rule(
        "07",
        "A wildcard stands only for a type that could be written as the argument",
        &[],
        &["class Box<T extends Throwable> { T item; public Box(T item) { this.item = item; } public T get() { return item; } } class A { Throwable f(Box<?> b) { return b.get(); } }"],
    ),
    rule(
        "07",
        "A type contains only itself",
        &[("open class Shape { } class Circle extends Shape { } class A { List<Shape> f(List<Circle> xs) { return xs; } }", "`List<Circle>` is not assignable to `List<Shape>`")],
        &["open class Shape { } class Circle extends Shape { } class A { List<Circle> f(List<Circle> xs) { return xs; } }"],
    ),
    rule(
        "07",
        "The program has no name for `X`",
        &[],
        &["class A { @Nullable Object f(List<?> xs) { var x = xs.get(0); return x; } Throwable g(List<? extends Throwable> xs) { var x = xs.get(0); return x; } }"],
    ),
    rule(
        "07",
        "Two arguments never supply the same unknown",
        &[("class A { static <T> void copy(List<T> from, List<T> to) { } void f(List<?> a) { copy(a, a); } }", "the type arguments of `copy` cannot be inferred from this call")],
        &["class A { static <T> Int count(List<T> xs) { return xs.size(); } Int f(List<?> a) { return count(a); } }"],
    ),
    rule(
        "07",
        "It is also not applicable when the type determined for `P` is not a supertype of one of its lower bounds",
        &[("class A { static <T extends Throwable> void g(T x) { } static void f(String s) { g(s); } }", "the type arguments of `g` cannot be inferred from this call")],
        &["class A { static <T extends Throwable> void g(T x) { } static void f(IllegalStateException s) { g(s); } }"],
    ),
    stated("07", "Inference never produces a type that the program could not have written", "It describes every result of inference; the examples of section 7.5 are held by tests/spec_examples.rs."),
    rule("08", "An instance comes only from a use of the annotation", &[("annotation M(Int n); class A { M f() { return new M(1); } }", "`new M` is rejected: an annotation value comes only from a use of the annotation")], &[]),
    rule(
        "08",
        "A value obtains a refinement only by the narrowing of",
        &[("@Refines annotation P; class A { @P Int f(Int n) { return n; } }", "`Int` is not assignable to `@P Int`")],
        &["@Refines annotation P; class A { @P Int f(@P Int n) { @P Int m = n; return m; } }"],
    ),
    rule("08", "A value does not carry its own qualifiers at run time", &[("@Refines annotation P; class C { Boolean f(Object o) { return o instanceof @P Int; } }", "a value does not carry its qualifiers at run time")], &[]),
    rule(
        "08",
        "and `unitPrice` may be sent only to an `Order` known to have it",
        &[("@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } } class C { Int f(Order o) { return o.unitPrice(); } }", "`unitPrice` is sent to a `Order`")],
        &["@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } } class C { Int f(@Priced Order o) { return o.unitPrice(); } }"],
    ),
    rule(
        "08",
        "A method whose receiver type has a refinement can be sent only to a receiver known to have it",
        &[("@Refines annotation Priced; class Order { public Int total(@Priced Order this) { return 1; } public Int twice() { return total() + total(); } }", "`total` is sent to a `Order`")],
        &["@Refines annotation Priced; class Order { public Int total(@Priced Order this) { return 1; } public Int twice(@Priced Order this) { return total() + this.total(); } }"],
    ),
    rule(
        "08",
        "A receiver whose type has a widening accepts only the methods that declare that widening on their receiver",
        &[("@Widens annotation Raw; class Order { public Int id() { return 1; } } class C { Int f(@Raw Order o) { return o.id(); } }", "`id` is sent to a `@Raw Order`")],
        &["@Widens annotation Raw; class Order { public Int id(@Raw Order this) { return 1; } } class C { Int f(@Raw Order o) { return o.id(); } }"],
    ),
    rule(
        "08",
        "whose receiver has no qualifier, as",
        &[("open class A { public open Int f() { return 1; } } class B extends A { @Override public Int f(@Nullable B this) { return 2; } }", "an override has the same receiver qualifiers as the method it overrides")],
        &["class A { @Override public String toString() { return \"a\"; } @Override public Int hashCode() { return 1; } @Override public Boolean equals(@Nullable Object o) { return true; } }"],
    ),
    rule(
        "08",
        "passes such a call with the local as its subject",
        &[(
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } } class A { static Int need(@Positive Int n) { return n; } static Int f(Int n, Int m) { if (Counts.isPositive(n)) { n = m; return need(n); } return 0; } }",
            "`Int` is not assignable to `@Positive Int`",
        )],
        &["@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } } class A { static Int need(@Positive Int n) { return n; } static Int f(Int n) { if (Counts.isPositive(n)) { return need(n); } return 0; } }"],
    ),
    rule(
        "08",
        "A field, an array element, and any other expression that is evaluated again are not narrowed",
        &[(
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } } class A { Int n = 1; static Int need(@Positive Int n) { return n; } Int f() { if (Counts.isPositive(n)) { return need(n); } return 0; } }",
            "`Int` is not assignable to `@Positive Int`",
        )],
        &["@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } } class A { Int n = 1; static Int need(@Positive Int n) { return n; } Int f() { var held = n; if (Counts.isPositive(held)) { return need(held); } return 0; } }"],
    ),
    rule(
        "08",
        "Code outside it can gain a refinement, or shed a widening, only through those methods",
        &[("@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } static Int f(Int n) { if (n > 0) { return need(n); } return 0; } }", "`Int` is not assignable to `@Positive Int`")],
        &[],
    ),
    stated("08", "The language does not check this", "It says what is left unchecked."),
    rule(
        "08",
        "Without `@Inherited`, a subclass carries the tag only when it is written there",
        &[("@Target(Site.TYPE) annotation Frozen; @Frozen open class Base { } class Sub extends Base { } class Snapshot<@Frozen T> { } class C { void f(Snapshot<Sub> s) { } }", "`Sub` does not carry the tag `@Frozen`")],
        &["@Inherited @Target(Site.TYPE) annotation Frozen; @Frozen open class Base { } class Sub extends Base { } class Snapshot<@Frozen T> { } class C { void f(Snapshot<Sub> s) { } }"],
    ),
    stated("08", "promises that the conversion is exact", "It is a promise by the author of the method."),
    stated("08", "The language does not check the promise", "It says what is left unchecked."),
    stated("08", "it raises an exception when the value has no exact counterpart", "It describes a convention; tests/behavior/Rules.cleat, line `from raises`, shows the numeric classes follow it."),
    stated("08", "A member that carries no annotation has no mirror", "Checked by tests/behavior/Rules.cleat, line `mirrors`."),
    rule(
        "08",
        "A program reaches a member this way only through an annotation",
        &[("class A { void f(Class c) { var m = c.getFields(); } }", "`Class` has no method named `getFields`")],
        &["annotation Mark; class A { Field[] f(Class c) { return c.<Mark>getAnnotatedFields(); } }"],
    ),
    rule(
        "08",
        "They do not describe the type of a field or a parameter",
        &[("class A { void f(Field x) { var t = x.getType(); } }", "`Field` has no method named `getType`"), ("class A { Object f(Class c) { return c.newInstance(); } }", "`Class` has no method named `newInstance`")],
        &[],
    ),
    stated("09", "evaluate only the operands that", "Checked by tests/behavior/Rules.cleat, lines `and`, `or` and `choice`."),
    rule("09", "A method does not declare what it raises, and `throws` is not a keyword", &[("class A { void f() throws Throwable { } }", "expected `{`, found `throws`")], &["class A { Int throws = 1; void f() { throw new IllegalStateException(); } }"]),
    stated("09", "A numeric operation has no result in its class, or an array length is negative", "A row of the table of exceptions, which tests/spec_tables.rs holds to the prelude."),
    stated("09", "An object is used in a state that does not allow the operation", "A row of the table of exceptions, which tests/spec_tables.rs holds to the prelude."),
    rule("09", "It has no result", &[("class A { public A() { return 1; } }", "`return` with a value is rejected in a constructor")], &["class A { public A() { } }"]),
    stated("09", "Field initializers run first, in source order, on entry to a constructor that does not call `this(...)`", "Checked by tests/behavior/Rules.cleat, the lines under `constructor`."),
    rule("09", "The compact form has no parameter list", &[("value class V { public Int a; public V(Int a) { } }", "a value class has one constructor, whose parameters are its fields")], &["value class V { public Int a; public V { } }"]),
    rule(
        "09",
        "It does not assign a field",
        &[("value class V { public Int a; public V { a = 1; } }", "`a` is not assignable here")],
        &["value class V { public Int a; public V { if (a < 0) { throw new IllegalArgumentException(\"negative\"); } } }"],
    ),
    stated("09", "A class literal does not initialize the class", "Checked by tests/behavior/Rules.cleat, line `class literal`."),
    stated("09", "The interfaces a class implements are not initialized with it", "Checked by tests/behavior/Rules.cleat, lines `class` and `interface`."),
    stated("09", "That can happen only during initialization", "It explains the sentence before it; tests/behavior/Statics.cleat reads a static field before it is assigned."),
    stated("09", "No class is added to a program while it runs", "It says what the language leaves out; no program can ask for it."),
    rule("09", "It is never `null`, and no element is `null`", &[], &["==== M.cleat
public class M { public static void main(String[] args) { var n = 0; for (String a : args) { n += a.length(); } } }"]),
    stated("09", "Threads that are still running do not keep the program alive", "Checked by tests/behavior/Rules.cleat, whose last thread would print after `main returns`."),
    rule("10", "A method does not declare what it raises", &[("class A { void f() throws Throwable { } }", "expected `{`, found `throws`")], &[]),
    rule(
        "10",
        "No value is rounded, truncated, or converted to a `String` unless the program calls a method that does it",
        &[
            ("class A { Int f(Float64 x) { return x; } }", "`Float64` is not assignable to `Int`"),
            ("class A { Float32 f(Float64 x) { return x; } }", "`Float64` is not assignable to `Float32`"),
            ("class A { String f(Int n) { return n; } }", "`Int` is not assignable to `String`"),
        ],
        &["class A { Int f(Float64 x) { return Int.from(x.truncate()); } String g(Int n) { return n.toString(); } }"],
    ),
    rule("10", "A program does not list the members of a class, call a method by name, or construct an object from a class object", &[("class A { void f(Class c) { var m = c.getMethods(); } }", "`Class` has no method named `getMethods`")], &[]),
    stated("10", "It agrees with `equals` and does not change while the program runs", "It binds what an implementation chooses."),
    rule(
        "11",
        "An identifier is not a keyword, a reserved word, or one of the three word literals",
        &[("class A { Int class = 1; }", "`class` is a keyword and cannot be a name"), ("class A { Int int = 1; }", "`int` is a reserved word"), ("class A { Int true = 1; }", "`true` is a keyword and cannot be a name")],
        &["class A { Int klass = 1; Int truth = 2; }"],
    ),
    rule("11", "begins a cast only when the token after `)` is not `-`, `++` or `--`", &[], &["class A { Int f(Int a, Int b) { return (a) - b; } Int g(Object o) { return (Int) o; } }"]),
    rule(
        "11",
        "are never mistaken for one",
        &[],
        &["class A { Boolean f(Object o, Int a, Int b) { var xs = new List<Int>(); var ys = Array.<Int>build(2, (i) -> i); return o instanceof List<Int> && a < b; } }"],
    ),
    stated("12", "Reading one does not initialize its class", "Checked by tests/behavior/Rules.cleat, line `constant`."),
    rule(
        "12",
        "`return`, `throw`, `break`, `continue` | Never",
        &[
            ("class A { Int f() { return 1; return 2; } }", UNREACHABLE),
            ("class A { void f() { throw new IllegalStateException(); f(); } }", UNREACHABLE),
            ("class A { void f() { while (true) { break; f(); } } }", UNREACHABLE),
            ("class A { void f() { while (true) { continue; f(); } } }", UNREACHABLE),
        ],
        &[],
    ),
    rule("12", "A constructor that leaves `super()` implicit has no `return`", &[("class A { public A() { return; } }", "in a constructor, `return;` is legal only after")], &["class A { public A() { super(); return; } }"]),
    rule(
        "12",
        "After the whole expression, only what `a` assigns is certain",
        &[("class A { Int f(Boolean p) { Int n; var r = p && (n = 1) > 0; return n; } }", UNASSIGNED), ("class A { Int f(Boolean p) { Int n; if (p && (n = 1) > 0) { return n; } return 0; } }", UNASSIGNED)],
        &["class A { Int f(Boolean p) { Int n; var r = (n = 1) > 0 && p; return n; } Int g(Boolean p) { Int n; var r = (n = 1) > 0 || p; return n; } }"],
    ),
    rule("12", "also requires that the local is never assigned after it is initialized", &[("class A { Function0<Int> f() { var n = 1; n = 2; return () -> n; } }", "a lambda may use `n` only if it is never assigned after it is initialized")], &[]),
    rule(
        "12",
        "When `false` it has only the facts that both have when `false`",
        &[("class A { Int f(@Nullable String s, Boolean b) { if (s != null && b) { return 0; } return s.length(); } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s, Boolean b) { if (s != null && b) { return s.length(); } return 0; } }"],
    ),
    rule(
        "12",
        "When `true` it has only the facts that both have when `true`",
        &[("class A { Int f(@Nullable String s, Boolean b) { if (s != null || b) { return s.length(); } return 0; } }", "`length` is sent to a `@Nullable String`")],
        &["class A { Int f(@Nullable String s, Boolean b) { if (s == null || b) { return 0; } return s.length(); } }"],
    ),
    stated("13", "A thread that is still running does not keep it alive", "Checked by tests/behavior/Rules.cleat, whose last thread would print after `main returns`."),
    stated("13", "It never returns part of one value and part of another", "It holds for every interleaving, which no run can enumerate. The compiler keeps a value-class instance behind one pointer, so a read or an assignment moves one word."),
    stated("13", "by a thread that does not hold the lock raises `IllegalStateException`", "Checked by tests/behavior/Threads.cleat, line `not held`."),
    stated("13", "Only the thread that is running the scope's `call` may fork", "Checked by tests/behavior/Rules.cleat, lines `fork from another thread raises` and `fork after the call raises`."),
    stated("13", "A `CancellationException` from a task is never reported by `call`", "Checked by tests/behavior/Rules.cleat, line `call reports`: the task forked first is cancelled, and `call` reports the second."),
    stated("13", "is never cancelled, and neither is the thread that runs `main`", "Checked by tests/behavior/Rules.cleat, lines `main is never cancelled` and `a started thread is never cancelled`."),
    stated("13", "No exception arrives between two statements that do not ask for it", "It holds for every interleaving, which no run can enumerate. `cancellation_is_raised_only_where_section_13_6_says` checks that one method raises it and only the listed methods call that one."),
    stated("13", "On a thread that is not a cancelled task, it does nothing", "Checked by tests/behavior/Rules.cleat, line `main is never cancelled`."),
];

// ---- the sentences ----

/// A chapter without its fenced blocks.
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

/// The sentences of a line of prose: a sentence ends at `.`, `:` or `;` where the next
/// word begins a new one.
fn sentences(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < chars.len() {
        if matches!(chars[i], '.' | ':' | ';') && i + 1 < chars.len() && chars[i + 1].is_whitespace() {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j].is_ascii_uppercase() || matches!(chars[j], '`' | '*' | '[' | '(')) {
                out.push(chars[start..=i].iter().collect::<String>().trim().to_string());
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    let last: String = chars[start..].iter().collect::<String>().trim().to_string();
    if !last.is_empty() {
        out.push(last);
    }
    out
}

fn states_a_requirement(s: &str) -> bool {
    let has_word = |w: &str| s.split(|c: char| !c.is_alphabetic()).any(|x| x.eq_ignore_ascii_case(w));
    s.contains("reject") || s.contains("is not written") || s.contains("does not hide") || s.contains("may not") || has_word("must") || has_word("cannot") || has_word("legal")
}

/// A sentence of the specification's prose.
struct Sentence {
    chapter: String,
    text: String,
    /// The header row of the table the sentence is a row of.
    table: Option<String>,
}

/// Every sentence of the specification's prose. The header row of a table names its
/// columns and is not a sentence.
fn prose_sentences() -> Vec<Sentence> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("spec");
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().and_then(|e| e.to_str()) == Some("md")).collect();
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let chapter = path.file_name().unwrap().to_string_lossy()[..2].to_string();
        let text = prose(&std::fs::read_to_string(&path).unwrap().replace("\r\n", "\n"));
        let lines: Vec<&str> = text.lines().map(|l| l.trim()).collect();
        let mut table: Option<String> = None;
        for (i, line) in lines.iter().enumerate() {
            if !line.starts_with('|') {
                table = None;
            }
            if line.is_empty() || line.starts_with('#') || line.starts_with("| ---") {
                continue;
            }
            if line.starts_with('|') && lines.get(i + 1).is_some_and(|next| next.starts_with("| ---")) {
                table = Some(line.to_string());
                continue;
            }
            let line = line.strip_prefix("- ").unwrap_or(line);
            // The number of a list item is not part of its sentence.
            let digits = line.chars().take_while(|c| c.is_ascii_digit()).count();
            let line = if digits > 0 && line[digits..].starts_with(". ") { &line[digits + 2..] } else { line };
            for s in sentences(line) {
                out.push(Sentence { chapter: chapter.clone(), text: s, table: table.clone() });
            }
        }
    }
    out
}

/// Every sentence of the specification's prose, with its chapter.
fn all_sentences() -> Vec<(String, String)> {
    prose_sentences().into_iter().map(|s| (s.chapter, s.text)).collect()
}

/// A sentence that forbids or limits without the words of a requirement.
fn states_a_restriction(s: &str) -> bool {
    // Code is not prose: `only` is a keyword of the language.
    let mut plain = String::new();
    let mut code = false;
    for c in s.chars() {
        if c == '`' {
            code = !code;
            if code {
                plain.push_str(" CODE ");
            }
        } else if !code {
            plain.push(c);
        }
    }
    let words: Vec<&str> = plain.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()).collect();
    let low: Vec<String> = words.iter().map(|w| w.to_lowercase()).collect();
    let seq = |ws: &[&str]| low.windows(ws.len()).any(|x| x.iter().zip(ws).all(|(a, b)| a == b));
    let no_x_may = (0..words.len().saturating_sub(2)).any(|i| low[i] == "no" && words[i + 1].chars().all(|c| c.is_lowercase()) && matches!(low[i + 2].as_str(), "may" | "is" | "are" | "extends" | "converts"));
    low.iter().any(|w| w == "only" || w == "never") || seq(&["does", "not"]) || seq(&["do", "not"]) || seq(&["is", "not", "a"]) || seq(&["are", "not"]) || seq(&["has", "no"]) || seq(&["have", "no"]) || no_x_may
}

/// The sentences that state a requirement.
fn requirements() -> Vec<(String, String)> {
    all_sentences().into_iter().filter(|(_, s)| states_a_requirement(s)).collect()
}

#[test]
fn every_requirement_the_specification_states_has_a_case() {
    let all = requirements();
    let mut missing = Vec::new();
    for (chapter, sentence) in &all {
        if !RULES.iter().any(|r| r.chapter == chapter && sentence.contains(r.phrase)) {
            missing.push(format!("chapter {chapter}: {sentence}"));
        }
    }
    assert!(missing.is_empty(), "{} sentences state a requirement and have no case:\n{}", missing.len(), missing.join("\n"));
    let mut stale = Vec::new();
    for r in RULES {
        let hits = all.iter().filter(|(c, s)| c == r.chapter && s.contains(r.phrase)).count();
        if hits != 1 {
            stale.push(format!("chapter {}: `{}` is in {hits} sentences", r.chapter, r.phrase));
        }
    }
    assert!(stale.is_empty(), "a case must name exactly one sentence:\n{}", stale.join("\n"));
    assert_eq!(all.len(), RULES.len());
    for r in RULES {
        let has_programs = !r.reject.is_empty() || !r.accept.is_empty();
        assert!(has_programs == r.note.is_empty(), "`{}` needs programs or a reason, and not both", r.phrase);
    }
    let with_programs = RULES.iter().filter(|r| r.note.is_empty()).count();
    println!("{} sentences state a requirement: {} have programs, {} cannot be broken by a program", all.len(), with_programs, all.len() - with_programs);
    for r in RULES.iter().filter(|r| !r.note.is_empty()) {
        println!("  {} \"{}\": {}", r.chapter, r.phrase, r.note);
    }
}

#[test]
fn every_restriction_the_specification_states_has_a_case() {
    let all: Vec<(String, String)> = all_sentences().into_iter().filter(|(_, s)| !states_a_requirement(s) && states_a_restriction(s)).collect();
    let mut missing = Vec::new();
    for (chapter, sentence) in &all {
        if !FURTHER.iter().any(|r| r.chapter == chapter && sentence.contains(r.phrase)) {
            missing.push(format!("{chapter}: {sentence}"));
        }
    }
    assert!(missing.is_empty(), "{} sentences restrict and have no case:\n{}", missing.len(), missing.join("\n"));
    // A case names one sentence of all the prose. Two sentences of chapter 6 have the
    // same words, and one case stands for both.
    let prose = all_sentences();
    let mut wrong = Vec::new();
    for r in FURTHER {
        let mut hits: Vec<&String> = prose.iter().filter(|(c, s)| c == r.chapter && s.contains(r.phrase)).map(|(_, s)| s).collect();
        hits.dedup();
        if hits.len() != 1 {
            wrong.push(format!("chapter {}: `{}` is in {} sentences", r.chapter, r.phrase, hits.len()));
        }
        if RULES.iter().any(|x| x.chapter == r.chapter && x.phrase == r.phrase) {
            wrong.push(format!("chapter {}: `{}` is in both tables", r.chapter, r.phrase));
        }
        let has_programs = !r.reject.is_empty() || !r.accept.is_empty();
        assert!(has_programs == r.note.is_empty(), "`{}` needs programs or a reason, and not both", r.phrase);
    }
    assert!(wrong.is_empty(), "a case must name exactly one sentence:\n{}", wrong.join("\n"));
    let with_programs = FURTHER.iter().filter(|r| r.note.is_empty()).count();
    println!("{} sentences restrict in other words; the table has {} cases: {} have programs, {} cannot be broken by a program", all.len(), FURTHER.len(), with_programs, FURTHER.len() - with_programs);
    for r in FURTHER.iter().filter(|r| !r.note.is_empty()) {
        println!("  {} \"{}\": {}", r.chapter, r.phrase, r.note);
    }
}

/// Whether a test of this name is written in this file under `tests`.
fn test_exists(file: &str, test: &str) -> bool {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join(format!("{file}.rs"));
    std::fs::read_to_string(path).is_ok_and(|text| text.contains(&format!("fn {test}()")) || text.contains(&format!("({test}, ")) || text.contains(&format!("    {test} => ")))
}

/// Every sentence of a finished chapter has a case: in one of the three tables, or as
/// a row of a table that one test holds whole.
#[test]
fn every_sentence_of_the_specification_has_a_case() {
    let cases = described();
    let all = prose_sentences();
    let mut missing = Vec::new();
    let mut by_table = 0;
    let mut others = 0;
    for s in all.iter().filter(|s| DESCRIBED.contains(&s.chapter.as_str())) {
        let named = RULES.iter().chain(FURTHER).chain(cases.iter().copied()).any(|r| r.chapter == s.chapter && s.text.contains(r.phrase));
        let in_table = s.table.as_ref().is_some_and(|header| TABLES.iter().any(|t| t.0 == s.chapter && header.contains(t.1)));
        if !states_a_requirement(&s.text) && !states_a_restriction(&s.text) {
            others += 1;
            if in_table && !named {
                by_table += 1;
            }
        }
        if !named && !in_table {
            missing.push(format!("{}: {}", s.chapter, s.text));
        }
    }
    assert!(missing.is_empty(), "{} sentences have no case:\n{}", missing.len(), missing.join("\n"));
    let mut wrong = Vec::new();
    for r in &cases {
        let mut hits: Vec<&String> = all.iter().filter(|s| s.chapter == r.chapter && s.text.contains(r.phrase)).map(|s| &s.text).collect();
        hits.dedup();
        if hits.len() != 1 {
            wrong.push(format!("chapter {}: `{}` is in {} sentences", r.chapter, r.phrase, hits.len()));
        } else if states_a_requirement(hits[0]) || states_a_restriction(hits[0]) {
            wrong.push(format!("chapter {}: `{}` names a sentence of another table", r.chapter, r.phrase));
        }
        if r.ways() != 1 {
            wrong.push(format!("chapter {}: `{}` needs programs, a run test, another test or a reason, and one of them", r.chapter, r.phrase));
        }
        if !r.held.0.is_empty() && !test_exists(r.held.0, r.held.1) {
            wrong.push(format!("chapter {}: `{}` names the test `{}` of tests/{}.rs, and there is none", r.chapter, r.phrase, r.held.1, r.held.0));
        }
    }
    for t in TABLES {
        if !test_exists(t.2, t.3) {
            wrong.push(format!("the table `{}` names the test `{}` of tests/{}.rs, and there is none", t.1, t.3, t.2));
        }
        if !all.iter().any(|s| s.chapter == t.0 && s.table.as_ref().is_some_and(|h| h.contains(t.1))) {
            wrong.push(format!("chapter {} has no table `{}`", t.0, t.1));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    let count = |f: &dyn Fn(&Rule) -> bool| cases.iter().filter(|r| f(r)).count();
    println!(
        "chapters {}: {} sentences define or describe. {} have programs, {} name a line of a run test, {} name another test, {} are rows of a table one test holds, {} have a reason",
        DESCRIBED.join(", "),
        others,
        count(&|r| r.has_programs()),
        count(&|r| !r.ran.0.is_empty()),
        count(&|r| !r.held.0.is_empty()),
        by_table,
        count(&|r| !r.note.is_empty())
    );
    for r in cases.iter().filter(|r| !r.note.is_empty()) {
        println!("  {} \"{}\": {}", r.chapter, r.phrase, r.note);
    }
}

/// A case that names a line of a run test names a line that program prints, and
/// `tests/behavior.rs` runs the program.
#[test]
fn every_line_a_case_names_is_printed_by_its_run_test() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let driver = std::fs::read_to_string(dir.join("behavior.rs")).unwrap();
    let mut wrong = Vec::new();
    let mut lines = 0;
    for r in RULES.iter().chain(FURTHER).chain(described()).filter(|r| !r.ran.0.is_empty()) {
        lines += 1;
        let (program, line) = r.ran;
        if !driver.contains(&format!("\"{program}\"")) {
            wrong.push(format!("tests/behavior.rs does not run `{program}`"));
        }
        let out = std::fs::read_to_string(dir.join("behavior").join(format!("{program}.out"))).unwrap_or_default();
        if !out.lines().any(|l| l.trim_start().starts_with(line)) {
            wrong.push(format!("{} \"{}\": tests/behavior/{program}.out has no line that begins `{line}`", r.chapter, r.phrase));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    println!("{lines} cases name a line of a run test, and each line is printed");
}

// ---- the programs ----

/// A case is one file, `T.cleat`, or several, each after a line `==== name`.
fn files(case: &str) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut name = "T.cleat".to_string();
    let mut text = String::new();
    for line in case.lines() {
        if let Some(n) = line.strip_prefix("==== ") {
            if !text.trim().is_empty() {
                out.push((PathBuf::from(&name), std::mem::take(&mut text)));
            }
            name = n.trim().to_string();
            text.clear();
        } else {
            text.push_str(line);
            text.push('\n');
        }
    }
    if !text.trim().is_empty() {
        out.push((PathBuf::from(&name), text));
    }
    out
}

fn diagnostics(case: &str) -> Vec<String> {
    match analyze_sources(&files(case)) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| format!("{}:{}:{}: {}", d.file.display(), d.line, d.column, d.message)).collect(),
    }
}

fn run(chapter: &str) {
    let mut failures = Vec::new();
    let mut rejected = 0;
    let mut accepted = 0;
    let described = described();
    let cases: Vec<&Rule> = RULES.iter().chain(FURTHER).chain(described.iter().copied()).filter(|r| r.chapter == chapter && r.has_programs()).collect();
    for r in &cases {
        let before = failures.len();
        for (case, words) in r.reject {
            rejected += 1;
            let d = diagnostics(case);
            if d.is_empty() {
                failures.push(format!("\"{}\"\n  must be rejected, and the checker accepts:\n  {case}", r.phrase));
            } else if !d.iter().any(|m| m.contains(words)) {
                failures.push(format!("\"{}\"\n  is rejected, and not for this rule:\n  {case}\n  expected: {words}\n  the checker says:\n  {}", r.phrase, d.join("\n  ")));
            }
        }
        for case in r.accept {
            accepted += 1;
            let d = diagnostics(case);
            if !d.is_empty() {
                failures.push(format!("\"{}\"\n  must be accepted:\n  {case}\n  and the checker says:\n  {}", r.phrase, d.join("\n  ")));
            }
        }
        let verdict = if failures.len() == before { "ok  " } else { "FAIL" };
        println!("{chapter} {verdict} {} rejected, {} accepted: {}", r.reject.len(), r.accept.len(), r.phrase);
    }
    println!("chapter {chapter}: {} cases with programs, {rejected} programs rejected each for its rule, {accepted} programs accepted", cases.len());
    assert!(failures.is_empty(), "{} programs of chapter {chapter} went wrong:\n\n{}", failures.len(), failures.join("\n\n"));
}

macro_rules! chapters {
    ($($name:ident => $chapter:literal),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                run($chapter);
            }
        )*
    };
}

chapters![
    chapter_01_source => "01",
    chapter_02_objects => "02",
    chapter_03_visibility => "03",
    chapter_04_methods => "04",
    chapter_05_null_and_unit => "05",
    chapter_06_numbers => "06",
    chapter_07_generics => "07",
    chapter_08_annotations => "08",
    chapter_09_execution => "09",
    chapter_10_omissions => "10",
    chapter_11_syntax => "11",
    chapter_12_flow => "12",
    chapter_13_concurrency => "13",
];

#[test]
fn a_file_that_is_not_utf8_is_rejected() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("not_utf8");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("Bad.cleat");
    std::fs::write(&file, [b'c', b'l', b'a', b's', b's', b' ', 0xff, 0xfe, b' ', b'{', b'}']).unwrap();
    let err = cleatc::analyze(&[file.as_path()]).err().expect("bytes that are not UTF-8");
    assert!(err.iter().any(|d| d.message.contains("UTF-8")), "{}", err[0].message);
    std::fs::write(&file, "class Bad { }").unwrap();
    assert!(cleatc::analyze(&[file.as_path()]).is_ok());
}

#[test]
fn an_entry_class_declares_exactly_one_main() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("entry_rule");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (name, text) in [
        ("Both", "public class Both { public static void main() { } public static void main(String[] args) { } }"),
        ("Neither", "public class Neither { public static void start() { } }"),
    ] {
        let file = dir.join(format!("{name}.cleat"));
        std::fs::write(&file, text).unwrap();
        let err = cleatc::build_roots(&[file.as_path()], name, &dir.join(format!("{name}.exe"))).expect_err(name);
        assert!(err.iter().any(|d| d.message.contains("not an entry class")), "{name}: {}", err[0].message);
    }
}

#[test]
fn a_deprecated_use_is_reported_from_another_unit() {
    let old = "==== Old.cleat\npublic class Old { @Deprecated(\"use fresh\") public static Int stale() { return 1; } public static Int own() { return stale(); } }\n";
    let alone = analyze_sources(&files(old)).ok().expect("the declaring unit");
    assert!(alone.warnings.is_empty(), "a use inside the declaring unit is not reported: {}", alone.warnings[0].message);
    let used = format!("{old}==== User.cleat\npublic class User {{ public static Int f() {{ return Old.stale(); }} }}\n");
    let program = analyze_sources(&files(&used)).ok().expect("a deprecated use does not reject the program");
    assert_eq!(program.warnings.len(), 1);
    let w = &program.warnings[0];
    assert!(w.file.ends_with("User.cleat") && w.message.contains("deprecated") && w.message.contains("use fresh"), "{}", w.message);
}

/// Section 13.6: a cancelled task raises `CancellationException` at a call of
/// `Thread.checkCancelled`, `sleep`, `join`, `Task.result` or `Condition.await`, and at
/// no others. One method of the prelude raises it, and only those call that method.
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
        uses += std::fs::read_to_string(entry.unwrap().path()).unwrap().matches("X_CANCELLATION").count();
    }
    assert_eq!(uses, 1);
}

/// Section 2.2: the prelude is written in the language. The checker takes its source
/// as it takes a program's.
#[test]
fn the_prelude_is_source_the_checker_accepts() {
    let prelude = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("prelude");
    let files = std::fs::read_dir(&prelude).unwrap().filter(|e| e.as_ref().unwrap().path().extension().and_then(|x| x.to_str()) == Some("cleat")).count();
    assert!(files > 50, "{files} files");
    if let Err(errors) = cleatc::analyze(&[prelude.as_path()]) {
        panic!("{}:{}: {}", errors[0].file.display(), errors[0].line, errors[0].message);
    }
}
