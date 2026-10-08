//! Chapter 10, Omissions: the sentences that define and describe.

use super::{held, rule, stated, Rule};

const LEFT: &str = "It leaves this to the implementation, and names the chapter that says what a program may rely on.";

pub const SENTENCES: &[Rule] = &[
    stated("10", "Chapters 1 through 9 and 11 through 13 are the language", "It says how the document is arranged."),
    stated("10", "This chapter lists what they leave out", "It says how the document is arranged."),
    // ---- 10.1 ----
    stated("10", "These are absent by design", "It introduces the list after it, whose items have their own cases."),
    rule("10", "**Primitive types and boxing.**", &[("class A { int x = 1; }", "`int` is a reserved word"), ("class A { boolean b = true; }", "`boolean` is a reserved word")], &["class A { Object f(Int n, Boolean b) { Object o = n; o = b; return o; } }"]),
    rule(
        "10",
        "**Raw types and erasure.**",
        &[("class A { void f(List xs) { } }", "there are no raw types")],
        &["class A { Boolean f(Object o) { return o instanceof List<Int>; } }"],
    ),
    rule(
        "10",
        "There are no nested, inner, local or anonymous classes",
        &[
            ("class A { class B { } }", "a type is not declared inside another type"),
            ("class A { static class B { } }", "a type is not declared inside another type"),
            ("class A { void f() { class B { } } }", "a type is not declared inside a method"),
            ("class A { Object f() { return new Object() { }; } }", "expected `;`, found `{`"),
        ],
        &[],
    ),
    rule("10", "A lambda covers the common use of an anonymous class", &[], &["interface Check { Boolean test(Int n); } class A { Check f() { return (n) -> n > 0; } }"]),
    rule(
        "10",
        "There is no `synchronized`, no `wait` or `notify` on `Object`, and no `volatile`",
        &[
            ("class A { synchronized void f() { } }", "there is no modifier `synchronized`"),
            ("class A { volatile Int x = 1; }", "there is no modifier `volatile`"),
            ("class A { void f(Object o) { o.wait(); } }", "`Object` has no method named `wait`"),
            ("class A { void f(Object o) { o.notify(); } }", "`Object` has no method named `notify`"),
        ],
        &[],
    ),
    rule(
        "10",
        "has `Lock`, `Condition` and atomic cells",
        &[],
        &["class A { final Lock lock = new Lock(); AtomicInt n = new AtomicInt(0); Atomic<String> s = new Atomic<String>(\"a\"); Condition c() { return lock.newCondition(); } }"],
    ),
    rule(
        "10",
        "Integer division is written by name",
        &[("class A { Int f(Int a, Int b) { return a / b; } }", "`Int` has no method `div`")],
        &["class A { Int f(Int a, Int b) { return a.floorDiv(b) + a.truncatingDiv(b); } }"],
    ),
    rule(
        "10",
        "unary `+` and a power operator",
        &[
            ("class A { Int f(Int a) { return a >>> 1; } }", "`>>>` is not an operator"),
            ("class A { Int f(Int n) { return +n; } }", "there is no unary `+`"),
            ("class A { Int f(Int a, Int b) { return a ** b; } }", "expected an expression, found `*`"),
        ],
        &[],
    ),
    rule("10", "**Switch fallthrough.**", &[("class A { void f(Int n) { switch (n) { case 1: break; } } }", "expected `->`, found `:`")], &["class A { void f(Int n) { switch (n) { case 1 -> { } default -> { } } } }"]),
    rule("10", "**Octal literals and literal suffixes.**", &[("class A { Int f() { return 017; } }", "a decimal numeral has no leading zero"), ("class A { Int f() { return 1L; } }", "a numeric literal has no suffix"), ("class A { Float32 f() { return 1.5f; } }", "a numeric literal has no suffix")], &[]),
    rule(
        "10",
        "and a serialization protocol built into the language",
        &[
            ("class A { Object f(Object o) { return o.clone(); } }", "`Object` has no method named `clone`"),
            ("class A { void f(Object o) { o.finalize(); } }", "`Object` has no method named `finalize`"),
            ("class A { void f(WeakReference<String> w) { } }", "there is no type named `WeakReference` here"),
            ("class A implements Serializable { }", "there is no type named `Serializable` here"),
        ],
        &[],
    ),
    rule("10", "A `Char` is one Unicode scalar", &[("class A { Char c = '\\uD83D'; }", "this escape names a surrogate")], &["class A { Char c = '\\u{1F600}'; Int n = '\\u{1F600}'.code(); }"]),
    rule("10", "**`default` on an interface method.**", &[("interface I { default Int f() { return 1; } }", "`default` is not written on an interface method")], &["interface I { Int f() { return 1; } }"]),
    rule(
        "10",
        "**Code that runs at compile time.**",
        &[("annotation Getter; class A { @Getter Int x = 1; } class B { Int f(A a) { return a.getX(); } }", "`A` has no method named `getX`")],
        &[],
    ),
    rule("10", "The compiler sees the whole program", &[("class A { void f() { var c = Class.forName(\"A\"); } }", "`Class` has no static method named `forName`"), ("class A { void f(ClassLoader l) { } }", "there is no type named `ClassLoader` here")], &[]),
    // ---- 10.2 ----
    stated("10", "These may be added later", "It introduces the list after it, whose items have their own cases."),
    rule("10", "Loading classes at run time, and compiling while the program runs", &[("class A { void f() { var c = Class.forName(\"A\"); } }", "`Class` has no static method named `forName`")], &[]),
    rule("10", "methods whose lambda arguments can `return` from the caller", &[("class A { inline void each(Function0<Unit> body) { body.invoke(); } }", "`inline` is a reserved word")], &[]),
    rule("10", "`inline` is reserved for them", &[("class A { Int inline = 1; }", "`inline` is a reserved word")], &[]),
    rule("10", "A null-safe member access such as `?.`, and string interpolation", &[("class A { Int f(@Nullable String s) { return s?.length(); } }", "expected an expression, found `.`")], &["class A { String f(Int n) { return \"${n} and $n are text\"; } }"]),
    rule("10", "Properties, or separate audiences for reading and assigning a field", &[("class A { public private Int x = 1; }", "a declaration has one audience"), ("class A { public Int x { get; set; } }", "expected `;`, found `{`")], &[]),
    rule(
        "10",
        "Mirrors that describe the types of fields and parameters, construct objects, or enumerate the types of a program",
        &[
            ("class A { void f(Field x) { var t = x.getType(); } }", "`Field` has no method named `getType`"),
            ("class A { void f(Parameter x) { var t = x.getType(); } }", "`Parameter` has no method named `getType`"),
            ("class A { Object f(Class c) { return c.newInstance(); } }", "`Class` has no method named `newInstance`"),
        ],
        &[],
    ),
    rule(
        "10",
        "Repeatable annotations, and declaration annotations on locals and type parameters",
        &[("annotation M; @M @M class A { }", "an annotation is written at most once in one position"), ("annotation M; class A { void f() { @M Int x = 1; } }", "a declaration annotation is not written on a local")],
        &[],
    ),
    rule(
        "10",
        "A qualifier among the modifiers of a local, and a tag that a type parameter requires, are in the language",
        &[],
        &["@Refines annotation P; annotation Tag; class Box<@Tag T> { } class A { void f(@P Int n) { @P Int m = n; @Nullable String s = null; final @Nullable String t = s; } }"],
    ),
    rule("10", "Pinning an object for longer than one foreign call, and reading memory through a `Pointer`", &[("class A { Int f(Pointer p) { return p.read(); } }", "`Pointer` has no method named `read`")], &[]),
    rule("10", "Callbacks from C into Cleat", &[("class A { foreign static void f(Function0<Int> g); }", "a parameter of a `foreign` method is a machine type or an array of one")], &[]),
    rule("10", "Vector, complex and matrix classes", &[("class A { void f(Vector v) { } }", "there is no type named `Vector` here"), ("class A { void f(Complex c, Matrix m) { } }", "there is no type named `Complex` here")], &[]),
    rule(
        "10",
        "They are library classes over `Numeric<T>`",
        &[],
        &["value class Complex<T extends Numeric<T>> { public T re; public T im; public Complex<T> plus(Complex<T> o) { return new Complex<T>(re + o.re, im + o.im); } } class A { Complex<Float64> f(Complex<Float64> a) { return a + a; } }"],
    ),
    // ---- 10.3 ----
    stated("10", "An implementation chooses each of the following", "It introduces the list after it."),
    stated("10", "A program may rely on what the chapter cited defines and on nothing more", "It limits what a program may assume, and no program can be shown to break it."),
    stated("10", "How an instance is stored", LEFT),
    stated("10", "How instantiations of a generic type are compiled", LEFT),
    stated("10", "How storage is reclaimed, and how much there is", LEFT),
    stated("10", "The characters of the default `toString` of a reference class, and of a lambda", LEFT),
    stated("10", "The value of a default `hashCode`, and of `hashCode` for `null`", "It leaves this to the implementation."),
    stated("10", "How a `Rational` is stored", LEFT),
    stated("10", "What is recorded about where an exception was created", LEFT),
    stated("10", "How threads are scheduled, and how many run at once", LEFT),
    stated("10", "How the entry class and the libraries for foreign methods are named when a program is built", LEFT),
    stated("10", "The platform's C calling convention", LEFT),
    stated("10", "The limit on the depth of calls", "It leaves this to the implementation."),
    held("10", "Exceeding it ends the program", "host", "a_full_stack_ends_the_program"),
];
