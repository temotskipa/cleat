//! Chapter 5, Null and Unit: the sentences that define and describe.

use super::{held, ran, rule, stated, Rule};

const RUN: &str = "Sentences05";

pub const SENTENCES: &[Rule] = &[
    // ---- 5.1 ----
    rule("05", "`Null` is a final value class with one instance", &[("class B extends Null { }", "`Null` is not a class that can be extended"), ("class A { Null f() { return new Null(); } }", "`null` is the one instance")], &["class A { Null f() { return null; } }"]),
    rule("05", "The literal `null` is that instance", &[], &["class A { Null f() { Null n = null; return n; } Boolean g(Null n) { return n == null; } }"]),
    rule("05", "It is the one class outside the hierarchy that", &[("class A { Object f(Null n) { return n; } }", "`Null` is not assignable to `Object`")], &["class A { Object f(String s, Int n, Unit u) { Object o = s; o = n; return u; } }"]),
    // ---- 5.2 ----
    rule("05", "`@Nullable` is a type-use annotation", &[("@Nullable class A { }", "`@Nullable` is a qualifier: it is written before a type"), ("class A { @Nullable public A() { } }", "a qualifier is written before a type, and a constructor has none"), ("enum E { @Nullable A }", "`@Nullable` is a qualifier: it is written before a type")], &["class A { @Nullable String f(@Nullable String s) { return s; } }"]),
    held("05", "It is a qualifier, declared in the prelude with the facility", "spec_examples", "annotations_nullable"),
    stated("05", "Two facts about it are fixed by the language", "It introduces the two facts, which the sentences of this section and of section 5.4 state."),
    rule("05", "contains every value of `T`, and `null`", &[], &["class A { void f(String s) { @Nullable String a = s; @Nullable String b = null; @Nullable String c = \"x\"; } }"]),
    rule("05", "`T` is a subtype of `@Nullable T`.", &[], &["class A { @Nullable String f(String s) { return s; } @Nullable Int g(Int n) { return n; } }"]),
    rule("05", "`Null` is a subtype of `@Nullable T` for every `T`", &[], &["class A<T> { @Nullable String f(Null n) { return n; } @Nullable List<Int> g(Null n) { return n; } @Nullable T h(Null n) { return n; } }"]),
    rule("05", "It is a subtype of no other type except itself", &[("class A { String f(Null n) { return n; } }", "`Null` is not assignable to `String`"), ("class A<T> { T f(Null n) { return n; } }", "`Null` is not assignable to `T`")], &["class A { Null f(Null n) { return n; } }"]),
    rule(
        "05",
        "`@Nullable Object` is a supertype of every type",
        &[],
        &["class A<T> { @Nullable Object f(String s, @Nullable Int n, Null z, List<Int> xs, T t, @Nullable T u) { @Nullable Object o = s; o = n; o = z; o = t; o = u; return xs; } }"],
    ),
    rule("05", "Writing `@Nullable` twice on one type use is the same as writing it once", &[], &["class A { @Nullable @Nullable String f(@Nullable String s) { return s; } @Nullable String g(@Nullable @Nullable String s) { return s; } }"]),
    rule("05", "A `String` may be passed where a `@Nullable String` is expected", &[], &["class A { Int f(@Nullable String s) { return 1; } Int g(String s) { return f(s) + f(\"x\"); } }"]),
    rule(
        "05",
        "`@Nullable` may mark any type use",
        &[],
        &["class A<T> { @Nullable String f = null; @Nullable T g(@Nullable T p) { @Nullable Int local = null; List<@Nullable String> xs = new List<@Nullable String>(); @Nullable String[] arr = new @Nullable String[1]; return p; } }"],
    ),
    rule(
        "05",
        "is an array whose elements may be `null`",
        &[("class A { @Nullable String[] g() { return null; } }", "`null` is not a value of")],
        &["class A { void f(@Nullable String[] a) { a[0] = null; } }"],
    ),
    rule(
        "05",
        "is an array reference that may itself be `null`",
        &[("class A { void f(String @Nullable [] a) { if (a != null) { a[0] = null; } } }", "`null` is not a value of `String`")],
        &["class A { String @Nullable [] g() { return null; } Int h(String @Nullable [] a) { return a == null ? 0 : a.length(); } }"],
    ),
    rule("05", "A type parameter with no bound accepts a `@Nullable` type argument", &[], &["class Box<T> { } class A { void f(Box<@Nullable String> b, List<@Nullable Int> xs) { } }"]),
    ran("05", "A `@Nullable` field with no initializer starts as `null`", RUN, "nullable field: true"),
    stated("05", "How an implementation represents `null` is not observable", "It says what no program can observe."),
    // ---- 5.3 ----
    rule(
        "05",
        "The receiver is declared in the first parameter position, with the name `this`",
        &[("class C { public Int f(Int n, C this) { return n; } }", "`this` is a keyword and cannot be a name")],
        &["class C { public Boolean isBlank(@Nullable C this) { return this == null; } public Int size(C this, Int n) { return n; } }"],
    ),
    rule("05", "`Object` declares four methods this way", &[], &["class A { String f(@Nullable Object o) { return o.toString() + o.hashCode() + o.equals(o) + o.identical(o); } }"]),
    rule("05", "`identical` is final, and", &[("class A { @Override public Boolean identical(@Nullable Object o) { return true; } }", "`identical` is final in `Object` and cannot be overridden")], &[]),
    rule(
        "05",
        "The other three are `equals`, `hashCode` and `toString`",
        &[],
        &["class A { @Override public Boolean equals(@Nullable Object o) { return o instanceof A; } @Override public Int hashCode() { return 7; } @Override public String toString() { return \"A\"; } }"],
    ),
    ran("05", "They are `open`, and `Null` declares the same three", RUN, "Null methods: null 0 true false"),
    ran("05", "When the receiver is `null`, the method that `Null` declares runs", RUN, "Null methods: null 0 true false"),
    rule(
        "05",
        "The parameter of `equals` is `@Nullable Object` on every class",
        &[("class A { @Override public Boolean equals(Object other) { return true; } }", "`equals` carries `@Override` and overrides nothing")],
        &["class A { Boolean f(String s, Int n, List<Int> xs, A a) { return s.equals(null) || n.equals(null) || xs.equals(null) || a.equals(null); } }"],
    ),
    ran("05", "An override of `equals` returns `false` for a `null` argument", RUN, "equals null: false false false false"),
    rule("05", "A final method of any class may declare a `@Nullable` receiver", &[], &["class C { public Int size(@Nullable C this) { return 1; } } open class D { public final Int size(@Nullable D this) { return 2; } } class A { Int f(@Nullable C c, @Nullable D d) { return c.size() + d.size(); } }"]),
    rule(
        "05",
        "and is narrowed like any other parameter",
        &[
            ("class C { public Int size(@Nullable C this) { return own(); } Int own() { return 1; } }", "`own` is sent to a `@Nullable C`"),
            ("class C { public Int size(@Nullable C this) { return this.own(); } Int own() { return 1; } }", "`own` is sent to a `@Nullable C`"),
            ("class C { Int n = 1; public Int size(@Nullable C this) { return n; } }", "the field `n` is read from a `@Nullable C`, which may be `null`"),
            ("class C { Int n = 1; public void set(@Nullable C this) { n = 2; } }", "the field `n` is assigned through a `@Nullable C`, which may be `null`"),
        ],
        &["class C { public Int size(@Nullable C this) { if (this == null) { return 0; } return own() + this.own(); } Int own() { return 1; } }", "class C { Int n = 1; public Int size(@Nullable C this) { if (this == null) { return 0; } n = 2; return n + this.n; } }"],
    ),
    // ---- 5.4 ----
    rule(
        "05",
        "are null tests, with the operands in either order",
        &[],
        &["class A { Int f(@Nullable String s) { if (null != s) { return s.length(); } return 0; } Int g(@Nullable String s) { if (null == s) { return 0; } return s.length(); } Boolean h(@Nullable String s) { return s == null; } Boolean k(@Nullable String s) { return s != null; } }"],
    ),
    stated("05", "defines the paths", "It points to section 12.4, whose sentences have their own cases."),
    rule(
        "05",
        "carry the outcome of the left operand into the right operand",
        &[("class A { Boolean f(@Nullable String x) { return x == null && x.isEmpty(); } }", "`isEmpty` is sent to")],
        &["class A { Boolean f(@Nullable String x) { return x != null && x.isEmpty(); } Boolean g(@Nullable String x) { return x == null || x.isEmpty(); } }"],
    ),
    rule(
        "05",
        "Narrowing applies to a local and to a parameter, including `this` in a method with a `@Nullable` receiver",
        &[],
        &["class A { @Nullable String g() { return null; } Int f(@Nullable String p) { @Nullable String s = g(); if (s != null && p != null) { return s.length() + p.length(); } return 0; } Int h(@Nullable A this) { return this == null ? 0 : f(null); } }"],
    ),
    rule(
        "05",
        "The program copies the value to a local and tests the local",
        &[],
        &["class A { @Nullable String s = null; @Nullable String[] items = new @Nullable String[1]; Int f() { var held = s; var first = items[0]; if (held != null && first != null) { return held.length() + first.length(); } return 0; } }"],
    ),
    rule(
        "05",
        "narrows `x` to `T` where the test is known `true`, by the same path rule",
        &[("class A { Int f(Object o) { return o.length(); } }", "`Object` has no method named `length`"), ("class A { Int f(Object o) { if (o instanceof String) { } return o.length(); } }", "`Object` has no method named `length`")],
        &["class A { Int f(Object o) { if (o instanceof String) { return o.length(); } return 0; } Int g(Object o) { if (!(o instanceof String)) { return 0; } return o.length(); } }"],
    ),
    ran("05", "`a ?? b` evaluates `a`", RUN, "coalesce: some the right operand"),
    ran("05", "If its value is not `null`, that value is the result and `b` is not evaluated", RUN, "coalesce, left is not null:"),
    ran("05", "Otherwise the result is the value of `b`", RUN, "coalesce: some the right operand"),
    rule(
        "05",
        "The expression has the type that `x != null ? x : b` would have",
        &[("class A { String h(@Nullable String s, @Nullable String t) { return s ?? t; } }", "`@Nullable String` is not assignable to `String`")],
        &["class A { String f(@Nullable String s) { return s ?? \"x\"; } @Nullable String g(@Nullable String s, @Nullable String t) { return s ?? t; } Object h(@Nullable String s, Object o) { return s ?? o; } }"],
    ),
    ran("05", "binds more loosely than `||` and more tightly than `?:`", RUN, "precedence: c false true"),
    // ---- 5.5 ----
    ran("05", "A cast is a class test.", RUN, "casts: a String is not an Int"),
    ran("05", "because the class of `null` is `Null`", RUN, "cast of null: raises"),
    rule("05", "It is the checked way from `@Nullable T` to `T`", &[("class A { String f(@Nullable String s) { return s; } }", "`@Nullable String` is not assignable to `String`")], &["class A { String f(@Nullable String s) { return (String) s; } }"]),
    ran("05", "`(@Nullable T) e` accepts `null`", RUN, "nullable cast: true"),
    ran("05", "`null instanceof T` is `false` for every `T`", RUN, "instanceof null: false false"),
    // ---- 5.6 ----
    ran("05", "The nullability of a type argument is part of the class at run time", RUN, "type arguments: true false true false cleat.List<@cleat.Nullable cleat.String>"),
    ran("05", "are different classes, and a cast or an `instanceof` tells them apart", RUN, "type arguments: true false true false"),
    rule(
        "05",
        "Neither is a subtype of the other",
        &[
            ("class A { List<@Nullable String> f(List<String> xs) { return xs; } }", "`List<String>` is not assignable to `List<@Nullable String>`"),
            ("class A { List<String> f(List<@Nullable String> xs) { return xs; } }", "`List<@Nullable String>` is not assignable to `List<String>`"),
        ],
        &[],
    ),
    rule("05", "Where a type parameter is declared `out`, the subtyping of", &[], &["class A { Iterable<@Nullable String> f(Iterable<String> xs) { return xs; } Iterable<@Nullable Object> g(List<String> xs) { return xs; } }"]),
    rule(
        "05",
        "`Iterator<String>` is a subtype of `Iterator<@Nullable String>`",
        &[("class A { Iterator<String> f(Iterator<@Nullable String> it) { return it; } }", "`Iterator<@Nullable String>` is not assignable to `Iterator<String>`")],
        &["class A { Iterator<@Nullable String> f(Iterator<String> it) { return it; } }"],
    ),
    ran("05", "tests against the type argument of that instantiation", RUN, "generic cast to String: raises"),
    ran("05", "When the argument is `String`, a `null` raises `ClassCastException`", RUN, "generic cast to String: raises"),
    ran("05", "When the argument is `@Nullable String`, a `null` passes", RUN, "generic cast to @Nullable String: true"),
    // ---- 5.7 ----
    ran("05", "`Unit` is a final value class with no fields and one instance", RUN, "Unit: true cleat.Unit"),
    rule("05", "`void` is how the result type `Unit` is written on a method", &[], &["class A { void f() { } Function0<Unit> g() { return () -> f(); } Unit h() { Unit u = f(); } }"]),
    rule(
        "05",
        "and writing `Unit` as a method result means the same as `void`",
        &[],
        &["open class A { public open void f() { } public open Unit g() { } } class B extends A { @Override public Unit f() { } @Override public void g() { } }"],
    ),
    rule("05", "The body of such a method may end without `return`, or with `return;`", &[], &["class A { void f() { } void g() { return; } Unit h(Boolean b) { if (b) { return; } } }"]),
    ran("05", "Either returns the `Unit` instance", RUN, "Unit: true cleat.Unit"),
    rule("05", "may stand alone as an expression statement, because there is nothing to use", &[], &["class A { void f() { } static Unit g() { } void h() { f(); g(); this.f(); A.g(); } }"]),
    rule(
        "05",
        "a lambda that returns nothing has the type `Function0<Unit>`",
        &[],
        &["class A { Int n = 0; Function0<Unit> f() { return () -> { n += 1; }; } Function1<Int, Unit> g() { return (k) -> { n += k; }; } List<Unit> h() { return new List<Unit>(); } }"],
    ),
    rule("05", "There is no class `Void`", &[("class A { Void f() { return; } }", "there is no type named `Void` here")], &[]),
];
