# 4. Methods

This chapter defines methods, the expressions that call them, and the statements of a method body. [Chapter 11](11-syntax.md) gives the syntax, including operator precedence.

## 4.1 Declaring a method

A method declaration has modifiers, optional type parameters, a result, a name, a parameter list, and a body.

- The result is a type, or `void`, which is how the result `Unit` is written, as [chapter 5](05-null-and-unit.md) defines.
- Parameter types and the result type are written. `var` is rejected in those positions. A parameter is a local of the method, and is assignable unless it is declared `final`.
- The modifiers are an audience from [chapter 3](03-visibility.md), `static`, `final`, `open`, `abstract` and `foreign`.
- A generic method writes its type parameters before the result, as [chapter 7](07-generics.md) defines.

A method of a class is final unless it is declared `open`: no subclass overrides it. `final` may be written to state that default. An `abstract` method has no body and appears only in an abstract class. `open final`, `abstract final` and `abstract static` are rejected. The methods of an interface are described in [chapter 2](02-objects.md).

A static method has no receiver. `this` and `super` are rejected in it. A static method and an instance method may share a name. A call is static when its receiver is a type name, and is an instance call when its receiver is a value.

**The receiver parameter.** An instance method may declare its receiver as a first parameter named `this`. Its type is the declaring class or interface, with qualifiers:

```java
public open Boolean equals(@Nullable Object this, @Nullable Object other)
```

The receiver parameter is not an argument position, and a call supplies no argument for it. [Chapter 8](08-annotations.md) says what its qualifiers mean. Without the declaration, the receiver's type is the declaring class with no qualifier. A static method and a constructor do not declare a receiver.

## 4.2 Overriding

A method of a class overrides a method of its superclass, or implements a method of an interface, when the two have the same name and the same signature: the same type parameters and the same parameter types, with the same qualifiers. Its result type is the result type of the method it overrides, or a subtype of it, so `Circle copy()` may override `Shape copy()`.

- Only an `open` method, an `abstract` method, or an interface method not declared `final` can be overridden. Declaring a method with the name and signature of a visible final method of a superclass is rejected.
- The overriding method carries `@Override`, as [chapter 8](08-annotations.md) requires.
- The overriding method is itself final unless it is declared `open`.
- Its audience follows [chapter 3](03-visibility.md).
- Its receiver has the same qualifiers as the receiver it overrides. The exception is an override of `equals`, `hashCode` or `toString`, whose receiver has none.
- It carries `@Discardable` exactly when the method it overrides does.

A static method overrides nothing. A static method of a class meets a static requirement of an interface, as chapter 2 describes, when it has that name and signature, and it then carries `@Override` too.

Inside an instance method, `super.name(args)` calls the superclass's method without dispatch.

## 4.3 Calls

A call is written `receiver.name(args)`, `Type.name(args)`, `super.name(args)` or `name(args)`. In the last form the method belongs to the enclosing class or to a class it inherits from. The receiver is `this` for an instance method and the class for a static method. Calling an instance method that way from a static method is rejected.

A call evaluates the receiver, then the arguments from left to right, and then runs the method with each parameter bound to the value of its argument. Assigning a parameter does not change a variable of the caller.

