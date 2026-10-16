# 6. Numbers and text

## 6.1 The numeric classes

The prelude declares these numeric classes: <!-- r[numbers.classes.prelude-declares-numeric] -->

| Classes | Values |
| --- | --- |
| `Int8`, `Int16`, `Int32`, `Int` | Signed integers of 8, 16, 32 and 64 bits <!-- r[numbers.classes.int8-int16-int32] --> |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned integers of those widths <!-- r[numbers.classes.uint8-uint16-uint32] --> |
| `Float32`, `Float64` | IEEE 754 binary32 and binary64 <!-- r[numbers.classes.float32-float64-ieee] --> |
| `Rational` | Every rational number, exactly <!-- r[numbers.classes.rational-number-exactly] --> |

The first two rows are the integer classes. <!-- r[numbers.classes.first-rows-integer] -->
The third row is the float classes. <!-- r[numbers.classes.third-row-float] -->

Each numeric class is a final value class. <!-- r[numbers.classes.numeric-class-final] -->
No numeric class extends another, and their only common superclass is `Object`. <!-- r[numbers.classes.no-numeric-class] -->
A value of one numeric class is never an instance of a different one. <!-- r[numbers.classes.value-numeric-class] -->
`Char` and `Boolean` are not numeric. <!-- r[numbers.classes.char-boolean-not] -->

`Int` is the everyday integer. <!-- r[numbers.classes.int-everyday-integer] -->
An array length, an index, a `String` position, a shift count and a hash code are `Int`. <!-- r[numbers.classes.array-length-index] -->
There is no class named `Int64`. <!-- r[numbers.classes.no-class-named] -->

## 6.2 The numeric interfaces

What the numeric classes share is declared by interfaces: <!-- r[numbers.interfaces.numeric-classes-share] -->

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

Every numeric class `C` implements `Numeric<C>` and `Ordered<C>`. <!-- r[numbers.interfaces.numeric-class-implements] -->
`Float32`, `Float64` and `Rational` also implement `Divisible`. <!-- r[numbers.interfaces.float32-float64-rational] -->
The integer classes do not. <!-- r[numbers.interfaces.integer-classes-not] -->

`zero` and `one` are static interface methods. <!-- r[numbers.interfaces.zero-one-static] -->
Each implementing class declares them, and generic code calls them through a type parameter, as [chapter 7](07-generics.md) defines. <!-- r[numbers.interfaces.implementing-class-declares] -->

