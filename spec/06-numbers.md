# 6. Numbers and text

## 6.1 The numeric classes

The prelude declares these numeric classes:

| Classes | Values |
| --- | --- |
| `Int8`, `Int16`, `Int32`, `Int` | Signed integers of 8, 16, 32 and 64 bits |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned integers of those widths |
| `Float32`, `Float64` | IEEE 754 binary32 and binary64 |
| `Rational` | Every rational number, exactly |

The first two rows are the integer classes. The third row is the float classes.

Each numeric class is a final value class. No numeric class extends another, and their only common superclass is `Object`. A value of one numeric class is never an instance of a different one. `Char` and `Boolean` are not numeric.

`Int` is the everyday integer. An array length, an index, a `String` position, a shift count and a hash code are `Int`. There is no class named `Int64`.

## 6.2 The numeric interfaces

What the numeric classes share is declared by interfaces:

```java
public interface Numeric<T> {
    T plus(T other);
    T minus(T other);
    T times(T other);
    T negate();
    static T zero();
    static T one();
}
public interface Divisible<T> extends Numeric<T> {
    T div(T other);
}
public interface Ordered<in T> {
    Boolean lessThan(T other);
    Boolean atMost(T other);
    Boolean greaterThan(T other);
    Boolean atLeast(T other);
    Int compare(T other);
}
```

Every numeric class `C` implements `Numeric<C>` and `Ordered<C>`. `Float32`, `Float64` and `Rational` also implement `Divisible`. The integer classes do not.

`zero` and `one` are static interface methods. Each implementing class declares them, and generic code calls them through a type parameter, as [chapter 7](07-generics.md) defines.

