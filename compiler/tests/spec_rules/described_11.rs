//! Chapter 11, Syntax: the sentences that define and describe.

use super::{held, ran, rule, stated, Rule, NOT_A_STATEMENT, NO_DEFAULT};

const RUN: &str = "Sentences11";

pub const SENTENCES: &[Rule] = &[
    held("11", "This chapter is the grammar", "spec_tables", "every_program_the_grammar_must_derive_is_parsed"),
    stated("11", "The other chapters add restrictions to what the grammar derives, and those restrictions stand", "It says how the chapters fit together; the restrictions have their own cases."),
    stated("11", "The notation uses `=` for a definition", "It explains the notation."),
    stated("11", "Words outside quotes name other rules", "It explains the notation."),
    // ---- 11.1 ----
    stated("11", "describes the tokens in prose", "It points to chapter 1, whose sentences have their own cases."),
    held("11", "This is their grammar", "spec_consistency", "the_grammar_uses_exactly_the_words_of_chapter_1"),
    rule("11", "A `letter` is a character in Unicode category L, and a `number` one in category N", &[], &["class A { Int \u{e9}\u{663} = 1; Int a1 = 2; Int \u{3b1}\u{3b2}2 = 3; }"]),
    rule(
        "11",
        "A `plain-character` is any scalar other than the closing quote, a backslash and a line break",
        &[("class A { String s = \"a\\\"; }", "this string literal is never closed")],
        &["class A { String s = \"a'b \u{1F600}\"; Char c = '\"'; }"],
    ),
    rule("11", "A block comment ends at the first `*/`", &[("/* a /* b */ c */ class A { }", "expected a class, an interface, an enum or an annotation, found `c`")], &["/* a /* b */ class A { /* ** / */ }"]),
    rule("11", "is read alone where it closes a type argument list", &[], &["class A { List<List<Int>> x = new List<List<Int>>(); Boolean f(Int a, Int b) { a >>= 1; return a >= b && a >> 1 > b; } }"]),
    // ---- 11.2 ----
    rule("11", "An annotation before the named type qualifies it", &[], &["class A { @Nullable String f(@Nullable String s) { return null; } }"]),
    rule(
        "11",
        "An annotation before a pair of brackets qualifies that array",
        &[("class A { String[] g() { return null; } }", "`null` is not a value of")],
        &["class A { String @Nullable [] g() { return null; } }"],
    ),
    rule(
        "11",
        "With several pairs, the first pair is the outermost array",
        &[("class A { Int[] @Nullable [] g() { return null; } }", "`null` is not a value of")],
        &["class A { void f(Int[] @Nullable [] a) { a[0] = null; } Int @Nullable [][] g() { return null; } }"],
    ),
    rule(
        "11",
        "is an array whose elements are arrays of `Int` or `null`",
        &[("class A { void f(Int[] @Nullable [] a) { a[0][0] = 1; } }", "`set` is sent to a `Int @Nullable []`, which may be `null`")],
        &["class A { void f(Int[] @Nullable [] a) { a[0] = null; a[1] = new Int[2]; var first = a[1]; if (first != null) { first[0] = 1; } } }"],
    ),
    // ---- 11.4 ----
    rule(
        "11",
        "and a modifier is written at most once",
        &[("class A { static static Int x = 1; }", "`static` is written twice"), ("public public class A { }", "a declaration has one audience"), ("class A { final final Int x = 1; }", "`final` is written twice")],
        &[],
    ),
    rule(
        "11",
        "A method ends in `;` in place of a block when it is abstract",
        &[("class A { Int f(); }", "the method `f` needs a body")],
        &["abstract class A { abstract Int f(); foreign static Int g(); } interface I { Int h(); }"],
    ),
    rule("11", "A constructor's identifier is the name of its class", &[("class A { public B() { } }", "expected a name, found `(`")], &["class A { public A() { } }"]),
    stated("11", "could also be read as the first annotation of the `type` that follows", "It names the two readings that the next two sentences settle."),
    rule("11", "For a qualifier the two readings mean the same", &[], &["class A { @Nullable public String f() { return null; } public @Nullable String g() { return null; } @Nullable String h = null; }"]),
    rule(
        "11",
        "A declaration annotation belongs to the declaration",
        &[("@Target(Site.TYPE) annotation M; class A { @M public Int f() { return 1; } }", "its `@Target` does not list this site")],
        &["@Target(Site.METHOD) annotation M; class A { @M public Int f() { return 1; } @M Int g() { return 2; } }"],
    ),
    // ---- 11.5 ----
    ran("11", "An `else` belongs to the nearest `if` that has none", RUN, "dangling else: 2 3"),
    rule("11", "A statement that can be read as a local declaration is one", &[], &["class A { void f() { List<Int> xs = new List<Int>(); Int[] a = new Int[1]; a[0] = 1; Map<String, List<Int>> m = new Map<String, List<Int>>(); } }"]),
    rule(
        "11",
        "An arm that can be read as `case type identifier` is a type arm",
        &[],
        &["class A { Int f(Object o) { switch (o) { case String s -> { return s.length(); } case List<Int> xs -> { return xs.size(); } default -> { return 0; } } } }"],
    ),
    // ---- 11.6 ----
    ran("11", "The rules run from the loosest binding to the tightest", RUN, "binding: 7 true"),
    ran("11", "Assignment, `?:` and `??` group to the right", RUN, "right: 5 5 2 c"),
    ran("11", "The other binary operators group to the left", RUN, "left: -5 1"),
    ran("11", "is a subtraction, and a negated operand is cast as", RUN, "cast or subtraction: 4 -5"),
    rule(
        "11",
        "begins a lambda when the token after its matching `)` is `->`",
        &[],
        &["class A { Int g(Int a, Int b) { Function1<Int, Int> f = (x) -> (x) + 1; Function2<Int, Int, Int> h = (x, y) -> (x) * (y); return (a) + (b) + f.invoke(a) + h.invoke(a, b); } }"],
    ),
    rule(
        "11",
        "begins type arguments when the tokens up to the matching `>` are type arguments",
        &[],
        &["class A { Class f() { return List<Int>.class; } Function1<List<Int>, Int> g() { return List<Int>::size; } Class h() { return Map<String, List<Int>>.class; } }"],
    ),
    rule("11", "Everywhere else it is the comparison operator", &[], &["class A { Boolean f(Int a, Int b, Int c) { return a < b && b > c; } Boolean g(Int List, Int Int2) { return List < Int2; } }"]),
    rule(
        "11",
        "is derived as an identifier and two selectors",
        &[],
        &["==== p/Q.cleat\npackage p; public class Q { public static Q one = new Q(); public Int n = 1; }\n==== A.cleat\nclass B { public Q2 b = new Q2(); } class Q2 { public Int c = 3; } class A { Int f(B a) { return a.b.c + p.Q.one.n; } }"],
    ),
    stated("11", "says which parts name a package, a type, a field or a variable", "It points to section 1.8, whose sentences have their own cases."),
    rule(
        "11",
        "is a variable, a field or an indexed element",
        &[("class A { Int g() { return 1; } void f() { g() = 1; } }", "the left side of an assignment is a variable, a field or an indexed element"), ("class A { void f() { 1++; } }", "`++` and `--` apply to a variable, a field or an indexed element"), ("class A { void f(Int a, Int b) { (a + b)++; } }", "`++` and `--` apply to a variable, a field or an indexed element")],
        &["class A { Int n = 0; void f(Int[] a, Int x) { x = 1; this.n = 2; n = 3; a[0] = 4; x++; a[0]--; n++; ++x; --this.n; x += 2; a[0] *= 2; } }"],
    ),
    rule("11", "An `expr` that stands as a statement is one that", &[("class A { void f(Int n) { n + 1; } }", NOT_A_STATEMENT), ("class A { void f(Int n) { n == 1; } }", NOT_A_STATEMENT)], &["class A { Int n = 0; void g() { } void f() { g(); n = 1; n++; } }"]),
    rule(
        "11",
        "directly before a numeric literal is part of the literal for the rules of",
        &[("class A { Int8 a = -(128); }", "the literal is outside the range of `Int8`"), ("class A { Int8 a = -129; }", "the literal is outside the range of `Int8`")],
        &["class A { Int8 a = -128; Int min = -9223372036854775808; Float64 x = -1.5; }"],
    ),
    rule("11", "and brackets after `[n]` belong to the element type", &[("class A { void f() { var a = new Int[2][]; } }", NO_DEFAULT)], &["class A { void f() { var a = new Int[2]; a[0] = 1; var b = new Int[2] @Nullable []; b[0] = null; b[1] = new Int[3]; Int[] @Nullable [] c = b; } }"]),
    ran("11", "lists the elements, and the type before the braces is an array type", RUN, "array creation: 3 2"),
    rule(
        "11",
        "is derived by the `selector` rule or by the",
        &[],
        &["class A { Function0<Int> f(String s) { return s::length; } Function1<String, Int> g() { return String::length; } Function0<Int> h() { return this::hashCode; } }"],
    ),
    ran("11", "It is a reference through a value when `x` names one, and through a type otherwise", RUN, "references: 3 5"),
];