The operator spellings of [chapter 4](04-methods.md) reach these methods: `a + b` is `a.plus(b)`, and `a < b` is `a.lessThan(b)`. <!-- r[numbers.interfaces.operator-spellings-reach] -->
An arithmetic method takes and returns its own class. <!-- r[numbers.interfaces.arithmetic-method-takes] -->
When the two operands of an operator have different numeric classes, one is first converted to the other if [section 6.7](#67-conversion) lists an implicit conversion between them: with an `Int32 i` and an `Int n`, `i + n` is an `Int` addition. <!-- r[numbers.interfaces.operands-operator-different] -->
With an `Int32 i` and a `Float64 f`, `i + f` is rejected, because neither class converts implicitly to the other, and the program converts one operand itself. <!-- r[numbers.interfaces.int32-float64-rejected] -->

A class a program declares may implement these interfaces, and its instances then take the same operators. <!-- r[numbers.interfaces.class-program-declares] -->
Nothing in this chapter depends on a class being declared by the prelude, except the literal rule of [section 6.3](#63-numeric-literals). <!-- r[numbers.interfaces.nothing-depends-class] -->

## 6.3 Numeric literals

An integer literal and a decimal literal are the lexical forms of [chapter 1](01-source.md). <!-- r[numbers.literals.integer-literal-decimal] -->
Each denotes its exact mathematical value. <!-- r[numbers.literals.denotes-exact-mathematical] -->
A `-` written directly before a numeric literal is part of that literal, so `-128` is one literal. <!-- r[numbers.literals.written-directly-before] -->

A literal expression is a numeric literal, a parenthesized literal expression, or an arithmetic operator applied to literal expressions. <!-- r[numbers.literals.literal-expression-numeric] -->

A numeric literal has no class of its own. <!-- r[numbers.literals.numeric-literal-no] -->
The literals of a literal expression take their class from the first of these rules that applies: <!-- r[numbers.literals.literals-literal-expression] -->

1. **Expected type.** The expression stands where a value of a numeric class `C` is expected: the initializer of a declaration whose written type is `C`, the right side of an assignment to a `C` location, an argument for a parameter of type `C`, the operand of `return` in a method whose result is `C`, or an element of an array creation whose element type is `C`. <!-- r[numbers.literals.expected-type-expression] -->
   Every literal in it has class `C`. <!-- r[numbers.literals.literal-class] -->
   `C?` gives the same expected type as `C`. <!-- r[numbers.literals.nullable-gives-expected] -->
2. **Other operand.** The expression is one operand of a binary operator, of a compound assignment or of `??`, or one arm of `?:`, and the other operand or arm is not a literal expression and has a numeric static type `C`. <!-- r[numbers.literals.operand-expression-binary] -->
   Every literal in it has class `C`. <!-- r[numbers.literals.literal-class-2] -->
   For `??`, a left operand of type `C?` counts as `C`. <!-- r[numbers.literals.coalesce-left-operand] -->
3. **Default.** Every literal in the expression has class `Rational` if any of them is a decimal literal, and class `Int` otherwise. <!-- r[numbers.literals.default-literal-expression] -->

An expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression. <!-- r[numbers.literals.expected-type-passes] -->

A literal of class `C` must denote a value of `C`: <!-- r[numbers.literals.literal-class-denote] -->

- When `C` is an integer class, the literal is an integer literal in the range of `C`. <!-- r[numbers.literals.integer-class-literal] -->
  A decimal literal is rejected there, including one such as `1.0`. <!-- r[numbers.literals.decimal-literal-rejected] -->
- When `C` is a float class, the literal denotes the value of `C` nearest to it, ties to even. <!-- r[numbers.literals.float-class-literal] -->
  A literal whose nearest value is an infinity is rejected. <!-- r[numbers.literals.literal-nearest-value] -->
- When `C` is `Rational`, the literal denotes its exact value. <!-- r[numbers.literals.rational-literal-denotes] -->

```java
Float64 dt = 0.001;      // Float64, by rule 1
total += 0.5 * mass;     // Float64 when mass is Float64, by rule 2
var price = 19.99;       // Rational, exactly 1999/100, by rule 3
var i = 0;               // Int, by rule 3
UInt8 mask = 0xFF;       // UInt8; 0x100 is rejected
var third = 1.0 / 3;     // Rational, exactly one third
var half = 1 / 2;        // rejected: both literals are Int, and Int has no div
```

Rounding a decimal literal to a float is the only implicit rounding in the language. <!-- r[numbers.literals.rounding-decimal-literal] -->
It happens only where the program wrote a float type. <!-- r[numbers.literals.happens-only-program] -->
Rule 1 applies to literals, not to variables: after `var x = 0.1;`, the declaration `Float64 y = x;` is rejected, because `x` is a `Rational`. <!-- r[numbers.literals.rule-applies-literals] -->

For a parameter whose type is a type parameter, [chapter 7](07-generics.md) determines the type from the rest of the call when it can, and the literal then has that expected type. <!-- r[numbers.literals.parameter-type-determines] -->
When nothing else determines it, the literal takes its default class. <!-- r[numbers.literals.nothing-else-determines] -->

When a method name is overloaded, a literal expression is applicable to a parameter of numeric class `C` when the rules above accept it at `C`. <!-- r[numbers.literals.method-name-overloaded] -->
If more than one method remains after [chapter 4](04-methods.md) resolution, the method whose parameter class is the literal's default class is chosen. <!-- r[numbers.literals.more-method-remains] -->
If there is none, the call is rejected. <!-- r[numbers.literals.none-call-rejected] -->

## 6.4 Integer arithmetic

`plus`, `minus`, `times` and `negate` on an integer class raise `ArithmeticException` when the mathematical result is not a value of the class. <!-- r[numbers.integer.plus-minus-times] -->
That includes negating the minimum of a signed class and negating a nonzero unsigned value. <!-- r[numbers.integer.includes-negating-minimum] -->

An integer class has no `div`, so `a / b` on two integers is rejected. <!-- r[numbers.integer.integer-class-no] -->
Integer division is written by name: <!-- r[numbers.integer.integer-division-written] -->

| Method | Result |
| --- | --- |
| `floorDiv` | The quotient rounded toward negative infinity <!-- r[numbers.integer.floordiv-quotient-rounded] --> |
| `mod` | `a - a.floorDiv(b) * b`. It is zero or has the sign of the divisor. `a % b` is `a.mod(b)` <!-- r[numbers.integer.mod-floordiv-zero] --> |
| `truncatingDiv` | The quotient rounded toward zero <!-- r[numbers.integer.truncatingdiv-quotient-rounded] --> |
| `truncatingRem` | `a - a.truncatingDiv(b) * b`. It is zero or has the sign of the dividend <!-- r[numbers.integer.truncatingrem-truncatingdiv-zero] --> |

Each takes and returns the receiver's class. <!-- r[numbers.integer.takes-returns-receiver] -->
All four raise `ArithmeticException` when the divisor is zero. <!-- r[numbers.integer.four-raise-arithmeticexception] -->
`floorDiv` and `truncatingDiv` also raise when the quotient is not a value of the class, which is the case for a signed minimum divided by negative one. <!-- r[numbers.integer.floordiv-truncatingdiv-raise] -->

`-7 % 3` is `2`. <!-- r[numbers.integer.rule] -->
`(-7).truncatingRem(3)` is `-1`, the value that `%` gives in Java and C. <!-- r[numbers.integer.truncatingrem-value-gives] -->

`wrappingPlus`, `wrappingMinus` and `wrappingTimes` return the low bits of the mathematical result, interpreted in the class. <!-- r[numbers.integer.wrappingplus-wrappingminus-wrappingtimes] -->
They do not raise. <!-- r[numbers.integer.not-raise] -->

`and`, `or`, `xor`, `complement`, `shiftLeft` and `shiftRight` are declared on the integer classes and on no other numeric class. <!-- r[numbers.integer.and-or-xor] -->
A shift count is an `Int`. <!-- r[numbers.integer.shift-count-int] -->
A count below zero, or not below the width of the class, raises `ArithmeticException`. <!-- r[numbers.integer.count-below-zero] -->
`shiftLeft` discards the bits it shifts out. <!-- r[numbers.integer.shiftleft-discards-bits] -->
`shiftRight` copies the sign bit on a signed class and fills with zero on an unsigned class. <!-- r[numbers.integer.shiftright-copies-sign] -->
`>>>` is not a spelling. <!-- r[numbers.integer.unsigned-shift-not-spelling] -->

## 6.5 Float arithmetic

`plus`, `minus`, `times`, `div` and `negate` on a float class follow IEEE 754, rounding to nearest with ties to even. <!-- r[numbers.float.plus-minus-times] -->
They do not raise. <!-- r[numbers.float.not-raise] -->
Overflow gives an infinity. <!-- r[numbers.float.overflow-gives-infinity] -->
Division by zero gives an infinity or NaN. <!-- r[numbers.float.division-zero-gives] -->

`floor`, `ceil`, `truncate` and `round` take no argument and return a value of the same class with no fractional part. <!-- r[numbers.float.floor-ceil-truncate] -->
`round` rounds to nearest, ties to even. <!-- r[numbers.float.round-rounds-nearest] -->
A NaN or an infinity is returned unchanged. <!-- r[numbers.float.nan-infinity-returned] -->
`sqrt` returns the correctly rounded square root. <!-- r[numbers.float.sqrt-returns-correctly] -->

A float class does not declare `mod`. <!-- r[numbers.float.float-class-not] -->

## 6.6 `Rational`

A `Rational` is one exact rational number. <!-- r[numbers.rational.rational-exact-number] -->
`plus`, `minus`, `times`, `div` and `negate` return the exact result. <!-- r[numbers.rational.plus-minus-times] -->
`div` raises `ArithmeticException` when the divisor is zero. <!-- r[numbers.rational.div-raises-arithmeticexception] -->
No operation rounds, wraps, or overflows. <!-- r[numbers.rational.no-operation-rounds] -->
An operation whose result cannot be stored raises `OutOfMemoryException`. <!-- r[numbers.rational.operation-result-stored] -->

| Method | Result |
| --- | --- |
| `isInteger()` | Whether the value is an integer <!-- r[numbers.rational.isinteger-whether-value] --> |
| `numerator()`, `denominator()` | The value in lowest terms, as two integer-valued `Rational`s. The denominator is positive <!-- r[numbers.rational.numerator-denominator-value] --> |
| `floor()`, `ceil()`, `truncate()` | The nearest integer in that direction <!-- r[numbers.rational.floor-ceil-truncate] --> |
| `round()` | The nearest integer, ties to even <!-- r[numbers.rational.round-nearest-integer] --> |
| `round(Int places)` | The nearest multiple of ten to the power `-places`, ties to even <!-- r[numbers.rational.round-int-places] --> |
| `floorDiv`, `mod` | As [section 6.4](#64-integer-arithmetic) defines them, for any two rationals with a nonzero divisor <!-- r[numbers.rational.floordiv-mod-defines] --> |
| `toDecimal(Int places)` | The digits of `round(places)`, with exactly `places` digits after the point <!-- r[numbers.rational.todecimal-int-places] --> |

`toString` returns decimal digits for an integer and `numerator/denominator` otherwise, so one half is `1/2`. <!-- r[numbers.rational.tostring-returns-decimal] -->

A program may observe the mathematical value of a `Rational`. <!-- r[numbers.rational.program-observe-mathematical] -->
How the value is stored is not observable, and every representation that preserves the value is correct. <!-- r[numbers.rational.how-value-stored] -->

## 6.7 Conversion

**Implicit conversion.** The numeric classes declare implicit conversions in three cases, with the `@Implicit` annotation of [chapter 8](08-annotations.md). <!-- r[numbers.conversion.implicit-conversion-numeric] -->
None of them can lose information. <!-- r[numbers.conversion.none-lose-information] -->

- An integer class converts to an integer class that holds every value of the first. <!-- r[numbers.conversion.integer-class-converts-holds] -->
- `Float32` converts to `Float64`. <!-- r[numbers.conversion.float32-converts-float64] -->
- An integer class converts to `Rational`. <!-- r[numbers.conversion.integer-class-converts-rational] -->

| From | Converts implicitly to <!-- r[numbers.conversion.table] --> |
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

No integer class converts implicitly to a float class. <!-- r[numbers.conversion.no-integer-class] -->

Each of these conversions is the method `T.from`, which cannot fail for these pairs. <!-- r[numbers.conversion.conversions-method-from] -->
[Chapter 4](04-methods.md#45-assignability) says where an implicit conversion is applied: where a value is assigned to a location of the other class, as in an initializer, an argument or the operand of `return`, and between the two operands of a binary operator. <!-- r[numbers.conversion.says-implicit-conversion] -->

```java
Int32 small = 7;
Int count = small;                 // converted
Rational price = 4.35;
Rational total = price * count;    // count is converted to Rational
Float64 ratio = small;             // rejected: an integer does not convert implicitly to a float
```

**Explicit conversion.** A conversion that could lose information is never implicit. <!-- r[numbers.conversion.explicit-conversion-lose] -->
The program calls it by name, and its plain form raises an exception when the value does not fit. <!-- r[numbers.conversion.program-calls-name] -->
A conversion that rounds, truncates or wraps has a name that says so. <!-- r[numbers.conversion.conversion-rounds-truncates] -->

Each numeric class `C` declares a static `from` for every numeric class. <!-- r[numbers.conversion.numeric-class-declares] -->
`C.from(x)` returns the value of `C` equal to `x`, and raises `ArithmeticException` when `C` has no such value. <!-- r[numbers.conversion.from-returns-value] -->
`Int.from(i32)` always succeeds. <!-- r[numbers.conversion.int-from-i32] -->
`Int32.from(n)` raises when `n` is outside 32 bits. <!-- r[numbers.conversion.int32-from-raises] -->
`Float64.from(r)` raises when the rational `r` is not exactly a `Float64`, which is the case for one tenth. <!-- r[numbers.conversion.float64-from-raises] -->
A NaN or an infinity has no value in an integer class or in `Rational`. <!-- r[numbers.conversion.nan-infinity-no] -->

`Float32.nearest(x)` and `Float64.nearest(x)` are the rounding conversions. <!-- r[numbers.conversion.float32-nearest-float64] -->
They accept every numeric class and return the nearest value of the float class, ties to even. <!-- r[numbers.conversion.accept-numeric-class] -->
A magnitude too large for the class gives an infinity. <!-- r[numbers.conversion.magnitude-too-large] -->

A float becomes an integer in two steps, so the rounding direction is written: `Int.from(f.floor())`. <!-- r[numbers.conversion.float-becomes-integer] -->

A cast is a class test and never converts. <!-- r[numbers.conversion.cast-class-test] -->
`(Int) x` is rejected when the static type of `x` is a different numeric class, because the test could not succeed. <!-- r[numbers.conversion.int-rejected-static] -->

## 6.8 Equality and order

`==` is `equals`, as [chapter 4](04-methods.md) defines. <!-- r[numbers.equality.equals-defines] -->
`equals` on a numeric class is `true` exactly when the argument is a value of the same class with the same numeric value. <!-- r[numbers.equality.equals-numeric-class] -->
On a float class, NaN equals NaN, and positive zero equals negative zero. <!-- r[numbers.equality.float-class-nan-equals] -->
`hashCode` agrees with `equals`. <!-- r[numbers.equality.hashcode-agrees-equals] -->

When the operands of `==` have two different numeric classes, one is converted to the other if [section 6.7](#67-conversion) lists an implicit conversion between them, and the two values are then compared. <!-- r[numbers.equality.operands-equals-different] -->
Otherwise `a == b` is rejected, and the program converts one operand first. <!-- r[numbers.equality.otherwise-equals-rejected] -->
A direct call of `equals` converts nothing, so it is `false` for values of two different classes. <!-- r[numbers.equality.direct-call-equals] -->

`lessThan`, `atMost`, `greaterThan` and `atLeast` compare numeric values. <!-- r[numbers.equality.lessthan-atmost-greaterthan] -->
On a float class they follow IEEE 754: each is `false` when either operand is NaN. <!-- r[numbers.equality.float-class-follow] -->

`compare` returns a negative `Int`, zero, or a positive `Int`. <!-- r[numbers.equality.compare-returns-negative] -->
It returns zero exactly when `equals` is `true`. <!-- r[numbers.equality.returns-zero-exactly] -->
On a float class NaN compares greater than every other value, so `compare` is a total order and is the order a sort uses. <!-- r[numbers.equality.float-class-nan-compares] -->

Each float class also declares `Int totalOrder(C other)`, the IEEE 754 totalOrder. <!-- r[numbers.equality.float-class-declares] -->
It distinguishes negative zero from positive zero and one NaN from another. <!-- r[numbers.equality.distinguishes-negative-zero] -->
It is not `==`. <!-- r[numbers.equality.not-equals] -->

## 6.9 `Char` and `String`

`Char` is a final value class. <!-- r[numbers.text.char-final-value] -->
A value is one Unicode scalar: 0 through 10FFFF, excluding the surrogates. <!-- r[numbers.text.value-unicode-scalar] -->
`Char.from(Int)` returns the scalar with that number, or raises `ArithmeticException`. <!-- r[numbers.text.char-from-int] -->
`code()` returns the number as an `Int`. <!-- r[numbers.text.code-returns-number] -->
`Char` implements `Ordered<Char>` by scalar number. <!-- r[numbers.text.char-implements-ordered] -->
It is not numeric: `'a' + 1` is rejected. <!-- r[numbers.text.not-numeric-rejected] -->

`String` is a final value class. <!-- r[numbers.text.string-final-value] -->
A value is a sequence of `Char`. <!-- r[numbers.text.value-sequence-char] -->

| Method | Result |
| --- | --- |
| `length()` | The number of scalars, as an `Int` <!-- r[numbers.text.length-number-scalars] --> |
| `get(Int index)` | The scalar at that zero-based index. `s[i]` is `s.get(i)` <!-- r[numbers.text.get-int-index] --> |
| `substring(Int begin, Int end)` | The scalars from `begin` inclusive to `end` exclusive <!-- r[numbers.text.substring-int-begin] --> |
| `plus(Object? other)` | The receiver followed by `other.toString()` <!-- r[numbers.text.plus-nullable-object] --> |
| `indexOf(Char c)`, `indexOf(String s)` | The first index of the argument, or `-1`. The empty string is found at `0` <!-- r[numbers.text.indexof-char-string] --> |
| `startsWith(String s)`, `endsWith(String s)` | Whether the receiver begins or ends with `s` <!-- r[numbers.text.startswith-string-endswith] --> |
| `isEmpty()` | Whether the length is zero <!-- r[numbers.text.isempty-whether-length] --> |
| `toUtf8()` | The UTF-8 encoding, as a new `UInt8[]` <!-- r[numbers.text.toutf8-utf-encoding] --> |
| `static fromUtf8(UInt8[] bytes)` | The string those bytes encode <!-- r[numbers.text.static-fromutf8-uint8] --> |

`get` and `substring` raise `IndexOutOfBoundsException` when an index is outside the string or `begin` is greater than `end`. <!-- r[numbers.text.get-substring-raise] -->
`fromUtf8` raises `IllegalArgumentException` when the bytes are not well-formed UTF-8. <!-- r[numbers.text.fromutf8-raises-illegalargumentexception] -->

`equals` and `hashCode` use the sequence of scalars. <!-- r[numbers.text.equals-hashcode-use] -->
`String` implements `Ordered<String>`: `compare` orders by the first scalar that differs, and by length when one string is a prefix of the other. <!-- r[numbers.text.string-implements-ordered] -->
`String` implements `Iterable<Char>`, so a `for` statement visits its scalars in order. <!-- r[numbers.text.string-implements-iterable] -->

No value is converted to a `String` implicitly. <!-- r[numbers.text.no-value-converted] -->
`"count: " + n` is `String.plus`, which calls `n.toString()`. <!-- r[numbers.text.count-string-plus] -->
`n + " items"` is rejected when `n` is an `Int`, because `Int.plus` has no `String` parameter. <!-- r[numbers.text.items-rejected-int] -->

These are the `String` methods the language defines. <!-- r[numbers.text.string-methods-language] -->
The prelude may declare more in ordinary source. <!-- r[numbers.text.prelude-declare-more] -->

## 6.10 Arrays

An array type is written `T[]` for any type `T`, including an array type. <!-- r[numbers.arrays.array-type-written] -->
`T[]` is the prelude class `Array<T>`, and a program may write either. <!-- r[numbers.arrays.prelude-class-array] -->
An array is a reference object. <!-- r[numbers.arrays.array-reference-object] -->
Its length is fixed at creation. <!-- r[numbers.arrays.length-fixed-creation] -->
`length()` returns an `Int`. <!-- r[numbers.arrays.length-returns-int] -->
`get` and `set` take an `Int` index and raise `IndexOutOfBoundsException` when it is out of range. <!-- r[numbers.arrays.get-set-take] -->
`a[i]` is `a.get(i)`, and `a[i] = e` is `a.set(i, e)`. <!-- r[numbers.arrays.get-set] -->
An array implements `Iterable<T>`. <!-- r[numbers.arrays.array-implements-iterable] -->

Arrays are invariant. <!-- r[numbers.arrays.arrays-invariant] -->
`String[]` is not a subtype of `Object[]`. <!-- r[numbers.arrays.string-not-subtype] -->
A store is typechecked on `T`, so no store fails at run time for its type. <!-- r[numbers.arrays.store-typechecked-no] -->
`T[]` does not contain `null` unless `T` is nullable. <!-- r[numbers.arrays.not-contain-null] -->

There are three ways to create an array: <!-- r[numbers.arrays.three-ways-create] -->

- `new T[n]` creates `n` elements, each the default of `T`. <!-- r[numbers.arrays.new-creates-elements] -->
  `n` is an `Int`, and a negative `n` raises `ArithmeticException`. <!-- r[numbers.arrays.int-negative-raises] -->
  The default is `null` when `T` is nullable, `zero()` when `T` is a numeric class, the scalar 0 when `T` is `Char`, `false` when `T` is `Boolean`, and the value built from those defaults when `T` is a value class whose fields all have one. <!-- r[numbers.arrays.default-null-nullable] -->
  That value is constructed once each time `new T[n]` creates an array, as [section 9.6](09-execution.md#96-constructing-a-value) says, and an exception that its constructor raises comes out of `new T[n]`. <!-- r[numbers.arrays.value-constructed-once] -->
  Every other `new T[n]` is rejected, because `T` has no default. <!-- r[numbers.arrays.new-rejected-because] -->
  When `T` is a type parameter, `new T[n]` is rejected and `new T?[n]` is accepted. <!-- r[numbers.arrays.type-parameter-new] -->
- `new T[] { e1, e2 }` evaluates the elements from left to right. <!-- r[numbers.arrays.new-e1-e2] -->
  It is legal for every `T`. <!-- r[numbers.arrays.legal] -->
- `Array.build(n, element)` creates `n` elements by calling `element` with each index from zero upward. <!-- r[numbers.arrays.array-build-element] -->
  It is declared `public static <T> T[] build(Int length, Function1<Int, T> element)` and is legal for every `T`. <!-- r[numbers.arrays.declared-public-static] -->

How an array stores its elements is not observable. <!-- r[numbers.arrays.how-array-stores] -->

## 6.11 `Boolean`

`Boolean` is a final value class with exactly two instances, the literals `true` and `false`. <!-- r[numbers.boolean.boolean-final-value] -->
It is not numeric. <!-- r[numbers.boolean.not-numeric] -->
It declares `and`, `or`, `xor` and `not`, which `&`, `|`, `^` and `!` spell. <!-- r[numbers.boolean.declares-and-or] -->
`equals` is `true` when both values are `true` or both are `false`. <!-- r[numbers.boolean.equals-true-values] -->

The conditions of `if`, `while`, `for`, `?:`, `&&`, `||` and `assert` have static type `Boolean`. <!-- r[numbers.boolean.conditions-if-while] -->
Those forms are syntax, defined in [chapter 4](04-methods.md). <!-- r[numbers.boolean.forms-syntax-defined] -->