The operator spellings of [chapter 4](04-methods.md) reach these methods: `a + b` is `a.plus(b)`, and `a < b` is `a.lessThan(b)`. An arithmetic method takes and returns its own class. When the two operands of an operator have different numeric classes, one is first converted to the other if [section 6.7](#67-conversion) lists an implicit conversion between them: with an `Int32 i` and an `Int n`, `i + n` is an `Int` addition. With an `Int32 i` and a `Float64 f`, `i + f` is rejected, because neither class converts implicitly to the other, and the program converts one operand itself.

A class a program declares may implement these interfaces, and its instances then take the same operators. Nothing in this chapter depends on a class being declared by the prelude, except the literal rule of [section 6.3](#63-numeric-literals).

## 6.3 Numeric literals

An integer literal and a decimal literal are the lexical forms of [chapter 1](01-source.md). Each denotes its exact mathematical value. A `-` written directly before a numeric literal is part of that literal, so `-128` is one literal.

A literal expression is a numeric literal, a parenthesized literal expression, or an arithmetic operator applied to literal expressions.

A numeric literal has no class of its own. The literals of a literal expression take their class from the first of these rules that applies:

1. **Expected type.** The expression stands where a value of a numeric class `C` is expected: the initializer of a declaration whose written type is `C`, the right side of an assignment to a `C` location, an argument for a parameter of type `C`, the operand of `return` in a method whose result is `C`, or an element of an array creation whose element type is `C`. Every literal in it has class `C`. `@Nullable C` gives the same expected type as `C`.
2. **Other operand.** The expression is one operand of a binary operator, of a compound assignment or of `??`, or one arm of `?:`, and the other operand or arm is not a literal expression and has a numeric static type `C`. Every literal in it has class `C`. For `??`, a left operand of type `@Nullable C` counts as `C`.
3. **Default.** Every literal in the expression has class `Rational` if any of them is a decimal literal, and class `Int` otherwise.

An expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression.

A literal of class `C` must denote a value of `C`:

- When `C` is an integer class, the literal is an integer literal in the range of `C`. A decimal literal is rejected there, including one such as `1.0`.
- When `C` is a float class, the literal denotes the value of `C` nearest to it, ties to even. A literal whose nearest value is an infinity is rejected.
- When `C` is `Rational`, the literal denotes its exact value.

```java
Float64 dt = 0.001;      // Float64, by rule 1
total += 0.5 * mass;     // Float64 when mass is Float64, by rule 2
var price = 19.99;       // Rational, exactly 1999/100, by rule 3
var i = 0;               // Int, by rule 3
UInt8 mask = 0xFF;       // UInt8; 0x100 is rejected
var third = 1.0 / 3;     // Rational, exactly one third
var half = 1 / 2;        // rejected: both literals are Int, and Int has no div
```

Rounding a decimal literal to a float is the only implicit rounding in the language. It happens only where the program wrote a float type. Rule 1 applies to literals, not to variables: after `var x = 0.1;`, the declaration `Float64 y = x;` is rejected, because `x` is a `Rational`.

For a parameter whose type is a type parameter, [chapter 7](07-generics.md) determines the type from the rest of the call when it can, and the literal then has that expected type. When nothing else determines it, the literal takes its default class.

When a method name is overloaded, a literal expression is applicable to a parameter of numeric class `C` when the rules above accept it at `C`. If more than one method remains after [chapter 4](04-methods.md) resolution, the method whose parameter class is the literal's default class is chosen. If there is none, the call is rejected.

## 6.4 Integer arithmetic

`plus`, `minus`, `times` and `negate` on an integer class raise `ArithmeticException` when the mathematical result is not a value of the class. That includes negating the minimum of a signed class and negating a nonzero unsigned value.

An integer class has no `div`, so `a / b` on two integers is rejected. Integer division is written by name:

| Method | Result |
| --- | --- |
| `floorDiv` | The quotient rounded toward negative infinity |
| `mod` | `a - a.floorDiv(b) * b`. It is zero or has the sign of the divisor. `a % b` is `a.mod(b)` |
| `truncatingDiv` | The quotient rounded toward zero |
| `truncatingRem` | `a - a.truncatingDiv(b) * b`. It is zero or has the sign of the dividend |

Each takes and returns the receiver's class. All four raise `ArithmeticException` when the divisor is zero. `floorDiv` and `truncatingDiv` also raise when the quotient is not a value of the class, which is the case for a signed minimum divided by negative one.

`-7 % 3` is `2`. `(-7).truncatingRem(3)` is `-1`, the value that `%` gives in Java and C.

`wrappingPlus`, `wrappingMinus` and `wrappingTimes` return the low bits of the mathematical result, interpreted in the class. They do not raise.

`and`, `or`, `xor`, `complement`, `shiftLeft` and `shiftRight` are declared on the integer classes and on no other numeric class. A shift count is an `Int`. A count below zero, or not below the width of the class, raises `ArithmeticException`. `shiftLeft` discards the bits it shifts out. `shiftRight` copies the sign bit on a signed class and fills with zero on an unsigned class. `>>>` is not a spelling.

## 6.5 Float arithmetic

`plus`, `minus`, `times`, `div` and `negate` on a float class follow IEEE 754, rounding to nearest with ties to even. They do not raise. Overflow gives an infinity. Division by zero gives an infinity or NaN.

`floor`, `ceil`, `truncate` and `round` take no argument and return a value of the same class with no fractional part. `round` rounds to nearest, ties to even. A NaN or an infinity is returned unchanged. `sqrt` returns the correctly rounded square root.

A float class does not declare `mod`.

## 6.6 `Rational`

A `Rational` is one exact rational number. `plus`, `minus`, `times`, `div` and `negate` return the exact result. `div` raises `ArithmeticException` when the divisor is zero. No operation rounds, wraps, or overflows. An operation whose result cannot be stored raises `OutOfMemoryException`.

| Method | Result |
| --- | --- |
| `isInteger()` | Whether the value is an integer |
| `numerator()`, `denominator()` | The value in lowest terms, as two integer-valued `Rational`s. The denominator is positive |
| `floor()`, `ceil()`, `truncate()` | The nearest integer in that direction |
| `round()` | The nearest integer, ties to even |
| `round(Int places)` | The nearest multiple of ten to the power `-places`, ties to even |
| `floorDiv`, `mod` | As [section 6.4](#64-integer-arithmetic) defines them, for any two rationals with a nonzero divisor |
| `toDecimal(Int places)` | The digits of `round(places)`, with exactly `places` digits after the point |

`toString` returns decimal digits for an integer and `numerator/denominator` otherwise, so one half is `1/2`.

A program may observe the mathematical value of a `Rational`. How the value is stored is not observable, and every representation that preserves the value is correct.

## 6.7 Conversion

**Implicit conversion.** The numeric classes declare implicit conversions in three cases, with the `@Implicit` annotation of [chapter 8](08-annotations.md). None of them can lose information.

- An integer class converts to an integer class that holds every value of the first.
- `Float32` converts to `Float64`.
- An integer class converts to `Rational`.

| From | Converts implicitly to |
| --- | --- |
| `Int8` | `Int16`, `Int32`, `Int`, `Rational` |
| `Int16` | `Int32`, `Int`, `Rational` |
| `Int32` | `Int`, `Rational` |
| `Int` | `Rational` |
| `UInt8` | `UInt16`, `UInt32`, `UInt64`, `Int16`, `Int32`, `Int`, `Rational` |
| `UInt16` | `UInt32`, `UInt64`, `Int32`, `Int`, `Rational` |
| `UInt32` | `UInt64`, `Int`, `Rational` |
| `UInt64` | `Rational` |
| `Float32` | `Float64` |

No integer class converts implicitly to a float class.

Each of these conversions is the method `T.from`, which cannot fail for these pairs. [Chapter 4](04-methods.md#45-assignability) says where an implicit conversion is applied: where a value is assigned to a location of the other class, as in an initializer, an argument or the operand of `return`, and between the two operands of a binary operator.

```java
Int32 small = 7;
Int count = small;                 // converted
Rational price = 4.35;
Rational total = price * count;    // count is converted to Rational
Float64 ratio = small;             // rejected: an integer does not convert implicitly to a float
```

**Explicit conversion.** A conversion that could lose information is never implicit. The program calls it by name, and its plain form raises an exception when the value does not fit. A conversion that rounds, truncates or wraps has a name that says so.

Each numeric class `C` declares a static `from` for every numeric class. `C.from(x)` returns the value of `C` equal to `x`, and raises `ArithmeticException` when `C` has no such value. `Int.from(i32)` always succeeds. `Int32.from(n)` raises when `n` is outside 32 bits. `Float64.from(r)` raises when the rational `r` is not exactly a `Float64`, which is the case for one tenth. A NaN or an infinity has no value in an integer class or in `Rational`.

`Float32.nearest(x)` and `Float64.nearest(x)` are the rounding conversions. They accept every numeric class and return the nearest value of the float class, ties to even. A magnitude too large for the class gives an infinity.

A float becomes an integer in two steps, so the rounding direction is written: `Int.from(f.floor())`.

A cast is a class test and never converts. `(Int) x` is rejected when the static type of `x` is a different numeric class, because the test could not succeed.

## 6.8 Equality and order

`==` is `equals`, as [chapter 4](04-methods.md) defines. `equals` on a numeric class is `true` exactly when the argument is a value of the same class with the same numeric value. On a float class, NaN equals NaN, and positive zero equals negative zero. `hashCode` agrees with `equals`.

When the operands of `==` have two different numeric classes, one is converted to the other if [section 6.7](#67-conversion) lists an implicit conversion between them, and the two values are then compared. Otherwise `a == b` is rejected, and the program converts one operand first. A direct call of `equals` converts nothing, so it is `false` for values of two different classes.

`lessThan`, `atMost`, `greaterThan` and `atLeast` compare numeric values. On a float class they follow IEEE 754: each is `false` when either operand is NaN.

`compare` returns a negative `Int`, zero, or a positive `Int`. It returns zero exactly when `equals` is `true`. On a float class NaN compares greater than every other value, so `compare` is a total order and is the order a sort uses.

Each float class also declares `Int totalOrder(C other)`, the IEEE 754 totalOrder. It distinguishes negative zero from positive zero and one NaN from another. It is not `==`.

## 6.9 `Char` and `String`

`Char` is a final value class. A value is one Unicode scalar: 0 through 10FFFF, excluding the surrogates. `Char.from(Int)` returns the scalar with that number, or raises `ArithmeticException`. `code()` returns the number as an `Int`. `Char` implements `Ordered<Char>` by scalar number. It is not numeric: `'a' + 1` is rejected.

`String` is a final value class. A value is a sequence of `Char`.

| Method | Result |
| --- | --- |
| `length()` | The number of scalars, as an `Int` |
| `get(Int index)` | The scalar at that zero-based index. `s[i]` is `s.get(i)` |
| `substring(Int begin, Int end)` | The scalars from `begin` inclusive to `end` exclusive |
| `plus(@Nullable Object other)` | The receiver followed by `other.toString()` |
| `indexOf(Char c)`, `indexOf(String s)` | The first index of the argument, or `-1`. The empty string is found at `0` |
| `startsWith(String s)`, `endsWith(String s)` | Whether the receiver begins or ends with `s` |
| `isEmpty()` | Whether the length is zero |
| `toUtf8()` | The UTF-8 encoding, as a new `UInt8[]` |
| `static fromUtf8(UInt8[] bytes)` | The string those bytes encode |

`get` and `substring` raise `IndexOutOfBoundsException` when an index is outside the string or `begin` is greater than `end`. `fromUtf8` raises `IllegalArgumentException` when the bytes are not well-formed UTF-8.

`equals` and `hashCode` use the sequence of scalars. `String` implements `Ordered<String>`: `compare` orders by the first scalar that differs, and by length when one string is a prefix of the other. `String` implements `Iterable<Char>`, so a `for` statement visits its scalars in order.

No value is converted to a `String` implicitly. `"count: " + n` is `String.plus`, which calls `n.toString()`. `n + " items"` is rejected when `n` is an `Int`, because `Int.plus` has no `String` parameter.

These are the `String` methods the language defines. The prelude may declare more in ordinary source.

## 6.10 Arrays

An array type is written `T[]` for any type `T`, including an array type. `T[]` is the prelude class `Array<T>`, and a program may write either. An array is a reference object. Its length is fixed at creation. `length()` returns an `Int`. `get` and `set` take an `Int` index and raise `IndexOutOfBoundsException` when it is out of range. `a[i]` is `a.get(i)`, and `a[i] = e` is `a.set(i, e)`. An array implements `Iterable<T>`.

Arrays are invariant. `String[]` is not a subtype of `Object[]`. A store is typechecked on `T`, so no store fails at run time for its type. `T[]` does not contain `null` unless `T` is `@Nullable`.

There are three ways to create an array:

- `new T[n]` creates `n` elements, each the default of `T`. `n` is an `Int`, and a negative `n` raises `ArithmeticException`. The default is `null` when `T` is `@Nullable`, `zero()` when `T` is a numeric class, the scalar 0 when `T` is `Char`, `false` when `T` is `Boolean`, and the value built from those defaults when `T` is a value class whose fields all have one. Every other `new T[n]` is rejected, because `T` has no default. When `T` is a type parameter, `new T[n]` is rejected and `new @Nullable T[n]` is accepted.
- `new T[] { e1, e2 }` evaluates the elements from left to right. It is legal for every `T`.
- `Array.build(n, element)` creates `n` elements by calling `element` with each index from zero upward. It is declared `public static <T> T[] build(Int length, Function1<Int, T> element)` and is legal for every `T`.

How an array stores its elements is not observable.

## 6.11 `Boolean`

`Boolean` is a final value class with exactly two instances, the literals `true` and `false`. It is not numeric. It declares `and`, `or`, `xor` and `not`, which `&`, `|`, `^` and `!` spell. `equals` is `true` when both values are `true` or both are `false`.

The conditions of `if`, `while`, `for`, `?:`, `&&`, `||` and `assert` have static type `Boolean`. Those forms are syntax, defined in [chapter 4](04-methods.md).
