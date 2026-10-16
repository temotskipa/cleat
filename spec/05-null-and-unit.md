# 5. Null and Unit

## 5.1 `null`

`null` is a value that belongs to no class. <!-- r[null.null-value.belongs-no-class] -->
The literal `null` is that value. <!-- r[null.null-class.literal-null-instance] -->
It is not an instance of `Object`, though every class is a subclass of `Object`, as [chapter 2](02-objects.md#25-the-root-class) says. <!-- r[null.null-value.not-instance-object] -->
A variable of type `Object` therefore cannot hold `null`. <!-- r[null.null-class.variable-type-object] -->

The literal `null` has the null type, which has no name, so a program never writes it. <!-- r[null.null-value.null-type-no-name] -->

## 5.2 Nullable types

A `?` after a type adds `null` to it. <!-- r[null.nullable.nullable-type-use] -->
The type `T?` contains every value of `T`, and `null`. <!-- r[null.nullable.type-nullable-contains] -->
A type written without `?` does not contain `null`. <!-- r[null.nullable.type-written-without] -->

- `T` is a subtype of `T?`. <!-- r[null.nullable.subtype-nullable] -->
  `T?` is not a subtype of `T`. <!-- r[null.nullable.nullable-not-subtype] -->
- The null type is a subtype of `T?` for every `T`. <!-- r[null.nullable.null-subtype-nullable] -->
  It is a subtype of no type written without `?`. <!-- r[null.nullable.subtype-no-type] -->
- `Object?` is a supertype of every type. <!-- r[null.nullable.nullable-object-supertype] -->
- `?` applied to a type that already contains `null` is that same type: where `T` stands for `String?`, `T?` is `String?`. <!-- r[null.nullable.nullable-applied-type] -->
- A qualifier stays on the type that `?` follows: `@Positive Int?` contains every `@Positive Int`, and `null`. <!-- r[null.nullable.qualifier-stays-type] -->

`String s = null` is rejected. <!-- r[null.nullable.string-null-rejected] -->
`String? s = null` is legal. <!-- r[null.nullable.nullable-string-null] -->
A `String` may be passed where a `String?` is expected. <!-- r[null.nullable.string-passed-nullable] -->

`?` may follow any type use: the type of a field, a local, a parameter or a result, a type argument, a use of a type parameter, and an array's element type. <!-- r[null.nullable.nullable-mark-type] -->
`String?[]` is an array whose elements may be `null`. <!-- r[null.nullable.nullable-string-array] -->
`String[]?` is an array reference that may itself be `null`. <!-- r[null.nullable.string-nullable-array] -->

A type parameter with no bound accepts a nullable type argument. <!-- r[null.nullable.type-parameter-no] -->
A type parameter whose bound is written without `?`, such as `T extends Object`, does not. <!-- r[null.nullable.type-parameter-bound] -->

A nullable field with no initializer starts as `null`. <!-- r[null.nullable.nullable-field-no] -->
Every other field has no default and is definitely assigned on every constructor path, as [chapter 9](09-execution.md) requires. <!-- r[null.nullable.field-no-default] -->

How an implementation represents `null` is not observable. <!-- r[null.nullable.how-implementation-represents] -->

## 5.3 Sends on a nullable receiver

A method may be sent to a receiver whose static type is `T?` only when the method declares its receiver with `?`. <!-- r[null.sends.method-sent-receiver] -->
The receiver is declared in the first parameter position, with the name `this`: <!-- r[null.sends.receiver-declared-first] -->

```java
public Boolean isBlank(String? this)
```

Every other send on a nullable receiver is rejected until the receiver is narrowed. <!-- r[null.sends.send-nullable-receiver] -->

`Object` declares four methods this way. <!-- r[null.sends.object-declares-four] -->
`identical` is final, and [chapter 2](02-objects.md#25-the-root-class) says what it answers for `null`. <!-- r[null.sends.identical-final-says] -->
The other three are `equals`, `hashCode` and `toString`. <!-- r[null.sends.three-equals-hashcode] -->
They are `open`, and an override runs only for a receiver that is an instance of its class. <!-- r[null.sends.open-null-declares] -->
When the receiver is `null`, the body that `Object` declares runs: <!-- r[null.sends.receiver-null-method] -->

```java
public open class Object {
    public open Boolean equals(Object? this, Object? other) { /* for null: true only when other is null */ }
    public open Int hashCode(Object? this) { /* for null: one unchanging Int */ }
    public open String toString(Object? this) { /* for null: the string "null" */ }
}
```

The parameter of `equals` is `Object?` on every class. <!-- r[null.sends.parameter-equals-nullable] -->
An override of `equals` returns `false` for a `null` argument. <!-- r[null.sends.override-equals-returns] -->

A final method of any class may declare a nullable receiver. <!-- r[null.sends.final-method-class] -->
Inside it, `this` has the type `C?` and is narrowed like any other parameter. <!-- r[null.sends.inside-this-type] -->
An `open` or `abstract` method other than those three must not declare one, because a `null` receiver has no class to select the body. <!-- r[null.sends.open-abstract-method] -->

## 5.4 Null tests and narrowing

`e == null` and `e != null` are null tests, with the operands in either order. <!-- r[null.tests.equals-null-not-equals] -->
A null test is not a send of `equals`. <!-- r[null.tests.null-test-not] -->
Its result depends only on whether the value of `e` is `null`, whatever `equals` the class of `e` declares. <!-- r[null.tests.result-depends-only] -->
A null test of an expression whose static type cannot contain `null` is rejected. <!-- r[null.tests.null-test-expression] -->

A local or parameter `x` whose declared type is `T?` is narrowed to `T` at a point in a method when every path to that point passes a null test of `x` with the not-null outcome and does not assign `x` after that test. <!-- r[null.tests.local-parameter-declared] -->
[Chapter 12](12-flow.md#124-narrowing) defines the paths. <!-- r[null.tests.defines-paths] -->

```java
Int? seen = counts[word];
if (seen == null) {
    counts[word] = 1;
} else {
    counts[word] = seen + 1;    // seen is an Int here
}
```

`&&` and `||` carry the outcome of the left operand into the right operand. <!-- r[null.tests.and-or-carry] -->
In `x != null && x.isEmpty()` and in `x == null || x.isEmpty()`, the right operand runs only when `x` is not `null`, so the send is legal. <!-- r[null.tests.not-equals-null-and] -->
A test whose `null` branch cannot complete normally narrows the code after it: <!-- r[null.tests.test-null-branch] -->

```java
if (value == null) {
    throw new ParseException("expected a number");
}
return new Num(value);          // value is not null here
```

Narrowing applies to a local and to a parameter, including `this` in a method with a nullable receiver. <!-- r[null.tests.narrowing-applies-local] -->
It does not apply to a field, an array element, or any other expression that is evaluated again, because another read may give a different value. <!-- r[null.tests.not-apply-field] -->
The program copies the value to a local and tests the local. <!-- r[null.tests.program-copies-value] -->

`x instanceof T` narrows `x` to `T` where the test is known `true`, by the same path rule. <!-- r[null.tests.instanceof-narrows-test] -->

**`??`.** `a ?? b` evaluates `a`. <!-- r[null.tests.coalesce-evaluates] -->
If its value is not `null`, that value is the result and `b` is not evaluated. <!-- r[null.tests.value-not-null] -->
Otherwise the result is the value of `b`. <!-- r[null.tests.otherwise-result-value] -->
The static type of `a` must be able to contain `null`. <!-- r[null.tests.static-type-able] -->
The expression has the type that `x != null ? x : b` would have, where `x` is a local that holds the value of `a`. <!-- r[null.tests.expression-type-not-equals] -->
`??` binds more loosely than `||` and more tightly than `?:`, and `a ?? b ?? c` is `a ?? (b ?? c)`. <!-- r[null.tests.coalesce-binds-more] -->

```java
counts[word] = (counts[word] ?? 0) + 1;
```

## 5.5 Casts and class tests

A cast is a class test. <!-- r[null.casts.cast-class-test] -->
`(T) e` raises `ClassCastException` when the value of `e` is `null` and `T` is written without `?`, because `null` is an instance of no class. <!-- r[null.casts.raises-classcastexception-value] -->
It is the checked way from `T?` to `T`. <!-- r[null.casts.checked-way-nullable] -->
`(T?) e` accepts `null`. <!-- r[null.casts.nullable-accepts-null] -->

`null instanceof T` is `false` for every `T`. <!-- r[null.casts.null-instanceof-false] -->
The type after `instanceof` has no `?`, and a `?` after it begins the operator `?:`. <!-- r[null.casts.writing-nullable-type] -->

## 5.6 Type arguments

The nullability of a type argument is part of the class at run time, as every qualifier in a type argument is. <!-- r[null.type-arguments.nullability-type-argument] -->
`List<String>` and `List<String?>` are different classes, and a cast or an `instanceof` tells them apart. <!-- r[null.type-arguments.list-string-nullable] -->
Neither is a subtype of the other. <!-- r[null.type-arguments.subtype] -->
Where a type parameter is declared `out`, the subtyping of [section 5.2](#52-nullable-types) passes through: `Iterator<String>` is a subtype of `Iterator<String?>`. <!-- r[null.type-arguments.type-parameter-declared] -->

Inside generic code, `(T) e` tests against the type argument of that instantiation. <!-- r[null.type-arguments.inside-generic-code] -->
When the argument is `String`, a `null` raises `ClassCastException`. <!-- r[null.type-arguments.argument-string-null] -->
When the argument is `String?`, a `null` passes. <!-- r[null.type-arguments.argument-nullable-string] -->

## 5.7 `Unit` and `void`

`Unit` is a final value class with no fields and one instance. <!-- r[null.unit.unit-final-value] -->
`void` is how the result type `Unit` is written on a method. <!-- r[null.unit.void-how-result] -->
A method declared `void` has result type `Unit`, and writing `Unit` as a method result means the same as `void`. <!-- r[null.unit.method-declared-void] -->

The body of such a method may end without `return`, or with `return;`. <!-- r[null.unit.body-method-end] -->
Either returns the `Unit` instance. <!-- r[null.unit.returns-unit-instance] -->
`return` with an operand is rejected in it. <!-- r[null.unit.return-operand-rejected] -->

A call whose result is `Unit` may stand alone as an expression statement, because there is nothing to use. <!-- r[null.unit.call-result-unit] -->
A call with any other result must be used, unless its method is marked `@Discardable`, as [chapter 4](04-methods.md) says. <!-- r[null.unit.call-result-used] -->

`Unit` is written where a type argument is needed: a lambda that returns nothing has the type `Function0<Unit>`. <!-- r[null.unit.unit-written-type] -->
There is no class `Void`. <!-- r[null.unit.no-class-void] -->

```java
public void log(String message) { /* returns the Unit instance */ }
```

```java
log("ok");                              // legal: nothing to use
Function0<Unit> later = () -> log("ok"); // legal: the result is Unit
```
