//! Programs the checker must reject, each with the rule it breaks. A case is the body of
//! a class `Probe` in the package `probe`, and types that stand beside it.

use cleatc::analyze_sources;
use std::path::PathBuf;

fn messages(members: &str, types: &str) -> Vec<String> {
    let text = format!("package probe;\nclass Probe {{\n{members}\n}}\n{types}\n");
    match analyze_sources(&[(PathBuf::from("Probe.cleat"), text)]) {
        Ok(_) => Vec::new(),
        Err(d) => d.into_iter().map(|d| format!("{}:{}: {}", d.line, d.column, d.message)).collect(),
    }
}

fn accepted(members: &str, types: &str) {
    let m = messages(members, types);
    assert!(m.is_empty(), "expected no diagnostics, and the checker says:\n{}", m.join("\n"));
}

fn rejected(members: &str, types: &str, fragment: &str) {
    let m = messages(members, types);
    assert!(!m.is_empty(), "expected a diagnostic that mentions `{fragment}`, and the checker accepts the program");
    assert!(
        m.iter().any(|x| x.contains(fragment)),
        "expected a diagnostic that mentions `{fragment}`, and the checker says:\n{}",
        m.join("\n")
    );
}

#[test]
fn a_plain_program_is_accepted() {
    accepted("static Int twice(Int n) { return n + n; }", "");
}

#[test]
fn null_is_not_a_string() {
    rejected("static void f() { String s = null; }", "", "`null` is not a value of `String`");
    accepted("static void f() { @Nullable String s = null; }", "");
}

#[test]
fn a_nullable_value_is_not_assignable_to_its_type() {
    rejected("static String f(@Nullable String s) { return s; }", "", "is not assignable");
}

#[test]
fn a_send_on_a_nullable_receiver_is_rejected_until_narrowed() {
    rejected("static Int f(@Nullable String s) { return s.length(); }", "", "may be `null`");
    accepted("static Int f(@Nullable String s) { if (s == null) { return 0; } return s.length(); }", "");
    accepted("static String f(@Nullable String s) { return s.toString(); }", "");
}

#[test]
fn a_null_test_of_a_value_that_cannot_be_null_is_rejected() {
    rejected("static Boolean f(String s) { return s == null; }", "", "cannot be `null`");
}

#[test]
fn an_integer_has_no_division_operator() {
    rejected("static Int f(Int a, Int b) { return a / b; }", "", "has no method `div`");
    accepted("static Int f(Int a, Int b) { return a.floorDiv(b) + a % b; }", "");
}

#[test]
fn numbers_of_two_classes_do_not_mix_without_a_conversion() {
    rejected("static Float64 f(Int32 i, Float64 x) { return i + x; }", "", "`Float64` is not assignable to `Int32`");
    accepted("static Int f(Int32 i, Int n) { return i + n; }", "");
    rejected("static Int32 f(Int n) { return n; }", "", "is not assignable");
    accepted("static Int32 f(Int n) { return Int32.from(n); }", "");
}

#[test]
fn a_literal_must_fit_its_class() {
    rejected("static void f() { UInt8 b = 0x100; }", "", "outside the range");
    rejected("static void f() { Int n = 1.5; }", "", "decimal literal");
    accepted("static void f() { Int8 low = -128; Float32 x = 0.1; }", "");
}

#[test]
fn a_constant_expression_that_would_raise_is_rejected() {
    rejected("static void f() { Int32 big = 2147483647 + 1; }", "", "ArithmeticException");
    // The mistake inside an argument is the one reported.
    rejected("static void f() { Console.println(Int8.from(128)); }", "", "ArithmeticException");
}

#[test]
fn text_is_never_made_implicitly() {
    rejected("static String f(Int n) { return n + \" items\"; }", "", "`String` is not assignable to `Int`");
    accepted("static String f(Int n) { return \"count: \" + n; }", "");
    rejected("static String f(Int n) { return n; }", "", "is not assignable");
}

