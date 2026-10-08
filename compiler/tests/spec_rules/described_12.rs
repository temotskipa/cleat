//! Chapter 12, Flow: the sentences that define and describe.

use super::{held, rule, stated, Rule, CONSTANT_RAISES, FALLS_OFF, FALSE_LOOP, FINAL_LOCAL, UNASSIGNED, UNREACHABLE};

const NULLABLE_SEND: &str = "`length` is sent to a `@Nullable String`";
const NOT_ASSIGNED_FIELD: &str = "is not assigned on every path to the `super(...)` call";

/// A `@Narrows` method for the cases of section 12.4.
macro_rules! positive {
    ($more:literal) => {
        concat!(
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } } ",
            "class A { static Int need(@Positive Int n) { return n; } ",
            $more,
            " }"
        )
    };
}

pub const SENTENCES: &[Rule] = &[
    stated("12", "This chapter defines four things the compiler works out from the text of a method", "It says what the chapter covers."),
    // ---- 12.1 ----
    rule(
        "12",
        "A constant expression has a value that the compiler computes",
        &[("class A { Int8 x = 100 + 100; }", CONSTANT_RAISES)],
        &["class A { Int8 x = 100 + 27; Int f(Int n) { return switch (n) { case 100 + 27 -> 1; default -> 0; }; } }"],
    ),
    stated("12", "It is one of:", "It introduces the list after it, whose lines have their own cases."),
    rule(
        "12",
        "a literal other than `null`;",
        &[("annotation M(String s); @M(null) class A { }", "`null` is not a value of `String`")],
        &["annotation M(String s, Int n, Boolean b, Char c); @M(\"x\", 1, true, 'c') class A { Int f(Int n) { return switch (n) { case 1 -> 1; default -> 0; }; } }"],
    ),
    rule("12", "a constant expression in parentheses;", &[], &["annotation M(Int n); @M((1)) class A { Int f(Int n) { return switch (n) { case (1) -> 1; case ((2)) -> 2; default -> 0; }; } }"]),
    rule(
        "12",
        "applied to constant expressions, when the method it spells belongs to a numeric class",
        &[("class A { Int f(Int n, Int k) { return switch (n) { case k + 1 -> 1; default -> 0; }; } }", "the constant of a `case` is a constant expression")],
        &["annotation M(String s, Boolean b); @M(\"a\" + \"b\", 1 < 2) class A { Int f(Int n) { return switch (n) { case 1 + 2 * 3 -> 1; case -4 -> 2; case 10 % 4 -> 3; default -> 0; }; } }"],
    ),
    rule(
        "12",
        "whose operands are constant expressions;",
        &[("annotation Flag(Boolean on); class A { public static Boolean g() { return true; } } @Flag(true && A.g()) class B { }", "an annotation argument is a constant expression, a class literal, or a use of an annotation")],
        &["annotation Flag(Boolean on); @Flag(true && !false || false) class A { Int f(Int n) { return switch (n) { case (true ? 1 : 2) -> 1; default -> 0; }; } }"],
    ),
    rule(
        "12",
        "a call of `from` or `nearest` on a numeric class, or of `Char.from`",
        &[],
        &["annotation W(Int8 n, Float64 x, Char c); @W(Int8.from(5), Float64.nearest(3), Char.from(65)) class A { Int8 small = Int8.from(27); }"],
    ),
    rule(
        "12",
        "the name of a constant field;",
        &[("class A { static Int LIMIT = 3; Int f(Int n) { return switch (n) { case LIMIT -> 1; default -> 0; }; } }", "the constant of a `case` is a constant expression")],
        &["class A { static final Int LIMIT = 3; Int f(Int n) { return switch (n) { case LIMIT -> 1; case A.LIMIT + 1 -> 2; default -> 0; }; } }"],
    ),
    rule(
        "12",
        "the name of an enum constant.",
        &[],
        &["enum E { X, Y } annotation M(E e); @M(E.X) @Target(Site.TYPE) annotation N; class A { Int f(E e) { return switch (e) { case E.X -> 1; case E.Y -> 2; }; } }"],
    ),
    rule(
        "12",
        "A constant field is a `static final` field whose type is a numeric class",
        &[
            ("class A { static Int g() { return 3; } static final Int LIMIT = g(); Int f(Int n) { return switch (n) { case LIMIT -> 1; default -> 0; }; } }", "the constant of a `case` is a constant expression"),
            ("class A { final Int LIMIT = 3; Int f(Int n) { return switch (n) { case LIMIT -> 1; default -> 0; }; } }", "the constant of a `case` is a constant expression"),
        ],
        &["class A { static final Int N = 3; static final Boolean B = true; static final Char C = 'c'; static final String S = \"s\"; static final Float64 X = 1.5; Int f(Int n) { return switch (n) { case N -> 1; default -> 0; }; } }"],
    ),
    rule(
        "12",
        "take their class by the rules of",
        &[],
        &["class A { static final Int8 SMALL = 5; static final Int WIDE = SMALL + 1; static final Float64 HALF = 1 / 2.0; Int f(Int n) { return switch (n) { case WIDE -> 1; default -> 0; }; } }"],
    ),
    rule("12", "`\"v\" + 2` is a constant `String`", &[], &["annotation M(String s); @M(\"v\" + 2) class A { static final String V2 = \"v\" + 2; }"]),
    rule(
        "12",
        "A constant expression calls no other method, reads no other field, and creates no array",
        &[
            ("annotation M(String s); @M(\" a \".trim()) class A { }", "an annotation argument is a constant expression, a class literal, or a use of an annotation"),
            ("annotation M(Int n); class H { public static Int count = 1; } @M(H.count) class A { }", "an annotation argument is a constant expression, a class literal, or a use of an annotation"),
            ("annotation M(Int n); @M(new Int[1].length()) class A { }", "an annotation argument is a constant expression, a class literal, or a use of an annotation"),
        ],
        &[],
    ),
    rule(
        "12",
        "Constant expressions are required as the constants of a `switch` arm and as the arguments of an annotation",
        &[
            ("class A { Int f(Int n, Int k) { return switch (n) { case k -> 1; default -> 0; }; } }", "the constant of a `case` is a constant expression"),
            ("annotation M(Int n); class A { public static Int g() { return 1; } } @M(A.g()) class B { }", "an annotation argument is a constant expression, a class literal, or a use of an annotation"),
        ],
        &["annotation M(Int n); @M(1 + 2) class A { Int f(Int n) { return switch (n) { case 1 + 2 -> 1; default -> 0; }; } }"],
    ),
    rule(
        "12",
        "A condition that is the constant `true` or `false` affects",
        &[("class A { static final Boolean ON = true; void g() { } void f() { while (ON) { } g(); } }", UNREACHABLE), ("class A { void f() { while (1 > 2) { } } }", FALSE_LOOP)],
        &["class A { void g() { } void f(Boolean on) { while (on) { } g(); } }"],
    ),
    // ---- 12.2 ----
    rule(
        "12",
        "The body of a method, of a constructor, of a lambda and of a static initializer is reachable",
        &[],
        &["class A { static Int n = 0; static { n = 1; } Int m = 0; public A() { m = 1; } void f() { m = 2; } Function0<Unit> g() { return () -> { m = 3; }; } }"],
    ),
    rule(
        "12",
        "it is the first statement of a reachable block, or when the statement before it can complete normally",
        &[("class A { Int f() { return 1; return 2; } }", UNREACHABLE), ("class A { void g() { } void f() { throw new IllegalStateException(); g(); } }", UNREACHABLE)],
        &["class A { void g() { } Int f() { g(); g(); return 1; } }"],
    ),
    rule("12", "A local declaration, an expression statement, `assert`, the empty statement", &[], &["class A { Int f() { var x = 1; x = 2; assert x > 0; ; return x; } }"]),
    rule(
        "12",
        "Its last statement can, or it is empty",
        &[("class A { Int f() { { return 1; } return 2; } }", UNREACHABLE)],
        &["class A { Int f() { { } { var x = 1; } return 2; } }"],
    ),
    rule("12", "| `if (c) s` | Always |", &[], &["class A { Int f(Boolean b) { if (b) { return 1; } return 2; } Int g(Boolean b) { if (b) return 1; return 2; } }"]),
    rule(
        "12",
        "`s` can or `t` can",
        &[("class A { Int f(Boolean b) { if (b) { return 1; } else { return 2; } return 3; } }", UNREACHABLE)],
        &["class A { Int f(Boolean b) { if (b) { return 1; } else { } return 3; } Int g(Boolean b) { if (b) { } else { return 2; } return 3; } }"],
    ),
    rule(
        "12",
        "| `while (c) s` | `c` is not the constant `true`, or a `break` leaves the loop |",
        &[("class A { void g() { } void f() { while (true) { } g(); } }", UNREACHABLE), ("class A { void g() { } void f() { while (true) { while (true) { break; } } g(); } }", UNREACHABLE)],
        &["class A { void g() { } void f(Boolean b) { while (b) { } g(); while (true) { break; } g(); a: while (true) { while (true) { break a; } } g(); } }"],
    ),
    rule(
        "12",
        "`c` is written and is not the constant `true`, or a `break` leaves the loop",
        &[("class A { void g() { } void f() { for (;;) { } g(); } }", UNREACHABLE), ("class A { void g() { } void f() { for (var i = 0; true; i++) { } g(); } }", UNREACHABLE)],
        &["class A { void g() { } void f() { for (var i = 0; i < 3; i++) { } g(); for (;;) { break; } g(); } }"],
    ),
    rule("12", "| `for (T x : e) s` | Always |", &[], &["class A { void g() { } void f(List<Int> xs) { for (Int x : xs) { return; } g(); } }"]),
    rule(
        "12",
        "Some arm's body can, or the switch is not exhaustive, or a `break` leaves it",
        &[
            ("class A { void g() { } Int f(Int n) { switch (n) { case 1 -> { return 1; } default -> { return 2; } } g(); return 3; } }", UNREACHABLE),
            ("class A { void g() { } Int f(Boolean b) { switch (b) { case true -> { return 1; } case false -> { return 2; } } g(); return 3; } }", UNREACHABLE),
        ],
        &["class A { void g() { } Int f(Int n) { switch (n) { case 1 -> { return 1; } } g(); switch (n) { case 2 -> { return 1; } default -> { } } g(); switch (n) { case 3 -> { return 1; } default -> { if (n > 5) { break; } return 2; } } return 3; } }"],
    ),
    rule(
        "12",
        "The `try` block or some `catch` block can, and the `finally` block can if there is one",
        &[
            ("class A { Int f() { try { return 1; } catch (Throwable e) { return 2; } return 3; } }", UNREACHABLE),
            ("class A { void g() { } void f() { try { } finally { throw new IllegalStateException(); } g(); } }", UNREACHABLE),
        ],
        &["class A { Int f() { try { return 1; } catch (Throwable e) { } return 3; } Int g() { try { } catch (Throwable e) { return 2; } finally { } return 3; } }"],
    ),
    rule(
        "12",
        "| `using` | Its block can |",
        &[("class R { public void close() { } } class A { void g() { } void f() { using (R r = new R()) { return; } g(); } }", UNREACHABLE)],
        &["class R { public void close() { } } class A { void g() { } void f() { using (R r = new R()) { } g(); } }"],
    ),
    rule(
        "12",
        "`s` can, or a `break label` leaves it",
        &[("class A { void g() { } void f() { a: { return; } g(); } }", UNREACHABLE)],
        &["class A { void g() { } void f(Boolean b) { a: { if (b) { break a; } return; } g(); c: { } g(); } }"],
    ),
    rule(
        "12",
        "The body of a loop is reachable when the loop is, unless its condition is the constant `false`",
        &[("class A { void g() { } void f() { while (false) { g(); } } }", FALSE_LOOP)],
        &["class A { void g() { } void f(Boolean b) { while (b) { g(); } while (true) { g(); break; } } }"],
    ),
    rule(
        "12",
        "The branches of an `if` are reachable when the `if` is, whatever its condition",
        &[],
        &["class A { static final Boolean OFF = false; void g() { } void f() { if (false) { g(); } else { g(); } if (true) { g(); } else { g(); } if (OFF) { g(); } } }"],
    ),
    rule(
        "12",
        "Each arm of a switch, each `catch` block and a `finally` block is reachable when its statement is",
        &[],
        &["class A { void g() { } void f(Int n) { switch (n) { case 1 -> { g(); } default -> { g(); } } try { g(); } catch (ArithmeticException e) { g(); } catch (Throwable e) { g(); } finally { g(); } } }"],
    ),
    rule("12", "A method whose result is `Unit` may complete by reaching the end of its body", &[("class A { Int f() { } }", FALLS_OFF)], &["class A { void f() { } Unit g() { } void h(Boolean b) { if (b) { return; } } }"]),
    rule(
        "12",
        "The same holds for a lambda with a block body",
        &[("class A { Function0<Int> f() { return () -> { }; } }", FALLS_OFF)],
        &["class A { Function0<Unit> f() { return () -> { }; } Function1<Int, Unit> g() { return (n) -> { if (n > 0) { return; } }; } }"],
    ),
    // ---- 12.3 ----
    rule(
        "12",
        "every path that reaches the point has passed its initializer or an assignment to it",
        &[("class A { Int f() { Int n; return n; } }", UNASSIGNED), ("class A { Int f(Boolean b) { Int n; if (b) { n = 1; } return n; } }", UNASSIGNED)],
        &["class A { Int f(Boolean b) { Int n = 0; Int m; m = 1; Int k; if (b) { k = 1; } else { k = 2; } return n + m + k; } }"],
    ),
    rule(
        "12",
        "and the name of a type arm are assigned where they are in scope",
        &[],
        &["class R { public void close() { } public Int n() { return 1; } } class A { Int f(Int p, List<Int> xs, Object o) { var t = p; for (Int x : xs) { t += x; } try { } catch (Throwable e) { t += e.message().length(); } using (R r = new R()) { t += r.n(); } switch (o) { case String s -> { t += s.length(); } default -> { } } return t; } }"],
    ),
    stated("12", "The paths are those of", "It points to section 12.2, whose sentences have their own cases."),
    rule(
        "12",
        "a local is assigned when it is assigned after `s` and after `t`",
        &[("class A { Int f(Boolean b) { Int n; if (b) { n = 1; } else { } return n; } }", UNASSIGNED)],
        &["class A { Int f(Boolean b) { Int n; if (b) { n = 1; } else { n = 2; } return n; } }"],
    ),
    rule(
        "12",
        "or by the condition when the condition is false",
        &[("class A { Int f(Int k) { Int n; while (k > 0) { n = k; k--; } return n; } }", UNASSIGNED)],
        &["class A { Int g() { return 0; } Int f() { Int n; while ((n = g()) > 0) { } return n; } Int h(Int k) { Int n = 0; while (k > 0) { k--; } return n; } }"],
    ),
    rule(
        "12",
        "a local is assigned when every arm assigns it and the switch is exhaustive",
        &[
            ("class A { Int f(Int k) { Int n; switch (k) { case 1 -> { n = 1; } case 2 -> { n = 2; } } return n; } }", UNASSIGNED),
            ("class A { Int f(Int k) { Int n; switch (k) { case 1 -> { n = 1; } default -> { } } return n; } }", UNASSIGNED),
        ],
        &["class A { Int f(Int k) { Int n; switch (k) { case 1 -> { n = 1; } default -> { n = 2; } } return n; } Int g(Boolean b) { Int n; switch (b) { case true -> { n = 1; } case false -> { n = 2; } } return n; } }"],
    ),
    rule(
        "12",
        "a local is assigned when it is assigned after the `try` block and after every `catch` block",
        &[("class A { Int f() { Int n; try { n = 1; } catch (ArithmeticException e) { n = 2; } catch (Throwable e) { } return n; } }", UNASSIGNED)],
        &["class A { Int f() { Int n; try { n = 1; } catch (ArithmeticException e) { n = 2; } catch (Throwable e) { n = 3; } return n; } }"],
    ),
    rule("12", "A local that the `finally` block assigns is assigned after the statement", &[], &["class A { Int f() { Int n; try { } finally { n = 1; } return n; } Int g() { Int n; try { } catch (Throwable e) { } finally { n = 1; } return n; } }"]),
    rule(
        "12",
        "`b` is evaluated with what `a` assigns",
        &[("class A { Boolean f(Boolean p) { Int n; return p && n > 0; } }", UNASSIGNED)],
        &["class A { Boolean f() { Int n; return (n = 1) > 0 && n > 0; } }"],
    ),
    rule("12", "`||` is the same.", &[("class A { Boolean f(Boolean p) { Int n; return p || n > 0; } }", UNASSIGNED)], &["class A { Int f(Boolean p) { Int n; var r = (n = 1) > 5 || n > 0; return n; } }"]),
    rule(
        "12",
        "a local is assigned when `c` assigns it or both arms do",
        &[("class A { Int f(Boolean b) { Int n; var r = b ? (n = 1) : 2; return n; } }", UNASSIGNED)],
        &["class A { Int f(Boolean b) { Int n; var r = b ? (n = 1) : (n = 2); return n; } Int g() { Int n; var r = (n = 1) > 0 ? 1 : 2; return n; } }"],
    ),
    rule("12", "A `final` local is assigned at most once", &[("class A { void f() { final Int n; n = 1; n = 2; } }", FINAL_LOCAL), ("class A { void f() { final Int n = 1; n = 2; } }", FINAL_LOCAL)], &["class A { Int f() { final Int n; n = 1; final Int m = 2; return n + m; } }"]),
    rule(
        "12",
        "has a constructor assign the fields of its class before it calls `super(...)`",
        &[("class A { Int n; public A() { super(); this.n = 1; } }", NOT_ASSIGNED_FIELD)],
        &["class A { Int n; public A() { this.n = 1; super(); } }"],
    ),
    rule(
        "12",
        "The rules for a local apply to each of those fields in that part of the body",
        &[("class A { Int n; public A(Boolean b) { if (b) { this.n = 1; } super(); } }", NOT_ASSIGNED_FIELD), ("class A { Int n; Int m; public A() { this.m = n; this.n = 1; super(); } }", "the field `n` is read before it is assigned")],
        &["class A { Int n; Int m; public A(Boolean b) { if (b) { this.n = 1; } else { this.n = 2; } this.m = n + 1; super(); } }"],
    ),
    rule(
        "12",
        "every field declared in the class is definitely assigned, by its initializer or by the body",
        &[("class A { Int n; public A() { } }", NOT_ASSIGNED_FIELD), ("class A { Int n; }", "is not assigned on every path to the `super(...)` call")],
        &["class A { Int n = 1; Int m; public A() { m = 2; } } class B { Int n; public B() { n = 1; super(); } }"],
    ),
    rule("12", "A `@Nullable` field is exempt and starts as `null`", &[], &["class A { @Nullable String s; @Nullable Int n; public A() { } } class B { @Nullable String s; }"]),
    rule(
        "12",
        "A final field is assigned at most once, by its initializer or by the body before the `super(...)` call, and nowhere else",
        &[
            ("class A { final Int n; public A() { this.n = 1; this.n = 2; super(); } }", "the final field `n` is assigned at most once"),
            ("class A { final Int n = 1; public A() { this.n = 2; super(); } }", "the final field `n` is assigned at most once"),
            ("class A { final Int n; public A() { this.n = 1; super(); this.n = 2; } }", "the field `n` is not assigned here: it is final"),
        ],
        &["class A { final Int n; final Int m = 2; public A(Boolean b) { if (b) { this.n = 1; } else { this.n = 3; } super(); } }"],
    ),
    rule(
        "12",
        "assigns no field of its own: the constructor it calls has assigned them all",
        &[("class A { Int n; public A() { this.n = 1; super(); } public A(Int k) { this.n = k; this(); } }", "`this(...)`, when written, is the first statement of the constructor")],
        &["class A { Int n; public A() { this.n = 1; super(); } public A(Int k) { this(); n = k; } }"],
    ),
    rule(
        "12",
        "every static field of the class is definitely assigned, by its initializer or by a static initializer block",
        &[("class A { static Int n; }", "the static field `n` is not assigned by its initializer or by a static initializer block"), ("class A { static Int n; static Int m = 1; static { m = 2; } }", "the static field `n` is not assigned by its initializer or by a static initializer block")],
        &["class A { static Int n; static Int m = 1; static { n = m + 1; } }"],
    ),
    rule("12", "A `@Nullable` static field is exempt", &[], &["class A { static @Nullable String s; static @Nullable Int n; }"]),
    held("12", "A static field that is read while initialization is still running is checked at run time", "behavior", "statics"),
    rule(
        "12",
        "The fields of a value class are assigned by its constructor's arguments",
        &[("value class V { public Int a; public Int b; } class A { V f() { return new V(1); } }", "constructor `V` takes 2 arguments, and 1 is written")],
        &["value class V { public Int a; public Int b; } class A { Int f() { return new V(1, 2).a + new V(1, 2).b; } }"],
    ),
    rule(
        "12",
        "Nothing in the class assigns one",
        &[("value class V { public Int a; public V { a = 1; } }", "`a` is not assignable here"), ("value class V { public Int a; public void f() { a = 2; } }", "a field of a value class is final")],
        &[],
    ),
    // ---- 12.4 ----
    rule(
        "12",
        "narrows a local or a parameter after a null test or an `instanceof`",
        &[],
        &[
            "class A { Int f(@Nullable String s, Object o) { if (s != null && o instanceof String) { return s.length() + o.length(); } return 0; } }",
            positive!("static Int f(Int n) { if (Counts.isPositive(n)) { return need(n); } return 0; }"),
        ],
    ),
    stated("12", "This section says where the narrowed type holds", "It says what the section covers."),
    stated("12", "A test gives a fact about a variable `x` on one of its outcomes", "It introduces the table after it, whose rows have their own cases."),
    rule(
        "12",
        "| `x != null` | When `true`, `x` is not `null` |",
        &[("class A { Int f(@Nullable String s) { if (s != null) { return 0; } else { return s.length(); } } }", "`length` is sent to")],
        &["class A { Int f(@Nullable String s) { if (s != null) { return s.length(); } return 0; } }"],
    ),
    rule(
        "12",
        "| `x == null` | When `false`, `x` is not `null` |",
        &[("class A { Int f(@Nullable String s) { if (s == null) { return s.length(); } return 0; } }", "`length` is sent to")],
        &["class A { Int f(@Nullable String s) { if (s == null) { return 0; } else { return s.length(); } } }"],
    ),
    rule(
        "12",
        "| `x instanceof T` | When `true`, `x` is a `T` |",
        &[("class A { Int f(Object o) { if (o instanceof String) { return 0; } else { return o.length(); } } }", "`Object` has no method named `length`")],
        &["class A { Int f(Object o) { if (o instanceof String) { return o.length(); } return 0; } }"],
    ),
    rule(
        "12",
        "When `true`, `x` gains or loses `Q`",
        &[(positive!("static Int f(Int n) { if (Counts.isPositive(n)) { return 0; } else { return need(n); } }"), "`Int` is not assignable to `@Positive Int`")],
        &[positive!("static Int f(Int n) { if (Counts.isPositive(n)) { return need(n); } return 0; }")],
    ),
    stated("12", "Conditions combine facts", "It introduces the list after it, whose items have their own cases."),
    rule(
        "12",
        "has when `true` the facts `c` has when `false`, and the reverse",
        &[("class A { Int f(@Nullable String s) { if (!(s != null)) { return s.length(); } return 0; } }", "`length` is sent to")],
        &["class A { Int f(@Nullable String s) { if (!(s == null)) { return s.length(); } return 0; } Int g(@Nullable String s) { if (!(s != null)) { return 0; } return s.length(); } }"],
    ),
    rule(
        "12",
        "has when `true` the facts of `a` and of `b` when `true`",
        &[],
        &["class A { Int f(@Nullable String s, @Nullable String t) { if (s != null && t != null) { return s.length() + t.length(); } return 0; } }"],
    ),
    rule("12", "`b` is checked with the facts `a` has when `true`", &[("class A { Boolean f(@Nullable String s) { return s == null && s.isEmpty(); } }", "`isEmpty` is sent to")], &["class A { Boolean f(@Nullable String s) { return s != null && s.isEmpty(); } }"]),
    rule(
        "12",
        "has when `false` the facts of `a` and of `b` when `false`",
        &[],
        &["class A { Int f(@Nullable String s, @Nullable String t) { if (s == null || t == null) { return 0; } return s.length() + t.length(); } }"],
    ),
    rule("12", "`b` is checked with the facts `a` has when `false`", &[("class A { Boolean f(@Nullable String s) { return s != null || s.isEmpty(); } }", "`isEmpty` is sent to")], &["class A { Boolean f(@Nullable String s) { return s == null || s.isEmpty(); } }"]),
    rule("12", "Parentheses change nothing", &[], &["class A { Int f(@Nullable String s) { if (((s != null))) { return (s).length(); } return 0; } Boolean g(@Nullable String s) { return (s == null) || (s.isEmpty()); } }"]),
    rule(
        "12",
        "A fact holds at a point when it holds on every path that reaches the point",
        &[("class A { Int f(@Nullable String s, Boolean b) { if (b) { if (s == null) { return 0; } } return s.length(); } }", NULLABLE_SEND)],
        &["class A { Int f(@Nullable String s, Boolean b) { if (b) { if (s == null) { return 0; } } else { if (s == null) { return 1; } } return s.length(); } }"],
    ),
    rule(
        "12",
        "hold at the start of `s`, and its facts when `false` hold at the start of `t`",
        &[],
        &["class A { Int f(@Nullable String s) { if (s == null) { return 0; } else { return s.length(); } } Int g(@Nullable String s) { if (s != null) { return s.length(); } else { return 0; } } }"],
    ),
    rule(
        "12",
        "a fact holds when it holds at the end of each branch that can complete normally",
        &[("class A { Int f(@Nullable String s) { if (s != null) { } else { } return s.length(); } }", NULLABLE_SEND)],
        &["class A { Int f(@Nullable String s) { if (s != null) { } else { throw new IllegalStateException(); } return s.length(); } Int g(@Nullable String s) { if (s == null) { return 0; } return s.length(); } }"],
    ),
    rule(
        "12",
        "hold at the start of the body, and its facts when `false` hold after the loop unless a `break` leaves it",
        &[("class A { @Nullable String g() { return null; } Int f(@Nullable String s, Boolean b) { while (s == null) { if (b) { break; } s = g(); } return s.length(); } }", NULLABLE_SEND)],
        &["class A { @Nullable String g() { return null; } Int f(@Nullable String s) { while (s == null) { s = g(); } return s.length(); } Int h(@Nullable String s) { var n = 0; while (s != null) { n += s.length(); s = g(); } for (var t = g(); t != null; t = g()) { n += t.length(); } return n; } }"],
    ),
    rule(
        "12",
        "hold in `a`, and its facts when `false` hold in `b`",
        &[("class A { Int f(@Nullable String s) { return s != null ? 0 : s.length(); } }", "`length` is sent to")],
        &["class A { Int f(@Nullable String s) { return s != null ? s.length() : 0; } Int g(@Nullable String s) { return s == null ? 0 : s.length(); } }"],
    ),
    rule(
        "12",
        "no fact comes from the test of `a`",
        &[("class A { Int f(@Nullable String s) { var t = s ?? \"x\"; return s.length(); } }", NULLABLE_SEND)],
        &["class A { Int f(@Nullable String s) { var t = s ?? \"x\"; return t.length(); } }"],
    ),
    rule("12", "gives the facts of `c` when `true` to what follows", &[("class A { Int f(@Nullable String s) { assert s == null; return s.length(); } }", "`length` is sent to")], &["class A { Int f(@Nullable String s, Object o) { assert s != null; assert o instanceof String : \"a string\"; return s.length() + o.length(); } }"]),
    rule(
        "12",
        "A fact about a variable holds inside a lambda that uses the variable",
        &[("class A { Function0<Int> f(@Nullable String s) { return () -> s.length(); } }", NULLABLE_SEND)],
        &["class A { Function0<Int> f(@Nullable String s, Object o) { if (s == null || !(o instanceof String)) { return () -> 0; } return () -> s.length() + o.length(); } }"],
    ),
    rule(
        "12",
        "An assignment to `x` ends every fact about `x`",
        &[
            ("class A { Int f(@Nullable String s, @Nullable String t) { if (s != null) { s = t; return s.length(); } return 0; } }", NULLABLE_SEND),
            ("class A { Int f(Object o, Object p) { if (o instanceof String) { o = p; return o.length(); } return 0; } }", "`Object` has no method named `length`"),
        ],
        &["class A { Int f(@Nullable String s, @Nullable String t) { if (s != null) { var n = s.length(); s = t; return n; } return 0; } }"],
    ),
];
