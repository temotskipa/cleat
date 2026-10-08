# 5. Null and Unit

## 5.1 `Null`

`Null` is a final value class with one instance. The literal `null` is that instance. `new Null()` is rejected.

`Null` is not a subclass of `Object`, and no class extends it. It is the one class outside the hierarchy that [chapter 2](02-objects.md) roots at `Object`. A variable of type `Object` therefore cannot hold `null`.

## 5.2 `@Nullable`

`@Nullable` is a type-use annotation. It is a qualifier, declared in the prelude with the facility [chapter 8](08-annotations.md) gives every qualifier. Two facts about it are fixed by the language, because `null` itself is built in: that `null` belongs to `@Nullable T`, and the null test of [section 5.4](#54-null-tests-and-narrowing).

The type `@Nullable T` contains every value of `T`, and `null`. A type written without `@Nullable` does not contain `null`.

- `T` is a subtype of `@Nullable T`. `@Nullable T` is not a subtype of `T`.
- `Null` is a subtype of `@Nullable T` for every `T`. It is a subtype of no other type except itself.
- `@Nullable Object` is a supertype of every type.
- Writing `@Nullable` twice on one type use is the same as writing it once.

`String s = null` is rejected. `@Nullable String s = null` is legal. A `String` may be passed where a `@Nullable String` is expected.

`@Nullable` may mark any type use: the type of a field, a local, a parameter or a result, a type argument, a use of a type parameter, and an array's element type. `@Nullable String[]` is an array whose elements may be `null`. `String @Nullable []` is an array reference that may itself be `null`.

A type parameter with no bound accepts a `@Nullable` type argument. A type parameter whose bound is written without `@Nullable`, such as `T extends Object`, does not.

A `@Nullable` field with no initializer starts as `null`. Every other field has no default and is definitely assigned on every constructor path, as [chapter 9](09-execution.md) requires.

How an implementation represents `null` is not observable.

## 5.3 Sends on a nullable receiver

A method may be sent to a receiver whose static type is `@Nullable T` only when the method declares its receiver `@Nullable`. The receiver is declared in the first parameter position, with the name `this`:

```java
public Boolean isBlank(@Nullable String this)
```

Every other send on a `@Nullable` receiver is rejected until the receiver is narrowed.

`Object` declares four methods this way. `identical` is final, and [chapter 2](02-objects.md#25-the-root-class) says what it answers for `null`. The other three are `equals`, `hashCode` and `toString`. They are `open`, and `Null` declares the same three. When the receiver is `null`, the method that `Null` declares runs:

```java
public final value class Null {
    public Boolean equals(@Nullable Object other) { /* true only when other is null */ }
    public Int hashCode() { /* one unchanging Int */ }
    public String toString() { /* the string "null" */ }
}
```

The parameter of `equals` is `@Nullable Object` on every class. An override of `equals` returns `false` for a `null` argument.

A final method of any class may declare a `@Nullable` receiver. Inside it, `this` has the type `@Nullable C` and is narrowed like any other parameter. An `open` or `abstract` method other than those three must not declare one, because a `null` receiver has no class to select the body.

## 5.4 Null tests and narrowing

`e == null` and `e != null` are null tests, with the operands in either order. A null test is not a send of `equals`. Its result depends only on whether the value of `e` is `null`, whatever `equals` the class of `e` declares. A null test of an expression whose static type cannot contain `null` is rejected.

A local or parameter `x` whose declared type is `@Nullable T` is narrowed to `T` at a point in a method when every path to that point passes a null test of `x` with the not-null outcome and does not assign `x` after that test. [Chapter 12](12-flow.md#124-narrowing) defines the paths.

```java
@Nullable Int seen = counts[word];
if (seen == null) {
    counts[word] = 1;
} else {
    counts[word] = seen + 1;    // seen is an Int here
}
```

`&&` and `||` carry the outcome of the left operand into the right operand. In `x != null && x.isEmpty()` and in `x == null || x.isEmpty()`, the right operand runs only when `x` is not `null`, so the send is legal. A test whose `null` branch cannot complete normally narrows the code after it:

```java
if (value == null) {
    throw new ParseException("expected a number");
}
return new Num(value);          // value is not null here
```

Narrowing applies to a local and to a parameter, including `this` in a method with a `@Nullable` receiver. It does not apply to a field, an array element, or any other expression that is evaluated again, because another read may give a different value. The program copies the value to a local and tests the local.

`x instanceof T` narrows `x` to `T` where the test is known `true`, by the same path rule.

**`??`.** `a ?? b` evaluates `a`. If its value is not `null`, that value is the result and `b` is not evaluated. Otherwise the result is the value of `b`. The static type of `a` must be able to contain `null`. The expression has the type that `x != null ? x : b` would have, where `x` is a local that holds the value of `a`. `??` binds more loosely than `||` and more tightly than `?:`, and `a ?? b ?? c` is `a ?? (b ?? c)`.

```java
counts[word] = (counts[word] ?? 0) + 1;
```

## 5.5 Casts and class tests

A cast is a class test. `(T) e` raises `ClassCastException` when the value of `e` is `null` and `T` is written without `@Nullable`, because the class of `null` is `Null`. It is the checked way from `@Nullable T` to `T`. `(@Nullable T) e` accepts `null`.

`null instanceof T` is `false` for every `T`. Writing `@Nullable` on the type after `instanceof` is rejected.

## 5.6 Type arguments

The nullability of a type argument is part of the class at run time, as every qualifier in a type argument is. `List<String>` and `List<@Nullable String>` are different classes, and a cast or an `instanceof` tells them apart. Neither is a subtype of the other. Where a type parameter is declared `out`, the subtyping of [section 5.2](#52-nullable) passes through: `Iterator<String>` is a subtype of `Iterator<@Nullable String>`.

Inside generic code, `(T) e` tests against the type argument of that instantiation. When the argument is `String`, a `null` raises `ClassCastException`. When the argument is `@Nullable String`, a `null` passes.

## 5.7 `Unit` and `void`

`Unit` is a final value class with no fields and one instance. `void` is how the result type `Unit` is written on a method. A method declared `void` has result type `Unit`, and writing `Unit` as a method result means the same as `void`.

The body of such a method may end without `return`, or with `return;`. Either returns the `Unit` instance. `return` with an operand is rejected in it.

A call whose result is `Unit` may stand alone as an expression statement, because there is nothing to use. A call with any other result must be used, unless its method is marked `@Discardable`, as [chapter 4](04-methods.md) says.

`Unit` is written where a type argument is needed: a lambda that returns nothing has the type `Function0<Unit>`. There is no class `Void`.

```java
public void log(String message) { /* returns the Unit instance */ }
```

```java
log("ok");                              // legal: nothing to use
Function0<Unit> later = () -> log("ok"); // legal: the result is Unit
```
