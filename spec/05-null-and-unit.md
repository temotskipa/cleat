# 5. Null and Unit

## 5.1 `Null`

`Null` is a final value class. It has one instance. The literal `null` is that instance. `new Null()` is rejected. `null.getClass()` is `Null.class`.

```java
public final value class Null {
    public Boolean equals(Object other) { /* true only for this instance */ }
    public Int32 hashCode()
    public String toString() { /* the string null */ }
}
```

`Null.equals` returns `true` only for that instance. `hashCode` returns one unchanging `Int32` on every call. A program may rely on that stability and on agreement with `equals`. It may not rely on which bits are set. Every implementation that returns one unchanging `Int32` is correct. [Chapter 10](10-deferred.md) records that bound. `toString` returns the string whose scalars are `n`, `u`, `l`, `l`. `hashCode` reads no field. It is the same value for the one instance, including when that instance is stored in `Object`.

`isNull` is the final `Object` method. On the null instance it returns `true`. A user class does not override it.

## 5.2 Non-null by default

A type written without `@Nullable` does not contain the null instance. `@Nullable` is a type-use annotation owned by the language. It may mark any type use, including a type argument and a use of a type parameter: `List<@Nullable String>`, `@Nullable T`. Duplicate `@Nullable` on the same use collapses to one.

`String s = null` is rejected. `@Nullable String s = null` is legal. A non-null value may be passed where `@Nullable` of that type is expected.

`@Nullable` on a reference type uses the empty reference as its representation of `null`. `@Nullable` on `Number` or another value type is a flattened optional: the payload and a presence flag, not a pointer. The language rule is the same in both representations.

An uninitialized `@Nullable` field starts as `null`. A non-null field has no default and is definitely assigned on every constructor path.

## 5.3 Sends on a nullable receiver

When the static type of a receiver is `@Nullable T`, the methods that may be sent without a narrowing are `equals`, `hashCode`, `toString`, `getClass`, `identical`, `isNull`, and `as`. Every other send is rejected until the receiver is narrowed to `T`.

`identical` remains rejected when `T` itself is a value type, by the rule in [chapter 2](02-objects.md). `@Nullable String` is a reference type that may be null, so `identical` is legal on it. `@Nullable Int32` is still a value type, so `identical` is rejected on it.

A failed `equals` against the literal `null` narrows the other operand from `@Nullable T` to `T`. The same narrowing applies when `isNull()` is `false`, when `!= null` is `true`, and on the else path of `== null` or of `isNull()` being `true`. A true `isNull()` or a true `== null` narrows the operand to `Null`.

Narrowing applies to a local or parameter that is not assigned in the narrowed region. It does not apply to a field, an array element, or any other location that is re-read. A field test reads the field and does not change the field's type on the next read.

`&&` carries the narrowings of its left operand into its right operand and into the region where the conjunction is known true. `||` carries a narrowing into the continuation only when both sides establish it. A narrowing from `instanceof` is defined in [section 5.4](#54-class-tests).

## 5.4 Class tests

`instanceof` and a true branch of `isInstance` narrow a stable local to the tested class. The test is the runtime class, including reified type arguments. A wildcard in the tested type matches as [chapter 7](07-generics.md) defines. A `Number` that would fit in 32 bits and is not an instance of class `Int32` makes `instanceof Int32` false. `(Int32) n` on that value raises `ClassCastException`. Width conversion is `Int32.from(n)`, which raises `ArithmeticException` when the value is not an integer in the `Int32` range.

`(String) null` raises `ClassCastException`, because the class of `null` is `Null`. There is no cast that strips `@Nullable`. Narrowing does that, and only for the locals [section 5.3](#53-sends-on-a-nullable-receiver) names.

## 5.5 `Unit`

`Unit` is a final value class with no fields and one instance. Every evaluation of `new Unit()` returns that instance. All `Unit` values are equal. A type argument may be `Unit`, so a generic type the program declares may be instantiated at `Unit`. There is no class `Void` and no type named `void`.

A method whose body has nothing to report returns that instance. The caller learns whether the instance is to be used from the way the result is written.

## 5.6 The `void` form

`void` is a method-result form, not a type. A method declared `void` returns the `Unit` instance. A direct call of that method must be an expression statement. Using the call as a value, including `return` of it from a method declared to return `Unit`, is rejected.

A method declared to return `Unit` returns the same instance. A direct call must be used. An expression statement that is only that call is rejected. The body of a `Unit` method must return an expression of type `Unit` on every path. `return;` in a `Unit` method is rejected. Falling off the end of a `Unit` method is rejected.

The body of a `void` method may end without `return`, or with `return;`. `return` of an expression in a `void` method is rejected. The method still returns the `Unit` instance to its caller inside the implementation. The caller of a direct call cannot name that instance.

An override repeats the form. A `void` method is not an override of a `Unit` method, and the reverse is rejected, even though both return the same instance.

Generics see `Unit` for both forms. The type of a functional method, of `Block.invoke`, and of a type argument is `Unit`, never `void`. The `void` restriction applies to a direct call of a method declared `void`. It does not apply to `invoke` on a `Block<Unit>` that was built from that method.

A statement-form `if`, `while`, `for`, or `try` may discard the `Unit` that `then`, `whileTrue`, or `catching` returns. That discard is part of the statement. It does not authorize discarding a direct call.

```java
public void log(String message) { /* body returns the Unit instance */ }
public Unit ready() { return new Unit(); }

log("ok");          /* legal: direct void call as a statement */
ready();            /* rejected: Unit result unused */
Unit u = ready();   /* legal */
Unit v = log("ok"); /* rejected: void call used as a value */
```