#[test]
fn equality_between_disjoint_types_is_rejected() {
    rejected("static Boolean f(String s, Int n) { return s == n; }", "", "disjoint");
    rejected("static Boolean f(Int32 i, Float64 x) { return i == x; }", "", "disjoint");
    accepted("static Boolean f(Int32 i, Int n) { return i == n; }", "");
}

#[test]
fn a_cast_between_disjoint_types_is_rejected() {
    rejected("static Int f(Float64 x) { return (Int) x; }", "", "disjoint");
    accepted("static String f(Object o) { return (String) o; }", "");
}

#[test]
fn a_result_must_be_used() {
    rejected("static Int g() { return 1; } static void f() { g(); }", "", "must be used");
    rejected("static void f(Int n) { n + 1; }", "", "not a statement");
    accepted("static void f(StringBuilder b) { b.append(\"x\"); }", "");
}

#[test]
fn every_statement_is_reachable() {
    rejected("static Int f() { return 1; return 2; }", "", "unreachable");
    rejected("static void f() { while (false) { } }", "", "constant `false`");
    rejected("static Int f(Boolean b) { if (b) { return 1; } }", "", "can reach the end");
}

#[test]
fn a_local_is_assigned_before_it_is_used() {
    rejected("static Int f(Boolean b) { Int n; if (b) { n = 1; } return n; }", "", "may not have been assigned");
    rejected("static void f() { final Int n = 1; n = 2; }", "", "final");
    rejected("static void f() { var n; }", "", "`var` needs an initializer");
    rejected("static void f() { var n = null; }", "", "`null` alone gives no type");
    rejected("static void f(Int n) { Int n = 2; }", "", "does not hide another");
}

#[test]
fn a_lambda_captures_only_what_never_changes() {
    rejected(
        "static Function0<Int> f() { var n = 1; n = 2; return () -> n; }",
        "",
        "never assigned after it is initialized",
    );
    rejected("static void f() { var g = () -> 1; }", "", "no type of its own");
    accepted("static Function0<Int> f(Int n) { return () -> n + 1; }", "");
}

#[test]
fn fields_are_assigned_before_super() {
    rejected("", "class Pair { final Int a; final Int b; public Pair(Int a) { this.a = a; } }", "`b` is not assigned");
    rejected(
        "",
        "class Late { final Int a; public Late(Int a) { this.a = a; super(); this.a = 2; } }",
        "final",
    );
    rejected("", "class Early { Int a; public Early() { helper(); this.a = 1; } void helper() { } }", "before `super(...)`");
    rejected("", "value class V { public Int a; public V { a = 2; } }", "final");
}

#[test]
fn overriding_follows_its_rules() {
    rejected(
        "",
        "open class A { public open Int f() { return 1; } } class B extends A { public Int f() { return 2; } }",
        "must carry `@Override`",
    );
    rejected("", "class A { @Override public Int f() { return 1; } }", "overrides nothing");
    rejected(
        "",
        "open class A { public Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }",
        "is final",
    );
    rejected("", "class A { } class B extends A { }", "A");
    rejected("", "interface I { Int f(); } class C implements I { }", "does not implement");
}

#[test]
fn audiences_are_enforced() {
    rejected(
        "static Int f(Other o) { return o.secret; }",
        "class Other { Int secret = 1; }",
        "outside its audience",
    );
    rejected("static void f(Other o) { o.hidden(); }", "class Other { void hidden() { } }", "outside its audience");
    rejected("static Other f() { return new Other(); }", "class Other { Other() { } }", "outside its audience");
    accepted("static Int f(Other o) { return o.shown; }", "class Other { package Int shown = 1; }");
}

