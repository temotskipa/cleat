//! Chapter 6, Numbers and text: the sentences that define and describe.

use super::{CONSTANT_RAISES, Rule, held, ran, rule, stated};

const RUN: &str = "Sentences06";
const INFER: &str = "cannot be inferred from this call";
const NOT_BOOLEAN: &str = "`Int` is not assignable to `Boolean`";
const METHODS: &str = "the_numeric_classes_declare_the_methods_chapter_6_lists";

pub const SENTENCES: &[Rule] = &[
    // ---- 6.1 ----
    held(
        "06",
        "The prelude declares these numeric classes",
        "spec_tables",
        METHODS,
    ),
    rule(
        "06",
        "| `Int8`, `Int16`, `Int32`, `Int` | Signed integers of 8, 16, 32 and 64 bits |",
        &[
            (
                "class A { void f() { Int8 a = 128; } }",
                "the literal is outside the range of `Int8`",
            ),
            (
                "class A { void f() { Int16 a = 32768; } }",
                "the literal is outside the range of `Int16`",
            ),
            (
                "class A { void f() { Int a = 9223372036854775808; } }",
                "the literal is outside the range of `Int`",
            ),
        ],
        &[
            "class A { void f() { Int8 a = -128; Int8 b = 127; Int16 c = -32768; Int16 d = 32767; Int32 e = -2147483648; Int32 g = 2147483647; Int h = -9223372036854775808; Int i = 9223372036854775807; } }",
        ],
    ),
    rule(
        "06",
        "| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned integers of those widths |",
        &[
            (
                "class A { void f() { UInt8 a = 256; } }",
                "the literal is outside the range of `UInt8`",
            ),
            (
                "class A { void f() { UInt8 a = -1; } }",
                "the literal is outside the range of `UInt8`",
            ),
            (
                "class A { void f() { UInt64 a = 18446744073709551616; } }",
                "the literal is outside the range of `UInt64`",
            ),
        ],
        &[
            "class A { void f() { UInt8 a = 0; UInt8 b = 255; UInt16 c = 65535; UInt32 d = 4294967295; UInt64 e = 18446744073709551615; } }",
        ],
    ),
    ran(
        "06",
        "| `Float32`, `Float64` | IEEE 754 binary32 and binary64 |",
        RUN,
        "floats: 0.30000000000000004 16777216.0",
    ),
    ran(
        "06",
        "| `Rational` | Every rational number, exactly |",
        RUN,
        "rationals: 1/2 true",
    ),
    stated(
        "06",
        "The first two rows are the integer classes",
        "It gives a name to eight classes, and the sentences that use the name have their own cases.",
    ),
    stated(
        "06",
        "The third row is the float classes",
        "It gives a name to two classes, and the sentences that use the name have their own cases.",
    ),
    rule(
        "06",
        "Each numeric class is a final value class",
        &[
            (
                "class A extends Int { }",
                "`Int` is not a class that can be extended",
            ),
            (
                "class A extends Rational { }",
                "`Rational` is not a class that can be extended",
            ),
        ],
        &[],
    ),
    rule(
        "06",
        "`Int` is the everyday integer",
        &[],
        &["class A { Int f() { var i = 0; Int n = i; return n + \"abc\".length(); } }"],
    ),
    rule(
        "06",
        "An array length, an index, a `String` position, a shift count and a hash code are `Int`",
        &[
            (
                "class A { Int32 f(Int[] xs) { return xs.length(); } }",
                "`Int` is not assignable to `Int32`",
            ),
            (
                "class A { Int f(Int[] xs, UInt64 i) { return xs[i]; } }",
                "`UInt64` is not assignable to `Int`",
            ),
        ],
        &[
            "class A { Int f(Int[] xs, String s, Int8 b) { Int a = xs.length(); Int c = s.indexOf('x'); Int d = s.hashCode(); return xs[a - 1] + b.shiftLeft(a) + c + d; } }",
        ],
    ),
    rule(
        "06",
        "There is no class named `Int64`",
        &[(
            "class A { void f() { Int64 x = 1; } }",
            "there is no type named `Int64` here",
        )],
        &[],
    ),
    // ---- 6.2 ----
    rule(
        "06",
        "What the numeric classes share is declared by interfaces",
        &[],
        &[
            "class A { static <T extends Numeric<T> & Ordered<T>> T larger(T a, T b) { return a < b ? b : a; } Int f() { return larger(1, 2); } Rational g() { return larger(0.5, 0.25); } }",
        ],
    ),
    rule(
        "06",
        "Every numeric class `C` implements `Numeric<C>` and `Ordered<C>`",
        &[(
            "class A { static <T extends Numeric<T> & Ordered<T>> void need(T x) { } void f(String s) { need(s); } }",
            INFER,
        )],
        &[
            "class A { static <T extends Numeric<T> & Ordered<T>> void need(T x) { } void f(Int8 a, Int16 b, Int32 c, Int d, UInt8 e, UInt16 g, UInt32 h, UInt64 i, Float32 j, Float64 k, Rational l) { need(a); need(b); need(c); need(d); need(e); need(g); need(h); need(i); need(j); need(k); need(l); } }",
        ],
    ),
    rule(
        "06",
        "`Float32`, `Float64` and `Rational` also implement `Divisible`",
        &[(
            "class A { static <T extends Divisible<T>> void need(T x) { } void f(Int d) { need(d); } }",
            INFER,
        )],
        &[
            "class A { static <T extends Divisible<T>> void need(T x) { } void f(Float32 j, Float64 k, Rational l) { need(j); need(k); need(l); } }",
        ],
    ),
    rule(
        "06",
        "`zero` and `one` are static interface methods",
        &[(
            "class A { Int f() { return Numeric.zero(); } }",
            "`zero` is a static requirement of `Numeric`; it is called through a type parameter, not on the interface",
        )],
        &[
            "class A { Int f() { return Int.zero() + Int.one(); } Rational g() { return Rational.zero() + Rational.one(); } static <T extends Numeric<T>> T h() { return T.zero(); } }",
        ],
    ),
    ran(
        "06",
        "Each implementing class declares them, and generic code calls them through a type parameter",
        RUN,
        "zero and one: 0 1, 0 1, 0.0 1.0, 0 1, Steps(0) Steps(1)",
    ),
    ran(
        "06",
        "reach these methods",
        RUN,
        "operators: plus lessThan minus negate times atMost atLeast greaterThan",
    ),
    ran(
        "06",
        "`a + b` is `a.plus(b)`, and `a < b` is `a.lessThan(b)`",
        RUN,
        "operators: plus lessThan",
    ),
    rule(
        "06",
        "An arithmetic method takes and returns its own class",
        &[(
            "class A { Int8 f(Int8 a, Int big) { return a + big; } }",
            "`Int` is not assignable to `Int8`",
        )],
        &[
            "class A { Int8 f(Int8 a, Int8 b) { Int8 c = a + b; Int8 d = a * b - a; return -c; } Float32 g(Float32 a) { return a + a; } }",
        ],
    ),
    ran(
        "06",
        "`i + n` is an `Int` addition",
        RUN,
        "mixed operands: 2147483648 cleat.Int",
    ),
    ran(
        "06",
        "A class a program declares may implement these interfaces, and its instances then take the same operators",
        RUN,
        "operators: plus lessThan minus negate times atMost atLeast greaterThan | 7 true -6 true",
    ),
    stated(
        "06",
        "Nothing in this chapter depends on a class being declared by the prelude",
        "It says what the chapter leaves out. The sentence before it has the case of a class the program declares.",
    ),
    // ---- 6.3 ----
    rule(
        "06",
        "An integer literal and a decimal literal are the lexical forms of",
        &[],
        &[
            "class A { void f() { Int a = 255; Int b = 0xFF; Rational c = 1.5; Rational d = 2.5e3; Int e = 1_000; } }",
        ],
    ),
    ran(
        "06",
        "Each denotes its exact mathematical value",
        RUN,
        "defaults: cleat.Rational cleat.Int cleat.Rational 1999/100 1/3",
    ),
    rule(
        "06",
        "A `-` written directly before a numeric literal is part of that literal, so `-128` is one literal",
        &[(
            "class A { void f() { Int8 x = -(128); } }",
            "the literal is outside the range of `Int8`",
        )],
        &["class A { void f() { Int8 x = -128; Int m = -9223372036854775808; } }"],
    ),
    rule(
        "06",
        "A literal expression is a numeric literal, a parenthesized literal expression, or an arithmetic operator applied to literal expressions",
        &[],
        &[
            "class A { void f() { Float64 x = (1 + 2) * 0.5; Int8 y = 100 + 27; Float32 z = -(1.5); } }",
        ],
    ),
    stated(
        "06",
        "The literals of a literal expression take their class from the first of these rules that applies",
        "It introduces the three rules after it, which have their own cases.",
    ),
    rule(
        "06",
        "The expression stands where a value of a numeric class `C` is expected",
        &[
            (
                "class A { static void take(UInt8 b) { } void g() { take(256); } }",
                "the literal is outside the range of `UInt8`",
            ),
            (
                "class A { Int8[] g() { return new Int8[] { 1, 200 }; } }",
                "the literal is outside the range of `Int8`",
            ),
        ],
        &[
            "class A { Float64 f = 0.5; static void take(UInt8 b) { } Float32 g() { Float64 dt = 0.001; f = 2.5; take(255); var xs = new Int8[] { 1, 2, -128 }; return 0.1; } }",
        ],
    ),
    rule(
        "06",
        "Every literal in it has class `C`",
        &[(
            "class A { void f() { Int8 x = 100 + 100; } }",
            CONSTANT_RAISES,
        )],
        &["class A { void f() { Float64 x = 1 + 0.5; Int8 y = 100 + 27; } }"],
    ),
    rule(
        "06",
        "`@Nullable C` gives the same expected type as `C`",
        &[(
            "class A { void f() { @Nullable UInt8 z = 256; } }",
            "the literal is outside the range of `UInt8`",
        )],
        &["class A { void f() { @Nullable Float64 x = 0.5; @Nullable UInt8 y = 255; } }"],
    ),
    rule(
        "06",
        "For `??`, a left operand of type `@Nullable C` counts as `C`",
        &[(
            "class A { UInt8 g(@Nullable UInt8 x) { return x ?? 256; } }",
            "the literal is outside the range of `UInt8`",
        )],
        &["class A { Float64 g(@Nullable Float64 x) { return x ?? 0.5; } }"],
    ),
    ran(
        "06",
        "Every literal in the expression has class `Rational` if any of them is a decimal literal, and class `Int` otherwise",
        RUN,
        "defaults: cleat.Rational cleat.Int cleat.Rational",
    ),
    rule(
        "06",
        "An expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression",
        &[
            (
                "class A { UInt8 g(Boolean c) { return c ? 1 : 256; } }",
                "the literal is outside the range of `UInt8`",
            ),
            (
                "class A { UInt8 g(Int n) { return switch (n) { case 1 -> 1; default -> 256; }; } }",
                "the literal is outside the range of `UInt8`",
            ),
        ],
        &[
            "class A { Float64 f(Boolean c, Int n) { Float64 a = (0.5); Float64 b = c ? 0.5 : 1; Float64 d = switch (n) { case 1 -> 0.5; default -> 2; }; return a + b + d; } }",
        ],
    ),
    rule(
        "06",
        "When `C` is an integer class, the literal is an integer literal in the range of `C`",
        &[(
            "class A { UInt8 g(UInt8 x) { return x + 256; } }",
            "the literal is outside the range of `UInt8`",
        )],
        &["class A { UInt8 g(UInt8 x) { Int8 a = 127; UInt16 b = 65535; return x + 255; } }"],
    ),
    ran(
        "06",
        "When `C` is a float class, the literal denotes the value of `C` nearest to it, ties to even",
        RUN,
        "the nearest float: 0.1 true 16777216.0",
    ),
    ran(
        "06",
        "When `C` is `Rational`, the literal denotes its exact value",
        RUN,
        "defaults: cleat.Rational cleat.Int cleat.Rational 1999/100",
    ),
    ran(
        "06",
        "determines the type from the rest of the call when it can, and the literal then has that expected type",
        "Sentences07",
        "a literal: cleat.Int8",
    ),
    ran(
        "06",
        "When nothing else determines it, the literal takes its default class",
        "Sentences07",
        "a literal: cleat.Int8 cleat.Int cleat.Rational",
    ),
    ran(
        "06",
        "a literal expression is applicable to a parameter of numeric class `C` when the rules above accept it at `C`",
        RUN,
        "overloads: Float64 Float64",
    ),
    ran(
        "06",
        "the method whose parameter class is the literal's default class is chosen",
        RUN,
        "overloads: Float64 Float64 Int Rational",
    ),
    // ---- 6.4 ----
    ran(
        "06",
        "That includes negating the minimum of a signed class and negating a nonzero unsigned value",
        RUN,
        "negate: raises raises 7",
    ),
    stated(
        "06",
        "Integer division is written by name",
        "It introduces the table after it, whose rows have their own cases.",
    ),
    ran(
        "06",
        "| `floorDiv` | The quotient rounded toward negative infinity |",
        RUN,
        "floorDiv: -3 -3 2",
    ),
    ran(
        "06",
        "| `mod` | `a - a.floorDiv(b) * b`.",
        RUN,
        "mod: 2 -2 1 0 2",
    ),
    ran(
        "06",
        "It is zero or has the sign of the divisor.",
        RUN,
        "mod: 2 -2 1 0",
    ),
    ran("06", "`a % b` is `a.mod(b)` |", RUN, "mod: 2 -2 1 0 2 2 -2"),
    ran(
        "06",
        "| `truncatingDiv` | The quotient rounded toward zero |",
        RUN,
        "truncatingDiv: -2 -2",
    ),
    ran(
        "06",
        "| `truncatingRem` | `a - a.truncatingDiv(b) * b`.",
        RUN,
        "truncatingRem: -1 1 0 -1",
    ),
    ran(
        "06",
        "It is zero or has the sign of the dividend |",
        RUN,
        "truncatingRem: -1 1 0",
    ),
    ran(
        "06",
        "Each takes and returns the receiver's class",
        RUN,
        "the receiver's class: cleat.Int8 cleat.Int8 cleat.UInt8 cleat.UInt8",
    ),
    ran(
        "06",
        "All four raise `ArithmeticException` when the divisor is zero",
        RUN,
        "a zero divisor: raises raises raises raises",
    ),
    ran("06", "`-7 % 3` is `2`", RUN, "remainders: 2"),
    ran(
        "06",
        "`(-7).truncatingRem(3)` is `-1`, the value that `%` gives in Java and C",
        RUN,
        "remainders: 2 -1",
    ),
    ran(
        "06",
        "return the low bits of the mathematical result, interpreted in the class",
        RUN,
        "wrapping: -128 127 44 44 254",
    ),
    rule(
        "06",
        "are declared on the integer classes and on no other numeric class",
        &[
            (
                "class A { Float64 f(Float64 x) { return x.and(x); } }",
                "`Float64` has no method named `and`",
            ),
            (
                "class A { Rational f(Rational x) { return x.shiftLeft(1); } }",
                "`Rational` has no method named `shiftLeft`",
            ),
            (
                "class A { Float64 f(Float64 x) { return x.complement(); } }",
                "`Float64` has no method named `complement`",
            ),
        ],
        &[
            "class A { Int f(Int a, Int8 b, UInt64 c) { return a.and(a).or(a).xor(a).complement().shiftLeft(1).shiftRight(1) + b.and(b) + Int.from(c.xor(c)); } }",
        ],
    ),
    rule(
        "06",
        "A shift count is an `Int`",
        &[
            (
                "class A { Int8 f(Int8 a, Float64 n) { return a.shiftLeft(n); } }",
                "`Float64` is not assignable to `Int`",
            ),
            (
                "class A { Int f(Int a, UInt64 n) { return a.shiftLeft(n); } }",
                "`UInt64` is not assignable to `Int`",
            ),
        ],
        &[
            "class A { Int8 f(Int8 a, Int n) { return a.shiftLeft(n); } UInt64 g(UInt64 a, Int n) { return a.shiftRight(n); } }",
        ],
    ),
    ran(
        "06",
        "A count below zero, or not below the width of the class, raises `ArithmeticException`",
        RUN,
        "a shift count: raises raises raises -128",
    ),
    ran(
        "06",
        "`shiftLeft` discards the bits it shifts out",
        RUN,
        "shifts: 4 144",
    ),
    ran(
        "06",
        "`shiftRight` copies the sign bit on a signed class and fills with zero on an unsigned class",
        RUN,
        "shifts: 4 144 -64 64",
    ),
    // ---- 6.5 ----
    ran(
        "06",
        "on a float class follow IEEE 754, rounding to nearest with ties to even",
        RUN,
        "ieee: 0.30000000000000004 0.30000000000000004 0.3333333333333333 -0.1 -0.1",
    ),
    ran(
        "06",
        "Overflow gives an infinity",
        RUN,
        "overflow: Infinity -Infinity Infinity",
    ),
    ran(
        "06",
        "Division by zero gives an infinity or NaN",
        RUN,
        "division by zero: Infinity -Infinity NaN",
    ),
    ran(
        "06",
        "take no argument and return a value of the same class with no fractional part",
        RUN,
        "no fraction: -3.0 -2.0 -2.0 -2.0 cleat.Float64 cleat.Float32",
    ),
    ran(
        "06",
        "`round` rounds to nearest, ties to even",
        RUN,
        "round: 2.0 2.0 4.0 2.0",
    ),
    ran(
        "06",
        "A NaN or an infinity is returned unchanged",
        RUN,
        "unchanged: NaN Infinity -Infinity NaN",
    ),
    ran(
        "06",
        "`sqrt` returns the correctly rounded square root",
        RUN,
        "sqrt: 1.4142135623730951 3.0",
    ),
    // ---- 6.6 ----
    ran(
        "06",
        "A `Rational` is one exact rational number",
        RUN,
        "the value: true true 1/2",
    ),
    ran(
        "06",
        "`div` and `negate` return the exact result",
        RUN,
        "exact results: 1/2 1/6 1/18 2 -1/3",
    ),
    ran(
        "06",
        "`div` raises `ArithmeticException` when the divisor is zero",
        RUN,
        "a zero divisor of a rational: raises",
    ),
    ran(
        "06",
        "No operation rounds, wraps, or overflows",
        RUN,
        "no overflow: 340282366920938463463374607431768211456 1/1494186269970473680896",
    ),
    ran(
        "06",
        "| `isInteger()` | Whether the value is an integer |",
        RUN,
        "isInteger: true false",
    ),
    ran(
        "06",
        "| `numerator()`, `denominator()` | The value in lowest terms",
        RUN,
        "lowest terms: 3 4 -3 4 true",
    ),
    ran(
        "06",
        "The denominator is positive |",
        RUN,
        "lowest terms: 3 4 -3 4",
    ),
    ran(
        "06",
        "| `floor()`, `ceil()`, `truncate()` | The nearest integer in that direction |",
        RUN,
        "nearest integers: -4 -3 -3 3 4 3",
    ),
    ran(
        "06",
        "| `round()` | The nearest integer, ties to even |",
        RUN,
        "round of a rational: 4 2 -4",
    ),
    ran(
        "06",
        "| `round(Int places)` | The nearest multiple of ten to the power `-places`, ties to even |",
        RUN,
        "round to places: 117/50 23/10 1200 333/1000",
    ),
    ran(
        "06",
        "| `floorDiv`, `mod` | As",
        RUN,
        "floorDiv and mod of rationals: 2 1/2 -3 1",
    ),
    ran(
        "06",
        "| `toDecimal(Int places)` | The digits of `round(places)`, with exactly `places` digits after the point |",
        RUN,
        "toDecimal: 2.34 4.000 0.3333 4",
    ),
    ran(
        "06",
        "`toString` returns decimal digits for an integer and `numerator/denominator` otherwise, so one half is `1/2`",
        RUN,
        "toString of a rational: 4 1/2 -7/2",
    ),
    ran(
        "06",
        "A program may observe the mathematical value of a `Rational`",
        RUN,
        "the value: true true 1/2",
    ),
    stated(
        "06",
        "How the value is stored is not observable, and every representation that preserves the value is correct",
        "It says that a choice of the implementation cannot be seen, so no program shows it.",
    ),
    // ---- 6.7 ----
    held(
        "06",
        "The numeric classes declare implicit conversions in three cases",
        "spec_tables",
        "the_implicit_conversions_are_exactly_the_table_of_section_6_7",
    ),
    ran(
        "06",
        "None of them can lose information",
        RUN,
        "implicit conversions: -128 -128 -128 -128 18446744073709551615 true",
    ),
    rule(
        "06",
        "An integer class converts to an integer class that holds every value of the first",
        &[
            (
                "class A { Int8 f(Int16 b) { return b; } }",
                "`Int16` is not assignable to `Int8`",
            ),
            (
                "class A { UInt8 f(Int8 b) { return b; } }",
                "`Int8` is not assignable to `UInt8`",
            ),
            (
                "class A { Int f(UInt64 b) { return b; } }",
                "`UInt64` is not assignable to `Int`",
            ),
            (
                "class A { Int8 f(UInt8 b) { return b; } }",
                "`UInt8` is not assignable to `Int8`",
            ),
        ],
        &[
            "class A { void f(Int8 a, Int16 b, Int32 c, UInt8 d, UInt16 e, UInt32 g) { Int16 x1 = a; Int32 x2 = a; Int x3 = a; Int32 x4 = b; Int x5 = b; Int x6 = c; UInt16 y1 = d; UInt32 y2 = d; UInt64 y3 = d; Int16 y4 = d; Int32 y5 = d; Int y6 = d; UInt32 z1 = e; UInt64 z2 = e; Int32 z3 = e; Int z4 = e; UInt64 w1 = g; Int w2 = g; } }",
        ],
    ),
    rule(
        "06",
        "`Float32` converts to `Float64`",
        &[(
            "class A { Float32 f(Float64 x) { return x; } }",
            "`Float64` is not assignable to `Float32`",
        )],
        &["class A { Float64 f(Float32 x) { return x; } }"],
    ),
    rule(
        "06",
        "An integer class converts to `Rational`",
        &[(
            "class A { Rational f(Float64 x) { return x; } }",
            "`Float64` is not assignable to `Rational`",
        )],
        &[
            "class A { void f(Int8 a, Int16 b, Int32 c, Int d, UInt8 e, UInt16 g, UInt32 h, UInt64 i) { Rational r = a; r = b; r = c; r = d; r = e; r = g; r = h; r = i; } }",
        ],
    ),
    rule(
        "06",
        "No integer class converts implicitly to a float class",
        &[
            (
                "class A { Float64 f(Int32 small) { return small; } }",
                "`Int32` is not assignable to `Float64`",
            ),
            (
                "class A { Float32 f(Int8 small) { return small; } }",
                "`Int8` is not assignable to `Float32`",
            ),
            (
                "class A { Float64 f(Int n) { return n; } }",
                "`Int` is not assignable to `Float64`",
            ),
        ],
        &[],
    ),
    rule(
        "06",
        "says where an implicit conversion is applied",
        &[],
        &[
            "class A { static Int take(Int n) { return n; } Rational f(Int32 small) { Int count = small; Int again = take(small); Rational price = 4.35; Rational total = price * count; return count; } }",
        ],
    ),
    rule(
        "06",
        "A conversion that rounds, truncates or wraps has a name that says so",
        &[],
        &[
            "class A { Float64 f(Rational r, Float64 x, Int8 a) { return Float64.nearest(r) + x.truncate() + x.round() + Float64.nearest(a.wrappingPlus(a)); } }",
        ],
    ),
    held(
        "06",
        "Each numeric class `C` declares a static `from` for every numeric class",
        "spec_tables",
        METHODS,
    ),
    ran(
        "06",
        "`Int.from(i32)` always succeeds",
        RUN,
        "from: 2147483647",
    ),
    ran(
        "06",
        "`Int32.from(n)` raises when `n` is outside 32 bits",
        RUN,
        "from: 2147483647 raises",
    ),
    ran(
        "06",
        "`Float64.from(r)` raises when the rational `r` is not exactly a `Float64`, which is the case for one tenth",
        RUN,
        "from: 2147483647 raises raises 0.5",
    ),
    ran(
        "06",
        "`Float32.nearest(x)` and `Float64.nearest(x)` are the rounding conversions",
        RUN,
        "nearest: 0.1 16777216.0 9007199254740992.0",
    ),
    rule(
        "06",
        "They accept every numeric class and return the nearest value of the float class, ties to even",
        &[(
            "class A { Float64 f(String s) { return Float64.nearest(s); } }",
            "no method `nearest` accepts the arguments (String)",
        )],
        &[
            "class A { void f(Int8 a, Int16 b, Int32 c, Int d, UInt8 e, UInt16 g, UInt32 h, UInt64 i, Float32 j, Float64 k, Rational l) { var x = Float64.nearest(a) + Float64.nearest(b) + Float64.nearest(c) + Float64.nearest(d) + Float64.nearest(e) + Float64.nearest(g) + Float64.nearest(h) + Float64.nearest(i) + Float64.nearest(j) + Float64.nearest(k) + Float64.nearest(l); var y = Float32.nearest(a) + Float32.nearest(b) + Float32.nearest(c) + Float32.nearest(d) + Float32.nearest(e) + Float32.nearest(g) + Float32.nearest(h) + Float32.nearest(i) + Float32.nearest(j) + Float32.nearest(k) + Float32.nearest(l); } }",
        ],
    ),
    ran(
        "06",
        "A magnitude too large for the class gives an infinity",
        RUN,
        "nearest: 0.1 16777216.0 9007199254740992.0 Infinity -Infinity",
    ),
    rule(
        "06",
        "A float becomes an integer in two steps, so the rounding direction is written",
        &[(
            "class A { Int f(Float64 x) { return x; } }",
            "`Float64` is not assignable to `Int`",
        )],
        &[
            "class A { Int f(Float64 x) { return Int.from(x.floor()) + Int.from(x.ceil()) + Int.from(x.round()) + Int.from(x.truncate()); } }",
        ],
    ),
    ran(
        "06",
        "`Int.from(f.floor())`",
        RUN,
        "two steps: 2 3 -3 raises",
    ),
    // ---- 6.8 ----
    ran(
        "06",
        "`==` is `equals`, as",
        RUN,
        "== is equals: equals true",
    ),
    ran(
        "06",
        "`equals` on a numeric class is `true` exactly when the argument is a value of the same class with the same numeric value",
        RUN,
        "equals: true false false true",
    ),
    ran(
        "06",
        "On a float class, NaN equals NaN, and positive zero equals negative zero",
        RUN,
        "float equals: true true true",
    ),
    ran(
        "06",
        "`hashCode` agrees with `equals`",
        RUN,
        "hashCode: true true true",
    ),
    ran(
        "06",
        "When the operands of `==` have two different numeric classes, one is converted to the other",
        RUN,
        "mixed ==: true true false",
    ),
    ran(
        "06",
        "A direct call of `equals` converts nothing, so it is `false` for values of two different classes",
        RUN,
        "mixed equals: false false",
    ),
    ran(
        "06",
        "`lessThan`, `atMost`, `greaterThan` and `atLeast` compare numeric values",
        RUN,
        "comparisons: true true false true true true",
    ),
    ran(
        "06",
        "each is `false` when either operand is NaN",
        RUN,
        "comparisons with NaN: false false false false false false",
    ),
    ran(
        "06",
        "`compare` returns a negative `Int`, zero, or a positive `Int`",
        RUN,
        "compare: negative zero positive negative positive",
    ),
    ran(
        "06",
        "It returns zero exactly when `equals` is `true`",
        RUN,
        "compare and equals: zero zero zero",
    ),
    ran(
        "06",
        "On a float class NaN compares greater than every other value, so `compare` is a total order and is the order a sort uses",
        RUN,
        "a total order: positive negative | -Infinity 0.0 1.0 Infinity NaN",
    ),
    ran(
        "06",
        "Each float class also declares `Int totalOrder(C other)`, the IEEE 754 totalOrder",
        RUN,
        "totalOrder: negative positive zero",
    ),
    ran(
        "06",
        "It distinguishes negative zero from positive zero and one NaN from another",
        RUN,
        "totalOrder: negative positive zero true",
    ),
    ran(
        "06",
        "It is not `==`",
        RUN,
        "totalOrder: negative positive zero true true",
    ),
    // ---- 6.9 ----
    rule(
        "06",
        "`Char` is a final value class",
        &[(
            "class A extends Char { }",
            "`Char` is not a class that can be extended",
        )],
        &[],
    ),
    ran(
        "06",
        "A value is one Unicode scalar: 0 through 10FFFF, excluding the surrogates",
        RUN,
        "Char.from: A 1114111 raises raises raises",
    ),
    ran(
        "06",
        "`Char.from(Int)` returns the scalar with that number, or raises `ArithmeticException`",
        RUN,
        "Char.from: A 1114111 raises",
    ),
    ran(
        "06",
        "`code()` returns the number as an `Int`",
        RUN,
        "code: 97 cleat.Int",
    ),
    ran(
        "06",
        "`Char` implements `Ordered<Char>` by scalar number",
        RUN,
        "the order of Char: true false negative negative",
    ),
    rule(
        "06",
        "It is not numeric:",
        &[
            (
                "class A { static <T extends Numeric<T>> void need(T x) { } void f() { need('a'); } }",
                INFER,
            ),
            (
                "class A { Char f(Char c) { return c.plus(c); } }",
                "`Char` has no method named `plus`",
            ),
        ],
        &[],
    ),
    rule(
        "06",
        "`String` is a final value class",
        &[(
            "class A extends String { }",
            "`String` is not a class that can be extended",
        )],
        &[],
    ),
    ran(
        "06",
        "A value is a sequence of `Char`",
        RUN,
        "a sequence of Char: 3 128512 b",
    ),
    ran(
        "06",
        "| `length()` | The number of scalars, as an `Int` |",
        RUN,
        "length: 5 0 cleat.Int",
    ),
    ran(
        "06",
        "| `get(Int index)` | The scalar at that zero-based index.",
        RUN,
        "get: e e h o",
    ),
    ran("06", "`s[i]` is `s.get(i)` |", RUN, "get: e e"),
    ran(
        "06",
        "| `substring(Int begin, Int end)` | The scalars from `begin` inclusive to `end` exclusive |",
        RUN,
        "substring: el [] hello",
    ),
    ran(
        "06",
        "| `plus(@Nullable Object other)` | The receiver followed by `other.toString()` |",
        RUN,
        "plus: hello there hello5 hellonull helloloud toString",
    ),
    ran(
        "06",
        "| `indexOf(Char c)`, `indexOf(String s)` | The first index of the argument, or `-1`.",
        RUN,
        "indexOf: 2 3 -1 -1",
    ),
    ran(
        "06",
        "The empty string is found at `0` |",
        RUN,
        "indexOf: 2 3 -1 -1 0",
    ),
    ran(
        "06",
        "| `startsWith(String s)`, `endsWith(String s)` | Whether the receiver begins or ends with `s` |",
        RUN,
        "startsWith and endsWith: true false true false",
    ),
    ran(
        "06",
        "| `isEmpty()` | Whether the length is zero |",
        RUN,
        "isEmpty: true false",
    ),
    ran(
        "06",
        "| `toUtf8()` | The UTF-8 encoding, as a new `UInt8[]` |",
        RUN,
        "toUtf8: 6 97 240 98 true",
    ),
    ran(
        "06",
        "| `static fromUtf8(UInt8[] bytes)` | The string those bytes encode |",
        RUN,
        "fromUtf8: hi true",
    ),
    ran(
        "06",
        "`get` and `substring` raise `IndexOutOfBoundsException` when an index is outside the string or `begin` is greater than `end`",
        RUN,
        "outside the string: raises raises raises raises",
    ),
    ran(
        "06",
        "`equals` and `hashCode` use the sequence of scalars",
        RUN,
        "the scalars of a string: true true false",
    ),
    ran(
        "06",
        "`String` implements `Ordered<String>`",
        RUN,
        "the order of String: negative positive negative zero true",
    ),
    ran(
        "06",
        "`compare` orders by the first scalar that differs, and by length when one string is a prefix of the other",
        RUN,
        "the order of String: negative positive negative",
    ),
    ran(
        "06",
        "`String` implements `Iterable<Char>`, so a `for` statement visits its scalars in order",
        RUN,
        "a for statement over a string: 97 128512 98",
    ),
    ran(
        "06",
        "is `String.plus`, which calls `n.toString()`",
        RUN,
        "string plus: count: 7 it is loud toString",
    ),
    held(
        "06",
        "These are the `String` methods the language defines",
        "spec_tables",
        METHODS,
    ),
    stated(
        "06",
        "The prelude may declare more in ordinary source",
        "It says what the prelude may add, and no program shows that it does not.",
    ),
    // ---- 6.10 ----
    rule(
        "06",
        "An array type is written `T[]` for any type `T`, including an array type",
        &[],
        &[
            "class A { void f(Int[][] grid, String[] names, List<Int>[] lists, Int[][][] cube) { Int[] row = grid[0]; Int cell = cube[0][0][0]; } }",
        ],
    ),
    rule(
        "06",
        "`T[]` is the prelude class `Array<T>`, and a program may write either",
        &[],
        &[
            "class A { Array<Int> f(Int[] xs) { return xs; } Int[] g(Array<Int> xs) { return xs; } Array<Array<String>> h(String[][] x) { return x; } }",
        ],
    ),
    ran(
        "06",
        "An array is a reference object",
        RUN,
        "a reference object: false true 9",
    ),
    rule(
        "06",
        "Its length is fixed at creation",
        &[
            (
                "class A { void f(Int[] xs) { xs.add(1); } }",
                "`Int[]` has no method named `add`",
            ),
            (
                "class A { void f(Int[] xs) { xs.resize(4); } }",
                "`Int[]` has no method named `resize`",
            ),
        ],
        &[],
    ),
    rule(
        "06",
        "`length()` returns an `Int`",
        &[(
            "class A { Int8 f(Int[] xs) { return xs.length(); } }",
            "`Int` is not assignable to `Int8`",
        )],
        &["class A { Int f(Int[] xs) { Int n = xs.length(); return n; } }"],
    ),
    ran(
        "06",
        "`get` and `set` take an `Int` index and raise `IndexOutOfBoundsException` when it is out of range",
        RUN,
        "an array's bounds: raises raises raises raises 3",
    ),
    ran(
        "06",
        "`a[i]` is `a.get(i)`, and `a[i] = e` is `a.set(i, e)`",
        RUN,
        "indexing: 7 7 8 8",
    ),
    ran(
        "06",
        "An array implements `Iterable<T>`",
        RUN,
        "a for statement over an array: 1 7 8",
    ),
    rule(
        "06",
        "Arrays are invariant",
        &[(
            "class A { Object[] f(String[] xs) { return xs; } }",
            "`String[]` is not assignable to `Object[]`",
        )],
        &[
            "class A { String[] f(String[] xs) { return xs; } Array<? extends Object> g(String[] xs) { return xs; } }",
        ],
    ),
    rule(
        "06",
        "A store is typechecked on `T`, so no store fails at run time for its type",
        &[(
            "class A { void f(String[] xs, Object o) { xs[0] = o; } }",
            "`Object` is not assignable to `String`",
        )],
        &["class A { void f(Object[] xs, String s) { xs[0] = s; } }"],
    ),
    stated(
        "06",
        "There are three ways to create an array",
        "It introduces the three ways after it, which have their own cases.",
    ),
    ran(
        "06",
        "`new T[n]` creates `n` elements, each the default of `T`",
        RUN,
        "the defaults: null 0 0.0 0 0 0 false Pair(0, false) 3",
    ),
    ran(
        "06",
        "`n` is an `Int`, and a negative `n` raises `ArithmeticException`",
        RUN,
        "a negative length: raises 0",
    ),
    ran(
        "06",
        "The default is `null` when `T` is `@Nullable`, `zero()` when `T` is a numeric class",
        RUN,
        "the defaults: null 0 0.0 0 0 0 false Pair(0, false)",
    ),
    ran(
        "06",
        "That value is constructed once each time `new T[n]` creates an array",
        RUN,
        "a default's constructor: made | made | 0 raises raises",
    ),
    ran(
        "06",
        "evaluates the elements from left to right",
        RUN,
        "listed elements: first second third 123",
    ),
    ran(
        "06",
        "creates `n` elements by calling `element` with each index from zero upward",
        RUN,
        "build: 0 1 2 3 | 0 1 4 9 4",
    ),
    stated(
        "06",
        "How an array stores its elements is not observable",
        "It says that a choice of the implementation cannot be seen, so no program shows it.",
    ),
    // ---- 6.11 ----
    rule(
        "06",
        "`Boolean` is a final value class with exactly two instances, the literals `true` and `false`",
        &[
            (
                "class A extends Boolean { }",
                "`Boolean` is not a class that can be extended",
            ),
            (
                "class A { Boolean f() { return new Boolean(); } }",
                "the constructor of `Boolean` is outside its audience here",
            ),
        ],
        &[],
    ),
    rule(
        "06",
        "It is not numeric.",
        &[
            (
                "class A { Int f(Boolean b) { return b + 1; } }",
                "`+` is the method `plus`, and `Boolean` has no method `plus`",
            ),
            (
                "class A { static <T extends Numeric<T>> void need(T x) { } void f() { need(true); } }",
                INFER,
            ),
        ],
        &[],
    ),
    ran(
        "06",
        "It declares `and`, `or`, `xor` and `not`, which `&`, `|`, `^` and `!` spell",
        RUN,
        "boolean operators: false true true false | false true true false",
    ),
    ran(
        "06",
        "`equals` is `true` when both values are `true` or both are `false`",
        RUN,
        "boolean equals: true true false false true",
    ),
    rule(
        "06",
        "The conditions of `if`, `while`, `for`, `?:`, `&&`, `||` and `assert` have static type `Boolean`",
        &[
            ("class A { void f(Int n) { if (n) { } } }", NOT_BOOLEAN),
            ("class A { void f(Int n) { while (n) { } } }", NOT_BOOLEAN),
            (
                "class A { void f(Int n) { for (var i = 0; n; i++) { } } }",
                NOT_BOOLEAN,
            ),
            (
                "class A { Int f(Int n) { return n ? 1 : 2; } }",
                NOT_BOOLEAN,
            ),
            (
                "class A { Boolean f(Int n, Boolean b) { return n && b; } }",
                NOT_BOOLEAN,
            ),
            (
                "class A { Boolean f(Int n, Boolean b) { return b || n; } }",
                NOT_BOOLEAN,
            ),
            ("class A { void f(Int n) { assert n; } }", NOT_BOOLEAN),
            (
                "class A { void f(@Nullable Boolean b) { if (b) { } } }",
                "`@Nullable Boolean` is not assignable to `Boolean`",
            ),
        ],
        &[
            "class A { Int f(Boolean a, Boolean b) { if (a) { } while (b) { break; } for (var i = 0; a; i++) { break; } assert a || b; return a && b ? 1 : 2; } }",
        ],
    ),
    stated(
        "06",
        "Those forms are syntax, defined in",
        "It points to chapter 4, whose sentences have their own cases.",
    ),
];
