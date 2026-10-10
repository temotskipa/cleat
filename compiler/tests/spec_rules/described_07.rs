//! Chapter 7, Generics: the sentences that define and describe.

use super::{Rule, ran, rule, stated};

const RUN: &str = "Sentences07";
const INFER: &str = "cannot be inferred from this call";
const UNKNOWN: &str = "is not assignable to `an unknown type`";
const OUT_CONSUMED: &str = "`T` is declared `out` and is written where a value is consumed";
const IN_PRODUCED: &str = "`T` is declared `in` and is written where a value is produced";
const OUT_BOTH: &str =
    "`T` is declared `out` and is written where a value is both produced and consumed";

/// A program after the two shapes most cases of this chapter use.
macro_rules! shapes {
    ($more:literal) => {
        concat!(
            "open class Shape { } class Circle extends Shape { } ",
            $more
        )
    };
}

/// A program after the shapes and an interface with an `in` parameter.
macro_rules! sinks {
    ($more:literal) => {
        concat!(
            "open class Shape { } class Circle extends Shape { } interface Snk<in T> { void put(T x); } ",
            "class Bin<T> implements Snk<T> { @Override public void put(T x) { } } ",
            $more
        )
    };
}

pub const SENTENCES: &[Rule] = &[
    // ---- 7.1 ----
    rule(
        "07",
        "A class, an interface and a method may declare type parameters",
        &[],
        &["class Box<T> { } interface Src<T> { } class A { static <T> T id(T x) { return x; } }"],
    ),
    rule(
        "07",
        "A use of a generic class or interface writes one type argument for each parameter",
        &[(
            "class Box<T> { } class A { void f(Box<String, Int> b) { } }",
            "`Box` takes 1 type arguments, not 2",
        )],
        &[
            "class Box<T> { public T v; public Box(T v) { this.v = v; } } class A { Box<String> f(String s) { return new Box<String>(s); } }",
        ],
    ),
    rule(
        "07",
        "The one exception is the class name that qualifies a static member",
        &[(
            "class Box<T> { public static Int count = 0; } class A { Int f() { return Box<Int>.count; } }",
            "expected an expression, found `.`",
        )],
        &[
            "class Box<T> { public static Int count = 0; public static Int next() { return 1; } } class A { Int f() { return Box.count + Box.next(); } }",
        ],
    ),
    rule(
        "07",
        "There are no raw types",
        &[(
            "class A { void f(List xs) { } }",
            "`List` is generic and is written with its type arguments; there are no raw types",
        )],
        &["class A { void f(List<Int> xs) { } }"],
    ),
    rule(
        "07",
        "A type argument is a type: a class or interface type, an array type, or a type parameter that is in scope, each with its qualifiers",
        &[(
            "class A { void f(List<Missing> xs) { } }",
            "there is no type named `Missing` here",
        )],
        &[
            "class A<T> { void f(List<String> a, List<Iterable<Int>> b, List<Int[]> c, List<T> d, List<@Nullable String> e) { } }",
        ],
    ),
    rule(
        "07",
        "In some positions a type argument may instead be a wildcard",
        &[],
        &["class A { void f(List<?> xs, List<? extends Object> ys, List<? super Int> zs) { } }"],
    ),
    rule(
        "07",
        "The type parameters of a class are in scope in its `extends` and `implements` clauses, its instance members and its constructors",
        &[],
        &[
            "interface Src<T> { T get(); } open class Base<T> { } class Box<T> extends Base<T> implements Src<T> { T v; Box(T v) { this.v = v; } @Override public T get() { return v; } }",
        ],
    ),
    rule(
        "07",
        "The type parameters of a method are in scope in its signature and its body",
        &[(
            "class A { static <T> T id(T x) { return x; } void g(T other) { } }",
            "there is no type named `T` here",
        )],
        &[
            "class A { static <T> List<T> one(T x) { List<T> xs = new List<T>(); xs.add(x); return xs; } }",
        ],
    ),
    ran(
        "07",
        "says what that means at run time",
        RUN,
        "classes: false true true",
    ),
    // ---- 7.2 ----
    rule(
        "07",
        "A type parameter may declare a bound, or several bounds joined by `&`",
        &[(
            "interface P { } interface Q { } class OnlyP implements P { } class A { static <T extends P & Q> T pick(T x) { return x; } static OnlyP f(OnlyP b) { return pick(b); } }",
            INFER,
        )],
        &[
            "interface P { } interface Q { } class Both implements P, Q { } class A { static <T extends P & Q> T pick(T x) { return x; } static Both f(Both b) { return pick(b); } }",
        ],
    ),
    rule(
        "07",
        "At most one bound is a class",
        &[(
            "open class C1 { } open class C2 { } class A { static <T extends C1 & C2> void f(T x) { } }",
            "at most one bound is a class; the others are interfaces",
        )],
        &["open class C1 { } class A { static <T extends C1> void f(T x) { } }"],
    ),
    rule(
        "07",
        "The others are interfaces",
        &[],
        &[
            "open class C1 { } interface P { } interface Q { } class A { static <T extends C1 & P & Q> void f(T x) { } static <U extends P & Q> void g(U x) { } }",
        ],
    ),
    rule(
        "07",
        "A type parameter with no bound has the bound `@Nullable Object`, so every type is an argument for it",
        &[],
        &[
            "class Box<T> { } class A { void f(Box<@Nullable String> a, Box<Int> b, Box<Null> c, Box<Box<@Nullable Object>> d) { } }",
        ],
    ),
    rule(
        "07",
        "A bound written without `@Nullable` excludes `@Nullable` arguments",
        &[(
            "class Box<T extends Object> { } class A { void f(Box<@Nullable String> a) { } }",
            "`@Nullable String` is not within the bound `Object` of the type parameter `T`",
        )],
        &["class Box<T extends Object> { } class A { void f(Box<String> a) { } }"],
    ),
    rule(
        "07",
        "Inside the declaration, a value of type `T` has the members of the bounds of `T`",
        &[(
            "class A { static <T> String f(T x) { return x.name(); } }",
            "`T` has no method named `name`",
        )],
        &[
            "interface Named { String name(); } class A { static <T extends Named> String f(T x) { return x.name(); } }",
        ],
    ),
    rule(
        "07",
        "With no bound, those are the methods of `Object` that take a `@Nullable` receiver",
        &[(
            "class A { static <T> Class f(T x) { return x.getClass(); } }",
            "`getClass` is sent to a `T`, which may be `null`",
        )],
        &[
            "class A { static <T> String f(T x) { return x.toString() + x.hashCode() + x.equals(x) + x.identical(x); } }",
        ],
    ),
    rule(
        "07",
        "A tag written before the name of a type parameter is a further requirement",
        &[(
            "@Target(Site.TYPE) annotation Frozen; class Snapshot<@Frozen T> { } class A { void f(Snapshot<String> s) { } }",
            "`String` does not carry the tag `@Frozen` that the type parameter requires",
        )],
        &[
            "@Target(Site.TYPE) annotation Frozen; @Frozen class Bill { } class Snapshot<@Frozen T> { } class A { void f(Snapshot<Bill> s) { } }",
        ],
    ),
    // ---- 7.3 ----
    rule(
        "07",
        "A type parameter of a class or an interface is invariant unless it is declared `out` or `in`",
        &[(
            "class Box<T> { } class A { Box<Object> f(Box<String> b) { return b; } }",
            "`Box<String>` is not assignable to `Box<Object>`",
        )],
        &["class Box<T> { } class A { Box<String> f(Box<String> b) { return b; } }"],
    ),
    rule(
        "07",
        "`T` may be the result type of a method and the type of a final field",
        &[
            ("interface Bad<out T> { void put(T x); }", OUT_CONSUMED),
            (
                "class Bad<out T> { T v; Bad(T v) { this.v = v; } }",
                OUT_BOTH,
            ),
        ],
        &["class Src<out T> { final T v; Src(T v) { this.v = v; } public T get() { return v; } }"],
    ),
    rule(
        "07",
        "`T` may be the type of a method parameter",
        &[("interface Bad<in T> { T get(); }", IN_PRODUCED)],
        &["interface Snk<in T> { void put(T x); }"],
    ),
    rule(
        "07",
        "An invariant parameter may be written in either kind of position",
        &[],
        &["interface Cell<T> { T get(); void put(T x); List<T> all(); }"],
    ),
    rule(
        "07",
        "When `T` is written as a type argument, its position is judged through the parameter that receives it",
        &[("interface Bad<out T> { List<T> all(); }", OUT_BOTH)],
        &["interface Src<out T> { Iterable<T> all(); }"],
    ),
    rule(
        "07",
        "An `out` parameter keeps the direction of the position",
        &[("interface Bad<in T> { Iterable<T> all(); }", IN_PRODUCED)],
        &["interface Src<out T> { Iterable<Iterable<T>> all(); }"],
    ),
    rule(
        "07",
        "An `in` parameter reverses it",
        &[(
            "interface Snk<in T> { void put(T x); } interface Bad<out T> { Snk<T> sink(); }",
            OUT_CONSUMED,
        )],
        &["interface Snk<in T> { void put(T x); } interface Src<out T> { void each(Snk<T> s); }"],
    ),
    rule(
        "07",
        "is a subtype of `G<B>` when this holds for each type parameter",
        &[(
            shapes!(
                "interface Fn<in A, out R> { R call(A a); } class X { Fn<Shape, String> f(Fn<Circle, String> g) { return g; } }"
            ),
            "`Fn<Circle, String>` is not assignable to `Fn<Shape, String>`",
        )],
        &[shapes!(
            "interface Fn<in A, out R> { R call(A a); } class X { Fn<Circle, Object> f(Fn<Shape, String> g) { return g; } }"
        )],
    ),
    rule(
        "07",
        "for an invariant parameter, `A` and `B` are the same type, with the same qualifiers, or `B` is a wildcard that contains `A` under",
        &[
            (
                "class A { List<Object> f(List<String> x) { return x; } }",
                "`List<String>` is not assignable to `List<Object>`",
            ),
            (
                "class A { List<@Nullable String> f(List<String> x) { return x; } }",
                "`List<String>` is not assignable to `List<@Nullable String>`",
            ),
        ],
        &[
            "class A { List<String> f(List<String> x) { return x; } List<? extends Object> g(List<String> x) { return x; } }",
        ],
    ),
    rule(
        "07",
        "for an `out` parameter, `A` is a subtype of `B`",
        &[(
            "class A { Iterable<String> f(Iterable<Object> x) { return x; } }",
            "`Iterable<Object>` is not assignable to `Iterable<String>`",
        )],
        &["class A { Iterable<Object> f(Iterable<String> x) { return x; } }"],
    ),
    rule(
        "07",
        "for an `in` parameter, `B` is a subtype of `A`",
        &[(
            sinks!("class A { Snk<Shape> f(Snk<Circle> x) { return x; } }"),
            "`Snk<Circle>` is not assignable to `Snk<Shape>`",
        )],
        &[sinks!(
            "class A { Snk<Circle> f(Snk<Shape> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "An instantiation is also a subtype of the supertypes its declaration names, with the arguments substituted",
        &[(
            "class A { Iterable<Int> f(List<String> xs) { return xs; } }",
            "`List<String>` is not assignable to `Iterable<Int>`",
        )],
        &["class A { Iterable<String> f(List<String> xs) { return xs; } }"],
    ),
    rule(
        "07",
        "is a subtype of `Iterable<String>`, and so of `Iterable<Object>`, because `Iterable` declares `out T`",
        &[],
        &[
            "class A { Iterable<String> f(List<String> xs) { return xs; } Iterable<Object> g(List<String> xs) { return xs; } }",
        ],
    ),
    // ---- 7.4 ----
    rule(
        "07",
        "A type argument may be a wildcard",
        &[],
        &["class A { static Int size(List<?> items) { return items.size(); } }"],
    ),
    rule(
        "07",
        "`? extends U` stands for some subtype of `U`",
        &[(
            shapes!(
                "class A { static Shape first(List<? extends Shape> xs) { return xs[0]; } static Shape f(List<String> c) { return first(c); } }"
            ),
            "`List<String>` is not assignable to `List<? extends Shape>`",
        )],
        &[shapes!(
            "class A { static Shape first(List<? extends Shape> xs) { return xs[0]; } static Shape f(List<Circle> c) { return first(c); } }"
        )],
    ),
    rule(
        "07",
        "`? super L` stands for some supertype of `L`",
        &[(
            shapes!(
                "class A { static void fill(List<? super Circle> sink) { sink.add(new Circle()); } static void f(List<String> a) { fill(a); } }"
            ),
            "`List<String>` is not assignable to `List<? super Circle>`",
        )],
        &[shapes!(
            "class A { static void fill(List<? super Circle> sink) { sink.add(new Circle()); } static void f(List<Shape> a, List<Object> b, List<Circle> c) { fill(a); fill(b); fill(c); } }"
        )],
    ),
    rule(
        "07",
        "`?` stands for some type that the parameter accepts",
        &[],
        &[
            "class A { static Int size(List<?> xs) { return xs.size(); } static Int f(List<String> a, List<@Nullable Int> b) { return size(a) + size(b); } }",
        ],
    ),
    rule(
        "07",
        "`U` and `L` are types within that bound",
        &[
            (
                "open class Shape { } class Box<T extends Shape> { } class A { void f(Box<? extends String> a) { } }",
                "`String` is not within the bound `Shape` of the type parameter `T`",
            ),
            (
                "open class Shape { } class Box<T extends Shape> { } class A { void f(Box<? super Object> a) { } }",
                "`Object` is not within the bound `Shape` of the type parameter `T`",
            ),
        ],
        &[shapes!(
            "class Box<T extends Shape> { } class A { void f(Box<? extends Circle> a, Box<? super Circle> b) { } }"
        )],
    ),
    rule(
        "07",
        "is the type of every `List<X>` whose `X` is a subtype of `Shape`",
        &[(
            "open class Shape { } class A { void f(List<Object> a) { List<? extends Shape> x = a; } }",
            "`List<Object>` is not assignable to `List<? extends Shape>`",
        )],
        &[shapes!(
            "class A { void f(List<Circle> a, List<Shape> b) { List<? extends Shape> x = a; List<? extends Shape> y = b; } }"
        )],
    ),
    stated(
        "07",
        "A type that has a wildcard among its arguments is a wildcard type",
        "It names a kind of type. The next sentence says what follows from the name, and has its own case.",
    ),
    rule(
        "07",
        "It is a type for variables and parameters and not a class",
        &[(
            "class A { Class f() { return List<?>.class; } }",
            "a wildcard is not written among the arguments of a class literal",
        )],
        &[
            "class A { List<?> field = new List<Int>(); List<?> f(List<?> p) { List<?> l = p; return l; } }",
        ],
    ),
    rule(
        "07",
        "A wildcard may be written wherever a type argument is written, with four exceptions that need a type",
        &[],
        &[
            "class A { List<?> field = new List<Int>(); List<?> f(List<?> p) { List<?> l = p; return l; } Boolean g(Object o) { return o instanceof List<?>; } List<List<?>> h(Object o) { return (List<List<?>>) o; } }",
        ],
    ),
    rule(
        "07",
        "the arguments of the class in `new`;",
        &[(
            "class A { List<?> f() { return new List<?>(); } }",
            "a wildcard is not written among the arguments of the class in `new`",
        )],
        &[],
    ),
    rule(
        "07",
        "the explicit type arguments of a call;",
        &[(
            "class A { static <T> T id(T x) { return x; } String f() { return A.<?>id(\"a\"); } }",
            "a wildcard is not written as an explicit type argument of a call",
        )],
        &[],
    ),
    rule(
        "07",
        "the arguments of a type that an `extends` or `implements` clause names;",
        &[
            (
                "class X implements Iterable<?> { @Override public Iterator<@Nullable Object> iterator() { return new List<@Nullable Object>().iterator(); } }",
                "a wildcard is not written for the arguments of a supertype",
            ),
            (
                "open class Base<T> { } class X extends Base<?> { }",
                "a wildcard is not written for the arguments of a superclass",
            ),
        ],
        &[],
    ),
    rule(
        "07",
        "the arguments of a class literal.",
        &[(
            "class A { Class f() { return List<?>.class; } }",
            "a wildcard is not written among the arguments of a class literal",
        )],
        &["class A { Class f() { return List<List<?>>.class; } }"],
    ),
    ran(
        "07",
        "creates a list whose elements are lists of any kind",
        RUN,
        "lists of any kind: 2 cleat.List<cleat.List<?>>",
    ),
    rule(
        "07",
        "A parameter declared `out` or `in` already varies in one direction, so a bounded wildcard adds nothing to it",
        &[],
        &[sinks!(
            "class A { Iterable<Shape> f(Iterable<? extends Shape> x) { return x; } Iterable<? extends Shape> g(Iterable<Shape> x) { return x; } Snk<Circle> h(Snk<? super Circle> x) { return x; } List<Snk<? super Circle>> i(List<Snk<Circle>> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "`?` may be written for any parameter",
        &[],
        &[
            "interface Snk<in T> { void put(T x); } class A { void f(List<?> a, Iterable<?> b, Snk<?> c) { } }",
        ],
    ),
    rule(
        "07",
        "is the same argument as `? extends B`, so `Iterable<?>` is `Iterable<@Nullable Object>`",
        &[],
        &[
            "class A { List<Iterable<?>> f(List<Iterable<@Nullable Object>> x) { return x; } List<Iterable<@Nullable Object>> g(List<Iterable<?>> x) { return x; } List<List<?>> h(List<List<? extends @Nullable Object>> x) { return x; } }",
            "open class Shape { } class Box<T extends Shape> { } class A { List<Box<?>> f(List<Box<? extends Shape>> x) { return x; } List<Box<? extends Shape>> g(List<Box<?>> x) { return x; } }",
        ],
    ),
    rule(
        "07",
        "is the type of every `Ordered`, which no other spelling gives",
        &[(
            "interface Snk<in T> { void put(T x); } class A { void f(Snk<@Nullable Object> s) { } void g(Snk<String> b) { f(b); } }",
            "`Snk<String>` is not assignable to `Snk<@Nullable Object>`",
        )],
        &[
            "open class Shape { } interface Snk<in T> { void put(T x); } class A { void f(Snk<?> s) { } void g(Snk<Shape> a, Snk<String> b, Snk<@Nullable Object> c) { f(a); f(b); f(c); } }",
        ],
    ),
    stated(
        "07",
        "compares the arguments of an invariant parameter for sameness",
        "It points back to section 7.3, whose sentences have their own cases.",
    ),
    stated(
        "07",
        "With wildcards the rule is containment",
        "It introduces the rule after it, which has its own case.",
    ),
    rule(
        "07",
        "is a subtype of `G<B>` when, for each invariant parameter, `B` contains `A`",
        &[(
            "class A { Map<? extends Object, String> f(Map<String, Int> m) { return m; } }",
            "`Map<String, Int>` is not assignable to `Map<? extends Object, String>`",
        )],
        &["class A { Map<? extends Object, ? super Int> f(Map<String, Int> m) { return m; } }"],
    ),
    rule(
        "07",
        "`? extends U` contains a type `S`, and the wildcard `? extends S`, when `S` is a subtype of `U`",
        &[(
            "open class Shape { } class A { List<? extends Shape> f(List<Object> x) { return x; } }",
            "`List<Object>` is not assignable to `List<? extends Shape>`",
        )],
        &[shapes!(
            "class A { List<? extends Shape> f(List<Circle> x) { return x; } List<? extends Shape> g(List<? extends Circle> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "`? super L` contains a type `S`, and the wildcard `? super S`, when `L` is a subtype of `S`",
        &[(
            shapes!("class A { List<? super Shape> g(List<? super Circle> x) { return x; } }"),
            "`List<? super Circle>` is not assignable to `List<? super Shape>`",
        )],
        &[shapes!(
            "class A { List<? super Circle> f(List<Shape> x) { return x; } List<? super Circle> g(List<? super Shape> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "`?` contains every argument",
        &[],
        &[shapes!(
            "class A { List<?> f(List<String> x) { return x; } List<?> g(List<? extends Shape> x) { return x; } List<?> h(List<? super Circle> x) { return x; } List<?> i(List<?> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "`List<Circle>` is a subtype of `List<?>`, of `List<? extends Shape>` and of `List<? super Circle>`",
        &[],
        &[shapes!(
            "class A { List<?> f(List<Circle> x) { return x; } List<? extends Shape> g(List<Circle> x) { return x; } List<? super Circle> h(List<Circle> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "`List<? extends Circle>` is a subtype of `List<? extends Shape>`",
        &[(
            shapes!("class A { List<? extends Circle> g(List<? extends Shape> x) { return x; } }"),
            "`List<? extends Shape>` is not assignable to `List<? extends Circle>`",
        )],
        &[shapes!(
            "class A { List<? extends Shape> g(List<? extends Circle> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "It is a subtype of `List<? extends List<?>>`",
        &[],
        &[shapes!(
            "class A { List<? extends List<?>> f(List<List<Circle>> x) { return x; } }"
        )],
    ),
    rule(
        "07",
        "A wildcard type has the supertypes that its declaration names, read as the result type of a member is read below",
        &[(
            shapes!("class A { Iterable<Circle> h(List<? extends Shape> xs) { return xs; } }"),
            "`List<? extends Shape>` is not assignable to `Iterable<Circle>`",
        )],
        &[shapes!(
            "class A { Iterable<Shape> f(List<? extends Shape> xs) { return xs; } Iterable<@Nullable Object> g(List<?> xs) { return xs; } }"
        )],
    ),
    ran(
        "07",
        "so a `for` statement visits its elements as shapes",
        RUN,
        "a for statement: 6",
    ),
    rule(
        "07",
        "`List<? extends Object>` accepts no list of a `@Nullable` type",
        &[(
            "class A { void f(List<? extends Object> xs) { } void g(List<@Nullable String> ys) { f(ys); } }",
            "`List<@Nullable String>` is not assignable to `List<? extends Object>`",
        )],
        &["class A { void f(List<? extends Object> xs) { } void g(List<String> ys) { f(ys); } }"],
    ),
    rule(
        "07",
        "`List<?>` accepts every list",
        &[],
        &[
            "class A { void f(List<?> xs) { } void g(List<@Nullable String> a, List<String> b, List<List<?>> c) { f(a); f(b); f(c); } }",
        ],
    ),
    stated(
        "07",
        "each wildcard stands for one unknown type, called `X` here",
        "It names the unknown type that the sentences after it reason with, and they have their own cases.",
    ),
    stated(
        "07",
        "The checker knows the bounds of `X` and nothing else",
        "It introduces the bounds, which the sentence after it lists and which has its own case.",
    ),
    rule(
        "07",
        "`X` is a subtype of `U` for `? extends U`, a supertype of `L` for `? super L`, and within the parameter's bound in every case",
        &[(
            shapes!("class A { Circle f(List<? extends Shape> xs) { return xs[0]; } }"),
            "`Shape` is not assignable to `Circle`",
        )],
        &[shapes!(
            "class Box<T extends Shape> { public T v; public Box(T v) { this.v = v; } } class A { Shape f(List<? extends Shape> xs) { return xs[0]; } void g(List<? super Circle> s) { s.add(new Circle()); } Shape h(Box<?> b) { return b.v; } }"
        )],
    ),
    rule(
        "07",
        "The member's signature is read with `X` in place of the type parameter",
        &[(
            shapes!("class A { void f(List<? extends Shape> xs) { xs.add(xs.get(0)); } }"),
            "`Shape` is not assignable to `an unknown type`",
        )],
        &["class A { Int n(List<?> xs) { return xs.size() + xs.get(0).hashCode(); } }"],
    ),
    rule(
        "07",
        "Where the parameter's type is `X`, a subtype of `L` is assignable when the wildcard is `? super L`",
        &[(
            shapes!("class A { void f(List<? super Circle> sink) { sink.add(new Shape()); } }"),
            "`Shape` is not assignable to `an unknown type`",
        )],
        &[
            "open class Shape { } open class Circle extends Shape { } class Dot extends Circle { } class A { void f(List<? super Circle> sink) { sink.add(new Circle()); sink.add(new Dot()); } }",
        ],
    ),
    rule(
        "07",
        "Nothing is assignable when the wildcard is `? extends U` or `?`",
        &[
            (
                shapes!("class A { void f(List<? extends Shape> xs) { xs.add(new Circle()); } }"),
                UNKNOWN,
            ),
            ("class A { void f(List<?> ys) { ys.add(\"s\"); } }", UNKNOWN),
        ],
        &[],
    ),
    rule(
        "07",
        "The result has the result type as read",
        &[],
        &[shapes!(
            "class A { Shape f(List<? extends Shape> shapes) { Shape first = shapes[0]; return first; } }"
        )],
    ),
    rule(
        "07",
        "`X` on its own, or as the argument of an `out` parameter, becomes its upper bound",
        &[],
        &[shapes!(
            "class A { Shape a(List<? extends Shape> xs) { return xs.get(0); } Iterator<Shape> b(List<? extends Shape> xs) { return xs.iterator(); } }"
        )],
    ),
    rule(
        "07",
        "`U`, or the parameter's bound",
        &[(
            "class A { Object c(List<?> xs) { return xs.get(0); } }",
            "`@Nullable Object` is not assignable to `Object`",
        )],
        &[
            "open class Shape { } class Box<T extends Shape> { T v; Box(T v) { this.v = v; } public T get() { return v; } } class A { @Nullable Object c(List<?> xs) { return xs.get(0); } Shape d(Box<?> b) { return b.get(); } }",
        ],
    ),
    rule(
        "07",
        "`X` as the argument of an invariant parameter becomes the wildcard it came from",
        &[(
            "open class Shape { } class Box<T> { public List<T> all() { return new List<T>(); } } class A { List<Shape> g(Box<? extends Shape> b) { return b.all(); } }",
            "`List<? extends Shape>` is not assignable to `List<Shape>`",
        )],
        &[
            "open class Shape { } class Box<T> { public List<T> all() { return new List<T>(); } } class A { List<? extends Shape> f(Box<? extends Shape> b) { return b.all(); } }",
        ],
    ),
    rule(
        "07",
        "`X` as the argument of an `in` parameter becomes `L` when it has that lower bound, and `?` otherwise",
        &[(
            sinks!(
                "class Box<T> { public Snk<T> sink() { return new Bin<T>(); } } class A { Snk<Shape> h(Box<? extends Shape> b) { return b.sink(); } }"
            ),
            "`Snk<?>` is not assignable to `Snk<Shape>`",
        )],
        &[sinks!(
            "class Box<T> { public Snk<T> sink() { return new Bin<T>(); } } class A { Snk<Circle> f(Box<? super Circle> b) { return b.sink(); } Snk<?> g(Box<? extends Shape> b) { return b.sink(); } }"
        )],
    ),
    rule(
        "07",
        "A field is read and assigned by the same reading",
        &[(
            shapes!(
                "class Cell<T> { public T v; public Cell(T v) { this.v = v; } } class A { void h(Cell<? extends Shape> c) { c.v = new Circle(); } }"
            ),
            UNKNOWN,
        )],
        &[shapes!(
            "class Cell<T> { public T v; public Cell(T v) { this.v = v; } } class A { Shape f(Cell<? extends Shape> c) { return c.v; } void g(Cell<? super Circle> c) { c.v = new Circle(); } }"
        )],
    ),
    rule(
        "07",
        "Each evaluation of an expression has an unknown of its own",
        &[(
            "class A { void f(List<?> items) { var x = items.get(0); items.add(x); } }",
            "`@Nullable Object` is not assignable to `an unknown type`",
        )],
        &[],
    ),
    rule(
        "07",
        "A generic method says it",
        &[],
        &[
            "class A { static <T> void swap(List<T> items, Int i, Int j) { T held = items[i]; items[i] = items[j]; items[j] = held; } static void swapFirstTwo(List<?> items) { swap(items, 0, 1); } }",
        ],
    ),
    rule(
        "07",
        "An argument of a wildcard type may be given for a parameter whose type has a type parameter of the method in that position",
        &[],
        &[
            "open class Shape { } class A { static <T> Int count(List<T> xs) { return xs.size(); } Int f(List<?> a, List<? extends Shape> b, List<? super Shape> c) { return count(a) + count(b) + count(c); } }",
        ],
    ),
    rule(
        "07",
        "The type parameter is then the unknown type of that argument",
        &[],
        &[
            "open class Shape { } class A { static <T> T firstOf(List<T> xs) { return xs[0]; } Shape f(List<? extends Shape> xs) { return firstOf(xs); } }",
        ],
    ),
    ran(
        "07",
        "At run time it is the type argument of the object that was passed",
        RUN,
        "the unknown type: cleat.List<t.Circle> cleat.List<cleat.String>",
    ),
    rule(
        "07",
        "The result type of such a call is written without `X`, as a member's is",
        &[(
            "open class Shape { } class A { static <T> List<T> same(List<T> xs) { return xs; } List<Shape> g(List<? extends Shape> xs) { return same(xs); } }",
            INFER,
        )],
        &[
            "open class Shape { } class A { static <T> List<T> same(List<T> xs) { return xs; } List<? extends Shape> f(List<? extends Shape> xs) { return same(xs); } }",
        ],
    ),
    // ---- 7.5 ----
    rule(
        "07",
        "A call of a generic method writes all of its type arguments or none of them",
        &[(
            "class A { static <P, Q> P two(P a, Q b) { return a; } String f() { return A.<String>two(\"a\", 1); } }",
            "method `two` has 2 type parameters; a call writes all of its type arguments or none",
        )],
        &[
            "class A { static <P, Q> P two(P a, Q b) { return a; } String f() { return A.<String, Int>two(\"a\", 1) + two(\"b\", 2); } }",
        ],
    ),
    rule(
        "07",
        "Written arguments follow the `.` of the call",
        &[(
            "class A { static <T> T id(T x) { return x; } String f() { return id<String>(\"a\"); } }",
            "`id` is not a variable or a field in scope",
        )],
        &[
            "class A { static <T> T id(T x) { return x; } <T> T mine(T x) { return x; } String f() { return A.<String>id(\"a\") + this.<String>mine(\"b\"); } }",
        ],
    ),
    rule(
        "07",
        "`Array.<String>build(n, f)`",
        &[],
        &["class A { String[] f(Int n) { return Array.<String>build(n, (i) -> \"x\"); } }"],
    ),
    rule(
        "07",
        "`new` always writes the type arguments of a generic class",
        &[(
            "class Box<T> { T v; public Box(T v) { this.v = v; } } class A { Object f() { return new Box(1); } }",
            "`Box` is generic and is written with its type arguments; there are no raw types",
        )],
        &[
            "class Box<T> { T v; public Box(T v) { this.v = v; } } class A { Box<Int> f() { return new Box<Int>(1); } }",
        ],
    ),
    rule(
        "07",
        "`new Box<Int>(1)`",
        &[],
        &[
            "class Box<T> { T v; public Box(T v) { this.v = v; } } class A { Box<Int> f() { var b = new Box<Int>(1); return b; } }",
        ],
    ),
    ran(
        "07",
        "When a call writes none, they are inferred",
        RUN,
        "a lower bound: cleat.String",
    ),
    stated(
        "07",
        "Each type parameter `P` of the method collects constraints from the call",
        "It introduces the constraints after it, which have their own cases.",
    ),
    ran(
        "07",
        "with or without qualifiers, makes `A` a lower bound of `P`",
        RUN,
        "a lower bound: cleat.String cleat.String t.Circle",
    ),
    ran(
        "07",
        "is matched against that type, through the supertype of the argument's type that instantiates the same generic type",
        RUN,
        "through a supertype: cleat.String",
    ),
    ran(
        "07",
        "The type argument found opposite `P` fixes `P` when the position is invariant",
        RUN,
        "an invariant position: cleat.String t.Shape",
    ),
    ran(
        "07",
        "It is a lower bound of `P` when the position is `out`, and an upper bound when it is `in`",
        RUN,
        "out and in: t.Shape t.Circle",
    ),
    ran(
        "07",
        "When `P` is nested more deeply, the positions on the way combine as",
        RUN,
        "nested: t.Shape",
    ),
    ran(
        "07",
        "A parameter type that writes `? extends P` puts `P` in an `out` position, and one that writes `? super P` puts it in an `in` position",
        RUN,
        "wildcard parameters: t.Shape t.Circle",
    ),
    ran(
        "07",
        "the wildcard's upper bound is what an `out` position finds, and its lower bound is what an `in` position finds",
        RUN,
        "wildcard arguments: t.Shape t.Circle",
    ),
    rule(
        "07",
        "A wildcard with no bound on that side gives no constraint",
        &[(
            "open class Shape { } class A { static <P> void consumer(List<? super P> a) { } void f(List<? extends Shape> xs) { consumer(xs); } }",
            INFER,
        )],
        &[
            "open class Shape { } class A { static <P> void consumer(List<? super P> a) { } void f(List<? super Shape> xs) { consumer(xs); } }",
        ],
    ),
    ran(
        "07",
        "In an invariant position the wildcard fixes `P` to the unknown type of that argument",
        RUN,
        "wildcard arguments: t.Shape t.Circle t.Circle",
    ),
    ran(
        "07",
        "The expected type of the call, when there is one, constrains `P` through the method's result type",
        RUN,
        "the expected type: cleat.String",
    ),
    ran(
        "07",
        "It is an upper bound when the result type is `P`, and it is matched as an argument is when the result type has `P` as a type argument",
        RUN,
        "the expected type: cleat.String cleat.List<cleat.String> t.Shape",
    ),
    ran(
        "07",
        "A lambda argument is considered after the other arguments",
        RUN,
        "a lambda: cleat.String -> cleat.Int",
    ),
    ran(
        "07",
        "A parameter type that the lambda writes fixes the type parameter in that position",
        RUN,
        "a lambda: cleat.String -> cleat.Int | cleat.String -> cleat.Int",
    ),
    rule(
        "07",
        "The type of the lambda's result is then a lower bound for the type parameter in the result position",
        &[(
            "class A { static <X, Y> List<Y> map(List<X> items, Function1<X, Y> f) { return new List<Y>(); } List<String> g(List<String> names) { return map(names, (s) -> s.length()); } }",
            "`Int` is not assignable to `String`",
        )],
        &[
            "class A { static <X, Y> List<Y> map(List<X> items, Function1<X, Y> f) { return new List<Y>(); } List<Int> g(List<String> names) { var lengths = map(names, (s) -> s.length()); return lengths; } }",
        ],
    ),
    ran(
        "07",
        "A numeric literal argument for a parameter of type `P` is considered last",
        RUN,
        "a literal: cleat.Int8",
    ),
    rule(
        "07",
        "If `P` is already determined, the literal is checked against that type, which is its expected type under",
        &[(
            "class A { static <P> P two(P a, P b) { return a; } Int8 f(Int8 small) { return two(small, 1000); } }",
            "the literal is outside the range of `Int8`",
        )],
        &[
            "class A { static <P> P two(P a, P b) { return a; } Int8 f(Int8 small) { return two(small, 1); } }",
        ],
    ),
    ran(
        "07",
        "Otherwise the literal's default class is a lower bound of `P`",
        RUN,
        "a literal: cleat.Int8 cleat.Int cleat.Rational",
    ),
    stated(
        "07",
        "Each `P` is then determined by the first of these rules that applies",
        "It introduces the three rules after it, which have their own cases.",
    ),
    ran(
        "07",
        "If the constraints fix `P`, `P` is that type",
        RUN,
        "an invariant position: cleat.String t.Shape",
    ),
    ran(
        "07",
        "If `P` has lower bounds, `P` is the lower bound that every other lower bound is a subtype of",
        RUN,
        "lower bounds: t.Shape t.Shape",
    ),
    ran(
        "07",
        "When no lower bound is such a type, `P` is the upper bound that the expected type gave, if it gave one",
        RUN,
        "no greatest lower bound: cleat.Object",
    ),
    ran(
        "07",
        "If `P` has an upper bound from an argument or from the expected type, `P` is that bound",
        RUN,
        "upper bounds: t.Shape",
    ),
    ran(
        "07",
        "When it has several, `P` is the one that is a subtype of all the others",
        RUN,
        "upper bounds: t.Shape t.Circle",
    ),
    rule(
        "07",
        "If no rule determines `P`, the method is not applicable to the call",
        &[(
            "class A { static <P> List<P> empty() { return new List<P>(); } void f() { var e = empty(); } }",
            INFER,
        )],
        &[
            "class A { static <P> List<P> empty() { return new List<P>(); } void f() { List<Int> e = empty(); } }",
        ],
    ),
    ran(
        "07",
        "The program then writes the type arguments",
        RUN,
        "written: t.Shape cleat.Object",
    ),
    rule(
        "07",
        "An implicit conversion plays no part in determining `P`",
        &[(
            "class A { static <P> P two(P a, P b) { return a; } void f(Int8 small, Int big) { var x = two(small, big); } }",
            INFER,
        )],
        &[
            "class A { static <P> P two(P a, P b) { return a; } void f(Int8 small, Int big) { var x = A.<Int>two(small, big); Int y = two(small, big); } }",
        ],
    ),
    ran(
        "07",
        "A type is inferred with its qualifiers",
        RUN,
        "with its qualifiers: @cleat.Nullable cleat.String",
    ),
    // ---- 7.6 ----
    ran(
        "07",
        "When a bound of `P` is an interface that requires a static method",
        RUN,
        "a static requirement: 3/2 6",
    ),
    ran(
        "07",
        "The method that runs is the one the class of the type argument has",
        RUN,
        "a static requirement: 3/2 6 Money(0) Weight(-1)",
    ),
    ran(
        "07",
        "`sum` of an `Iterable<Rational>` runs `Rational.zero()`",
        RUN,
        "a static requirement: 3/2",
    ),
    rule(
        "07",
        "No other static member is reached through a type parameter",
        &[
            (
                "class K { public static Int count = 0; } class A { static <T extends K> Int f() { return T.count; } }",
                "only a static requirement is reached through a type parameter",
            ),
            (
                "class K { public static Int next() { return 1; } } class A { static <T extends K> Int f() { return T.next(); } }",
                "no bound of `T` requires a static method named `next`",
            ),
        ],
        &["class A { static <T extends Numeric<T>> T zero() { return T.zero(); } }"],
    ),
    // ---- 7.7 ----
    ran(
        "07",
        "Type arguments exist at run time",
        RUN,
        "classes: false true true",
    ),
    ran(
        "07",
        "Each instantiation of a generic class is a class of its own",
        RUN,
        "classes: false",
    ),
    ran(
        "07",
        "have different class objects, and `getClass()` on their instances returns different results",
        RUN,
        "classes: false true true",
    ),
    ran(
        "07",
        "The qualifiers of a type argument are part of it",
        RUN,
        "names: cleat.Map<cleat.String, cleat.Int> cleat.List<@cleat.Nullable cleat.String>",
    ),
    ran(
        "07",
        "A cast and an `instanceof` test type arguments by the subtype rules of",
        RUN,
        "tests: a cast fails, true true true true false",
    ),
    ran(
        "07",
        "`(List<String>) o` fails for a `List<Int>`",
        RUN,
        "tests: a cast fails",
    ),
    ran(
        "07",
        "`o instanceof Iterable<Object>` is `true` for a `List<String>`",
        RUN,
        "tests: a cast fails, true",
    ),
    ran(
        "07",
        "`o instanceof List<?>` is `true` for every list",
        RUN,
        "tests: a cast fails, true true true true",
    ),
    ran(
        "07",
        "Inside a generic class or method, a type parameter stands for its argument",
        RUN,
        "a type parameter: true false",
    ),
    ran(
        "07",
        "`(T) e` and `e instanceof T` test against that argument",
        RUN,
        "a type parameter: true false accepted refused cleat.Array<@cleat.Nullable cleat.String>",
    ),
    ran(
        "07",
        "`T[]` is `Array<T>`, an invariant generic class like any other",
        RUN,
        "arrays: cleat.Array<cleat.String> false true false",
    ),
    ran(
        "07",
        "`o instanceof String[]` is `false` for an `Object[]`",
        RUN,
        "arrays: cleat.Array<cleat.String> false",
    ),
    ran(
        "07",
        "An array of some subtype of `Shape` is written `Array<? extends Shape>`",
        RUN,
        "arrays: cleat.Array<cleat.String> false true false true",
    ),
    ran(
        "07",
        "A wildcard type may be a type argument, and it is then part of the class as any argument is",
        RUN,
        "a wildcard argument: false",
    ),
    ran(
        "07",
        "`List<List<?>>` and `List<List<String>>` are different classes",
        RUN,
        "a wildcard argument: false cleat.List<cleat.List<cleat.String>>",
    ),
    ran(
        "07",
        "`getName()` of an instantiation is the name of the generic type",
        RUN,
        "names: cleat.Map<cleat.String, cleat.Int>",
    ),
    ran(
        "07",
        "The name of a type argument is the `getName()` of its class, preceded by each qualifier it carries",
        RUN,
        "names: cleat.Map<cleat.String, cleat.Int> cleat.List<@cleat.Nullable cleat.String> cleat.List<@cleat.Nullable @t.Positive cleat.Int>",
    ),
    ran(
        "07",
        "A `List` of nullable strings is `cleat.List<@cleat.Nullable cleat.String>`",
        RUN,
        "names: cleat.Map<cleat.String, cleat.Int> cleat.List<@cleat.Nullable cleat.String>",
    ),
    ran(
        "07",
        "A wildcard inside a type argument is named `?`, or `? extends ` or `? super ` followed by the name of its bound",
        RUN,
        "wildcard names: cleat.List<cleat.List<?>> cleat.List<cleat.List<? extends t.Shape>> cleat.List<cleat.List<? super t.Circle>>",
    ),
    ran(
        "07",
        "Two spellings of one type have one name",
        RUN,
        "one name: cleat.List<cleat.Iterable<t.Shape>> cleat.List<cleat.List<?>> true true",
    ),
    stated(
        "07",
        "How an implementation represents instantiations, with shared code or with a copy for each, is not observable",
        "It says that a choice of the implementation cannot be seen, so no program shows it.",
    ),
    // ---- 7.8 ----
    ran(
        "07",
        "A signature keeps its type arguments",
        RUN,
        "signatures: strings ints",
    ),
    rule(
        "07",
        "are different signatures, and one class may declare both",
        &[(
            "class A { void accept(List<String> items) { } void accept(List<String> other) { } }",
            "the method `accept` is declared twice with the same parameter types",
        )],
        &["class A { void accept(List<String> items) { } void accept(List<Int> items) { } }"],
    ),
    stated(
        "07",
        "There is no bridge method",
        "It says what an implementation need not make. A program sees only the methods it declared, which the sentences around it show.",
    ),
    rule(
        "07",
        "A class implements a method of a generic interface by declaring it with the interface's type arguments substituted",
        &[
            (
                "interface Snk<T> { void put(T x); } class S implements Snk<String> { @Override public void put(Object x) { } }",
                "`put` carries `@Override` and overrides nothing",
            ),
            (
                "interface Src<T> { T get(); } class S implements Src<String> { @Override public Object get() { return \"s\"; } }",
                "the result `Object` is not `String` or a subtype of it",
            ),
        ],
        &[
            "interface Src<T> { T get(); } interface Snk<T> { void put(T x); } class S implements Src<String>, Snk<Int> { @Override public String get() { return \"s\"; } @Override public void put(Int x) { } }",
        ],
    ),
    ran(
        "07",
        "A class that implements `Ordered<Money>` declares `Int compare(Money other)`",
        RUN,
        "substituted: -1",
    ),
    rule(
        "07",
        "A class implements a given generic interface with one set of type arguments",
        &[(
            "interface Src<T> { T get(); } open class Base implements Src<String> { @Override public String get() { return \"s\"; } } class D extends Base implements Src<Int> { }",
            "`Src` is implemented with two sets of type arguments",
        )],
        &[
            "interface Src<T> { T get(); } open class Base implements Src<String> { @Override public String get() { return \"s\"; } } class D extends Base implements Src<String> { }",
        ],
    ),
    ran(
        "07",
        "A superclass method and an interface method with the same signature are one method, and the class implements it once",
        RUN,
        "one method: one method, one method",
    ),
];
