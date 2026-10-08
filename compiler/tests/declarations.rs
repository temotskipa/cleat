//! Declarations and source text the checker must reject (chapters 1, 2, 3, 8 and 9).
//! Each case is whole files, and names a phrase of the diagnostic it must draw.

use cleatc::analyze_sources;
use std::path::PathBuf;

fn diagnostics(files: &[(&str, &str)]) -> Vec<String> {
    let sources: Vec<(PathBuf, String)> = files.iter().map(|(n, t)| (PathBuf::from(n), t.to_string())).collect();
    match analyze_sources(&sources) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| format!("{}:{}:{}: {}", d.file.display(), d.line, d.column, d.message)).collect(),
    }
}

thread_local! {
    static FAILURES: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn fail(message: String) {
    FAILURES.with(|f| f.borrow_mut().push(message));
}

/// Reports every case of the test that went wrong, together.
fn done() {
    let all = FAILURES.with(|f| std::mem::take(&mut *f.borrow_mut()));
    assert!(all.is_empty(), "{} cases failed:

{}", all.len(), all.join("

"));
}

fn rejected(text: &str, fragment: &str) {
    rejected_files(&[("T.cleat", text)], fragment);
}

fn rejected_files(files: &[(&str, &str)], fragment: &str) {
    let d = diagnostics(files);
    if d.is_empty() {
        fail(format!("expected a diagnostic that mentions `{fragment}`, and the checker accepts:
{}", files.last().unwrap().1));
    } else if !d.iter().any(|m| m.contains(fragment)) {
        fail(format!("expected a diagnostic that mentions `{fragment}` for:
{}
and the checker says:
{}", files.last().unwrap().1, d.join("
")));
    }
}

fn accepted(text: &str) {
    let d = diagnostics(&[("T.cleat", text)]);
    if !d.is_empty() {
        fail(format!("expected no diagnostics for:
{text}
and the checker says:
{}", d.join("
")));
    }
    done();
}

#[test]
fn extension_follows_section_2_4() {
    rejected("class A { } class B extends A { }", "final");
    rejected("sealed class A { }", "permits");
    rejected("sealed class A permits B { } class B { }", "extend");
    rejected("sealed class A permits B { } class B extends A { } class C extends A { }", "does not permit");
    rejected("abstract final class A { }", "contradicts");
    rejected("open final class A { }", "contradicts");
    rejected("sealed open class A permits B { } class B extends A { }", "sealed open");
    rejected("class A extends Enum { }", "Enum");
    rejected("final interface I { }", "final interface");
    accepted("open class A { } class B extends A { } sealed class S permits T { } class T extends S { }");
    done();
}

#[test]
fn value_classes_and_enums_keep_their_shape() {
    rejected("open class A { } value class V extends A { }", "value class");
    rejected("value enum E { A }", "enum");
    rejected("enum E { A; public E() { } }", "private");
    rejected("enum E<T> { A }", "");
    rejected("value class V { public Int a = 1; }", "initializer");
    rejected("value class V { public Int a; public V(Int a) { } }", "value class");
    done();
}

#[test]
fn interfaces_follow_section_2_6() {
    rejected("interface I { Int count; }", "initializer");
    rejected("interface I { private void f(); }", "public");
    rejected("interface I { final void f(); }", "final");
    rejected("interface I { I() { } }", "constructor");
    rejected("interface I { static Int zero(); } class C implements I { }", "static method");
    rejected("interface I { static Int zero(); } class C implements I { public static Int zero() { return 0; } }", "@Override");
    rejected("interface A { Int f() { return 1; } } interface B { Int f() { return 2; } } class C implements A, B { }", "two interfaces");
    rejected("class C implements Ordered<Int>, Ordered<String> { }", "Ordered");
    accepted("interface I { Int LIMIT = 3; Int f(); Int g() { return f() + LIMIT; } } class C implements I { @Override public Int f() { return 1; } }");
    done();
}

#[test]
fn methods_and_members_follow_their_modifier_rules() {
    rejected("class A { abstract void f(); }", "abstract");
    rejected("abstract class A { abstract final void f(); }", "abstract final");
    rejected("class A { open static void f() { } }", "static");
    rejected("class A { void f(Int a) { } void f(Int b) { } }", "twice");
    rejected("class A { Int x; String x; }", "twice");
    rejected("class A { } class A { }", "twice");
    rejected("class A { void f(); }", "body");
    rejected("class A { static void f() { var t = this; } }", "static");
    rejected("class A { @Intrinsic void f(); }", "@Intrinsic");
    rejected("class A { foreign void f(); }", "static");
    accepted("class A { Int f(@Nullable A this) { return 1; } }");
    rejected("open class A { open Int f(@Nullable A this) { return 1; } }", "@Nullable");
    done();
}

#[test]
fn audiences_follow_chapter_3() {
    rejected("private class A { }", "a type is");
    rejected("protected class A { }", "a type is");
    rejected("class A { private only(B) Int x = 1; } class B { }", "only");
    rejected_files(&[("a/A.cleat", "package a; class Hidden { }"), ("b/B.cleat", "package b; import a.Hidden; class B { }")], "Hidden");
    rejected_files(&[("a/A.cleat", "package a; package class Mid { }"), ("b/B.cleat", "package b; class B { a.Mid m; }")], "Mid");
    rejected_files(&[("A.cleat", "public class Other { }")], "Other.cleat");
    rejected_files(&[("A.cleat", "public class A { } public class B { }")], "");
    rejected(
        "open class A { protected void f() { } } class B extends A { void g(A other) { other.f(); } }",
        "audience",
    );
    accepted("open class A { protected void f() { } } class B extends A { void g(B other) { other.f(); f(); } }");
    done();
}

#[test]
fn source_text_follows_chapter_1() {
    rejected("class A { void f() { int x = 1; } }", "int");
    rejected("class A { void f() { var when = 1; } }", "");
    rejected("class A { Int f() { return 07; } }", "");
    rejected("class A { Int f() { return 1L; } }", "");
    rejected("class A { Int f(Int a) { return a >>> 1; } }", ">>>");
    rejected("class A { Int f(Int a) { return +a; } }", "");
    rejected("class A { String f() { return \"\\q\"; } }", "");
    rejected("class A { class B { } }", "");
    rejected("import a.Missing; class A { }", "Missing");
    rejected_files(
        &[("a/X.cleat", "package a; public class X { }"), ("b/X.cleat", "package b; public class X { }"), ("C.cleat", "import a.X; import b.X; class C { }")],
        "X",
    );
    rejected("package cleat; class Mine { }", "cleat");
    accepted("class A { Int open = 1; Int value = 2; Int f() { var only = open + value; return only; } }");
    done();
}

#[test]
fn annotations_follow_chapter_8() {
    rejected("@Refines annotation Q(Int x);", "elements");
    rejected("annotation A(Int n); @A(n = 1, m = 2) class C { }", "no element");
    rejected("annotation A(Int n); @A class C { }", "no default");
    rejected("annotation A(Int n); class C { static Int k() { return 1; } @A(k()) void f() { } }", "constant");
    rejected("annotation A; @A @A class C { }", "at most once");
    rejected("annotation A(List<Int> items);", "element");
    rejected("annotation A; class C implements A { }", "not an interface");
    rejected("class C implements Annotation { }", "Annotation");
    rejected("annotation A; class C { void f() { var a = new A(); } }", "annotation");
    rejected("@Refines @Widens annotation Q;", "");
    rejected("@Refines(Q.class) annotation Q;", "below itself");
    rejected("@Refines annotation Q; class C { @Narrows(Q.class) public static Int f(Int n) { return n; } }", "Boolean");
    rejected("class C { @Implicit static C from(Int n) { return new C(); } }", "@Implicit");
    accepted("@Refines annotation Q; @Refines(Q.class) annotation R; class C { static @Q Int f(@R Int n) { return n; } }");
    done();
}

#[test]
fn statements_follow_their_rules() {
    rejected("class A { void f() { break; } }", "break");
    rejected("class A { void f() { a: while (true) { } b: while (true) { break a; } } }", "label");
    rejected("class A { void f(List<Int> items) { for (Int x : items) { x = 1; } } }", "");
    rejected("class A { void f(Int n) { for (Int x : n) { } } }", "Iterable");
    rejected("class A { void f(Object o) { switch (o) { case String s -> { } case 1 -> { } } } }", "not both");
    rejected("class A { void f(Int n) { switch (n) { default -> { } case 1 -> { } } } }", "last");
    rejected("class A { void f(@Nullable Int n) { switch (n) { default -> { } } } }", "@Nullable");
    rejected("class A { void f(String s) { using (String t = s) { } } }", "close");
    rejected("class A { void f(Object o) { Boolean b = o instanceof @Nullable String; } }", "@Nullable");
    rejected("class A { @Nullable String s; Int f() { if (s != null) { return s.length(); } return 0; } }", "may be `null`");
    rejected("class A { A() { return; } }", "return");
    rejected("class A { void f() { Function1<Int, Int> g = (a, b) -> a; } }", "parameters");
    rejected("class A { void f() { Function1<Int, Int> g = (String a) -> 1; } }", "");
    rejected("class A { void f() { var n = 0; Function0<Unit> g = () -> { n = 1; }; } }", "lambda");
    rejected("class A { Int f(Int n) { return n.length; } }", "length");
    rejected("class A { void f() { Int x = 1; { Int x = 2; } } }", "does not hide");
    rejected("class A { Int a = b; Int b = 1; }", "before it is assigned");
    done();
}

#[test]
fn generics_follow_chapter_7() {
    rejected("open class B<T> { } class A extends B<?> { }", "wildcard");
    rejected("class A { <out T> void f() { } }", "variance");
    rejected("class A<T> { static T make() { throw new IllegalStateException(); } }", "");
    rejected("class A<T, T> { }", "");
    rejected("class A<T extends Int & String> { }", "");
    rejected("interface Out<out T> { } class A { void f(Out<? super Int> o) { } }", "? super");
    rejected("class A<T> { T[] make(Int n) { return new T[n]; } }", "no default");
    rejected("class A { void f(List<Int> a, List<String> b) { a = b; } }", "is not assignable");
    accepted("class A<T> { @Nullable T[] make(Int n) { return new @Nullable T[n]; } <R extends Ordered<R>> R pick(R a, R b) { return a < b ? a : b; } }");
    done();
}
