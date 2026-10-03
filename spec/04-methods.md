# 4. Methods

## 4.1 Declaration

```
method     = {method-mod} [type-params] result identifier "(" [params] ")" (block | ";")
method-mod = audience | "only" ... | "static" | "final" | "open"
           | "abstract" | "inline" | "foreign"
result     = "void" | type
params     = param {"," param}
param      = {param-mod} type identifier
           | {param-mod} type "..." identifier
```

Parameter types and the result type are written. `var` is rejected in those positions. A parameter is a local of the method. It is reassignable unless declared `final`.

A concrete method is final unless declared `open`. `final` may be written to state the default. `open final` is rejected. An `abstract` method has no body, is overridable, and is not `final` and not `inline`. An abstract method appears only in an abstract class or an interface.

An override matches the overridden signature exactly, including the result type, type parameters, and whether the result is written `void` or `Unit`. A covariant result is rejected. The overriding method is itself final unless declared `open`.

A static method has no receiver instance. Using `this` or `super` in a static method is rejected. A static method may overload an instance method. The call is static when the receiver expression is a type name, and instance when the receiver expression is a value.

## 4.2 Calls

A call evaluates the receiver, then the arguments from left to right, then enters the method. Arguments are values. A parameter receives the argument object. Assignment to the parameter does not assign to the caller's variable. Laziness is a block argument, not a calling convention.

The receiver of a static call is the compile-time class. The receiver of an instance call is the value of the receiver expression. An instance call whose compile-time method is final may be lowered to a direct call. That lowering is available to every final method. It is not a privilege of a prelude type.

