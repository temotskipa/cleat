# 6. Numbers and aggregates

## 6.1 `Number`

`Number` is a concrete sealed value class. It is not generic. User classes cannot extend it. The permitted subclasses are the fixed classes of [section 6.3](#63-fixed-widths), and they are declared by the prelude. An integer that does not fit in a machine width remains a `Number` with heap digits. There is no `BigInt` class.

An instance whose class is `Number` stores one exact quantity. The quantity is an integer or a rational. The representation of an integer is chosen after every operation that produces one. A non-negative integer uses the smallest of u8, u16, u32, and u64 that holds it. A negative integer uses the smallest of i8, i16, i32, and i64 that holds it. When a signed width and the unsigned width of the same size both hold the value, the unsigned width is used. One and 200 are u8. 2^63 is u64. Negative one is i8. -2^63 is i64. A value outside those ranges uses heap digits. The class stays `Number` when the representation changes. A rational that is not an integer is an exact rational. A program may observe that mathematical value, including that `1 / 2` is one half and that the literal `0.1` is one tenth. It may not observe the stored form of a non-integer. Every exact representation that preserves those values is correct, and the integer representation in this paragraph is not part of that choice. [Chapter 10](10-deferred.md) records the bound. `Number[]` has one element layout. A narrow integer is a tag inside that layout, not a smaller slot.

`Number` never wraps and never traps. An operation whose integer result no longer fits the current width is stored by the rule above. Negation of a non-negative value stores a signed width when the result is negative. Negation of a signed width's minimum stores the next signed width that holds it. Division is exact: `1 / 2` is one half, and the literal `0.1` is one tenth, not a binary approximation. Division by zero raises `ArithmeticException`.

`var n = 1` has type `Number`. `var x = 1 + 2` has type `Number`. In an expected-type context of a fixed numeric type, a bare literal or an operator fold of bare literals uses that fixed type when the value fits: `Int32 i = 1` and `Int32 i = 1 + 2` resolve on `Int32`. If the value does not fit, the program is rejected.

## 6.2 Methods of `Number`

`Number` defines `plus`, `minus`, `times`, `div`, `negate`, `unaryPlus`, `lessThan`, `greaterThan`, `atMost`, and `atLeast` against `Number`. `%` is rejected on `Number`, because exact division leaves a zero remainder and `rem` is not defined for it.

Comparisons and `equals` use the mathematical value. `hashCode` agrees with `equals` across representations: a u8 one, an i8 one, and an i64 one are equal and hash the same.

`pow(Int32 exponent)` returns `Number`. A negative exponent returns the exact reciprocal when the base is not zero, and raises `ArithmeticException` when the base is zero. `sqrt()`, `sin()`, and `pow(Float64 exponent)` return `Float64`. Bitwise methods and shifts are not defined on `Number`.

`static Number zero()` returns the additive identity. `static Number from(Number n)` returns `n`. `Int32.from(Number n)` and the same method on each fixed class are specified in [section 6.3](#63-fixed-widths). A fixed `from` changes the class only when the value fits.

## 6.3 Fixed widths

The prelude declares these sealed subclasses of `Number`:

```
Int8 Int16 Int32 Int64
UInt8 UInt16 UInt32 UInt64
Float32 Float64
```

Each is a value class. A fixed integer or float instance does not widen. Its representation is exactly those bits. On the signed integers and the unsigned integers, an operation whose mathematical result is not representable in the type raises `ArithmeticException`. That includes negation of the minimum signed value, negation of a nonzero unsigned value, and division by zero. Float operations are specified later in this section and follow IEEE 754 instead of this paragraph.

`div` on a fixed integer type requires exact divisibility and returns the quotient in that type. `5.div(2)` on `Int32` raises `ArithmeticException`. `rem` returns the remainder in that type, with the sign of the dividend, and raises `ArithmeticException` when the divisor is zero. The remainder of a representable pair is representable.

`truncatingDiv` divides toward zero. It raises `ArithmeticException` when the divisor is zero, and when the truncated quotient is not representable in the type, which is the case for the signed minimum divided by negative one. `truncatingRem` satisfies `a.equals(a.truncatingDiv(b).times(b).plus(a.truncatingRem(b)))` for every pair that completes normally, and the remainder has the sign of the dividend.

Each numeric class defines `static` `zero()` returning the additive identity of that class.

`wrappingPlus`, `wrappingMinus`, and `wrappingTimes` compute in the bit width and do not raise. The result is the low bits of the mathematical value, interpreted in the type. `wrappingMinus` of the signed minimum and negative one, and `wrappingTimes` of that minimum and negative one, wrap. They do not raise. No other wrapping operations are declared.

`Float32` and `Float64` follow IEEE 754, including NaN, infinities, and rounding. Their `div` by zero does not raise. Overflow is the IEEE infinity of the appropriate sign. `rem` is the IEEE remainder.

An arithmetic method of a fixed numeric class takes and returns that class. `Int32.plus(Int32)` returns `Int32`. `Float64.plus(Float64)` returns `Float64`. The same pattern holds for `minus`, `times`, `div`, `rem`, and the comparisons. When the static type of the receiver is a fixed class, the method is applicable only when the argument's static type is that same class, or the argument is a bare literal that adopts that class because the value fits exactly. A `Number`, a `Float64`, or a different fixed width is not applicable to `Int32.plus`. The call is rejected. There is no promotion and no narrowing. `Int32 i; Float64 f; i + f` is rejected. `f + i` is rejected. The program writes `Float64.from(i)` when every `Int32` value of that use is exactly a `Float64`, which is true of every `Int32`. It writes `Float64.round` when the conversion may round, which is the case for a `Number` that is not a dyadic rational of that width, and for an `Int64` above the exact integer range of `Float64`. `Int32.from(f)` is the exact narrowing and raises `ArithmeticException` when `f` is not an integer in range. `f + 1` adopts the literal as `Float64`, because one is exact. `f + 0.1` is rejected, because the literal `0.1` is not an exact `Float64`. `i + 1.5` is rejected, because `1.5` is not an `Int32`.

`Number.plus(Number)` and the other `Number` methods apply when the static receiver is `Number`. A fixed class is a `Number`, so `Number n; n + f` is exact rational arithmetic. A finite float contributes the exact rational its IEEE value denotes. A non-finite float raises `ArithmeticException`. The result is `Number`, not `Float64`. A composite uses the same applicability rule on its element type. `Vector<Int32>` does not accept a `Float64` lane or a `Float64` broadcast. `Vector<Float64> v; v * 2` adopts the literal as `Float64`.

`from` on each fixed class accepts `Number` and the other numeric classes. It returns an instance of the target class when the mathematical value is exactly representable in that class, and raises `ArithmeticException` otherwise. `Float64.from` of a value with no exact IEEE representation raises rather than rounding. `Number.from(Number n)` returns `n`. An `Int32` argument is returned as that same value with static type `Number`. That return is the upcast.

`Float32.round(Number n)` and `Float64.round(Number n)` are the rounding conversions. They round to nearest, ties to even, and overflow to the IEEE infinity of the appropriate sign. A value that is already a float of that width is returned unchanged, including NaN and infinities. `from` remains the exact conversion.

`instanceof Int32` is true only for an instance of class `Int32`. A `Number` that fits in 32 bits is not an `Int32`. A cast is that same class test. `(Int32) n` raises `ClassCastException` when `n` is a `Number`. `(Char) n` raises `ClassCastException`. Neither cast changes the width or the scalar. `Int32.from(n)` and `Char.from(n)` are the conversions, and each raises `ArithmeticException` when the value does not fit. An expected-type bare literal such as `Int32 i = 1` adopts `Int32` when the value fits and is not a cast. Foreign code receives a fixed class or a fixed machine shape, never an un-narrowed `Number`.

## 6.4 Equality across numeric classes

`equals` is mathematical. An `Int8` one equals an `Int32` one, and their hash codes match. A finite `Float32` or `Float64` equals an exact number only when it represents that number exactly. `Float64.from` is not how a lossy decimal is spelled, because `from` refuses an inexact value. A `Float64` produced by an IEEE operation equals an exact `Number` only when the IEEE value is exactly that quantity. `1.0` equals `1`. A `Float64` that is the IEEE rounding of one tenth does not equal the literal `0.1`.

`equals` is an equivalence. `NaN.equals(NaN)` is `true`. Positive zero and negative zero are equal. `hashCode` agrees. `equals` is the meaning of `==`. `totalOrder` is not `==`. Each of `Float32` and `Float64` defines `Int32 totalOrder` against its own type, implementing IEEE 754 totalOrder. Negative zero precedes positive zero. A NaN is ordered by its sign bit and then by its payload, and every NaN lies outside the finite numbers and the infinities in the direction IEEE 754 specifies. The result is negative, zero, or positive.

A non-finite float compares `lessThan`, `greaterThan`, `atMost`, and `atLeast` as `false` against every value, including itself.

`Char` and `Boolean` are not numeric. They do not extend `Number`. A `Char` has comparison methods ordered by Unicode scalar value. It has no `plus`. `'a' + 1` is rejected. There is no implicit conversion from any numeric type to `String`. `1 + "count: "` is rejected, because `Number.plus` has no `String` parameter.

## 6.5 Bitwise methods

`and`, `or`, `xor`, `complement`, `shiftLeft`, and `shiftRight` are defined on the fixed integer classes, signed and unsigned. They are not defined on `Number`, `Float32`, or `Float64`. A shift count is `Int32`. The shift methods on a fixed width use the width's bit count; a count outside `0` inclusive through `width - 1` inclusive raises `ArithmeticException`. `>>>` is not a spelling. An unsigned type shifts with `shiftRight` as a logical shift. A signed type's `shiftRight` is an arithmetic shift.

## 6.6 Composites delegate

A composite of numbers uses the component type's operations. It does not define a second arithmetic. `Vector`, `Complex`, and `Matrix` follow that rule. `+`, `-`, `*`, `/`, and `%` lower to the same method names as everywhere else.

```java
value class Vector<T extends Number> {
    public Vector<T> plus(Vector<T> other)
    public Vector<T> minus(Vector<T> other)
    public Vector<T> times(Vector<T> other)
    public Vector<T> times(T scalar)
    public Vector<T> div(Vector<T> other)
    public Vector<T> div(T scalar)
    public Vector<T> rem(Vector<T> other)
    public T dot(Vector<T> other)
}
```

Each lane uses `T`'s method. `Vector<Number>` widens a lane that no longer fits its current representation. `Vector<Int32>` raises `ArithmeticException` when a lane's result does not fit. Unequal lengths raise `ArithmeticException`. A one-element vector is not a scalar. `v * 2` selects `times(T)` and broadcasts. `v * new Vector(2)` selects `times(Vector<T>)` and does not broadcast.

`new Vector(1, 2, 3)` has type `Vector<Number>`. The element type of a vector literal is the common type of the arguments under assignability. Mixing `Int32` and `Number` yields `Vector<Number>` only when every argument is assignable to `Number`, which it is, and the constructor is `Vector<Number>`. There is no implicit lift of an existing `Vector<Int32>` to `Vector<Number>`. `as` on a vector is `Object.as`, the class test, and it is final. `lanes.as(Number.class)` and `(Vector<Number>) lanes` both fail when `lanes` is a `Vector<Int32>`, because that class is not `Vector<Number>` and not `Number`. Scalar `Int32` still lifts to `Number` by subtyping.

The copying lift is a different method:

```java
public <U extends Number> Vector<U> map(Class<U> elementType)
```

`map` evaluates the receiver once, then each lane from left to right. Every lane conversion is the send `U.from`, for a fixed class and for `Number`. There is no second conversion. `Number.from` returns its argument, which is the upcast. A fixed `from` raises `ArithmeticException` when the lane is not exactly representable. A raised `from` converts no later lane. An empty vector evaluates the receiver and the class argument and returns an empty `Vector<U>`. It calls `from` on no lane, so `from` cannot raise. The argument is a non-null `Class<U>`. A null argument is rejected because the parameter is not `@Nullable`. A receiver whose static type is `@Nullable Vector<T>` is rejected until it is narrowed, like any other send. A receiver whose type contains a wildcard is captured first, as [chapter 7](07-generics.md) defines. `Class` is `out`, so `Class<Int32>` is a subtype of `Class<Number>`. With no expected type, `lanes.map(Int32.class)` solves `U` as both `Int32` and `Number`. That is not one solution, and the call is rejected. The program writes `lanes.<Int32>map(Int32.class)`. `lanes.map(Number.class)` has one solution, `U=Number`, because `Number` is the only type in the bound that is a supertype of `Number`. An expected type `Vector<Int32>` determines `U` as `Int32`, and `Int32.class` is applicable. An expected type `Vector<Number>` determines `U` as `Number`, and `Int32.class` is applicable by the `out` subtyping. The conversion is still `Number.from`. A `Class<?>` argument does not determine `U`. The result is monomorphized when `U` is a concrete fixed class. `Vector<Number>` keeps the one `Number` lane layout. `map` is final. A lowering is correct when it matches this send, including a final `U.from` lowered to the conversion [chapter 9](09-execution.md) allows. Overload resolution does not prefer `map` over `as`: the names differ. `lanes.<Int32>map(Int32.class)` is the lift to `Vector<Int32>`. `lanes.map(Number.class)` is the lift to `Vector<Number>`. A `Vector<Int32>` stored in an `Object` slot does not gain a converting `as`. The send is still the class test.

`equals` on a composite is component `equals`. Two `Vector<Number>` values are equal when each lane is the same quantity, regardless of stored width.

```java
value class Complex<T extends Number> {
    public T real
    public T imag
    public Complex<T> plus(Complex<T> other)
    public Complex<T> minus(Complex<T> other)
    public Complex<T> times(Complex<T> other)
    public Complex<T> times(T scalar)
    public Complex<T> div(T scalar)
}
```

`times(Complex<T>)` uses `T.plus` and `T.times` only.

```java
value class Matrix<T extends Number> {
    public Matrix<T> plus(Matrix<T> other)
    public Matrix<T> times(Matrix<T> other)
    public Matrix<T> times(T scalar)
}
```

`times(Matrix<T>)` is the matrix product and uses `T.plus` and `T.times` only. A shape mismatch raises `ArithmeticException`.

A reduction folds with the element operation. `dot` multiplies lanes with `times` and folds with `plus`, starting from `T.zero()` when the length is zero. Further named reductions are not required.

`Matrix<T>` is constructed from a rectangular `Vector<T>[]` of rows, or from `T[][]`. Unequal row lengths raise `ArithmeticException`.

## 6.7 Machine shapes

The only machine shapes are `Int32x4` and `Float64x4`. Both are final value classes. They do not extend `Vector`, they do not widen, and they are not subclasses of `Number`. A lane operation that does not fit raises `ArithmeticException` on `Int32x4` and follows IEEE on `Float64x4`. No other machine shape is declared. A missing shape name is rejected as a missing type. An ordinary class a program declares is not a machine shape, and foreign code and a SIMD register do not receive it as one. A program does not invent a shape by syntax.

Each defines `toVector()` returning `Vector<Int32>` or `Vector<Float64>`. The reverse construction from a vector raises `ArithmeticException` when the length is not 4. Foreign code and a SIMD register receive the machine shape, not `Vector<Number>`.

`Vector<Int32>` and `Vector<Float64>` are monomorphized to flat lanes. A constant length that matches a machine register may be lowered to a machine vector instruction. `Vector<Number>` evaluates lane-wise on `Number` and does not pack mixed widths into one register.

## 6.8 `String` and `Char`

`Char` is a final value class. A value is one Unicode scalar, in 0..10FFFF excluding surrogates. `Char.from(Int32)` and `Char.from(Number)` return that scalar or raise `ArithmeticException`. `Char` is not a subtype of `Number`.

`String` is a final value class. It is a sequence of `Char`. `length()` returns `Int32` and raises `ArithmeticException` on construction of a string longer than `Int32` can count. `get(Int32 index)` returns the scalar at that zero-based index and raises `IndexOutOfBoundsException` when the index is out of range. `plus(Object other)` returns a new string that is the receiver followed by `other.toString()`. `equals` and `hashCode` use the sequence of scalars. `toString` returns the receiver.

`isEmpty()` is `length().equals(0)`. `substring(Int32 begin, Int32 end)` returns the scalars from `begin` inclusive to `end` exclusive, and raises `IndexOutOfBoundsException` when the range is outside the string or `begin` is greater than `end`. `compare(String other)` returns a negative `Int32`, zero, or a positive `Int32` by the first scalar that differs, and by length when one string is a prefix of the other. `indexOf(Char c)` and `indexOf(String s)` return the first index of that argument, or `-1` when it does not occur. `indexOf` of the empty string returns `0`. `startsWith(String prefix)` and `endsWith(String suffix)` report those positions. These are the `String` methods. A further operation is a method a program declares, not an implicit conversion.

## 6.9 Arrays

An array type is written `T[]` for any type `T`, including an array type. An array is a reference object. Its length is fixed at creation. `length()` returns `Int32`. `get` and `set` use an `Int32` index and raise `IndexOutOfBoundsException` when the index is out of range. An array implements `Iterable<T>`.

Arrays are invariant. `String[]` is not a subtype of `Object[]`. A store is typechecked on `T`, so there is no separate array-store failure. An array of a value type stores flat values. `Number[]` uses the one element layout of [section 6.1](#61-number). An array of a reference type stores references. `T[]` does not contain `null` unless `T` is `@Nullable`.

`new T[n]` evaluates `n`, requires a non-negative `Int32`, and produces an array of that length. A negative length raises `ArithmeticException`. It is accepted when `T` is `@Nullable U`, in which case every element starts as `null`; when `T` is a numeric class, in which case every element is that class's `zero()`; when `T` is `Char`, in which case every element is the scalar 0; and when `T` is a composite value type whose fields are all of those kinds, in which case every element is built from those zeros. Every other `new T[n]` is rejected. An array of a type without that default is written `new T[] { e1, e2 }`, which evaluates the elements left to right. The element type of that literal is the written `T`. `new int[n]` is rejected because `int` is reserved.

## 6.10 `Boolean`

`Boolean` is a final value class with exactly two instances, the literals `true` and `false`. It is not numeric. Its methods are the control and Boolean methods of [chapter 4](04-methods.md). `equals` returns `true` only when both instances are `true` or both are `false`. That is equality of the two values. It is not `identical`. `identical` is rejected on a `Boolean` receiver and on a `Boolean` argument, because `Boolean` is a value type, by [chapter 2](02-objects.md). `==` is this `equals`. `hashCode` agrees: `true` and `false` have different hash codes, and each is stable.
