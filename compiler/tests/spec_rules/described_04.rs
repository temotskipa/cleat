//! Chapter 4, Methods: the sentences that define and describe.

use super::{
    DISCARDED, FALLS_OFF, FINAL_LOCAL, FOREIGN_PARAM, Rule, UNASSIGNED, held, ran, rule, stated,
};

const RUN: &str = "Sentences04";
const AMBIGUOUS: &str =
    "is ambiguous: 2 methods accept these arguments and none is more specific than the rest";
const NOTHING_OVERRIDDEN: &str = "carries `@Override` and overrides nothing";
const NEEDS_OVERRIDE: &str = "overrides or implements a method and must carry `@Override`";
const EXHAUSTIVE: &str =
    "a switch expression is exhaustive: add the missing arms or a `default` arm";
const NOT_FUNCTIONAL: &str = "a lambda stands where a functional interface is expected";
const DISJOINT: &str = "the two types are disjoint, so no pair of values is equal";
const DISCARDABLE: &str =
    "a method carries `@Discardable` exactly when the method it overrides does";
const NULLABLE_SEND: &str =
    "which may be `null`; only a method that declares a `@Nullable` receiver is sent to it";

/// A program after the shapes most cases of this chapter use.
macro_rules! shapes {
    ($more:literal) => {
        concat!(
            "open class Shape { } class Circle extends Shape { } class Square extends Shape { } ",
            $more
        )
    };
}

