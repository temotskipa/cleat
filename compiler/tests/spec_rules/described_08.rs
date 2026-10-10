//! Chapter 8, Annotations: the sentences that define and describe.

use super::{DISCARDED, Rule, SITE, held, ran, rule, stated};

const RUN: &str = "Sentences08";
const ONCE: &str = "an annotation is written at most once in one position";
const BEFORE: &str =
    "a declaration annotation is written before the modifiers of the declaration it marks";
const ELEMENT: &str = "an element is a numeric class, Boolean, Char, String, an enum, Class, an annotation, or an array of one of those";
const IMPLICIT: &str = "an `@Implicit` method is a static method of a value class `T` with one parameter of another value class, the result `T`, and no type parameters";
const NULLABLE_SEND: &str = "`length` is sent to a `@Nullable String`";
const PLAIN_TO_POSITIVE: &str = "`Int` is not assignable to `@Positive Int`";

/// The refinement `Positive`, its `@Narrows` method and a method that needs it, for
/// the cases of section 8.5.
macro_rules! positive {
    ($more:literal) => {
        concat!(
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } ",
            "static Int need(@Positive Int n) { return n; } ",
            $more,
            " }"
        )
    };
}

pub const SENTENCES: &[Rule] = &[
    // ---- 8.1 ----
    rule(
        "08",
        "An annotation is a modifier whose name a program can declare",
        &[],
        &["annotation Mine; @Mine class A { }"],
    ),
    rule(
        "08",
        "It is written before the thing it modifies, as `final` and `public` are",
        &[
            (
                "annotation M; class A { public final Int x @M = 1; }",
                "expected `;`, found `@`",
            ),
            (
                "annotation M; class A { public final Int @M x = 1; }",
                "expected a name, found `@`",
            ),
        ],
        &["annotation M; class A { @M public final Int x = 1; }"],
    ),
    stated(
        "08",
        "There are two kinds",
        "It introduces the two kinds after it, which have their own cases.",
    ),
    rule(
        "08",
        "A qualifier modifies a type, and it changes what the compiler accepts for values of that type",
        &[(
            "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } static Int f(Int n) { return need(n); } }",
            PLAIN_TO_POSITIVE,
        )],
        &[
            "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } static Int f(@Positive Int n) { return need(n); } }",
        ],
    ),
    rule(
        "08",
        "A declaration annotation marks a type declaration, a field, a method, a constructor or a parameter",
        &[],
        &["annotation M; @M class A { @M Int x = 1; @M A() { } @M void f(@M Int p) { } }"],
    ),
    ran(
        "08",
        "The program can read it at run time",
        RUN,
        "read at run time: people",
    ),
    rule(
        "08",
        "The declaration annotations of the prelude also have rules that the compiler applies",
        &[(
            "class A { @Override void f() { } }",
            "`f` carries `@Override` and overrides nothing",
        )],
        &["class A { @Override public String toString() { return \"a\"; } }"],
    ),
    rule(
        "08",
        "A program declares annotations of both kinds",
        &[],
        &["@Refines annotation Q; annotation M; @M class A { Int f(@Q Int n) { return n; } }"],
    ),
    held(
        "08",
        "lists the ones the prelude declares",
        "spec_tables",
        "the_prelude_declares_the_annotations_of_section_8_10",
    ),
    rule(
        "08",
        "No annotation runs code while the program is compiled",
        &[(
            "annotation M(Int n); class A { public static Int g() { return 1; } } @M(A.g()) class B { }",
            "an annotation argument is a constant expression, a class literal, or a use of an annotation",
        )],
        &[
            "annotation M(Int n); class A { public static final Int G = 1; } @M(A.G + 1) class B { }",
        ],
    ),
    // ---- 8.2 ----
    rule(
        "08",
        "An annotation declaration is a type declaration under the file rules of",
        &[
            (
                "==== a/Wrong.cleat\npackage a; public annotation Mark;",
                "the public type `Mark` must be in a file named `Mark.cleat`",
            ),
            (
                "==== a/Mark.cleat\npackage a; annotation Mark;\n==== b/B.cleat\npackage b; @a.Mark class B { }",
                "there is no type named `a.Mark`",
            ),
        ],
        &[
            "==== a/Mark.cleat\npackage a; public annotation Mark;\n==== b/B.cleat\npackage b; import a.Mark; @Mark class B { }",
        ],
    ),
    rule(
        "08",
        "A declaration may list elements in a header",
        &[],
        &["annotation Route(String path, Int priority = 0); @Route(\"/x\") class A { }"],
    ),
    rule(
        "08",
        "The type of an element is a numeric class, `Boolean`, `Char`, `String`, an enum, `Class`, an annotation, or an array of one of those, with no qualifier",
        &[
            ("annotation M(Object o);", ELEMENT),
            ("annotation M(@Nullable String s);", ELEMENT),
            ("annotation M(List<Int> l);", ELEMENT),
            ("annotation M(Int[][] l);", ELEMENT),
        ],
        &[
            "enum Mode { FAST } annotation In(String v); annotation M(Int a, UInt8 b, Float64 c, Rational r, Boolean d, Char e, String f, Mode g, Class h, In i, Int[] j, String[] k, Mode[] l, Class[] m, In[] n); class A { }",
        ],
    ),
    ran(
        "08",
        "An element may declare a default",
        RUN,
        "arguments: full_name/10",
    ),
    ran(
        "08",
        "An annotation declaration declares a final value class that implements the prelude interface `Annotation`",
        RUN,
        "the value: true true true",
    ),
    ran(
        "08",
        "Its elements are its fields, and each is public",
        RUN,
        "arguments: full_name/10 years/3",
    ),
    rule(
        "08",
        "A use supplies its arguments in element order",
        &[
            (
                "annotation Route(String path, Int priority = 0); @Route(\"/x\", priority = 2) class A { }",
                "expected `)`, found `=`",
            ),
            (
                "annotation Route(String path, Int priority = 0); @Route(path = \"/x\", 2) class A { }",
                "expected a name, found `2`",
            ),
        ],
        &[
            "annotation Route(String path, Int priority = 0); @Route(\"/x\", 2) class A { } @Route(path = \"/y\") class B { } @Route(priority = 3, path = \"/z\") class C { }",
        ],
    ),
    ran(
        "08",
        "An argument is a constant expression of",
        RUN,
        "kinds of argument: SAFE t.Person within",
    ),
    ran(
        "08",
        "For an array element it is a list of those in braces, and a single one may be written without the braces",
        RUN,
        "kinds of argument: SAFE t.Person within 1 one 2",
    ),
    rule(
        "08",
        "A use with no arguments writes no parentheses",
        &[(
            "annotation M; @M() class A { }",
            "an annotation with no arguments writes no parentheses",
        )],
        &["annotation M; @M class A { }"],
    ),
    stated(
        "08",
        "An annotation is written in one of two places",
        "It introduces the two places after it, which have their own cases.",
    ),
    rule(
        "08",
        "A declaration annotation is written before the modifiers of the declaration it marks",
        &[
            ("annotation M; class A { public @M Int x = 1; }", BEFORE),
            (
                "annotation M; class A { public @M static void f() { } }",
                BEFORE,
            ),
            (
                "annotation M; class A { void f(final @M Int p) { } }",
                BEFORE,
            ),
            ("annotation M; class A { public @M A() { } }", BEFORE),
            ("annotation M; open @M class A { }", BEFORE),
        ],
        &[
            "annotation M; @M class A { @M Int x = 1; @M A() { } @M void f(@M Int p) { } } @M interface I { } @M enum E { @M ONE } @M annotation N;",
            "annotation M; class A { @M public final Int x = 1; @M public static void f(@M final Int p) { } @M public A() { } } @M open class B { }",
        ],
    ),
    rule(
        "08",
        "A qualifier is written directly before the type it qualifies",
        &[(
            "annotation M; class A { Int f(List<@M Int> xs) { return 1; } }",
            "`@M` is not a qualifier, so it is not written on a type",
        )],
        &[
            "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } Int f(@Positive Int n) { return need(n); } }",
        ],
    ),
    rule(
        "08",
        "`@Positive Int`, `List<@Positive Int>`",
        &[],
        &[
            "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } Int f(@Positive Int n, List<@Positive Int> xs) { return need(n) + need(xs[0]); } }",
        ],
    ),
    rule(
        "08",
        "With an array type it qualifies the element type",
        &[(
            "class A { Int f(@Nullable String[] xs) { return xs[0].length(); } }",
            NULLABLE_SEND,
        )],
        &[
            "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } Int f(@Positive Int[] xs) { return need(xs[0]) + xs.length(); } }",
        ],
    ),
    rule(
        "08",
        "`Int @Positive []` qualifies the array",
        &[
            (
                "@Refines annotation Positive; class A { static Int need(@Positive Int n) { return n; } Int f(Int @Positive [] xs) { return need(xs[0]); } }",
                PLAIN_TO_POSITIVE,
            ),
            (
                "class A { Int f(Int @Nullable [] xs) { return xs.length(); } }",
                "`length` is sent to a `Int @Nullable []`",
            ),
        ],
        &["class A { Int f(Int @Nullable [] xs) { return xs == null ? 0 : xs.length(); } }"],
    ),
    rule(
        "08",
        "A qualifier may also be written among the modifiers of a field, a method, a parameter or a local",
        &[],
        &[
            "class A { @Nullable public String s; @Nullable public String find() { return null; } void f(@Nullable final String p) { @Nullable final String l = p; } }",
        ],
    ),
    rule(
        "08",
        "It then means what it would mean directly before that declaration's type, which for a method is its result type",
        &[(
            "class A { @Nullable public String find() { return null; } Int g() { return find().length(); } }",
            NULLABLE_SEND,
        )],
        &[
            "class A { @Nullable public String find() { return null; } Int g() { var s = find(); return s == null ? 0 : s.length(); } }",
        ],
    ),
    rule(
        "08",
        "are the same declaration",
        &[(
            "class A { @Nullable public String find(String key) { return null; } public @Nullable String find(String key) { return null; } }",
            "the method `find` is declared twice with the same parameter types",
        )],
        &[
            "open class A { @Nullable public open String find(String key) { return null; } } class B extends A { @Override public @Nullable String find(String key) { return null; } }",
        ],
    ),
    rule(
        "08",
        "An annotation is written at most once in one position",
        &[
            ("annotation M; @M @M class A { }", ONCE),
            (
                "annotation M(Int n = 0); class A { void f(@M(1) @M(2) Int p) { } }",
                ONCE,
            ),
            ("class A { @Nullable @Nullable String s; }", ONCE),
        ],
        &[
            "annotation M; annotation N; @M @N class A { @M Int x = 1; @M void f(@M @Nullable String p) { } }",
        ],
    ),
    rule(
        "08",
        "A declaration's modifiers and the annotations directly before its type are one position",
        &[
            ("class A { @Nullable public @Nullable String s; }", ONCE),
            (
                "@Refines annotation P; class A { Int f(@P @P Int n) { return n; } }",
                ONCE,
            ),
            (
                "class A { void f() { @Nullable @Nullable String s = null; } }",
                ONCE,
            ),
            (
                "class A { void f(List<@Nullable @Nullable String> xs) { } }",
                ONCE,
            ),
            ("class A { String @Nullable @Nullable [] xs; }", ONCE),
        ],
        &[
            "class A { @Nullable String @Nullable [] xs; void f(List<@Nullable String> a, @Nullable List<@Nullable String> b) { } }",
        ],
    ),
    rule(
        "08",
        "`@Target` on the declaration of a declaration annotation lists the sites where it may be written",
        &[(
            "@Target(Site.METHOD) annotation Route; @Route class A { }",
            SITE,
        )],
        &[
            "@Target(Site.METHOD) annotation Route; class A { @Route void f() { } }",
            "@Target({Site.TYPE, Site.FIELD}) annotation Both; @Both class A { @Both Int x = 1; }",
        ],
    ),
    held(
        "08",
        "The sites are the constants of the prelude enum `Site`",
        "spec_tables",
        "the_sites_of_section_8_2_are_the_constants_of_site",
    ),
    rule(
        "08",
        "A declaration annotation with no `@Target` may be written at every site",
        &[],
        &[
            "annotation M; @M class A { @M Int x = 1; @M A() { } @M void f(@M Int p) { } } @M interface I { } @M enum E { @M ONE } @M annotation N;",
        ],
    ),
    // ---- 8.3 ----
    rule(
        "08",
        "A qualifier is an annotation declared with exactly one of two meta-annotations",
        &[(
            "@Refines @Widens annotation Q;",
            "a qualifier is declared with exactly one of `@Refines` and `@Widens`",
        )],
        &[
            "@Refines annotation Q; @Widens annotation W; class A { Int f(@Q Int n, @W Int m) { return n; } }",
        ],
    ),
    rule(
        "08",
        "`@Refines` declares a refinement",
        &[],
        &["@Refines annotation Q; class A { Int f(@Q Int n) { Int m = n; return m; } }"],
    ),
    rule(
        "08",
        "also places `Q` below the refinement `R`, so `@Q T` is a subtype of `@R T`",
        &[(
            "@Refines annotation R; @Refines(R.class) annotation Q; class A { static Int need(@Q Int n) { return n; } Int f(@R Int n) { return need(n); } }",
            "`@R Int` is not assignable to `@Q Int`",
        )],
        &[
            "@Refines annotation R; @Refines(R.class) annotation Q; class A { static Int need(@R Int n) { return n; } Int f(@Q Int n) { return need(n); } }",
        ],
    ),
    rule(
        "08",
        "Several refinements may be listed",
        &[],
        &[
            "@Refines annotation R; @Refines annotation S; @Refines({R.class, S.class}) annotation Q; class A { static Int needR(@R Int n) { return n; } static Int needS(@S Int n) { return n; } Int f(@Q Int n) { return needR(n) + needS(n); } }",
        ],
    ),
    rule(
        "08",
        "`@Widens` declares a widening",
        &[(
            "@Widens annotation Raw; class A { String g(@Raw String s) { return s; } }",
            "`@Raw String` is not assignable to `String`",
        )],
        &["@Widens annotation Raw; class A { @Raw String f(String s) { return s; } }"],
    ),
    rule(
        "08",
        "A type may carry several qualifiers",
        &[],
        &[
            "@Refines annotation P; @Refines annotation E; class A { static Int needP(@P Int n) { return n; } static Int needE(@E Int n) { return n; } Int f(@P @E Int n) { return needP(n) + needE(n); } }",
        ],
    ),
    rule(
        "08",
        "A qualifier applied to a type that already has it leaves the type unchanged",
        &[(
            "@Refines annotation R; @Refines(R.class) annotation Q; class Box<T> { public @R T get(@R T x) { return x; } } class A { static Int needQ(@Q Int n) { return n; } Int f(Box<Int> b, @R Int n) { return needQ(b.get(n)); } }",
            "`@R Int` is not assignable to `@Q Int`",
        )],
        &[
            "@Refines annotation P; class Box<T> { public @P T get(@P T x) { return x; } } class A { static Int need(@P Int n) { return n; } Int f(Box<@P Int> b, @P Int n) { return need(b.get(n)); } }",
            "@Refines annotation R; @Refines(R.class) annotation Q; class Box<T> { public @R T get(@R T x) { return x; } } class A { static Int needQ(@Q Int n) { return n; } Int f(Box<@Q Int> b, @Q Int n) { return needQ(b.get(n)); } }",
        ],
    ),
    rule(
        "08",
        "`S` is a subtype of `T` when that holds for the two types with their qualifiers removed",
        &[(
            "@Refines annotation P; class A { String f(@P Int n) { return n; } }",
            "`@P Int` is not assignable to `String`",
        )],
        &["@Refines annotation P; class A { Object f(@P Int n) { return n; } }"],
    ),
    rule(
        "08",
        "every refinement on `T` is also on `S`, or is above a refinement on `S`",
        &[(
            "@Refines annotation P; class A { static Int need(@P Int n) { return n; } Int g(Int n) { return need(n); } }",
            "`Int` is not assignable to `@P Int`",
        )],
        &[
            "@Refines annotation R; @Refines(R.class) annotation Q; class A { static Int need(@R Int n) { return n; } Int f(@Q Int a, @R Int b) { return need(a) + need(b); } }",
        ],
    ),
    rule(
        "08",
        "every widening on `S` is also on `T`",
        &[(
            "@Widens annotation Raw; class A { Object f(@Raw String s) { return s; } }",
            "`@Raw String` is not assignable to `Object`",
        )],
        &["@Widens annotation Raw; class A { @Raw Object f(@Raw String s) { return s; } }"],
    ),
    rule(
        "08",
        "Qualifiers are part of the types in a signature",
        &[(
            "@Refines annotation P; class A { Int f(@P Int n) { return n; } Int g(Int n) { return f(n); } }",
            "`Int` is not assignable to `@P Int`",
        )],
        &[
            "@Refines annotation P; class A { Int f(@P Int n) { return n; } Int g(@P Int n) { return f(n); } }",
        ],
    ),
    rule(
        "08",
        "An override repeats them",
        &[(
            "@Refines annotation P; open class A { public open Int f(@P Int n) { return n; } } class B extends A { @Override public Int f(Int n) { return n; } }",
            "`f` carries `@Override` and overrides nothing",
        )],
        &[
            "@Refines annotation P; open class A { public open Int f(@P Int n) { return n; } } class B extends A { @Override public Int f(@P Int n) { return n; } }",
        ],
    ),
    rule(
        "08",
        "`var` infers a type with its qualifiers",
        &[(
            "class A { @Nullable String maybe() { return null; } Int f() { var s = maybe(); return s.length(); } }",
            NULLABLE_SEND,
        )],
        &[
            "@Refines annotation P; class A { static Int need(@P Int n) { return n; } Int f(@P Int n) { var m = n; return need(m); } }",
        ],
    ),
    rule(
        "08",
        "Qualifiers are part of a type argument",
        &[
            (
                "@Refines annotation P; class A { List<@P Int> f(List<Int> xs) { return xs; } }",
                "`List<Int>` is not assignable to `List<@P Int>`",
            ),
            (
                "@Refines annotation P; class A { List<Int> f(List<@P Int> xs) { return xs; } }",
                "`List<@P Int>` is not assignable to `List<Int>`",
            ),
        ],
        &["@Refines annotation P; class A { List<@P Int> f(List<@P Int> xs) { return xs; } }"],
    ),
    rule(
        "08",
        "A parameter declared `out` or `in` uses the subtype rule above, and so does the bound of a wildcard",
        &[
            (
                "@Refines annotation P; interface Source<out T> { T get(); } class A { Source<@P Int> f(Source<Int> s) { return s; } }",
                "`Source<Int>` is not assignable to `Source<@P Int>`",
            ),
            (
                "@Refines annotation P; class A { List<? extends @P Int> h(List<Int> xs) { return xs; } }",
                "`List<Int>` is not assignable to `List<? extends @P Int>`",
            ),
        ],
        &[
            "@Refines annotation P; interface Source<out T> { T get(); } interface Sink<in T> { void put(T x); } class A { Source<Int> f(Source<@P Int> s) { return s; } Sink<@P Int> g(Sink<Int> s) { return s; } List<? extends Int> h(List<@P Int> xs) { return xs; } }",
        ],
    ),
    ran(
        "08",
        "The qualifiers of a type argument are part of the reified argument",
        RUN,
        "a qualified type argument: true false false",
    ),
    ran(
        "08",
        "are different classes at run time, and a cast or an `instanceof` tells them apart",
        RUN,
        "a qualified type argument: true false false, a cast is accepted refused",
    ),
    ran(
        "08",
        "gives the one exception",
        RUN,
        "a cast to a nullable type: accepts null, refuses null",
    ),
    // ---- 8.4 ----
    rule(
        "08",
        "A method may declare its receiver as a first parameter named `this`",
        &[],
        &[
            "@Refines annotation Priced; class Order { public Int unitPrice(@Priced Order this) { return 1; } }",
        ],
    ),
    rule(
        "08",
        "Its type is the declaring class, with qualifiers",
        &[(
            "@Refines annotation Priced; class Order { public Int f(@Priced String this) { return 1; } }",
            "the receiver parameter has the type of the declaring class",
        )],
        &[
            "@Refines annotation Priced; class Order { public Int f(@Priced Order this) { return 1; } public Boolean absent(@Nullable Order this) { return this == null; } }",
        ],
    ),
    rule(
        "08",
        "A method with no receiver parameter has the declaring class, without qualifiers, as its receiver type",
        &[(
            "class Order { Int f() { return 1; } static Int g(@Nullable Order o) { return o.f(); } }",
            "`f` is sent to a `@Nullable Order`",
        )],
        &["class Order { Int f() { return 1; } static Int g(Order o) { return o.f(); } }"],
    ),
    stated(
        "08",
        "Three consequences follow from",
        "It introduces the three consequences after it, which have their own cases.",
    ),
    rule(
        "08",
        "Every method can be sent to a receiver that has a refinement",
        &[],
        &[
            "@Refines annotation Priced; class Order { Int f() { return 1; } static Int g(@Priced Order o) { return o.f() + o.hashCode(); } }",
        ],
    ),
    rule(
        "08",
        "An override has the same receiver qualifiers as the method it overrides",
        &[(
            "@Refines annotation Priced; open class Order { public open Int price(@Priced Order this) { return 1; } } class Rush extends Order { @Override public Int price() { return 2; } }",
            "an override has the same receiver qualifiers as the method it overrides",
        )],
        &[
            "@Refines annotation Priced; open class Order { public open Int price(@Priced Order this) { return 1; } } class Rush extends Order { @Override public Int price(@Priced Rush this) { return 2; } }",
        ],
    ),
    // ---- 8.5 ----
    rule(
        "08",
        "marks a method whose result is `Boolean`",
        &[(
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Int isPositive(Int n) { return n; } }",
            "a `@Narrows` method returns `Boolean`",
        )],
        &[
            "@Refines annotation Positive; class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } }",
        ],
    ),
    rule(
        "08",
        "The subject of the method is its receiver, or its first parameter when the method is static",
        &[],
        &[
            "@Refines annotation Priced; class Order { @Narrows(Priced.class) public Boolean isPriced() { return true; } public Int price(@Priced Order this) { return 1; } static Int g(Order o) { if (o.isPriced()) { return o.price(); } return 0; } }",
            positive!("static Int f(Int n) { if (isPositive(n)) { return need(n); } return 0; }"),
        ],
    ),
    rule(
        "08",
        "A `true` result narrows the subject",
        &[(
            positive!("static Int f(Int n) { if (!isPositive(n)) { return need(n); } return 0; }"),
            PLAIN_TO_POSITIVE,
        )],
        &[positive!(
            "static Int f(Int n) { return isPositive(n) ? need(n) : 0; }"
        )],
    ),
    rule(
        "08",
        "when `Q` is a refinement, the subject gains `Q`",
        &[],
        &[positive!(
            "static Int f(Int n) { if (isPositive(n)) { @Positive Int m = n; return need(m); } return 0; }"
        )],
    ),
    rule(
        "08",
        "when `Q` is a widening, the subject loses `Q`",
        &[(
            "@Widens annotation Raw; class Checks { @Narrows(Raw.class) public static Boolean isClean(@Raw String s) { return true; } } class A { String f(@Raw String s) { if (!Checks.isClean(s)) { return s; } return \"\"; } }",
            "`@Raw String` is not assignable to `String`",
        )],
        &[
            "@Widens annotation Raw; class Checks { @Narrows(Raw.class) public static Boolean isClean(@Raw String s) { return true; } } class A { String f(@Raw String s) { if (Checks.isClean(s)) { return s; } return \"\"; } }",
        ],
    ),
    rule(
        "08",
        "A `false` result narrows nothing",
        &[(
            positive!(
                "static Int f(Int n) { if (isPositive(n)) { return 0; } else { return need(n); } }"
            ),
            PLAIN_TO_POSITIVE,
        )],
        &[],
    ),
    rule(
        "08",
        "Narrowing follows the path rule of",
        &[(
            positive!(
                "static Int f(Int n) { if (isPositive(n)) { n = n - 1; return need(n); } return 0; }"
            ),
            PLAIN_TO_POSITIVE,
        )],
        &[positive!(
            "static Int f(Int n) { if (!isPositive(n)) { return 0; } return need(n); } static Int h(Int n) { while (!isPositive(n)) { n = n + 1; } return need(n); }"
        )],
    ),
    rule(
        "08",
        "method is declared in the package that declares `Q`",
        &[(
            "==== a/Positive.cleat\npackage a; @Refines public annotation Positive;\n==== b/Counts.cleat\npackage b; import a.Positive; public class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } }",
            "a `@Narrows` method is declared in the package that declares its qualifier",
        )],
        &[
            "==== a/Positive.cleat\npackage a; @Refines public annotation Positive;\n==== a/Counts.cleat\npackage a; public class Counts { @Narrows(Positive.class) public static Boolean isPositive(Int n) { return n > 0; } }",
        ],
    ),
    stated(
        "08",
        "That package decides what `Q` means",
        "It says who is answerable for a qualifier's meaning. The rule that enforces it is the sentence before it.",
    ),
    stated(
        "08",
        "is responsible for it, as the author of `equals` is responsible for the obligations of",
        "It places an obligation on an author, and the sentence before it says the language does not check it.",
    ),
    // ---- 8.6 ----
    rule(
        "08",
        "`@Nullable` is declared in the prelude as a widening",
        &[(
            "class A { String g(@Nullable String s) { return s; } }",
            "`@Nullable String` is not assignable to `String`",
        )],
        &["class A { @Nullable String f(String s) { return s; } }"],
    ),
    rule(
        "08",
        "and its rule for sends on a nullable receiver is the receiver rule of",
        &[(
            "class A { Int f(@Nullable String s) { return s.length(); } }",
            NULLABLE_SEND,
        )],
        &[
            "class A { Boolean absent(@Nullable A this) { return this == null; } static Boolean g(@Nullable A a) { return a.absent(); } }",
        ],
    ),
    stated(
        "08",
        "`@Nullable` differs from a qualifier a program declares in two ways",
        "It introduces the two ways after it, which have their own cases.",
    ),
    stated(
        "08",
        "Both follow from `null` being built into the language",
        "It gives the reason for the two differences, which have their own cases.",
    ),
    rule(
        "08",
        "`@Nullable T` contains `null`",
        &[(
            "class A { String s = null; }",
            "`null` is not a value of `String`",
        )],
        &["class A { @Nullable String s = null; }"],
    ),
    rule(
        "08",
        "No other qualifier adds a value to a type",
        &[(
            "@Widens annotation Raw; class A { @Raw String s = null; }",
            "`null` is not a value of `@Raw String`",
        )],
        &[],
    ),
    rule(
        "08",
        "For that reason a cast may name `@Nullable` outside a type argument",
        &[],
        &["class A { @Nullable String f(Object o) { return (@Nullable String) o; } }"],
    ),
    rule(
        "08",
        "narrow without a `@Narrows` method, and they narrow on both outcomes",
        &[],
        &[
            "class A { Int f(@Nullable String s) { if (s == null) { return 0; } return s.length(); } Int g(@Nullable String s) { if (s != null) { return s.length(); } return 0; } Int h(@Nullable String s) { return s == null ? 0 : s.length(); } }",
        ],
    ),
    // ---- 8.7 ----
    rule(
        "08",
        "A declaration annotation written on a class, an interface or an enum is a tag of that type",
        &[(
            "@Target(Site.TYPE) annotation Frozen; class A { } class S<@Frozen T> { } class U { void f(S<A> a) { } }",
            "`A` does not carry the tag `@Frozen` that the type parameter requires",
        )],
        &[
            "@Target(Site.TYPE) annotation Frozen; @Frozen class A { } @Frozen interface I { } @Frozen enum E { ONE } class S<@Frozen T> { } class U { void f(S<A> a, S<I> i, S<E> e) { } }",
        ],
    ),
    stated(
        "08",
        "A tag is a claim by the author of the declaration",
        "It says what a tag stands for. The compiler checks that a tag is written, and never what it claims.",
    ),
    rule(
        "08",
        "A type parameter may require a tag",
        &[],
        &[
            "@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class Pair<@Frozen A, B> { <@Frozen C> void f(C c) { } }",
        ],
    ),
    rule(
        "08",
        "The requirement names the annotation and no arguments, and a use with any arguments meets it",
        &[(
            "@Target(Site.TYPE) annotation Kind(String name = \"k\"); class S<@Kind(\"x\") T> { }",
            "a requirement names the annotation and no arguments",
        )],
        &[
            "@Target(Site.TYPE) annotation Kind(String name = \"k\"); @Kind(\"special\") class A { } @Kind class B { } class S<@Kind T> { } class U { void f(S<A> a, S<B> b) { } }",
        ],
    ),
    stated(
        "08",
        "gives the rule for instantiation",
        "It points to chapter 7, whose sentences have their own cases.",
    ),
    rule(
        "08",
        "A tag declared with `@Inherited` is also carried by every declaration that extends or implements a marked one",
        &[],
        &[
            "@Target(Site.TYPE) @Inherited annotation Frozen; @Frozen open class A { } class B extends A { } @Frozen interface I { } class C implements I { } class S<@Frozen T> { } class U { void f(S<B> b, S<C> c) { } }",
        ],
    ),
    ran(
        "08",
        "A use written on the declaration itself takes the place of an inherited one",
        RUN,
        "an inherited tag: people pupils",
    ),
    // ---- 8.8 ----
    rule(
        "08",
        "`@Override` marks a method that overrides a superclass method or implements an interface method",
        &[],
        &[
            "open class B { public open void f() { } } interface I { void g(); } class D extends B implements I { @Override public void f() { } @Override public void g() { } }",
        ],
    ),
    rule(
        "08",
        "`@Discardable` marks a method whose result a caller may ignore",
        &[(
            "class A { Int f() { return 1; } void g() { f(); } }",
            DISCARDED,
        )],
        &["class A { @Discardable Int f() { return 1; } void g() { f(); } }"],
    ),
    rule(
        "08",
        "A call of it may stand alone as a statement, as",
        &[(
            "class A { public static Int f() { return 1; } } class B { void g() { A.f(); } }",
            DISCARDED,
        )],
        &[
            "class A { @Discardable public static Int f() { return 1; } } class B { void g() { A.f(); } }",
        ],
    ),
    ran(
        "08",
        "`@Implicit` marks a static method that the compiler may call on its own to convert a value",
        RUN,
        "implicit: Meters(3)",
    ),
    rule(
        "08",
        "The method is declared in a value class `T`",
        &[
            (
                "class Wrap { public Int n; public Wrap(Int n) { this.n = n; } @Implicit public static Wrap from(Int whole) { return new Wrap(whole); } }",
                IMPLICIT,
            ),
            (
                "value class Meters { public Int length; @Implicit public Meters from(Int whole) { return new Meters(whole); } }",
                IMPLICIT,
            ),
        ],
        &[
            "value class Meters { public Int length; @Implicit public static Meters from(Int whole) { return new Meters(whole); } } class A { Meters f() { return 3; } }",
        ],
    ),
    rule(
        "08",
        "It has one parameter, whose type is another value class `S`, the result type `T`, and no type parameters",
        &[
            (
                "value class Meters { public Int length; @Implicit public static Meters from(Int whole, Int more) { return new Meters(whole); } }",
                IMPLICIT,
            ),
            (
                "value class Meters { public Int length; @Implicit public static Meters from(Object whole) { return new Meters(1); } }",
                IMPLICIT,
            ),
            (
                "value class Meters { public Int length; @Implicit public static Meters from(Meters whole) { return whole; } }",
                IMPLICIT,
            ),
            (
                "value class Meters { public Int length; @Implicit public static Int from(Int whole) { return whole; } }",
                IMPLICIT,
            ),
            (
                "value class Meters { public Int length; @Implicit public static <T> Meters from(Int whole) { return new Meters(whole); } }",
                IMPLICIT,
            ),
        ],
        &[],
    ),
    ran(
        "08",
        "It declares an implicit conversion from `S` to `T`",
        RUN,
        "implicit: Meters(3) Meters(5)",
    ),
    stated(
        "08",
        "A conversion that can fail or lose information is not marked `@Implicit`",
        "It is a promise by the method's author, and the paragraph says the language does not check it.",
    ),
    held(
        "08",
        "`@Intrinsic` marks a method or a constructor of the prelude whose body the implementation supplies",
        "prelude_intrinsics",
        "every_intrinsic_of_the_prelude_is_listed_with_its_reason",
    ),
    held(
        "08",
        "The declaration ends in `;` where its body would be",
        "spec_rules",
        "the_prelude_is_source_the_checker_accepts",
    ),
    rule(
        "08",
        "`@Deprecated` marks a type, a field, a method or a constructor",
        &[("class A { void f(@Deprecated Int p) { } }", SITE)],
        &[
            "@Deprecated class Old { @Deprecated Int x = 1; @Deprecated Old() { } @Deprecated(\"use g\") void f() { } }",
        ],
    ),
    // ---- 8.9 ----
    ran(
        "08",
        "A program reads a declaration annotation at run time",
        RUN,
        "read at run time: people",
    ),
    ran(
        "08",
        "and reaches the type's annotated fields and methods through mirrors",
        RUN,
        "fields: name age",
    ),
    ran(
        "08",
        "The type argument `A` names the annotation that a call asks for",
        RUN,
        "methods: greet | quiet",
    ),
    ran(
        "08",
        "`getAnnotation` returns the use of `A` on the declaration that the receiver stands for, or `null` when there is none",
        RUN,
        "none: true",
    ),
    ran(
        "08",
        "On a class object it includes a tag the type inherits under",
        RUN,
        "an inherited tag: people",
    ),
    ran(
        "08",
        "Every instantiation of a generic type returns the same result",
        RUN,
        "generic: boxes boxes true",
    ),
    ran(
        "08",
        "`getAnnotatedFields` returns a mirror for each field that the type declares with a use of `A`, in source order",
        RUN,
        "fields: name age count(static) hidden fixed",
    ),
    ran(
        "08",
        "Static fields and enum constants are included",
        RUN,
        "enum constants: LOW(static)",
    ),
    ran(
        "08",
        "`getAnnotatedMethods` returns a mirror for each method that the type declares with a use of `A`, in source order",
        RUN,
        "methods: greet | quiet twice(static) fails same secret",
    ),
    ran(
        "08",
        "`getSuperclass` returns the class object of the superclass",
        RUN,
        "superclass: t.Person",
    ),
    ran(
        "08",
        "It returns `null` for `Object` and for an interface",
        RUN,
        "superclass: t.Person true true",
    ),
    ran(
        "08",
        "`getInterfaces` returns the class objects of the interfaces that the declaration names, in the order written",
        RUN,
        "interfaces: 2 t.Shape t.Labelled",
    ),
    ran(
        "08",
        "A reader that wants the annotated members a type inherits follows these",
        RUN,
        "declared here: 0, and in the superclass: 5",
    ),
    ran(
        "08",
        "`getParameters` returns a mirror for each parameter of the method, in order",
        RUN,
        "parameters: 2 other times whom true",
    ),
    ran(
        "08",
        "A qualifier belongs to a type and is not read this way",
        RUN,
        "a qualifier: 0 true",
    ),
    ran(
        "08",
        "Each call returns a new annotation value, and an array element of it is a new array",
        RUN,
        "a new value: true",
    ),
    ran(
        "08",
        "A program that modifies such an array changes no later result",
        RUN,
        "a new value: true changed one",
    ),
    ran(
        "08",
        "`get` reads the field, and `set` assigns it",
        RUN,
        "get and set: 36 37 37",
    ),
    ran(
        "08",
        "`invoke` calls the method with the arguments given, dispatching as",
        RUN,
        "invoke: Ada greets Bo x2 | the student Cy greets Bo",
    ),
    ran(
        "08",
        "For a `void` method that is the `Unit` value",
        RUN,
        "a void method: true true",
    ),
    ran(
        "08",
        "`target` is the instance, and it is not used for a static member",
        RUN,
        "a static member: 42 42 5 5",
    ),
    ran(
        "08",
        "An exception that the method raises propagates unchanged",
        RUN,
        "an exception: from the method",
    ),
    stated(
        "08",
        "A mirror checks what the compiler would have checked for a direct use",
        "It introduces the table after it, whose rows have their own cases.",
    ),
    ran(
        "08",
        "| The member is not `public`, or `set` names a final field | `IllegalAccessException` |",
        RUN,
        "illegal access: a field that is not public, a method that is not public, a final field",
    ),
    ran(
        "08",
        "| `target` is not an instance of the declaring type, or a value or an argument is not of the type the member declares | `ClassCastException` |",
        RUN,
        "class cast: the target, a value, an argument",
    ),
    ran(
        "08",
        "| The number of arguments is not the number of parameters, or the method is generic | `IllegalArgumentException` |",
        RUN,
        "illegal argument: the number of arguments, a generic method",
    ),
    ran(
        "08",
        "The type test is the test of a cast",
        RUN,
        "the test of a cast: a subclass is accepted, null is refused for a String",
    ),
    ran(
        "08",
        "No implicit conversion applies, so an `Int32` is not accepted for an `Int` parameter",
        RUN,
        "the test of a cast: a subclass is accepted, null is refused for a String, an Int32 is refused for an Int",
    ),
    ran(
        "08",
        "Mirrors read annotations and use the members that carry them",
        RUN,
        "get and set: 36 37",
    ),
    // ---- 8.10 ----
    ran(
        "08",
        "These declarations are ordinary prelude source, and a program reads the declaration annotations among them as it reads its own",
        RUN,
        "the prelude's annotations: 1 TYPE true use Person",
    ),
    stated(
        "08",
        "Their rules are the compiler's",
        "It points to sections 8.2 to 8.8, whose sentences have their own cases.",
    ),
    stated(
        "08",
        "An annotation that a program declares has the rules of this chapter and no others",
        "It sums up sections 8.2, 8.7 and 8.9, whose sentences have their own cases.",
    ),
    stated(
        "08",
        "What it means beyond that is decided by the code that reads it",
        "It leaves a meaning to the program, and so states nothing a compiler or a run can show.",
    ),
];