[Section 4.4](#44-overloading) chooses the method at compile time from static types. When the chosen method is `open`, `abstract` or an interface method, the receiver's class at run time selects the body, as [chapter 2](02-objects.md) describes.

The static type of the receiver must be a subtype of the method's receiver type. A receiver whose type is `@Nullable` therefore accepts only the methods that declare a `@Nullable` receiver, until it is narrowed.

A call is an expression, and its type is the method's result type. A call whose result is `Unit` may stand alone as a statement. So may a call of a method marked `@Discardable`, such as an `append` that returns its receiver for chaining. A call of any other method must be used: as an operand, an argument, an initializer, or the right side of an assignment. A statement that would discard its result is rejected.

## 4.4 Overloading

Two methods of one class may share a name when their parameter lists differ in length or in the type of a parameter. Types that differ only in qualifiers do not count as different, and neither do result types.

A call chooses among the visible methods of that name that the receiver's static type declares or inherits.

1. A method is applicable when the call has as many arguments as the method has parameters and each argument is assignable to its parameter under [section 4.5](#45-assignability), after the inference of [chapter 7](07-generics.md) for a generic method. [Chapter 6](06-numbers.md) says when a numeric literal is applicable, and [section 4.8](#48-lambdas-and-method-references) says when a lambda is.
2. One applicable method is more specific than another when each of its parameter types is a subtype of the other's corresponding parameter type.
3. If one applicable method is more specific than every other, it is chosen. Otherwise the call is rejected, and the program writes a cast or explicit type arguments to choose. The one exception is a call with a numeric literal argument, where [chapter 6](06-numbers.md#63-numeric-literals) chooses the method of the literal's default class.

A method with a fixed number of parameters is more specific than a varargs method.

Resolution first considers only the methods that are applicable without an implicit conversion. If there is none, it considers the methods that are applicable when an argument may be converted, as [section 4.5](#45-assignability) allows. In that second round, a class counts as more specific than a class it converts to.

## 4.5 Assignability

A value of static type `S` is assignable to a location of type `T` when `S` is a subtype of `T`. Subtyping is the reflexive, transitive closure of `extends` and `implements`, together with the rules for generic types in [chapter 7](07-generics.md) and for qualifiers in [chapter 8](08-annotations.md), of which `@Nullable` in [chapter 5](05-null-and-unit.md) is one.

**Implicit conversion.** A value of class `S` is also assignable to a location of type `T` or `@Nullable T` when `T` declares an implicit conversion from `S`. That is a static method marked `@Implicit`, as [chapter 8](08-annotations.md) defines, and the compiler inserts a call of it. The numeric classes declare the implicit conversions that [chapter 6](06-numbers.md#67-conversion) lists, and a program's value classes may declare their own.

At most one conversion is applied to a value. Nothing else is converted: not the receiver of a call, not a `@Nullable` value, and not a type argument, so an `Int32[]` is never an `Int[]`.

No other method is ever inserted to make a value fit: not to produce a `String`, not to narrow or round a number, not to copy into a superclass. A numeric literal takes its class from where it stands, as chapter 6 defines.

**Expected type.** An expression has an expected type when it is the initializer of a declaration with a written type, the right side of an assignment, an argument for a parameter of a chosen method, the operand of `return`, the expression body of a lambda, or an element of an array creation. The expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression. Numeric literals, lambdas, and the inference of chapter 7 use it.

**Disjoint types.** Two types are disjoint when no value can belong to both. Leaving aside every qualifier but `@Nullable`:

- two class types are disjoint unless one is a subtype of the other;
- a class type and an interface type are disjoint when the class is final and is not a subtype of the interface;
- two `@Nullable` types are never disjoint, because both contain `null`;
- two interface types are never disjoint, and a type parameter is disjoint from nothing.

`==`, casts and `instanceof` are rejected between disjoint types, as the following sections say.

## 4.6 Operators

An operator is a call of a method on its left operand, or on its only operand. The expression is legal exactly when that call is legal, and it has the call's result type. Any class may declare these methods.

| Spelling | Call |
| --- | --- |
| `a + b` | `a.plus(b)` |
| `a - b` | `a.minus(b)` |
| `a * b` | `a.times(b)` |
| `a / b` | `a.div(b)` |
| `a % b` | `a.mod(b)` |
| `-a` | `a.negate()` |
| `a < b` | `a.lessThan(b)` |
| `a <= b` | `a.atMost(b)` |
| `a > b` | `a.greaterThan(b)` |
| `a >= b` | `a.atLeast(b)` |
| `a == b` | `a.equals(b)` |
| `a & b` | `a.and(b)` |
| `a \| b` | `a.or(b)` |
| `a ^ b` | `a.xor(b)` |
| `~a` | `a.complement()` |
| `a << b` | `a.shiftLeft(b)` |
| `a >> b` | `a.shiftRight(b)` |
| `!a` | `a.not()` |
| `a[i]` | `a.get(i)` |
| `a[i] = e` | `a.set(i, e)` |

There is no unary `+`, no `>>>`, and no power operator.

When the two operands of a binary operator other than a shift have different classes, and exactly one of them converts implicitly to the class of the other, it is converted first. With an `Int32 i` and an `Int n`, `i + n` is an `Int` addition.

**Equality.** `a == b` is `a.equals(b)`, for a reference class and a value class alike. `a != b` is `!(a == b)`. Two cases differ:

- When one operand is the literal `null`, the expression is the null test of [chapter 5](05-null-and-unit.md), not a call.
- When the static types of the operands are disjoint, the expression is rejected, because it would be `false` for every pair of values. Two classes are not rejected when one converts implicitly to the other.

Reference identity is `identical`, a method of `Object`.

**Compound assignment.** `a += b` is `a = a + b`, except that the location `a` is evaluated once. The same holds for `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=` and `>>=`.

**Increment.** `a++` and `++a` are `a = a + 1`, and `a--` and `--a` are `a = a - 1`, with the location evaluated once. The literal takes its class from `a` by the rule of chapter 6. The prefix form yields the updated value, and the postfix form yields the value from before the update. If the method raises, the location is unchanged.

## 4.7 Other expressions

**`&&` and `||`.** Both operands have type `Boolean`. `a && b` evaluates `a`. If it is `false`, the result is `false` and `b` is not evaluated. Otherwise the result is the value of `b`. `a || b` evaluates `a`. If it is `true`, the result is `true` and `b` is not evaluated. Otherwise the result is the value of `b`. These two are not methods. `&` and `|` on `Boolean` are the methods `and` and `or`, and they evaluate both operands.

**`?:`.** In `c ? a : b`, `c` has type `Boolean` and is evaluated once. Then exactly one of `a` and `b` is evaluated, and its value is the result. When the expression has an expected type, both arms are checked against it and it is the type of the expression. Otherwise the type is the type of one arm, and the other arm must be assignable to it.

**`??`.** `a ?? b` is the value of `a` unless that is `null`, and the value of `b` otherwise. [Chapter 5](05-null-and-unit.md#54-null-tests-and-narrowing) defines it.

**Casts.** `(T) e` evaluates `e` and tests the class of its value. The test succeeds when that class is a subtype of `T`, including the type arguments of a generic class. A successful cast yields the same value with static type `T`. A failed cast raises `ClassCastException`. [Chapter 5](05-null-and-unit.md) says what a cast does with `null`, and [chapter 8](08-annotations.md) says which qualifiers `T` may carry.

A cast never converts a value. It does not change the class of a number. A cast is rejected when `T` and the static type of `e` are disjoint, because the test could not succeed.

**`instanceof`.** `e instanceof T` is `true` when the value of `e` is not `null` and its class is a subtype of `T`. It is rejected when `T` and the static type of `e` are disjoint. Where it is known `true`, it narrows a local, as chapter 5 describes.

**`new`.** `new C(args)` creates an instance of the class `C`, by the construction rules of [chapter 9](09-execution.md). The constructor is chosen as [section 4.4](#44-overloading) chooses a method. A generic class is written with its type arguments: `new List<String>()`. Array creation is in [chapter 6](06-numbers.md).

**Assignment.** `x = e` evaluates `e` and stores its value in the local or parameter `x`. `r.f = e` evaluates `r`, then `e`, and stores into the field. `a[i] = e` evaluates `a`, `i` and `e` in that order and calls `a.set(i, e)`. The value of `e` must be assignable to the location. An assignment is an expression: its value is the value stored, and its type is the type of the location. It may stand alone as a statement.

## 4.8 Lambdas and method references

A functional interface is an interface with exactly one abstract method and no static requirement. The prelude declares the general ones:

```java
public interface Function0<out R> { R invoke(); }
public interface Function1<in A, out R> { R invoke(A a); }
public interface Function2<in A, in B, out R> { R invoke(A a, B b); }
```

A lambda is written `(parameters) -> expression` or `(parameters) -> block`. It has no type of its own. It stands where the expected type is a functional interface, and it becomes an instance of that interface whose one method is the lambda. `var f = () -> 1;` is rejected, because nothing says which interface is meant.

- The lambda has as many parameters as the interface's method. A parameter's type may be omitted, and is then the method's parameter type. A written type must be that type.
- The value of an expression body is the result, and must be assignable to the method's result type. When that result type is `Unit`, an expression body may instead be any expression that could stand as a statement, and its value is discarded. A block body returns with `return`, under the rules for a method with that result. `return` inside a lambda returns from the lambda. `break` and `continue` in a lambda do not reach a statement outside it.
- Inside a lambda, `this` is the `this` of the enclosing method.

**Capture.** A lambda may use a local or a parameter of an enclosing method or lambda only if that variable is never assigned after it is initialized. Using any other local in a lambda is rejected. The lambda holds the values those variables had when the lambda expression was evaluated. A lambda may read and assign fields through `this`.

**The instance.** The instance belongs to a final value class that the lambda expression declares implicitly. Its fields are the captured values and `this`. A lambda therefore has no identity, and two evaluations of one lambda expression that capture equal values are equal. Its `toString` is chosen by the implementation.

**In overload resolution.** A lambda argument is applicable to a parameter whose type is a functional interface whose method has that many parameters, with the same types wherever the lambda writes them. The body of the lambda plays no part in choosing the method. Chapter 7 says how a lambda takes part in inference.

**Method references.** A method reference is a short form of a lambda, and stands in the same places.

| Reference | Lambda |
| --- | --- |
| `Type::name`, for a static method | `(args) -> Type.name(args)` |
| `Type::name`, for an instance method | `(receiver, args) -> receiver.name(args)` |
| `expr::name` | `(args) -> v.name(args)`, where `v` is the value of `expr` when the reference is evaluated |
| `Type::new` | `(args) -> new Type(args)` |

The parameter types of the interface's method choose among overloads.

## 4.9 Statements

**Local declarations.** `T name = e;` declares a local of type `T` and initializes it. `T name;` declares a local with no value, and [chapter 12](12-flow.md) requires an assignment before any use. `var name = e;` gives the local the static type of `e`, with its qualifiers. `var` needs an initializer, and `var name = null;` is rejected, because the type would be `Null`. A local is assignable unless it is declared `final`. [Chapter 1](01-source.md) gives its scope.

**Expression statements.** An assignment, an increment, a decrement, a call whose result is `Unit`, and a call of a `@Discardable` method may each stand as a statement. Every other expression is rejected as a statement, because its value would be discarded.

**`if` and `while`.** `if (c) s else t` evaluates `c`, which has type `Boolean`, and runs `s` when it is `true` and `t` otherwise. Without `else`, nothing runs when `c` is `false`. `while (c) s` evaluates `c` before each iteration, runs `s` while it is `true`, and completes when it is `false`.

**`for`.** `for (init; c; update) s` runs `init` once. Before each iteration it evaluates `c`, and completes when `c` is `false`. A missing `c` is `true`. Then it runs `s`, and then `update`. `update` also runs after a `continue` of this loop. It does not run after `break`, `return` or a raised exception. A local declared in `init` is in scope in `c`, `s` and `update`.

**`for` over a collection.** `for (T x : e) s` requires the static type of `e` to be a subtype of `Iterable<U>`, where `U` is assignable to `T`. It evaluates `e` and calls `iterator()` once. Then, while `hasNext()` is `true`, it binds `x` to `next()` and runs `s`. `x` is not assignable. `var` may be written for `T`.

```java
public interface Iterator<out T> { Boolean hasNext(); T next(); }
public interface Iterable<out T> { Iterator<T> iterator(); }
```

**`switch`.** `switch (selector) { arms }` evaluates the selector once and runs the first arm that matches. No arm continues into the next. An arm is one of three kinds:

- `case c1, c2 -> body` matches when the selector equals one of the constants under `equals`. The constants are constant expressions of the selector's type. The selector's type is an integer class, `Char`, `Boolean`, `String` or an enum. Writing one constant twice in a switch is rejected.
- `case T name -> body` matches when the selector is an instance of `T`, as `instanceof` tests. In the body, `name` is a local of type `T` that holds the selector and is not assignable. `T` must not be disjoint from the selector's type. An arm that cannot match because an earlier arm names `T` or a supertype of `T` is rejected.
- `default -> body` matches when no other arm does. A switch has at most one, written last.

One switch uses constant arms or type arms, not both. The selector's type is not `@Nullable`.

In a switch statement, each body is a statement. If no arm matches, the switch completes normally. In a switch expression, each body is an expression or a `throw` statement, and the switch must be exhaustive. Its type follows the rule for `?:`, applied across all arms.

A switch is exhaustive when it has a `default` arm, when its constants are both `true` and `false`, when its constants are every constant of an enum, or when its type arms cover the selector's type. Type arms cover a type `S` when one of them names `S` or a supertype of `S`. They also cover `S` when `S` is a sealed interface or a sealed abstract class and they cover every type in its `permits` clause.

```java
static Rational eval(Expr expr) {
    return switch (expr) {
        case Num n -> n.value;
        case Neg n -> -eval(n.operand);
        case Binary b -> apply(b.op, eval(b.left), eval(b.right));
    };
}
```

**Labels, `break` and `continue`.** `break` leaves the innermost enclosing `while`, `for` or switch statement. `continue` starts the next iteration of the innermost enclosing loop. A labeled statement is `label: statement`. `break label` completes that statement, and `continue label` continues it, which requires it to be a loop. A label that does not enclose the `break` or `continue` is rejected.

**`return` and `throw`.** `return e;` returns the value of `e`, which must be assignable to the method's result type. `return;` returns from a method whose result is `Unit`. `throw e;` raises the value of `e`, whose static type is `Throwable` or a subclass of it. [Chapter 9](09-execution.md) defines exceptions, `try` and `using`.

**`assert`.** `assert c;` evaluates `c`, which has type `Boolean`, and raises `AssertionException` when it is `false`. `assert c : detail;` also evaluates `detail` in that case and gives `detail.toString()` to the exception as its message. Assertions are always checked. No mode removes them.

## 4.10 Varargs

A parameter `T... name` is a parameter of type `T[]` and is the last parameter. When a call chooses a varargs method, a single trailing argument that is assignable to `T[]` is passed as that array. Otherwise the trailing arguments are evaluated from left to right and stored in a new `T[]`. No trailing arguments give a zero-length array.

## 4.11 Foreign methods

A method declared `foreign` is also declared `static` and has no body. Its body is a C function. Each parameter type and the result type is a machine type:

| Cleat type | C type |
| --- | --- |
| `Boolean` | `_Bool` |
| `Char` | `uint32_t` |
| `Int8`, `Int16`, `Int32`, `Int` | `int8_t`, `int16_t`, `int32_t`, `int64_t` |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t` |
| `Float32`, `Float64` | `float`, `double` |
| `Pointer` | `void *` |
| A value class whose fields are all machine types | A struct of those fields in source order, with the platform's C layout, passed by value |

A result written `void` is the C result `void`.

A parameter, but not a result, may also be `T[]` where `T` is a machine type. The C function receives a pointer to the first element, with the elements laid out consecutively as C lays out `T`. It may read and write the array's elements through that pointer until the call returns, and not afterwards. Its writes are in the array when the call returns. The length is not passed. A program that needs it passes `a.length()` as another argument. For an empty array the pointer must not be used.

Every other type is rejected on a foreign method. That includes `String`, `Rational`, a reference class, an interface, a type parameter, and a `@Nullable` type. A lambda cannot be passed, so a C function cannot call back into Cleat.

`Pointer` is a final value class that holds an address. `equals` compares addresses, and `Pointer.zero()` is the null address. Cleat does not read or write through a `Pointer`. A program passes it back to foreign methods. Memory that C code allocates is not collected.

The call uses the platform's C calling convention for those types. The C symbol is the method's name, unless the method carries `@Symbol("name")`, which names another. The implementation is told which libraries supply the symbols when the program is linked.

C code is outside this specification. A C function that writes outside the memory it was given, keeps a pointer after the call, or unwinds through the call has no defined behavior.