pub const SENTENCES: &[Rule] = &[
    stated(
        "04",
        "This chapter defines methods, the expressions that call them, and the statements of a method body",
        "It says what the chapter covers.",
    ),
    stated(
        "04",
        "gives the syntax, including operator precedence",
        "It points to chapter 11, whose sentences have their own cases.",
    ),
    // ---- 4.1 ----
    rule(
        "04",
        "A method declaration has modifiers, optional type parameters, a result, a name, a parameter list, and a body",
        &[
            ("class A { Int f(); }", "the method `f` needs a body"),
            (
                "class A { f(Int x) { return 1; } }",
                "expected a name, found `(`",
            ),
        ],
        &["class A { public static <T> T id(T x) { return x; } Int plain() { return 1; } }"],
    ),
    rule(
        "04",
        "The result is a type, or `void`, which is how the result `Unit` is written",
        &[],
        &[
            "class A { void f() { } Unit g() { } Int h() { Function0<Unit> k = () -> f(); return 1; } }",
        ],
    ),
    rule(
        "04",
        "Parameter types and the result type are written",
        &[(
            "class A { Int f(x) { return 1; } }",
            "expected a name, found `)`",
        )],
        &["class A { Int f(Int x, String s) { return x; } }"],
    ),
    rule(
        "04",
        "A parameter is a local of the method, and is assignable unless it is declared `final`",
        &[(
            "class A { Int f(final Int n) { n = 1; return n; } }",
            FINAL_LOCAL,
        )],
        &["class A { Int f(Int n) { n = n + 1; return n; } }"],
    ),
    rule(
        "04",
        "The modifiers are an audience from",
        &[(
            "class A { synchronized Int f() { return 1; } }",
            "there is no modifier `synchronized`",
        )],
        &[
            "abstract class A { public static Int a() { return 1; } public final Int b() { return 1; } public open Int c() { return 1; } public abstract Int d(); foreign static Int32 abs(Int32 n); protected Int e() { return 1; } package Int g() { return 1; } private Int h() { return 1; } }",
        ],
    ),
    rule(
        "04",
        "A generic method writes its type parameters before the result",
        &[(
            "class A { static T id<T>(T x) { return x; } }",
            "expected `;`, found `<`",
        )],
        &["class A { static <T> T id(T x) { return x; } }"],
    ),
    rule(
        "04",
        "A method of a class is final unless it is declared `open`",
        &[(
            "open class A { public Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }",
            "`f` is final in `A` and cannot be overridden",
        )],
        &[
            "open class A { public open Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }",
        ],
    ),
    rule(
        "04",
        "`final` may be written to state that default",
        &[],
        &["class A { public final Int f() { return 1; } }"],
    ),
    stated(
        "04",
        "The methods of an interface are described in",
        "It points to chapter 2, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "A static method and an instance method may share a name",
        RUN,
        "a shared name: static instance",
    ),
    rule(
        "04",
        "A call is static when its receiver is a type name, and is an instance call when its receiver is a value",
        &[
            (
                "class Target { public static String who() { return \"s\"; } public String who(Int n) { return \"i\"; } } class A { String f(Target t) { return Target.who(1); } }",
                "method `who` takes 0 arguments, and 1 is written",
            ),
            (
                "class Target { public static String who() { return \"s\"; } public String who(Int n) { return \"i\"; } } class A { String f(Target t) { return t.who(); } }",
                "method `who` takes 1 argument, and 0 are written",
            ),
        ],
        &[
            "class Target { public static String who() { return \"s\"; } public String who(Int n) { return \"i\"; } } class A { String f(Target t) { return Target.who() + t.who(1); } }",
        ],
    ),
    rule(
        "04",
        "An instance method may declare its receiver as a first parameter named `this`",
        &[],
        &["class A { public Boolean absent(@Nullable A this) { return this == null; } }"],
    ),
    rule(
        "04",
        "Its type is the declaring class or interface, with qualifiers",
        &[
            (
                "class A { public Boolean absent(@Nullable String this) { return true; } }",
                "the receiver parameter has the type of the declaring class",
            ),
            (
                "@Refines annotation Priced; interface Order { Int price(@Priced Order this); } class A { Int f(Order o) { return o.price(); } }",
                "`price` is sent to a `Order`, and its receiver is declared `@Priced Order`",
            ),
        ],
        &[
            "@Refines annotation Priced; interface Order { Int price(@Priced Order this); } class A { Int f(@Priced Order o) { return o.price(); } }",
        ],
    ),
    rule(
        "04",
        "The receiver parameter is not an argument position, and a call supplies no argument for it",
        &[(
            "class A { public Boolean absent(@Nullable A this) { return this == null; } static Boolean g(@Nullable A a) { return a.absent(a); } }",
            "method `absent` takes 0 arguments, and 1 is written",
        )],
        &[
            "class A { public Boolean absent(@Nullable A this) { return this == null; } static Boolean g(@Nullable A a) { return a.absent(); } }",
        ],
    ),
    stated(
        "04",
        "says what its qualifiers mean",
        "It points to chapter 8, whose sentences have their own cases.",
    ),
    rule(
        "04",
        "Without the declaration, the receiver's type is the declaring class with no qualifier",
        &[(
            "class A { public Int f() { return 1; } static Int g(@Nullable A a) { return a.f(); } }",
            NULLABLE_SEND,
        )],
        &["class A { public Int f() { return 1; } static Int g(A a) { return a.f(); } }"],
    ),
    // ---- 4.2 ----
    rule(
        "04",
        "A method of a class overrides a method of its superclass, or implements a method of an interface, when the two have the same name and the same signature",
        &[(
            "open class S { public open Int f(Int n) { return 1; } } class D extends S { @Override public Int f(String n) { return 2; } }",
            NOTHING_OVERRIDDEN,
        )],
        &[
            "open class S { public open Int f(Int n) { return 1; } } interface I { Int g(); } class D extends S implements I { @Override public Int f(Int n) { return 2; } @Override public Int g() { return 3; } }",
        ],
    ),
    rule(
        "04",
        "Its result type is the result type of the method it overrides, or a subtype of it",
        &[(
            "open class Shape { public open Shape copy() { return new Shape(); } } class Circle extends Shape { @Override public Object copy() { return new Circle(); } }",
            "the result `Object` is not `Shape` or a subtype of it",
        )],
        &[
            "open class Shape { public open Shape copy() { return new Shape(); } } class Circle extends Shape { @Override public Circle copy() { return new Circle(); } }",
        ],
    ),
    rule(
        "04",
        "The overriding method carries `@Override`, as",
        &[(
            "open class S { public open Int f() { return 1; } } class D extends S { public Int f() { return 2; } }",
            NEEDS_OVERRIDE,
        )],
        &[
            "open class S { public open Int f() { return 1; } } class D extends S { @Override public Int f() { return 2; } }",
        ],
    ),
    rule(
        "04",
        "The overriding method is itself final unless it is declared `open`",
        &[(
            "open class S { public open Int f() { return 1; } } open class D extends S { @Override public Int f() { return 2; } } class E extends D { @Override public Int f() { return 3; } }",
            "`f` is final in `D` and cannot be overridden",
        )],
        &[
            "open class S { public open Int f() { return 1; } } open class D extends S { @Override public open Int f() { return 2; } } class E extends D { @Override public Int f() { return 3; } }",
        ],
    ),
    rule(
        "04",
        "Its audience follows",
        &[(
            "open class S { public open Int f() { return 1; } } class D extends S { @Override Int f() { return 2; } }",
            "an overriding method has the audience of the method it overrides, or a wider one",
        )],
        &[
            "open class S { protected open Int f() { return 1; } } class D extends S { @Override public Int f() { return 2; } }",
        ],
    ),
    rule(
        "04",
        "Its receiver has the same qualifiers as the receiver it overrides",
        &[(
            "@Refines annotation Priced; open class Order { public open Int price(@Priced Order this) { return 1; } } class Rush extends Order { @Override public Int price() { return 2; } }",
            "an override has the same receiver qualifiers as the method it overrides",
        )],
        &[
            "@Refines annotation Priced; open class Order { public open Int price(@Priced Order this) { return 1; } } class Rush extends Order { @Override public Int price(@Priced Rush this) { return 2; } }",
        ],
    ),
    rule(
        "04",
        "The exception is an override of `equals`, `hashCode` or `toString`, whose receiver has none",
        &[(
            "class A { @Override public String toString(@Nullable A this) { return \"a\"; } }",
            "an override of `toString` declares an ordinary receiver, with no qualifier",
        )],
        &[
            "class A { @Override public String toString() { return \"a\"; } @Override public Boolean equals(@Nullable Object o) { return true; } @Override public Int hashCode() { return 1; } }",
        ],
    ),
    rule(
        "04",
        "It carries `@Discardable` exactly when the method it overrides does",
        &[
            (
                "open class S { @Discardable public open Int f() { return 1; } } class D extends S { @Override public Int f() { return 2; } }",
                DISCARDABLE,
            ),
            (
                "open class S { public open Int f() { return 1; } } class D extends S { @Override @Discardable public Int f() { return 2; } }",
                DISCARDABLE,
            ),
        ],
        &[
            "open class S { @Discardable public open Int f() { return 1; } } class D extends S { @Override @Discardable public Int f() { return 2; } }",
        ],
    ),
    rule(
        "04",
        "A static method overrides nothing",
        &[(
            "open class S { public static Int f() { return 1; } } class D extends S { @Override public static Int f() { return 2; } }",
            NOTHING_OVERRIDDEN,
        )],
        &[
            "open class S { public static Int f() { return 1; } } class D extends S { public static Int f() { return 2; } }",
        ],
    ),
    rule(
        "04",
        "A static method of a class meets a static requirement of an interface",
        &[(
            "interface Zeroed<T> { static T zero(); } value class M implements Zeroed<M> { public Int n; public static M zero() { return new M(0); } }",
            NEEDS_OVERRIDE,
        )],
        &[
            "interface Zeroed<T> { static T zero(); } value class M implements Zeroed<M> { public Int n; @Override public static M zero() { return new M(0); } }",
        ],
    ),
    ran(
        "04",
        "`super.name(args)` calls the superclass's method without dispatch",
        RUN,
        "overriding: circle circle shape",
    ),
    // ---- 4.3 ----
    rule(
        "04",
        "A call is written `receiver.name(args)`, `Type.name(args)`, `super.name(args)` or `name(args)`",
        &[],
        &[
            "open class S { public open Int f() { return 1; } public static Int g() { return 2; } } class D extends S { @Override public Int f() { return super.f() + this.f2() + S.g() + f2() + g(); } Int f2() { return 0; } }",
        ],
    ),
    rule(
        "04",
        "In the last form the method belongs to the enclosing class or to a class it inherits from",
        &[(
            "class A { Int f() { return missing(); } }",
            "`A` has no method named `missing`",
        )],
        &[
            "open class S { public Int inherited() { return 1; } } class D extends S { Int own() { return 2; } Int f() { return inherited() + own(); } }",
        ],
    ),
    ran(
        "04",
        "The receiver is `this` for an instance method and the class for a static method",
        RUN,
        "no receiver written: circle a static method",
    ),
    ran(
        "04",
        "A call evaluates the receiver, then the arguments from left to right, and then runs the method",
        RUN,
        "a call: receiver first second body 3",
    ),
    ran(
        "04",
        "chooses the method at compile time from static types",
        RUN,
        "static types: object shape circle",
    ),
    ran(
        "04",
        "the receiver's class at run time selects the body",
        RUN,
        "overriding: circle circle",
    ),
    rule(
        "04",
        "A call is an expression, and its type is the method's result type",
        &[(
            "class A { Int count() { return 1; } void f(A a) { String s = a.count(); } }",
            "`Int` is not assignable to `String`",
        )],
        &[
            "class A { Int count() { return 1; } void f(A a) { Int n = a.count(); Rational r = a.count() + 1; } }",
        ],
    ),
    rule(
        "04",
        "A call whose result is `Unit` may stand alone as a statement",
        &[],
        &["class A { void log() { } void f(A a) { a.log(); log(); } }"],
    ),
    rule(
        "04",
        "So may a call of a method marked `@Discardable`",
        &[(
            "class Text { public Text append(String s) { return this; } } class A { void f(Text t) { t.append(\"a\"); } }",
            DISCARDED,
        )],
        &[
            "class Text { @Discardable public Text append(String s) { return this; } } class A { void f(Text t) { t.append(\"a\").append(\"b\"); } }",
        ],
    ),
    // ---- 4.4 ----
    ran(
        "04",
        "Two methods of one class may share a name when their parameter lists differ in length or in the type of a parameter",
        RUN,
        "overloads: none number text two",
    ),
    ran(
        "04",
        "A call chooses among the visible methods of that name that the receiver's static type declares or inherits",
        RUN,
        "inherited overloads: own inherited",
    ),
    rule(
        "04",
        "A method is applicable when the call has as many arguments as the method has parameters and each argument is assignable to its parameter",
        &[
            (
                "class A { static Int f(Int a, String b) { return 1; } Int g() { return f(1); } }",
                "method `f` takes 2 arguments, and 1 is written",
            ),
            (
                "class A { static Int f(Int a, String b) { return 1; } Int g() { return f(1, 2); } }",
                "`Int` is not assignable to `String`",
            ),
        ],
        &[
            "class A { static Int f(Int a, String b) { return 1; } Int g() { return f(1, \"b\"); } }",
        ],
    ),
    stated(
        "04",
        "says when a numeric literal is applicable",
        "It points to chapter 6 and to section 4.8, whose sentences have their own cases.",
    ),
    rule(
        "04",
        "One applicable method is more specific than another when each of its parameter types is a subtype of the other's corresponding parameter type",
        &[(
            shapes!(
                "class A { static Int f(Shape a, Object b) { return 1; } static Int f(Object a, Shape b) { return 2; } Int g(Circle c) { return f(c, c); } }"
            ),
            AMBIGUOUS,
        )],
        &[shapes!(
            "class A { static Int f(Shape a, Object b) { return 1; } static Int f(Object a, Shape b) { return 2; } Int g(Circle c) { return f((Object) c, c); } }"
        )],
    ),
    ran(
        "04",
        "If one applicable method is more specific than every other, it is chosen",
        RUN,
        "static types: object shape circle",
    ),
    ran(
        "04",
        "The one exception is a call with a numeric literal argument",
        "Sentences06",
        "overloads: Float64 Float64 Int Rational",
    ),
    ran(
        "04",
        "A method with a fixed number of parameters is more specific than a varargs method",
        RUN,
        "fixed and varargs: fixed varargs-2 varargs-0",
    ),
    ran(
        "04",
        "it considers the methods that are applicable when an argument may be converted",
        RUN,
        "the second round: Int",
    ),
    ran(
        "04",
        "In that second round, a class counts as more specific than a class it converts to",
        RUN,
        "the second round: Int Int16",
    ),
    // ---- 4.5 ----
    rule(
        "04",
        "A value of static type `S` is assignable to a location of type `T` when `S` is a subtype of `T`",
        &[(
            shapes!("class A { Circle f(Shape s) { return s; } }"),
            "`Shape` is not assignable to `Circle`",
        )],
        &[shapes!(
            "class A { Shape f(Circle c) { Shape s = c; Object o = s; return c; } }"
        )],
    ),
    rule(
        "04",
        "Subtyping is the reflexive, transitive closure of `extends` and `implements`",
        &[],
        &[
            "interface I { } interface J extends I { } open class B implements J { } class C extends B { } class A { void f(C c) { C same = c; B b = c; J j = c; I i = c; Object o = i; } }",
        ],
    ),
    ran(
        "04",
        "A value of class `S` is also assignable to a location of type `T` or `@Nullable T` when `T` declares an implicit conversion from `S`",
        RUN,
        "an implicit conversion: from(3) from(4) Meters(3) Meters(4)",
    ),
    ran(
        "04",
        "and the compiler inserts a call of it",
        RUN,
        "an implicit conversion: from(3)",
    ),
    held(
        "04",
        "and a program's value classes may declare their own",
        "spec_tables",
        "the_implicit_conversions_are_exactly_the_table_of_section_6_7",
    ),
    rule(
        "04",
        "At most one conversion is applied to a value",
        &[(
            "value class Meters { public Int length; @Implicit public static Meters from(Int whole) { return new Meters(whole); } } class A { Meters f(Int8 n) { return n; } }",
            "`Int8` is not assignable to `Meters`",
        )],
        &[
            "value class Meters { public Int length; @Implicit public static Meters from(Int whole) { return new Meters(whole); } } class A { Meters f(Int n) { @Nullable Meters m = n; return n; } }",
        ],
    ),
    rule(
        "04",
        "No other method is ever inserted to make a value fit",
        &[
            (
                "class A { String f() { return 5; } }",
                "`Int` is not assignable to `String`",
            ),
            (
                "class A { Int8 f(Int n) { return n; } }",
                "`Int` is not assignable to `Int8`",
            ),
            (
                "class A { Int f(Float64 x) { return x; } }",
                "`Float64` is not assignable to `Int`",
            ),
            (
                shapes!("class A { Circle f(Shape s) { Circle c = s; return c; } }"),
                "`Shape` is not assignable to `Circle`",
            ),
        ],
        &[],
    ),
    ran(
        "04",
        "A numeric literal takes its class from where it stands",
        "Sentences06",
        "defaults: cleat.Rational cleat.Int cleat.Rational",
    ),
    rule(
        "04",
        "An expression has an expected type when it is the initializer of a declaration with a written type",
        &[],
        &[
            "class A { static Int take(UInt8 b) { return 1; } Float64 f() { Float64 x = 1; x = 2; var xs = new Float64[] { 1, 2 }; Function0<Float64> g = () -> 3; Function1<Int, Int> h = (n) -> n + 1; return take(255) == 1 ? 4 : 5; } }",
        ],
    ),
    rule(
        "04",
        "The expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression",
        &[],
        &[
            "class A { Function1<Int, Int> f(Boolean c, Int n) { Function1<Int, Int> a = (c ? (x) -> x + 1 : (x) -> x - 1); return switch (n) { case 1 -> (x) -> x; default -> (x) -> x * 2; }; } }",
        ],
    ),
    rule(
        "04",
        "Numeric literals, lambdas, and the inference of chapter 7 use it",
        &[],
        &[
            "class A { static <P> List<P> empty() { return new List<P>(); } void f() { Float64 x = 1; Function0<Int> g = () -> 1; List<String> e = empty(); } }",
        ],
    ),
    rule(
        "04",
        "Two types are disjoint when no value can belong to both",
        &[(
            "class A { Boolean f(String s, Int n) { return s == n; } }",
            DISJOINT,
        )],
        &["class A { Boolean f(String s, Object o) { return s == o; } }"],
    ),
    rule(
        "04",
        "Leaving aside every qualifier but `@Nullable`",
        &[],
        &[
            "@Refines annotation P; class A { Boolean f(@P Int a, Int b, @Nullable Int c) { return a == b || b == c; } }",
        ],
    ),
    rule(
        "04",
        "two class types are disjoint unless one is a subtype of the other;",
        &[(
            shapes!("class A { Boolean f(Circle c, Square s) { return c == s; } }"),
            DISJOINT,
        )],
        &[shapes!(
            "class A { Boolean f(Circle c, Shape s) { return c == s; } }"
        )],
    ),
    // ---- 4.6 ----
    ran(
        "04",
        "Any class may declare these methods",
        RUN,
        "every operator: plus minus times div mod negate lessThan atMost greaterThan atLeast and or xor complement shiftLeft shiftRight not get set",
    ),
    rule(
        "04",
        "There is no unary `+`, no `>>>`, and no power operator",
        &[
            (
                "class A { Int f(Int a) { return +a; } }",
                "there is no unary `+`",
            ),
            (
                "class A { Int f(Int a) { return a >>> 1; } }",
                "`>>>` is not an operator",
            ),
            (
                "class A { Int f(Int a) { return a ** 2; } }",
                "expected an expression, found `*`",
            ),
        ],
        &[],
    ),
    rule(
        "04",
        "and exactly one of them converts implicitly to the class of the other, it is converted first",
        &[(
            "class A { Float64 f(Int32 i, Float64 x) { return i + x; } }",
            "`Float64` is not assignable to `Int32`",
        )],
        &[
            "class A { Int f(Int32 i, Int n) { return i + n; } Rational g(Int n, Rational r) { return n * r; } Int h(Int a, Int8 count) { return a << count; } }",
        ],
    ),
    ran(
        "04",
        "`i + n` is an `Int` addition",
        "Sentences06",
        "mixed operands: 2147483648 cleat.Int",
    ),
    ran(
        "04",
        "`a == b` is `a.equals(b)`, for a reference class and a value class alike",
        RUN,
        "equality: equals equals true true",
    ),
    ran(
        "04",
        "`a != b` is `!(a == b)`",
        RUN,
        "inequality: equals false",
    ),
    stated(
        "04",
        "Two cases differ",
        "It introduces the two cases after it, which have their own cases here and in the other tables.",
    ),
    ran(
        "04",
        "When one operand is the literal `null`, the expression is the null test of",
        RUN,
        "a null test: [] true true",
    ),
    ran(
        "04",
        "Reference identity is `identical`, a method of `Object`",
        RUN,
        "identity: false true",
    ),
    ran(
        "04",
        "`a += b` is `a = a + b`, except that the location `a` is evaluated once",
        RUN,
        "compound assignment: index holder 7",
    ),
    ran(
        "04",
        "The same holds for `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=` and `>>=`",
        RUN,
        "compound operators: 10 30 2 2 6 3 12 6 3/2",
    ),
    ran(
        "04",
        "`a++` and `++a` are `a = a + 1`, and `a--` and `--a` are `a = a - 1`, with the location evaluated once",
        RUN,
        "increment: 5 7 7 5 5 | index 8",
    ),
    ran(
        "04",
        "The literal takes its class from `a` by the rule of chapter 6",
        RUN,
        "the literal's class: 2.5 -1/2 6",
    ),
    ran(
        "04",
        "The prefix form yields the updated value, and the postfix form yields the value from before the update",
        RUN,
        "increment: 5 7 7 5",
    ),
    ran(
        "04",
        "If the method raises, the location is unchanged",
        RUN,
        "an increment that raises: raises 127",
    ),
    // ---- 4.7 ----
    rule(
        "04",
        "Both operands have type `Boolean`",
        &[(
            "class A { Boolean f(Int n, Boolean b) { return n && b; } }",
            "`Int` is not assignable to `Boolean`",
        )],
        &["class A { Boolean f(Boolean a, Boolean b) { return a && b || !a; } }"],
    ),
    ran("04", "`a && b` evaluates `a`", RUN, "and: a a b a b"),
    ran(
        "04",
        "If it is `false`, the result is `false` and `b` is not evaluated",
        RUN,
        "and: a a b a b | false",
    ),
    ran(
        "04",
        "Otherwise the result is the value of `b`",
        RUN,
        "and: a a b a b | false false true",
    ),
    ran("04", "`a || b` evaluates `a`", RUN, "or: a a b a b"),
    ran(
        "04",
        "If it is `true`, the result is `true` and `b` is not evaluated",
        RUN,
        "or: a a b a b | true true false",
    ),
    ran(
        "04",
        "`&` and `|` on `Boolean` are the methods `and` and `or`, and they evaluate both operands",
        RUN,
        "both operands: a b a b | false true",
    ),
    ran(
        "04",
        "`c` has type `Boolean` and is evaluated once",
        RUN,
        "conditional: c a c b",
    ),
    ran(
        "04",
        "Then exactly one of `a` and `b` is evaluated, and its value is the result",
        RUN,
        "conditional: c a c b | 1 2",
    ),
    rule(
        "04",
        "When the expression has an expected type, both arms are checked against it and it is the type of the expression",
        &[(
            "class A { UInt8 f(Boolean c) { return c ? 1 : 256; } }",
            "the literal is outside the range of `UInt8`",
        )],
        &[shapes!(
            "class A { Float64 f(Boolean c) { Float64 x = c ? 1 : 0.5; Shape s = c ? new Circle() : new Square(); return x; } }"
        )],
    ),
    ran(
        "04",
        "`a ?? b` is the value of `a` unless that is `null`, and the value of `b` otherwise",
        RUN,
        "??: right left",
    ),
    stated(
        "04",
        "defines it",
        "It points to chapter 5, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "`(T) e` evaluates `e` and tests the class of its value",
        RUN,
        "casts: true 4 raises",
    ),
    ran(
        "04",
        "The test succeeds when that class is a subtype of `T`, including the type arguments of a generic class",
        RUN,
        "casts: true 4 raises circle refused",
    ),
    ran(
        "04",
        "A successful cast yields the same value with static type `T`",
        RUN,
        "casts: true 4",
    ),
    ran(
        "04",
        "A failed cast raises `ClassCastException`",
        RUN,
        "casts: true 4 raises",
    ),
    stated(
        "04",
        "says what a cast does with `null`",
        "It points to chapters 5 and 8, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "`e instanceof T` is `true` when the value of `e` is not `null` and its class is a subtype of `T`",
        RUN,
        "instanceof: true false false true true false",
    ),
    rule(
        "04",
        "Where it is known `true`, it narrows a local",
        &[(
            "class A { Int f(Object o) { return o.length(); } }",
            "`Object` has no method named `length`",
        )],
        &[
            "class A { Int f(Object o) { if (o instanceof String) { return o.length(); } return 0; } }",
        ],
    ),
    ran(
        "04",
        "`new C(args)` creates an instance of the class `C`, by the construction rules of",
        RUN,
        "constructors: none number text",
    ),
    ran(
        "04",
        "The constructor is chosen as",
        RUN,
        "constructors: none number text",
    ),
    rule(
        "04",
        "A generic class is written with its type arguments",
        &[(
            "class A { Object f() { return new List(); } }",
            "`List` is generic and is written with its type arguments",
        )],
        &["class A { List<String> f() { return new List<String>(); } }"],
    ),
    rule(
        "04",
        "`new List<String>()`",
        &[],
        &["class A { Int f() { var names = new List<String>(); return names.size(); } }"],
    ),
    stated(
        "04",
        "Array creation is in",
        "It points to chapter 6, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "`x = e` evaluates `e` and stores its value in the local or parameter `x`",
        RUN,
        "assignment: 5 5",
    ),
    ran(
        "04",
        "`r.f = e` evaluates `r`, then `e`, and stores into the field",
        RUN,
        "assignment: 5 5 7 cleat.List<cleat.Int8> | receiver value",
    ),
    ran(
        "04",
        "`a[i] = e` evaluates `a`, `i` and `e` in that order and calls `a.set(i, e)`",
        RUN,
        "assignment: 5 5 7 cleat.List<cleat.Int8> | receiver value grid index element set",
    ),
    ran(
        "04",
        "An assignment is an expression: its value is the value stored, and its type is the type of the location",
        RUN,
        "assignment: 5 5 7 cleat.List<cleat.Int8>",
    ),
    rule(
        "04",
        "It may stand alone as a statement",
        &[],
        &[
            "class A { Int x = 0; void f(Int[] xs, A other) { var n = 0; n = 1; x = 2; other.x = 3; xs[0] = 4; } }",
        ],
    ),
    // ---- 4.8 ----
    rule(
        "04",
        "A functional interface is an interface with exactly one abstract method and no static requirement",
        &[
            (
                "interface Two { Int a(); Int b(); } class A { Two f() { return () -> 1; } }",
                NOT_FUNCTIONAL,
            ),
            (
                "interface Req { Int a(); static Int zero(); } class A { Req f() { return () -> 1; } }",
                NOT_FUNCTIONAL,
            ),
        ],
        &[
            "interface One { Int a(); Int twice() { return a() * 2; } } class A { One f() { return () -> 1; } }",
        ],
    ),
    rule(
        "04",
        "The prelude declares the general ones",
        &[],
        &[
            "class A { void f() { Function0<Int> a = () -> 1; Function1<Int, Int> b = (x) -> x; Function2<Int, Int, Int> c = (x, y) -> x + y; } }",
        ],
    ),
    rule(
        "04",
        "A lambda is written `(parameters) -> expression` or `(parameters) -> block`",
        &[],
        &[
            "class A { void f() { Function1<Int, Int> a = (x) -> x + 1; Function1<Int, Int> b = (x) -> { return x + 1; }; Function1<Int, Int> c = (Int x) -> x; } }",
        ],
    ),
    ran(
        "04",
        "It stands where the expected type is a functional interface, and it becomes an instance of that interface whose one method is the lambda",
        RUN,
        "a lambda: 6 5 word true",
    ),
    rule(
        "04",
        "The lambda has as many parameters as the interface's method",
        &[
            (
                "class A { void f() { Function1<Int, Int> a = () -> 1; } }",
                "the lambda has 0 parameters, and `invoke` of the interface has 1",
            ),
            (
                "class A { void f() { Function1<Int, Int> a = (x, y) -> x; } }",
                "the lambda has 2 parameters, and `invoke` of the interface has 1",
            ),
        ],
        &["class A { void f() { Function2<Int, Int, Int> a = (x, y) -> x + y; } }"],
    ),
    rule(
        "04",
        "A parameter's type may be omitted, and is then the method's parameter type",
        &[(
            "class A { void f() { Function1<Int, Int> a = (s) -> s.length(); } }",
            "`Int` has no method named `length`",
        )],
        &["class A { void f() { Function1<String, Int> a = (s) -> s.length(); } }"],
    ),
    ran(
        "04",
        "an expression body may instead be any expression that could stand as a statement, and its value is discarded",
        RUN,
        "a Unit result: 1 3",
    ),
    rule(
        "04",
        "A block body returns with `return`, under the rules for a method with that result",
        &[(
            "class A { void f() { Function1<Int, Int> a = (x) -> { if (x > 0) { return 1; } }; } }",
            FALLS_OFF,
        )],
        &[
            "class A { void f() { Function1<Int, Int> a = (x) -> { if (x > 0) { return 1; } return 0; }; } }",
        ],
    ),
    ran(
        "04",
        "`return` inside a lambda returns from the lambda",
        RUN,
        "return in a lambda: positive other, and the method goes on",
    ),
    ran(
        "04",
        "Inside a lambda, `this` is the `this` of the enclosing method",
        RUN,
        "this in a lambda: 1 2 2",
    ),
    ran(
        "04",
        "The lambda holds the values those variables had when the lambda expression was evaluated",
        RUN,
        "captured values: 0 10 20",
    ),
    ran(
        "04",
        "A lambda may read and assign fields through `this`",
        RUN,
        "this in a lambda: 1 2 2",
    ),
    ran(
        "04",
        "The instance belongs to a final value class that the lambda expression declares implicitly",
        RUN,
        "the instance: true false true",
    ),
    ran(
        "04",
        "Its fields are the captured values and `this`",
        RUN,
        "the instance: true false",
    ),
    stated(
        "04",
        "Its `toString` is chosen by the implementation",
        "It leaves a text to the implementation, so no program shows a text it must be.",
    ),
    ran(
        "04",
        "A lambda argument is applicable to a parameter whose type is a functional interface whose method has that many parameters",
        RUN,
        "lambdas in overloads: none an-Int a-String",
    ),
    rule(
        "04",
        "The body of the lambda plays no part in choosing the method",
        &[(
            "class A { static Int f(Function1<Int, Int> g) { return 1; } static Int f(Function1<Int, String> g) { return 2; } Int h() { return f((x) -> x + 1); } }",
            AMBIGUOUS,
        )],
        &[
            "class A { static Int f(Function1<Int, Int> g) { return 1; } static Int f(Function1<String, Int> g) { return 2; } Int h() { return f((Int x) -> x + 1); } }",
        ],
    ),
    stated(
        "04",
        "Chapter 7 says how a lambda takes part in inference",
        "It points to chapter 7, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "A method reference is a short form of a lambda, and stands in the same places",
        RUN,
        "method references: positive",
    ),
    ran(
        "04",
        "| `Type::name`, for a static method | `(args) -> Type.name(args)` |",
        RUN,
        "method references: positive",
    ),
    ran(
        "04",
        "| `Type::name`, for an instance method | `(receiver, args) -> receiver.name(args)` |",
        RUN,
        "method references: positive | 3 true",
    ),
    ran(
        "04",
        "| `expr::name` | `(args) -> v.name(args)`, where `v` is the value of `expr` when the reference is evaluated |",
        RUN,
        "method references: positive | 3 true | made 4",
    ),
    ran(
        "04",
        "| `Type::new` | `(args) -> new Type(args)` |",
        RUN,
        "method references: positive | 3 true | made 4 | 7",
    ),
    ran(
        "04",
        "The parameter types of the interface's method choose among overloads",
        RUN,
        "method references: positive | 3 true | made 4 | 7 | number text",
    ),
    // ---- 4.9 ----
    ran(
        "04",
        "`T name = e;` declares a local of type `T` and initializes it",
        RUN,
        "locals: 2 3 5",
    ),
    rule(
        "04",
        "`T name;` declares a local with no value",
        &[("class A { Int f() { Int a; return a; } }", UNASSIGNED)],
        &["class A { Int f() { Int a; a = 1; return a; } }"],
    ),
    rule(
        "04",
        "`var name = e;` gives the local the static type of `e`, with its qualifiers",
        &[(
            "class A { @Nullable String m() { return null; } Int f() { var s = m(); return s.length(); } }",
            NULLABLE_SEND,
        )],
        &["class A { String m() { return \"s\"; } Int f() { var s = m(); return s.length(); } }"],
    ),
    rule(
        "04",
        "A local is assignable unless it is declared `final`",
        &[(
            "class A { Int f() { final Int a = 1; a = 2; return a; } }",
            FINAL_LOCAL,
        )],
        &["class A { Int f() { var a = 1; a = 2; return a; } }"],
    ),
    stated(
        "04",
        "gives its scope",
        "It points to chapter 1, whose sentences have their own cases.",
    ),
    rule(
        "04",
        "An assignment, an increment, a decrement, a call whose result is `Unit`, and a call of a `@Discardable` method may each stand as a statement",
        &[],
        &[
            "class Text { @Discardable public Text append(String s) { return this; } } class A { Int x = 0; void g() { } void f(Text t) { var n = 0; n = 1; n++; n--; ++n; --n; g(); t.append(\"a\"); x += 1; } }",
        ],
    ),
    ran(
        "04",
        "`if (c) s else t` evaluates `c`, which has type `Boolean`, and runs `s` when it is `true` and `t` otherwise",
        RUN,
        "if: then else",
    ),
    ran(
        "04",
        "Without `else`, nothing runs when `c` is `false`",
        RUN,
        "if: then else ",
    ),
    ran(
        "04",
        "`while (c) s` evaluates `c` before each iteration, runs `s` while it is `true`, and completes when it is `false`",
        RUN,
        "while: c c c | 0 1",
    ),
    ran(
        "04",
        "`for (init; c; update) s` runs `init` once",
        RUN,
        "for: init c s update c s update c",
    ),
    ran(
        "04",
        "Before each iteration it evaluates `c`, and completes when `c` is `false`",
        RUN,
        "for: init c s update c s update c",
    ),
    ran(
        "04",
        "A missing `c` is `true`",
        RUN,
        "for without a condition: 3",
    ),
    ran(
        "04",
        "Then it runs `s`, and then `update`",
        RUN,
        "for: init c s update",
    ),
    ran(
        "04",
        "`update` also runs after a `continue` of this loop",
        RUN,
        "for without a condition: 3, and after continue: update update update 3",
    ),
    rule(
        "04",
        "A local declared in `init` is in scope in `c`, `s` and `update`",
        &[(
            "class A { Int f() { for (var i = 0; i < 3; i++) { } return i; } }",
            "`i` is not a variable or a field in scope",
        )],
        &[
            "class A { Int f() { var total = 0; for (var i = 0; i < 3; i++) { total += i; } return total; } }",
        ],
    ),
    rule(
        "04",
        "requires the static type of `e` to be a subtype of `Iterable<U>`, where `U` is assignable to `T`",
        &[
            (
                "class A { void f() { for (Int x : 5) { } } }",
                "a `for` statement visits an `Iterable`, and `Int` is not one",
            ),
            (
                "class A { void f(List<Int> xs) { for (String s : xs) { } } }",
                "`Int` is not assignable to `String`",
            ),
        ],
        &[
            "class A { void f(List<String> xs) { for (Object o : xs) { } for (String s : xs) { } } }",
        ],
    ),
    ran(
        "04",
        "It evaluates `e` and calls `iterator()` once",
        RUN,
        "for over a collection: iterator hasNext",
    ),
    ran(
        "04",
        "Then, while `hasNext()` is `true`, it binds `x` to `next()` and runs `s`",
        RUN,
        "for over a collection: iterator hasNext next body hasNext next body hasNext | 12",
    ),
    rule(
        "04",
        "`x` is not assignable",
        &[(
            "class A { void f(List<Int> xs) { for (Int x : xs) { x = 1; } } }",
            FINAL_LOCAL,
        )],
        &["class A { Int f(List<Int> xs) { var t = 0; for (Int x : xs) { t += x; } return t; } }"],
    ),
    ran("04", "`var` may be written for `T`", RUN, "for with var: 6"),
    ran(
        "04",
        "evaluates the selector once and runs the first arm that matches",
        RUN,
        "switch: low low three high | selector selector selector selector",
    ),
    ran(
        "04",
        "No arm continues into the next",
        RUN,
        "selectors: b true 2 amber",
    ),
    stated(
        "04",
        "An arm is one of three kinds",
        "It introduces the three kinds after it, which have their own cases.",
    ),
    ran(
        "04",
        "matches when the selector equals one of the constants under `equals`",
        RUN,
        "switch: low low three",
    ),
    rule(
        "04",
        "The constants are constant expressions of the selector's type",
        &[
            (
                "class A { Int f(Int n, Int k) { switch (n) { case k -> { return 1; } default -> { return 2; } } } }",
                "the constant of a `case` is a constant expression",
            ),
            (
                "class A { Int f(Int n) { switch (n) { case \"one\" -> { return 1; } default -> { return 2; } } } }",
                "`String` is not assignable to `Int`",
            ),
        ],
        &[
            "class A { static final Int LIMIT = 3; Int f(Int n) { switch (n) { case 1 + 1 -> { return 1; } case LIMIT -> { return 3; } default -> { return 2; } } } }",
        ],
    ),
    rule(
        "04",
        "The selector's type is an integer class, `Char`, `Boolean`, `String` or an enum",
        &[(
            "class A { Int f(Float64 x) { switch (x) { case 1.5 -> { return 1; } default -> { return 2; } } } }",
            "constant arms need a selector that is an integer class, `Char`, `Boolean`, `String` or an enum, and this is a `Float64`",
        )],
        &[
            "enum E { ONE, TWO } class A { Int f(Int8 a, UInt64 b, Char c, Boolean d, String e, E g) { var n = 0; switch (a) { case 1 -> n = 1; default -> n = 2; } switch (b) { case 1 -> n = 1; default -> n = 2; } switch (c) { case 'x' -> n = 1; default -> n = 2; } switch (d) { case true -> n = 1; case false -> n = 2; } switch (e) { case \"s\" -> n = 1; default -> n = 2; } switch (g) { case E.ONE -> n = 1; case E.TWO -> n = 2; } return n; } }",
        ],
    ),
    ran(
        "04",
        "matches when the selector is an instance of `T`, as `instanceof` tests",
        RUN,
        "type arms: a String of 4, a circle, something else",
    ),
    rule(
        "04",
        "In the body, `name` is a local of type `T` that holds the selector and is not assignable",
        &[(
            "class A { Int f(Object o) { switch (o) { case String s -> { s = \"x\"; return 1; } default -> { return 0; } } } }",
            FINAL_LOCAL,
        )],
        &[
            "class A { Int f(Object o) { switch (o) { case String s -> { return s.length(); } default -> { return 0; } } } }",
        ],
    ),
    ran(
        "04",
        "`default -> body` matches when no other arm does",
        RUN,
        "switch: low low three high",
    ),
    rule(
        "04",
        "A switch has at most one, written last",
        &[
            (
                "class A { Int f(Int n) { switch (n) { default -> { return 0; } default -> { return 1; } } } }",
                "`default` is the last arm of a switch",
            ),
            (
                "class A { Int f(Int n) { switch (n) { default -> { return 0; } case 1 -> { return 1; } } } }",
                "`default` is the last arm of a switch",
            ),
        ],
        &[
            "class A { Int f(Int n) { switch (n) { case 1 -> { return 1; } default -> { return 0; } } } }",
        ],
    ),
    rule(
        "04",
        "One switch uses constant arms or type arms, not both",
        &[(
            "class A { Int f(Object o) { switch (o) { case String s -> { return 1; } case 1 -> { return 2; } default -> { return 0; } } } }",
            "one switch uses constant arms or type arms, not both",
        )],
        &[],
    ),
    rule(
        "04",
        "The selector's type is not `@Nullable`",
        &[(
            "class A { Int f(@Nullable String s) { switch (s) { case \"a\" -> { return 1; } default -> { return 0; } } } }",
            "the selector of a switch is not `@Nullable`, and this is a `@Nullable String`",
        )],
        &[
            "class A { Int f(@Nullable String s) { if (s == null) { return -1; } switch (s) { case \"a\" -> { return 1; } default -> { return 0; } } } }",
        ],
    ),
    rule(
        "04",
        "In a switch statement, each body is a statement",
        &[],
        &[
            "class A { Int f(Int n) { var out = 0; switch (n) { case 1 -> out = 1; case 2 -> { out = 2; } case 3 -> out++; default -> throw new IllegalStateException(\"x\"); } return out; } }",
        ],
    ),
    ran(
        "04",
        "If no arm matches, the switch completes normally",
        RUN,
        "selectors: b true 2 amber and no arm matched",
    ),
    rule(
        "04",
        "Its type follows the rule for `?:`, applied across all arms",
        &[(
            "class A { String f(Int n) { return switch (n) { case 1 -> \"one\"; default -> 2; }; } }",
            "`Int` is not assignable to `String`",
        )],
        &[shapes!(
            "class A { Float64 f(Int n) { Float64 x = switch (n) { case 1 -> 1; default -> 0.5; }; var s = switch (n) { case 1 -> new Shape(); default -> new Circle(); }; Shape t = s; return x; } }"
        )],
    ),
    rule(
        "04",
        "A switch is exhaustive when it has a `default` arm, when its constants are both `true` and `false`, when its constants are every constant of an enum, or when its type arms cover the selector's type",
        &[
            (
                "enum E { ONE, TWO } class A { Int f(E e) { return switch (e) { case E.ONE -> 1; }; } }",
                EXHAUSTIVE,
            ),
            (
                "class A { Int f(Int n) { return switch (n) { case 1 -> 1; }; } }",
                EXHAUSTIVE,
            ),
            (
                "class A { Int f(Boolean b) { return switch (b) { case true -> 1; }; } }",
                EXHAUSTIVE,
            ),
        ],
        &[
            "enum E { ONE, TWO } class A { Int f(Int n, Boolean b, E e, Object o) { var a = switch (n) { case 1 -> 1; default -> 0; }; var c = switch (b) { case true -> 1; case false -> 0; }; var d = switch (e) { case E.ONE -> 1; case E.TWO -> 2; }; var g = switch (o) { case Object x -> 1; }; return a + c + d + g; } }",
        ],
    ),
    rule(
        "04",
        "Type arms cover a type `S` when one of them names `S` or a supertype of `S`",
        &[(
            shapes!("class A { Int f(Shape c) { return switch (c) { case Circle s -> 1; }; } }"),
            EXHAUSTIVE,
        )],
        &[shapes!(
            "class A { Int f(Circle c) { return switch (c) { case Shape s -> 1; }; } Int g(Circle c) { return switch (c) { case Circle x -> 2; }; } }"
        )],
    ),
    rule(
        "04",
        "They also cover `S` when `S` is a sealed interface or a sealed abstract class and they cover every type in its `permits` clause",
        &[(
            "sealed interface Expr permits Num, Neg { } value class Num implements Expr { public Int value; } value class Neg implements Expr { public Expr operand; } class A { Int f(Expr e) { return switch (e) { case Num n -> 1; }; } }",
            EXHAUSTIVE,
        )],
        &[
            "sealed interface Expr permits Num, Neg { } value class Num implements Expr { public Int value; } value class Neg implements Expr { public Expr operand; } class A { Int f(Expr e) { return switch (e) { case Num n -> 1; case Neg n -> 2; }; } }",
            "sealed abstract class Node permits Leaf, Fork { } class Leaf extends Node { } class Fork extends Node { } class A { Int f(Node e) { return switch (e) { case Leaf n -> 1; case Fork k -> 2; }; } }",
        ],
    ),
    ran(
        "04",
        "`break` leaves the innermost enclosing `while`, `for` or switch statement",
        RUN,
        "break and continue: 00 10 20",
    ),
    ran(
        "04",
        "`continue` starts the next iteration of the innermost enclosing loop",
        RUN,
        "break and continue: 00 10 20 | 1 3",
    ),
    ran(
        "04",
        "A labeled statement is `label: statement`",
        RUN,
        "labels: 00 01 10 11 in out",
    ),
    rule(
        "04",
        "`break label` completes that statement, and `continue label` continues it, which requires it to be a loop",
        &[(
            "class A { void f() { block: { continue block; } } }",
            "`continue` with a label continues a loop, and this label is not on a loop",
        )],
        &[
            "class A { void f() { outer: for (var i = 0; i < 2; i++) { continue outer; } inner: { break inner; } } }",
        ],
    ),
    ran(
        "04",
        "`return;` returns from a method whose result is `Unit`",
        RUN,
        "return: before",
    ),
    ran(
        "04",
        "`throw e;` raises the value of `e`, whose static type is `Throwable` or a subclass of it",
        RUN,
        "throw: thrown",
    ),
    stated(
        "04",
        "defines exceptions, `try` and `using`",
        "It points to chapter 9, whose sentences have their own cases.",
    ),
    ran(
        "04",
        "`assert c;` evaluates `c`, which has type `Boolean`, and raises `AssertionException` when it is `false`",
        RUN,
        "assert: passes raises []",
    ),
    ran(
        "04",
        "`assert c : detail;` also evaluates `detail` in that case and gives `detail.toString()` to the exception as its message",
        RUN,
        "assert: passes raises [] [a detail]",
    ),
    ran(
        "04",
        "Assertions are always checked",
        RUN,
        "assert: passes raises",
    ),
    stated(
        "04",
        "No mode removes them",
        "It says what an implementation does not offer. `cleatc` has no option that does, and the run tests show the assertion checked.",
    ),
    // ---- 4.10 ----
    rule(
        "04",
        "A parameter `T... name` is a parameter of type `T[]` and is the last parameter",
        &[(
            "class A { static Int f(Int... rest, String label) { return 1; } }",
            "a varargs parameter is the last parameter",
        )],
        &[
            "class A { static Int f(String label, Int... rest) { Int[] same = rest; return rest.length(); } Int g(Int[] xs) { return f(\"a\") + f(\"a\", 1) + f(\"a\", 1, 2) + f(\"a\", xs); } }",
        ],
    ),
    ran(
        "04",
        "a single trailing argument that is assignable to `T[]` is passed as that array",
        RUN,
        "varargs: one two three 3 3 true",
    ),
    ran(
        "04",
        "Otherwise the trailing arguments are evaluated from left to right and stored in a new `T[]`",
        RUN,
        "varargs: one two three 3 3",
    ),
    ran(
        "04",
        "No trailing arguments give a zero-length array",
        RUN,
        "varargs: one two three 3 3 true 0",
    ),
    // ---- 4.11 ----
    ran("04", "Its body is a C function", RUN, "foreign: 5"),
    held(
        "04",
        "Each parameter type and the result type is a machine type",
        "spec_tables",
        "the_machine_types_of_section_4_11_are_accepted_on_a_foreign_method",
    ),
    held(
        "04",
        "A result written `void` is the C result `void`",
        "host",
        "foreign_structs",
    ),
    rule(
        "04",
        "A parameter, but not a result, may also be `T[]` where `T` is a machine type",
        &[(
            "class A { foreign static UInt8[] make(Int32 n); }",
            "the result of a `foreign` method is a machine type or `void`, and `UInt8[]` is neither",
        )],
        &[
            "class A { foreign static void f(Int32[] xs, Int count); foreign static UInt64 strlen(UInt8[] text); }",
        ],
    ),
    ran(
        "04",
        "The C function receives a pointer to the first element, with the elements laid out consecutively as C lays out `T`",
        RUN,
        "foreign: 5 9000000000 2",
    ),
    stated(
        "04",
        "It may read and write the array's elements through that pointer until the call returns, and not afterwards",
        "It says what a C function may do, and C code is outside what a Cleat program can show.",
    ),
    ran(
        "04",
        "Its writes are in the array when the call returns",
        RUN,
        "foreign: 5 9000000000 2 770",
    ),
    ran(
        "04",
        "The length is not passed",
        RUN,
        "foreign: 5 9000000000 2",
    ),
    held(
        "04",
        "A program that needs it passes `a.length()` as another argument",
        "host",
        "foreign_structs",
    ),
    rule(
        "04",
        "That includes `String`, `Rational`, a reference class, an interface, a type parameter, and a `@Nullable` type",
        &[
            (
                "class A { foreign static Int32 f(String s); }",
                FOREIGN_PARAM,
            ),
            (
                "class A { foreign static Int32 f(Rational r); }",
                FOREIGN_PARAM,
            ),
            (
                "class A { foreign static Int32 f(Object o); }",
                FOREIGN_PARAM,
            ),
            (
                "interface I { } class A { foreign static Int32 f(I i); }",
                FOREIGN_PARAM,
            ),
            (
                "class A { foreign static <T> Int32 f(T t); }",
                FOREIGN_PARAM,
            ),
            (
                "class A { foreign static Int32 f(@Nullable Int32 n); }",
                FOREIGN_PARAM,
            ),
        ],
        &[],
    ),
    rule(
        "04",
        "`Pointer` is a final value class that holds an address",
        &[(
            "class A extends Pointer { }",
            "`Pointer` is not a class that can be extended",
        )],
        &[],
    ),
    ran(
        "04",
        "`equals` compares addresses, and `Pointer.zero()` is the null address",
        RUN,
        "foreign: 5 9000000000 2 770 false true",
    ),
    rule(
        "04",
        "A program passes it back to foreign methods",
        &[],
        &[
            "class A { foreign static Pointer malloc(UInt64 n); foreign static void free(Pointer p); void f() { var p = malloc(8); free(p); var same = p == Pointer.zero(); } }",
        ],
    ),
    stated(
        "04",
        "Memory that C code allocates is not collected",
        "It says what the collector leaves alone, and a program has no way to see storage that C allocated.",
    ),
    held(
        "04",
        "The call uses the platform's C calling convention for those types",
        "host",
        "foreign_structs",
    ),
    ran(
        "04",
        "The C symbol is the method's name, unless the method carries `@Symbol(\"name\")`, which names another",
        RUN,
        "foreign: 5 9000000000",
    ),
    held(
        "04",
        "The implementation is told which libraries supply the symbols when the program is linked",
        "host",
        "foreign_structs",
    ),
    stated(
        "04",
        "C code is outside this specification",
        "It says what the specification leaves out.",
    ),
];