`foreign` marks a method with no source body. The body is supplied by foreign code. The signature is the platform C ABI for the mapping in [section 4.11](#411-foreign-signatures). `Number`, an unspecialized generic value type, a wildcard, and a Cleat reference are rejected on a `foreign` method. A `Block` is not a machine type and is rejected as a foreign parameter, so a foreign method has no Cleat callback.

## 4.3 Overloading

Two methods of one class may share a name when their parameter lists differ in number or in a parameter type. The signature includes the parameter types after substitution of any explicit type arguments. Return type and `void` versus `Unit` do not distinguish overloads.

Resolution discards every method that is not applicable. A method is applicable when the argument count matches and each argument is assignable to the corresponding parameter under [section 4.4](#44-assignability), using the inference of [chapter 7](07-generics.md). A numeric literal or an operator expression whose operands are numeric literals prefers an applicable fixed numeric parameter type in which the value fits over a parameter type of `Number`. Among the methods that remain, a method is more specific when each of its parameter types is a subtype of the other method's corresponding parameter type. If exactly one most specific method remains, it is chosen. Otherwise the call is rejected and the program writes explicit type arguments, as in `Box.<Int32>of(1)`.

There is no boxing conversion in overload resolution. There are no raw types. A value whose static type is a value type is already that type; storing it in `Object` is an upcast, not a conversion invented by resolution.

## 4.4 Assignability

A value of type `S` is assignable to a variable of type `T` when `S` is a subtype of `T`, or when the value is a bare numeric literal or a fold of bare numeric literals and the operators of [chapter 6](06-numbers.md) and `T` is a fixed numeric type in which that value fits.

Subtyping is the reflexive transitive closure of `extends` and `implements`, plus the generic rules of [chapter 7](07-generics.md), plus the nullability rule that a type `T` is assignable to `@Nullable T`. The reverse is rejected. The literal `null` is assignable to every `@Nullable` type and to no other type.

No method is inserted to convert `S` to `String`, to a different numeric width, or to a superclass by copying. An implicit conversion from `Int32` to `Number` is subtyping: `Int32` extends `Number`. An implicit conversion from `Vector<Int32>` to `Vector<Number>` does not exist.

## 4.5 Operator spellings

An operator spelling is a method of the left operand, or of the single operand. The right operand is the argument. The spelling exists on an expression when the resolved method exists. No prelude type is rewritten by a second rule.

| Spelling | Method |
| --- | --- |
| `a + b` | `a.plus(b)` |
| `a - b` | `a.minus(b)` |
| `a * b` | `a.times(b)` |
| `a / b` | `a.div(b)` |
| `a % b` | `a.rem(b)` |
| `-a` | `a.negate()` |
| `+a` | `a.unaryPlus()` |
| `a < b` | `a.lessThan(b)` |
| `a > b` | `a.greaterThan(b)` |
| `a <= b` | `a.atMost(b)` |
| `a >= b` | `a.atLeast(b)` |
| `a == b` | `a.equals(b)` |
| `a != b` | the negation of `a.equals(b)` |
| `a & b` | `a.and(b)` |
| `a \| b` | `a.or(b)` |
| `a ^ b` | `a.xor(b)` |
| `~a` | `a.complement()` |
| `a << b` | `a.shiftLeft(b)` |
| `a >> b` | `a.shiftRight(b)` |
| `a[i]` | `a.get(i)` |
| `a[i] = e` | `a.set(i, e)` |
| `!a` | `a.not()` |

`!=` is not a method. It is the Boolean negation of `equals`. `==` is `equals` for every type, reference or value.

Precedence, tightest first, is the expression layers of [chapter 11](11-syntax.md):

| Operators | Association |
| --- | --- |
| postfix `[]` `.` `()` `++` `--` | left |
| unary `+` `-` `~` `!` prefix `++` `--` cast `new` | right |
| `*` `/` `%` | left |
| `+` `-` | left |
| `<<` `>>` | left |
| `<` `>` `<=` `>=` `instanceof` | left |
| `==` `!=` | left |
| `&` | left |
| `^` | left |
| `\|` | left |
| `&&` | left |
| `\|\|` | left |
| `?:` | right |
| `=` and the compound assignments | right |

`&&` has lower precedence than `&`. A compound assignment `a += b` evaluates the location of `a` once and is `a = a.plus(b)` on that location. The same pattern applies to `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, and `>>=`.

`&&`, `||`, `if`, `?:`, `throw`, `new`, casts, and `instanceof` are also spellings of methods. They are specified with control flow in [section 4.8](#48-control-spellings) because their arguments are blocks or class literals.

## 4.6 Blocks

A block is an object. The prelude defines three classes, and the last type parameter is the result:

```java
public final value class Block<T> {
    public T invoke();
}
public final value class Block<A, T> {
    public T invoke(A a);
}
public final value class Block<A, B, T> {
    public T invoke(A a, B b);
}
```

`Block` is one prelude family, selected by the number of type arguments. A user package declares at most one type of a given simple name. The natural type of a lambda is one of these classes. A lambda expression is `( [params] ) -> expr` or `( [params] ) -> block`. Parameter types may be omitted when an expected type provides them. A lambda with no expected type and an untyped parameter is rejected. `() -> 1` has type `Block<Number>`.

A functional interface is an interface with exactly one abstract method. Where the expected type is a functional interface and the lambda's parameter and result types match that method, the lambda is converted to the interface. An exact `Block` type beats that conversion. If two functional interfaces are applicable and no exact `Block` is expected, the conversion is rejected.

A lambda is not a class. A non-inline lambda captures `this` and each local it uses. A captured local is not assigned after its initializer. An assignment to such a local anywhere in its scope is rejected. An inline lambda is not an escaping value; the effectively-final rule does not apply to it, and it may assign locals of the enclosing method.

A `Block` of three or more type parameters is rejected. A lambda of three or more parameters is rejected unless a functional interface is the expected type and the conversion applies. That conversion still does not make the lambda a class.

## 4.7 `inline`

`inline` may be written on a final method, including a final static method. `inline` on an `open`, `abstract`, or non-final method is rejected.

A parameter whose type is a `Block` may be declared `inline`. An inline block parameter is not stored, is not returned, and is not passed to a parameter that is not itself inline. The method is typechecked under that restriction. The compiler substitutes the caller's block at the call. The substitution is part of the language rule for `return`, `break`, and `continue`, not an optional optimization.

In an inline block, `return` propagates to the enclosing method through inline callers that do not handle it. `break` and `continue` propagate to the enclosing `while` or `for` written in the source. `ensuring` handles those three effects as [chapter 9](09-execution.md) defines, runs its cleanup, and then propagates them. No other method handles them. In a non-inline lambda, `return` leaves the lambda, and `break` or `continue` is rejected.

`Boolean.then`, `Boolean.andAlso`, `Boolean.orElse`, `Block.whileTrue`, `catching`, and `ensuring` are inline methods. Their block parameters are inline. A user method with the same discipline writes `inline` and receives the same control rule. The types are not privileged.

## 4.8 Control spellings

Condition and loop spellings:

```java
public final value class Boolean {
    public inline <T> T then(inline Block<T> ifTrue, inline Block<T> ifFalse)
    public inline Boolean andAlso(inline Block<Boolean> other)
    public inline Boolean orElse(inline Block<Boolean> other)
    public Boolean and(Boolean other)
    public Boolean or(Boolean other)
    public Boolean xor(Boolean other)
    public Boolean not()
}

public static inline Unit whileTrue(
    inline Block<Boolean> condition,
    inline Block<Unit> body)
```

`whileTrue` is a static method of `Block`.

| Source | Send |
| --- | --- |
| `if (c) a else b`, as an expression | `c.then(() -> a, () -> b)` |
| `c ? a : b` | `c.then(() -> a, () -> b)` |
| `c && d` | `c.andAlso(() -> d)` |
| `c \|\| d` | `c.orElse(() -> d)` |
| `while (c) body` | `Block.whileTrue(() -> c, () -> body)` |

A statement `if (c) stmtA else stmtB` is `c.then(() -> { stmtA }, () -> { stmtB })`. A missing `else` is an empty second block. Each statement block's result is `Unit`. `return` inside either block propagates as an inline `return`.

`c` in `if`, `?:`, `&&`, and `||` is evaluated once. The chosen branch is evaluated. The other branch is not. `while` evaluates the condition block on every iteration, then the body block when the condition is `true`, and stops when the condition is `false`. The body block's result is `Unit`.

The result type of an `if` expression or of `?:` is the result type of `then` after inference. The two branches must agree under that inference or the expression is rejected. An `if` statement, a `while` statement, and a `for` statement discard the `Unit` result of the send. That discard belongs to the statement form. It is not a discard of a direct call, and [chapter 5](05-null-and-unit.md) defines the difference. An `if` expression does not discard its result.

`for (T x : e) body` evaluates `e`, sends `iterator()`, and then behaves as `while` on `hasNext` and a body that binds `x` to `next()` and then runs `body`. `x` is not reassignable. The static type of `e` must have a method `iterator()` returning a type with `hasNext()` and `next()`. The prelude interfaces are:

```java
interface Iterator<out T> {
    Boolean hasNext();
    T next();
}
interface Iterable<out T> {
    Iterator<T> iterator();
}
```

`for (init; condition; update) body` runs `init` once. A missing condition is the constant `true`. Before each iteration the condition runs. When it is `false`, the loop completes normally. Otherwise the body runs. The update runs after the body completes normally and after a `continue` that targets this loop. The update does not run after `break`, `return`, or `throw`. A local declared in `init` is in scope in the condition, the body, and the update. `break` and `continue` of this loop follow [section 4.12](#412-labels-switch-assert-and-increment).

`throw e` is `e.raise()`. The static type of `e` must be `Throwable` or a subclass. `raise` does not complete normally.

`new C(args)` is `C.new(args)`, a static call on the class object.

`(T) e` is `e.as(T.class)` when `T` contains no wildcard. It is the class test of [chapter 2](02-objects.md). It is not a numeric conversion and it does not convert `Char`. `(Int32) n` on a `Number` raises `ClassCastException` even when the quantity fits in 32 bits. `(Char) 65` raises `ClassCastException`. The width change is `Int32.from`, or an expected-type bare literal. The scalar conversion is `Char.from`. When `T` contains a wildcard, the cast is the test in [chapter 7](07-generics.md) and is not a class-literal send. A cast inside a constant expression uses this same test at compile time. A constant cast that would raise is rejected, as [chapter 12](12-flow.md) defines. The receiver is evaluated once. `==` is `equals` for the result, as for every other value.

`e instanceof T` is `T.class.isInstance(e)` when `T` contains no wildcard. When `T` contains a wildcard, the test is the same wildcard test. The result is `Boolean`.

## 4.9 Locals and statements

A local declaration is `type name = expr ;` or `var name = expr ;` or the same with `final`. `var` infers the type of `expr`, including type annotations that are part of that type, and excluding annotations that attach to a declaration rather than a type. `var` with a written type is rejected. `var` on a field, parameter, or result is rejected. `var name = null` is rejected, because the literal has no class to infer beyond the separate nullability rule, and a local of type `Null` is written `Null name = null`.

A local is reassignable unless declared `final`. A `final` local is assigned exactly once, by its initializer.

Statements are local declarations, expression statements, `if`, `while`, `for`, `return`, `break`, `continue`, `throw`, `try`, `using`, and blocks. An expression statement is an assignment or a method call. A call used as an expression statement is under the `void` rule of [chapter 5](05-null-and-unit.md). Any other unused result is rejected, including a discarded `Int32` and a discarded `Unit` from a method declared to return `Unit`.

`break` and `continue` with no label name the innermost enclosing `while` or `for`. `break` with no label may also name the innermost enclosing `switch`. A label is the form in [section 4.12](#412-labels-switch-assert-and-increment).

## 4.10 Assignment

`name = e` evaluates `e` and binds the object to the local or parameter. `field = e` and `receiver.field = e` are setter calls, or slot writes where [chapter 2](02-objects.md) gives slot access. `a[i] = e` is `set`. The left side is evaluated before the right side. A location is evaluated once.

## 4.11 Foreign signatures

A foreign parameter or result is a machine type: `Boolean`, `Char`, a fixed integer, `Float32`, `Float64`, a machine shape such as `Int32x4`, or `Pointer`. `Pointer` is a final value class holding an address. The collector does not trace it. `equals` compares addresses. Cleat does not dereference a `Pointer`. The program passes it to foreign methods.

| Cleat type | C type |
| --- | --- |
| `Boolean` | `_Bool` |
| `Char` | `uint32_t`, one scalar |
| `Int8`, `Int16`, `Int32`, `Int64` | the matching signed integer |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | the matching unsigned integer |
| `Float32`, `Float64` | `float`, `double` |
| `Int32x4`, `Float64x4` | the platform vector of that lane type, passed by value |
| `Pointer` | `void *` |

A foreign method may also take or return a value class whose fields are all machine types. The layout is source order, natural alignment of each field, no header, and trailing padding to the alignment of the strictest field. A value class that does not meet that restriction is rejected on a foreign signature. The call uses the platform C ABI for those C types and that struct layout. Which registers the platform uses is the platform ABI. The mapping in the table is not platform-defined.

Memory allocated by foreign code is not collected. A returned `Pointer` is an address the collector ignores. The program releases it by a foreign method if the foreign code requires release. `Pointer` is unchecked: it has no length, no element type, and no pin. The checked forms are [section 4.14](#414-checked-foreign-memory).

## 4.14 Checked foreign memory

A machine type for this section is a type legal as a foreign parameter, other than `Pointer`. A value class whose fields are all machine types is a machine type. `Pointer`, a Cleat reference, `Number`, a wildcard, and a `Block` are not.

```java
public final value class Span<T> {
    public Int32 length()
}
public final class Pin<T> {
    public static <T> Pin<T> of(T[] elements)
    public Span<T> span()
    public void close()
}
public final class ForeignBuffer<T> {
    public static <T> ForeignBuffer<T> adopt(
        Pointer address, Int32 length, Block<Pointer, Unit> release)
    public Span<T> span()
    public void close()
}
```

`Span<T>`, `Pin<T>`, and `ForeignBuffer<T>` are rejected when `T` is not a machine type. `Span` is a value. It holds an address and a length and is not traced. `equals` compares that address and that length. Two empty spans are equal. `Pin` and `ForeignBuffer` are reference classes. Their `close` methods are declared `void`, so `using` applies.

`Pin.of` evaluates the array once. The array is non-null. A null array is rejected because the parameter is not `@Nullable`. The length is the array's length. An empty array is legal: the address is the null address and the length is zero. The array object is pinned. A moving collection must not relocate that object or its elements until `close`. The pin is visible to every thread. `close` removes the pin. A second `close` raises `IllegalStateException`. `span` after `close` raises `IllegalStateException`. `close` on a pin that is still inside a foreign call on any thread raises `IllegalStateException`.

`ForeignBuffer.adopt` evaluates the address, the length, and the release block from left to right. A negative length raises `IllegalArgumentException` and does not call `release`. A null address is legal only when the length is zero. Otherwise it raises `IllegalArgumentException`. The release block is non-null. `close` marks the buffer closed and then invokes `release` once with the address. The `Unit` result of that invoke is used. If `release` raises, the buffer stays closed and the exception propagates. A second `close` raises `IllegalStateException` and does not invoke `release` again. `span` after `close` raises `IllegalStateException`. The collector does not call `close` and does not free the address. Forgetting `close` leaks the foreign memory. There is no `finalize`.

A `Span` from `span()` is confined to the lifetime of that pin or buffer. Storing it in a field, returning it, or passing it to a parameter that is not a foreign parameter and not an `inline` parameter is rejected. An `inline` method that receives it is checked by the same rule. Passing it to a foreign parameter is legal. The foreign parameter `Span<T>` is two C parameters, in order: a pointer to `T`, then an `int32_t` length. It is not one `Pointer`. An empty span passes the null address and the length zero. A foreign result of type `Span<T>` is rejected. Foreign code returns a `Pointer` and an `Int32`, and Cleat code passes them to `adopt`.

The address of a pin is valid only while the pin is open and only for the foreign calls reached from that lifetime. The address of a buffer is valid only while the buffer is open. Cleat does not dereference either address. A foreign function that stores the address and uses it after `close` is outside the lifetime. Cleat does not make that later use defined.

A foreign call does not lock the pinned array. A race between Cleat and foreign code on those elements remains a race under [chapter 13](13-concurrency.md). The pin keeps the address stable. It does not make the elements atomic beyond the rule that chapter already gives them. A `Vector` or other value stored in an `Object` slot is not an array and is not a `Pin`. `Pin.of` is not a conversion from `Object`.

## 4.12 Labels, switch, assert, and increment

A labeled statement is `identifier : statement`. `break identifier` completes that statement normally. `continue identifier` continues the labeled statement, which must be a `while` or a `for`. A label that does not enclose the `break` or `continue` is rejected. A `continue` of a statement that is not a loop is rejected.

`switch (selector) { arms }` evaluates the selector once. Each arm is `case constant -> statement` or `default -> statement`. An arm does not fall through. The end of an arm completes the switch unless the arm itself completes abruptly. Constants are constant expressions of the selector's type, compared with `equals`. The selector's static type is a fixed integer, `Char`, `Boolean`, `String`, or an enum. `Number` and the floating types are rejected as selectors. Duplicate constants are rejected. A statement switch with no matching arm completes normally. An expression switch is exhaustive or it is rejected. Exhaustive means a `default` arm, both Boolean constants, or every constant of an enum. An expression switch is an expression. Its type is the common type of the arm expressions under [chapter 7](07-generics.md) inference. A statement switch is a statement.

`assert condition` evaluates `condition`, which has type `Boolean`. When the value is `false`, it raises `AssertionException`. `assert condition : detail` also evaluates `detail` in that case and attaches `detail.toString()` to the exception. Assertions are always checked. There is no mode that removes them.

`++ location` and `location ++` evaluate the location once. The update is `location = location.plus(one)`. `one` is the literal `1` adopted to the static type of the location when that type is numeric and `1` fits, and is rejected otherwise. Prefix yields the updated value. Postfix yields the value from before the update. `--` is the same with `minus`. If `plus` or `minus` raises, the location is unchanged.

## 4.13 Varargs and method references

A parameter `T... name` is a parameter of type `T[]` and is the last parameter. At a call, a fixed-arity applicable method is more specific than a varargs method. When the varargs method is chosen, a single trailing argument assignable to `T[]` is passed as that array. Otherwise the trailing arguments are evaluated left to right and stored in a new array of type `T`. Zero trailing arguments produce a zero-length array. `new T[] { }` is legal for every `T`, including a `T` that [chapter 6](06-numbers.md) rejects for `new T[n]`.

A method reference is `Type::name`, `expr::name`, or `Type::new`. It has an expected type, which is a `Block` or a functional interface. Without an expected type it is rejected. `Type::name` for a static method is the lambda `(args) -> Type.name(args)`. `Type::name` for an instance method is the lambda `(receiver, args) -> receiver.name(args)`. `expr::name` evaluates `expr` once, when the reference is evaluated, and is the lambda `(args) -> captured.name(args)`. `Type::new` is `(args) -> new Type(args)`. Overload resolution uses the expected function type. A method reference is not a class. Capture of locals follows the lambda rule.