#[test]
fn generics_are_checked() {
    rejected("static void f() { List raw = new List<Int>(); }", "", "no raw types");
    rejected("static void f(List<String> a) { List<Object> b = a; }", "", "is not assignable");
    accepted("static void f(List<String> a) { Iterable<Object> b = a; }", "");
    rejected("static void f() { var x = new List<?>(); }", "", "wildcard");
    rejected("static void f(List<? extends Object> a) { a.add(\"x\"); }", "", "is not assignable to `an unknown type`");
    rejected(
        "static <T extends Numeric<T>> T zero() { return T.zero(); } static void f() { String s = Probe.<String>zero(); }",
        "",
        "bound",
    );
    rejected("static <T> T make() { throw new IllegalStateException(); } static void f() { make(); }", "", "cannot be inferred");
}

#[test]
fn variance_is_checked_at_the_declaration() {
    rejected("", "interface Source<out T> { void put(T item); }", "declared `out`");
    rejected("", "interface Sink<in T> { T take(); }", "declared `in`");
}

#[test]
fn switches_are_checked() {
    rejected(
        "static Int f(Color c) { return switch (c) { case Color.RED -> 1; }; }",
        "enum Color { RED, GREEN }",
        "exhaustive",
    );
    rejected(
        "static Int f(Color c) { return switch (c) { case Color.RED -> 1; case Color.RED -> 2; default -> 3; }; }",
        "enum Color { RED, GREEN }",
        "twice",
    );
    rejected(
        "static Int f(Object o) { return switch (o) { case Object x -> 1; case String s -> 2; }; }",
        "",
        "cannot match",
    );
    accepted(
        "static Int f(Color c) { return switch (c) { case Color.RED -> 1; case Color.GREEN -> 2; }; }",
        "enum Color { RED, GREEN }",
    );
}

#[test]
fn qualifiers_are_enforced() {
    let types = "@Refines annotation Even; class Evens { @Narrows(Even.class) public static Boolean isEven(Int n) { return n % 2 == 0; } }";
    rejected("static @Even Int f(Int n) { return n; }", types, "is not assignable");
    accepted("static Int f(@Even Int n) { return n; }", types);
    accepted("static @Even Int f(Int n) { if (Evens.isEven(n)) { return n; } throw new IllegalStateException(); }", types);
    rejected("static Int f(Object o) { return (@Even Int) o; }", types, "qualifier");
}

#[test]
fn tags_and_targets_are_enforced() {
    let types = "@Target(Site.TYPE) annotation Frozen; class Holder<@Frozen T> { } @Frozen value class Coin { public Int cents; }";
    rejected("static void f(Holder<String> h) { }", types, "does not carry the tag");
    accepted("static void f(Holder<Coin> h) { }", types);
    rejected("@Frozen static void f() { }", types, "`@Target`");
}

#[test]
fn exceptions_and_try_are_checked() {
    rejected("static void f() { throw \"no\"; }", "", "`throw` raises a `Throwable`");
    rejected(
        "static void f() { try { } catch (Throwable e) { } catch (ArithmeticException e) { } }",
        "",
        "can never run",
    );
    rejected("static Int f() { try { return 1; } finally { return 2; } }", "", "`finally`");
}

#[test]
fn value_classes_and_enums_keep_their_rules() {
    rejected("", "open value class V { public Int a; }", "value class");
    rejected("static Color f() { return new Color(); }", "enum Color { RED }", "enum");
    rejected("static void f(V v) { v.a = 2; }", "value class V { public Int a; }", "final");
    rejected("static void f() { var x = new Shape(); }", "abstract class Shape { }", "abstract");
}

#[test]
fn static_and_instance_are_kept_apart() {
    rejected("Int n = 1; static Int f() { return n; }", "", "static");
    rejected("void g() { } static void f() { g(); }", "", "instance method");
    rejected("static Int f(String s) { return String.length(); }", "", "instance method");
}

#[test]
fn foreign_methods_take_machine_types() {
    rejected("foreign static Int strlen(String s);", "", "machine type");
    accepted("foreign static Float64 sqrt(Float64 x);", "");
}
