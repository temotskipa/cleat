# Foundations for the revision

The decision record for the revision, last updated 2026-10-08. The [README](../README.md) lists which chapters of `spec/` have been revised, and [goal.md](goal.md) states what finishing the revision means.

The nine programs in [programs/](programs/) are written in the language these recommendations produce. Each of them builds and runs, and `compiler/tests/programs.rs` checks what it prints. Claims about other languages are from memory and should be checked before any of them reaches the spec.

## Settled in conversation

- **Purpose.** Java as it would be designed today with no legacy.
- **Consistency.** Every value is an instance of a class. Built-in types follow the same typing, lookup and generics rules as user types. Only their method bodies may be intrinsic.
- **Annotations as modifiers.** An annotation can change what the compiler checks or produces for the thing it marks, and that meaning is defined in the language, not by an outside tool. `@Nullable` is the first case, so nullability is spelled `@Nullable T`.
- **Control flow is syntax.** `if`, `while`, `for`, `try`, `throw`, `new` and casts are statements and expressions, not message sends. Operators remain methods.
- **v1 compiles ahead of time.** `Loader` comes later.

## Decisions

"Decided" means you chose it. "Assumed" means the rewrite proceeds on my recommendation until you say otherwise.

| # | Question | Recommendation | Main alternative | Status |
| --- | --- | --- | --- | --- |
| 1 | Null model | `@Nullable T` is `T` plus `null`; `Null` sits outside `Object`; `@Nullable` is an ordinary qualifier declared in the prelude | Keep `@Nullable` as a special case the language owns | Spelling decided; model assumed |
| 2 | Numeric classes | Unrelated final value classes that share an interface `Numeric<T>`; no `Number` superclass | Keep the tower and add special inheritance rules for it | Assumed |
| 3 | Everyday integer | `Int`, 64-bit, for literals, lengths and indexes | `Int32`, as in Java | Decided |
| 4 | Literals | Typed by context; with no context, `Int` and exact `Rational` | Decimal literals default to `Float64` | Decided |
| 5 | Integer division | No `/` on integer types; `floorDiv` and `%` instead | Truncating `/`, as in Java | Decided |
| 6 | Memory model | No torn values, safe initialization, ordering from locks and atomics; no sequential consistency | Keep sequential consistency | Decided |
| 7 | Generics | Drop use-site wildcards; inference picks the most specific type | Keep wildcards and capture | Inference assumed. Wildcards decided: kept, beside `in` and `out` |
| 8 | Lambdas | Instances of single-method interfaces; `inline` blocks later | Keep `Block` and `inline` in v1 | Decided |
| 9 | v1 scope | The scope table below | | Assumed; fields decided |
| 10 | Annotation powers | Type qualifiers and declaration checks; drop `Expander` and statement rewriting | Keep generators and wrappers for a later version | Decided. Also decided: a program or a library may declare its own annotations, as in Java, and read them at run time. The reading API is assumed |
| 11 | Sharing fields | One superclass; interfaces have no fields | Interfaces with fields, or full multiple inheritance | Decided |
| 12 | Constructor order | A constructor assigns its fields, then calls `super` | `super` first, with `this` barred in constructors of extensible classes | Decided |
| 13 | `@Override` | Required on every overriding or implementing method | Optional, or no marker | Decided |
| 14 | Result type of an override | May be a subtype of the overridden result | Exact match only | Decided: subtype allowed |
| 15 | `%` on integers | Floored, pairing with `floorDiv` | Truncated, as in Java | Decided |
| 16 | Unused results | Must be used unless the method is `@Discardable` | Always must use; `@MustUse` opt-in; free to discard | Decided |
| 17 | Implicit conversions | None | Lossless conversions among the numeric classes | Decided: lossless conversions are implicit, declared with `@Implicit` so a program's value classes may declare them too. A conversion that can lose information is called by name and raises by default |
| 18 | Null operators | Add `??` | Also `?.`; or narrowing only | Decided: `??` |
| 19 | Program end | The program ends when `main` returns | Wait for every thread | Decided |

The name is still open. It can wait.

## Null (decision 1)

**Recommendation.**

- `@Nullable T` contains every value of `T`, and `null`. `T` is a subtype of `@Nullable T`. Writing the annotation twice changes nothing.
- `null` is the one instance of `Null`. `Null` is not a subclass of `Object`. `@Nullable Object` is the top type.
- A method may be called on a `@Nullable` receiver only when the method declares a `@Nullable` receiver. In the prelude those are `equals`, `hashCode`, `toString` and the final `identical`. `equals` takes `@Nullable Object`, which is what lets `x == null` typecheck.
- Narrowing works as the spec has it now: `== null`, `!= null` and `instanceof` narrow a local or parameter that is not reassigned in the region. A cast `(T) e` from `@Nullable T` is the checked form and raises on `null`.
- Nullability is part of a reified type argument. `List<String>` and `List<@Nullable String>` are different classes.
- `@Nullable` is declared in the prelude with the same facility a user qualifier gets. The next section says what that takes.

**Why.** This removes both contradictions in the current text. Today `Null` extends `Object`, so `Null n = null; Object o = n;` typechecks and puts `null` in a non-null type. And `x == null` passes `null` to `equals(Object)`, whose parameter is non-null.

**Grammar point.** `@Nullable String[]` is an array of nullable strings, as in Java. The grammar also needs a way to write a nullable array of strings. Java writes `String @Nullable []`.

**Known limit.** `Map.get` returns `@Nullable V`, so it cannot tell "absent" from "present and null" when `V` is itself nullable. Kotlin and C# accept the same limit.

## Annotations as modifiers (decision 10)

**Four kinds of modifier.**

| Kind | Examples | What the compiler does |
| --- | --- | --- |
| Type qualifier | `@Nullable`, `@Positive`, `@Tainted` | Checks a declared subtype order; the annotation is data |
| Declaration check | `@Override`, `@Deprecated` | Validates a declared constraint; the annotation is data |
| Generator | `@Data`, `@Builder` | Runs annotation code that adds members |
| Wrapper | `@Transactional`, `@Cached` | Runs annotation code that wraps a method body |

**Recommendation.** The annotation system is the first two kinds. Drop `Expander` and `RewritesStatement`.

- **Generators are not needed.** Lombok patches gaps Java had, and Cleat does not have them: a value class already supplies `equals`, `hashCode`, `toString` and a constructor, fields need no accessors, and `var` exists. The one remaining use is code derived from a class's shape, such as serialization. A library now does that at run time, through the mirrors of the fields it annotates.
- **Wrappers are sugar for passing a lambda.** `lock.exclusive(() -> ...)` already does what a `@Synchronized` annotation would.

Dropping both means the compiler never has to run Cleat code while it compiles, which was the most expensive part of chapter 8 to build.

**The test.** `@Nullable` must be declarable in the prelude using only what a user qualifier may use. For that, a qualifier needs three powers:

1. **A place in the subtype order.** Every `T` is a subtype of `@Nullable T`.
2. **A receiver rule.** A method may be called on a qualified receiver only if the method declares that it accepts one. This replaces the fixed list of seven methods in chapter 5 with a rule every qualifier can use.
3. **Narrowing by a test.** A method can declare that its result narrows the receiver or an argument. A user's `@Positive` is then narrowed by `isPositive()` the way `@Nullable` is narrowed by a null test.

The Checker Framework offers all three for Java, from outside the compiler. This is the workaround the goal replaces.

**What stays special.** `null` the value is built in. Two facts about `@Nullable` therefore go on the list of admitted exceptions: it is the one qualifier that adds a value to a type, and `== null` narrows without a declared test method.

**As written in chapter 8.** The three powers are `@Refines` and `@Widens` for the subtype order, a receiver parameter named `this` for the receiver rule, and `@Narrows` for tests. Every qualifier in a type argument is reified, not only `@Nullable`, so a cast cannot smuggle a plain `List<Int>` into a `List<@Positive Int>`.

**Annotations a program declares (decided).** A program or a library declares its own annotations, as in Java: with elements, with `@Target`, and read at run time. An annotation is a final value class whose fields are its elements. Reading needs some reflection, which decision 9 had left out, so chapter 8 adds the smallest API that makes a declared annotation useful. These parts of it are assumed:

- **Reflection is reached through annotations.** `Class` returns mirrors only for the fields and methods that carry the annotation asked for. A member with no annotation has no mirror, so the cost in the compiled program follows what the program annotates, and a compiler can still drop unused code. Java's reflection over every member is what makes its ahead-of-time compilers need configuration files.
- **A mirror uses public members only.** `get`, `set` and `invoke` raise `IllegalAccessException` on any other member, so reflection gives no access that chapter 3 denies. The alternative is that writing the annotation on a member is the author's consent to its readers, which would let `@Test void adds()` run without `public`. That can be allowed later without breaking a program.
- **Mirrors do not describe types or construct objects.** That is enough for a test runner, a validator and a serializer. A deserializer or an injection container needs field types and constructors, which is a larger design.

**What the earlier chapter 8 lacked.**

- The grammar has no place for an annotation on a class, a method or a parameter, including the `@Target` that every annotation declaration must carry.
- Qualifiers get three sentences, and `Qualifier` declares no methods. None of the three powers above is defined.
- `Expander` and `RewritesStatement` could not do their job as written: an expander cannot see the members of the class it expands, and no grammar position can carry a statement-rewriting annotation.

## Numbers (decisions 2 to 5)

### Structure (decision 2)

**Recommendation.** The numeric classes are unrelated final value classes:

```
Int8  Int16  Int32  Int        signed; Int is 64-bit
UInt8 UInt16 UInt32 UInt64     unsigned
Float32 Float64                IEEE 754
Rational                       exact, unbounded
```

What they share is an interface, not a superclass:

```java
public interface Numeric<T> {
    T plus(T other);
    T minus(T other);
    T times(T other);
    T negate();
    static T zero();
    static T one();
}
```

`Int` implements `Numeric<Int>`, `Float64` implements `Numeric<Float64>`, and a user's `Money` or `Complex` can implement it on equal terms. C# does generic arithmetic this way.

**Why.** The current tower contradicts the language's own rules in two places.

- `Int32` extends `Number`, and `Number` declares `plus(Number)`. By ordinary inheritance `Int32` has that method, so `i + f` is accepted, though chapter 6 says it is rejected.
- Generic numeric code cannot be typed. Under `T extends Number`, `a + b` on two values of type `T` resolves to `Number.plus(Number)` and returns `Number`, not `T`. `Vector<T>.plus` as chapter 6 writes it does not typecheck.

Removing the superclass also removes the need for a class tag and a width tag inside every `Number`, and the rule that an `Int8` one equals an `Int32` one.

**What stays the same.** Integer overflow raises `ArithmeticException`, and wrapping is by name (`wrappingPlus`). The only implicit conversions are the lossless ones of decision 17. `T.from(x)` converts exactly or raises, and rounding is by name.

**One new rule.** `a == b` is rejected when the two static types can never be equal, such as `Int` and `Float64`. Kotlin does the same.

### The everyday integer (decision 3)

**Recommendation.** `Int` is 64-bit. Integer literals default to it, and array lengths, indexes and `String` positions use it. There is no separate `Int64` name.

**Why.** Java's 32-bit `int` and its two-billion-element array limit are legacy. Swift, Go and Dart all use a 64-bit everyday integer on 64-bit machines. With one type for literals and indexes, `for (var i = 0; i < xs.length(); i++) xs[i]` typechecks, which it does not today.

**Cost.** The compiler's first milestone is built around `Int32`. The alternative keeps `Int32` as the everyday type.

### Literals (decision 4)

**Recommendation.** A numeric literal takes its type from, in order:

1. the expected type at its position: a declared type, a parameter type, a return type;
2. the other operand, when it is one operand of a binary operator;
3. a default: `Int` for an integer literal, `Rational` for a decimal literal. Chapter 6 states this for a whole literal expression: with no context, every literal in `1.0 / 3` is a `Rational`.

```java
Float64 dt = 0.001;        // Float64, from the declared type
total += 0.5 * mass;       // Float64, from the other operand
var price = 19.99;         // Rational: exactly 1999/100
var i = 0;                 // Int
UInt8 mask = 0xFF;         // UInt8; 0x100 is rejected
```

An integer literal that does not fit its type is rejected. A decimal literal in a float position is rounded to the nearest float. That is the one place rounding is implicit, and it happens only where the programmer wrote a float type.

**Why.** Today `Float64 x = 0.1` is rejected and `var i = 0` is an exact rational that cannot index an array. This rule keeps the property that `0.1` with no stated type is exactly one tenth, and removes both papercuts.

**Alternative.** Default decimal literals to `Float64`, as most languages do. Then `0.1 + 0.2 == 0.3` is false unless a `Rational` type is written.

### Integer division (decision 5)

**Recommendation.** Integer types have no `/`. They have `floorDiv`, and `%` for the remainder. `/` exists on `Float32`, `Float64` and `Rational`, where it is that type's real division.

**Why.** Today `Int32` division raises at run time unless it is exact, so `(lo + hi) / 2` fails only for odd sums. Removing the method turns that into a compile error with an obvious fix. It also prevents `sum / count` from silently truncating. Dart and Python 3 also keep integer division apart from `/`, though there `/` on two integers gives a float.

**Alternative.** Truncating `/` as in Java, Kotlin, Swift and Rust. It is the most familiar choice and the least strict.

### `Rational`

`Rational` is the single exact class. It needs the operations the current `Number` lacks: `floor`, `ceil`, `truncate`, `round(places)`, `isInteger`, `numerator`, `denominator`, `floorDiv` and `%`. How it stores its value is not observable and is not part of the spec.

`Vector`, `Complex`, `Matrix` and the machine shapes leave the language spec. They become library classes over `Numeric<T>`, later.

## Memory model (decision 6)

**Recommendation.** Four guarantees, and no more:

1. **No torn or invented values.** A read returns a value that some write stored in that location.
2. **Safe initialization.** A thread that obtains a reference to an object never sees a field before the constructor assigned it.
3. **Ordering from synchronization.** Releasing a lock, an atomic operation, starting a thread and joining one each order what came before them ahead of what comes after, as in Java's model.
4. **Otherwise, stale reads are possible.** Two threads that share a location with no synchronization may see older writes. Nothing worse happens.

**Why not sequential consistency.** It needs a fence or locked instruction around most shared stores. A published experiment that made every Java field volatile measured roughly 30% slowdown on x86, as I recall. Guarantee 2 keeps the part of the current rule that matters for safety.

**Multi-word values.** Guarantee 1 has a cost for a value wider than a machine word, such as `Rational`, `@Nullable Int` or a user `Vec3`, when it sits in a mutable field or an array element. The implementation may box it there or copy it atomically. Locals and `final` fields can be flat at no such cost. The present compiler boxes every value class. A later opt-in could let a plain-data value class accept tearing in exchange for flat arrays, which is where Java's Valhalla project ended up.

**Cancellation.** A cancelled task should raise `CancellationException` only at blocking operations and at explicit checks, not at arbitrary safepoints. An exception that can appear between any two statements breaks invariants in the way `Thread.stop` did.

## Generics and lambdas (decisions 7 and 8)

**Wildcards (decided: kept).** Declaration-site `in` and `out` cover the types that vary by nature: `Iterable<out T>`, `Comparator<in T>`. Wildcards cover an invariant type used in one direction, such as `List<? extends Shape>`, and "any instantiation", `List<?>`. How the two fit together is assumed:

- On an `out` parameter `G<? extends U>` is the same type as `G<U>`, and `? super` is rejected. The `in` case mirrors it. `?` is legal everywhere.
- Capture is kept small. The type of an expression never mentions an unknown type: a result is read as the wildcard's bound. The unknown gets a name only when a wildcard-typed argument is passed to a generic method.
- Because type arguments exist at run time, `o instanceof List<? extends Shape>` is a real test, and a generic method called with a `List<?>` receives the object's actual type argument.

**Inference.** The current rule asks for "exactly one solution", which by its own reasoning rejects `id("s")`, since `String` and `Object` both fit. Replace it with:

- An argument in an invariant position, such as `List<T>`, fixes `T`.
- Otherwise each argument gives a lower bound, and `T` is the one bound that is a supertype of all the others.
- If there is no such bound, the expected type decides. If there is none, the call is rejected and the program writes the argument.
- A lambda argument is checked after the others and contributes its result type.

**Static members through a type parameter.** `T.zero()` is legal when `T`'s bound declares the static method. `Numeric<T>` depends on this, so the interface chapter has to define static interface methods properly.

**Lambdas.** A lambda is an instance of an interface with one abstract method, chosen by the expected type. The prelude declares a few such interfaces (`Function0`, `Function1`, `Function2`, `Comparator`) in ordinary source. This removes `Block`, a class that is overloaded by its number of type arguments, which no user class may be. `return` inside a lambda returns from the lambda.

`inline` blocks were needed when `if` and `while` were defined as method calls. They are now an independent feature that buys user-written control structures. I would add them after v1.

## Scope of v1 (decision 9)

| Area | v1 |
| --- | --- |
| Packages, imports, files | Keep |
| Classes, interfaces, enums; final and private by default | Keep |
| Audiences, including `only` lists | Keep; fix the grammar so the clause has one position |
| `private(this)` | Drop |
| Value classes | Keep; always final, no value-class inheritance |
| Operators as methods, overloading, varargs, method references | Keep |
| Fields | Decided: plain fields; drop the generated getter and setter description. Split read and write audiences, or properties, are the options if the convention proves heavy |
| `void` and `Unit` | One concept: `void` is how a `Unit` result is written |
| Control flow | Syntax; sibling `catch` clauses behave as in Java |
| `switch` | Keep; add type patterns, exhaustive over a sealed type |
| Exceptions, `try`, `using`, `assert`, labels | Keep; `Throwable` gains a message |
| Reified generics with `in` and `out` | Keep |
| Use-site wildcards | Keep, beside `in` and `out` (decision 7) |
| Lambdas | Single-method interfaces |
| `inline` blocks with non-local `return` | Later |
| Threads, `Lock`, atomics, `Scope` | Keep; add a way to wait for a condition |
| `foreign` methods, machine types, `Pointer` | Keep; say how a method names its C symbol |
| `Pin`, `Span`, `ForeignBuffer` | Later; v1 pins an array argument for the length of the call |
| Annotation declarations, targets, type qualifiers, declaration checks | Keep, and rewrite as a core chapter (decision 10) |
| Expanders and statement rewriting | Drop (decision 10) |
| Reflection | `getClass()`, class names, and mirrors of annotated fields and methods (decision 10) |
| `Vector`, `Complex`, `Matrix`, machine shapes | Later, as a library |
| `Loader` and the JIT | Later |
| Normative storage details | Drop: `Number`'s widths, which methods are omitted |

Three small rules change with this table. `value` becomes a keyword only before `class`, so it can name a field or parameter, as the spec's own signatures already use it. An interface may be empty, because a sealed interface with no methods is the natural root of a sum type. A lambda needs an expected type.

## The consistency test

The prelude is written as `.cleat` files. [programs/List.cleat](programs/List.cleat) is the first one, and it needs nothing a user class could not have. Where a prelude type needs something special, either the language gains the feature or the case goes on this list. The list so far:

1. Literals construct prelude types, and numeric literals take their type from context.
2. `null` is built in. `@Nullable` is the one qualifier that adds a value, and `== null` narrows without a declared test.
3. Array storage is built in, including the default elements of `new T[n]`.
4. Prelude method bodies may be intrinsic: arithmetic, array access, threads, I/O.
5. `Object`, `Null` and `Enum` are fixed roots.
6. The prelude's annotations have rules built into the compiler: `@Refines`, `@Widens`, `@Target`, `@Narrows`, `@Inherited`, `@Override`, `@Discardable`, `@Implicit`, `@Deprecated`, `@Intrinsic` and `@Symbol`.
7. The mirrors of chapter 8 have intrinsic bodies, and `Annotation` is implemented by annotations alone.

Implicit numeric conversion is not on the list. The numeric classes declare their conversions with `@Implicit`, and a program's value classes may do the same.

Syntax that maps to a named method or interface is not on the list, because user types take part equally: operators, `for` over `Iterable`, `using` with `close`, and lambdas.

## What the programs needed

**Prelude the spec does not have.**

- Output and input: `Console.println`, `Console.error`, `File.readText`, `Process.exit`.
- Collections: `List<T>`, `Map<K, V>` with `Entry<K, V>`, `Comparator<T>`.
- Text: `StringBuilder`; `String` iteration, `split`, `trim`, `toLowerCase`; `Char.isLetter`, `isDigit`, `isWhitespace`; conversion between `String` and UTF-8 bytes.
- Numbers: `Int.parse` and `Rational.parse` returning a `@Nullable` result; `Rational.round` and `toDecimal`; `Float64.sqrt` and `toFixed`.
- Exceptions: a message on `Throwable`; `IOException`.

**Language additions.**

- `switch` over types, exhaustive for a sealed interface ([Calc.cleat](programs/Calc.cleat)).
- Static interface methods called through a type parameter ([Ledger.cleat](programs/Ledger.cleat)).
- An array of non-null references with a run-time length. `new String[n]` has no default element, so the prelude needs a builder such as `Array.build(n, (i) -> ...)`.

**What writing them showed.**

- `"count: " + n` works and `n + " items"` does not, because `+` is a method of the left operand. String interpolation is the obvious later fix.
- With implicit lossless conversion (decision 17), [Ledger.cleat](programs/Ledger.cleat) multiplies a `Rational` price by an `Int` count directly. Before that decision it needed `Rational.from(quantity)`.
- `??` (decision 18) replaced a five-line null test in two programs with `(counts[word] ?? 0) + 1`.
- Generic containers narrow `@Nullable T` to `T` with a cast, `(T) slots[i]`. It is correct even when `T` is nullable, because the type argument exists at run time.
- Three of the first six programs name a field or local `value`.
- [Shares.cleat](programs/Shares.cleat) declares a qualifier and a tag. Obtaining a `@Positive Int` always takes a call to its test, even for a literal `5`.

## Calls made while rewriting

Every item in this section is Assumed: I chose it without a ruling from you, it stands until you say otherwise, and it can be reversed. The ones you have since ruled on are in the table above and are no longer listed here. The items about the chapters are in the revised text of `spec/`. The items about the prelude, the checker and the compiler are in the source.

**Chapter 2, objects.**

- `identical` is defined for every pair of values. Two value-class instances are identical when their fields are, so the result never depends on storage. The earlier text rejected it on value types and left boxed values unspecified.
- `Class` is not generic. It has `getName()` and the annotation readers of chapter 8. `getClass()` returns `Class`, with no special typing rule.
- `as` and `isNull` are gone from `Object`. Casts and null tests are syntax.
- The default `toString` of a value class has a fixed form, `Point(1, 2)`.
- An interface may require a static method of its implementers. This is what `Numeric<T>.zero()` needs.
- An enum constant may pass constructor arguments, as in `EARTH(5.97e24, 6.37e6)`. Without that, an enum's fields could not differ between constants.
- Field hiding is unchanged from the earlier text.

**Chapter 4, methods.**

- A lambda is an instance of an implicit final value class. It has no identity, and two evaluations that capture equal values are equal.
- `for (T x : e)` requires `e` to be an `Iterable`. The earlier text accepted any type with an `iterator()` method.
- `==`, casts and `instanceof` are rejected between types that share no value, such as two unrelated classes.
- A switch uses constant arms or type arms, not both, and its selector is not `@Nullable`. A switch expression must be exhaustive. A switch statement need not be.
- Assignment remains an expression, as in Java, and a local may be declared without an initializer.
- There is no unary `+`.
- A `foreign` method is static. An array argument is passed as a pointer to its elements, with no length, so a declaration can match a C signature such as `read(int, void *, size_t)` directly. `@Symbol` names the C function when the method's name cannot.

**Chapter 7, generics.**

- Static members of a generic class are shared by all its instantiations and are named without type arguments, as in Java: `Array.build(n, f)`. Chapter 2 was adjusted to say so.
- A call writes all of a generic method's type arguments or none, Java-style after the dot: `Array.<String>build(n, f)`. `new` always writes them, and there is no `<>` shorthand.
- In inference, a numeric literal is considered last, so `pair(x, 1)` gives a `List<Float64>` when `x` is a `Float64`.
- Only a static requirement of an interface can be called through a type parameter. There is no `T.class`.
- A class implements a generic interface with one set of type arguments.
- `getName()` writes type arguments with their qualifiers.
- A wildcard is rejected in four places that need a type: the class in `new`, explicit type arguments of a call, a type named by `extends` or `implements`, and a class literal.
- A wildcard may stand for a parameter that requires a tag. The earlier text rejected `Box<?>` there.
- A wildcard-typed argument may fix a method's type parameter to its unknown type. Two arguments never share an unknown.

**Chapter 9, execution.**

- A class that declares no constructor has an implicit one, with the audience of the class.
- Instance initializer blocks are gone.
- A value class may write its one constructor in a compact form, `public Range { ... }`, to give it an audience or a check. Value-class fields have no initializers.
- `Throwable` has a message, a cause and a list of suppressed exceptions. The message is the empty string when none was given.
- `catch` clauses behave as in Java. `return`, `break` and `continue` are rejected in a `finally` block.
- In `using`, an exception from the block wins and one from `close` is suppressed on it. The earlier text had it the other way round.
- Reading a static field before it is assigned raises `ClassInitializationException`. Two classes that use each other during initialization are not rejected for that alone.
- `Process.exit` is defined. Running out of stack ends the program and cannot be caught.
- `Loader`, the JIT, and the collector's design are no longer in the chapter. `main` has two forms.

**Chapter 5, null.**

- `x == null` is a built-in test, not a call of `equals`, so an `equals` override cannot break narrowing.
- A final method may declare a `@Nullable` receiver. Inside it, `this` may be `null`.
- A type parameter with no bound accepts a `@Nullable` type argument.

**Chapter 6, numbers.**

- Floats have no `%`.
- The rounding conversion is `Float64.nearest`. `round()` rounds to an integer value.
- `T[]` is the class `Array<T>`, and `Array.build` creates an array of non-null references.

**Chapter 8, annotations.**

- An annotation's elements are written in a header, `annotation Route(String path, Int priority = 0);`, and are read as fields, `route.path`. A qualifier has no elements.
- `@Target` takes constants of the enum `Site`. Without it, a declaration annotation may be written at every site. Locals and type parameters are not sites.
- A tag is any declaration annotation on a type declaration. `@Inherited` passes through interfaces as well as superclasses, and two inherited uses that disagree must be settled by writing the annotation.
- Reading is `c.<A>getAnnotation()`, typed by the method's reified type argument, so `Class` stays non-generic. `getAnnotatedFields` and `getAnnotatedMethods` return a type's own members in source order, and `getSuperclass` and `getInterfaces` lead to the inherited ones.
- `invoke` lets the method's exception through unchanged, returns `Unit` for a `void` method, applies no implicit conversion, and refuses a generic method.
- There are no repeatable or generic annotations, and no retention setting: every declaration annotation can be read.
- A `@Narrows` method must be declared in the package that declares its qualifier.
- Qualifiers in type arguments are reified. A value does not carry its own qualifiers, so `(@Positive Int) n` is rejected. Mirrors do not read qualifiers.
- A qualifier must describe a fact that never changes for a value. The language does not check this.
- `@Implicit` is limited to keep it tame. The method sits in the target class, both classes are value classes, it is not generic, and at most one conversion applies to a value. That it is exact and never raises is a promise by its author, which the language does not check.
- Integers do not convert implicitly to floats, because the everyday `Int` does not fit a float exactly.
- With mixed numeric operands, `i == n` compares after conversion while `i.equals(n)` is `false`, as in Java.

**Chapter 12, flow.**

- The branches of an `if` are reachable whatever its condition, so `if (DEBUG) { ... }` compiles when `DEBUG` is the constant `false`. The earlier text rejected the dead branch. A loop whose condition is the constant `false` is still rejected.
- `return;` in a constructor is legal only after a written `super(...)` or `this(...)` call.
- Every static field, not only a final one, is assigned by the end of static initialization.
- A constant expression may call `from`, `nearest` and `Char.from`. One that would raise is rejected.
- Narrowing has a section of its own. `assert c;` narrows what follows, and `??` narrows nothing.

**Chapter 13, concurrency.**

- `Thread.start` and `Scope.fork` take a `Function0`, and `Scope.call` a `Function1<Scope, T>`. With no `inline`, a `return` in a scope's body returns from the lambda.
- `Lock` can be taken again by the thread that holds it. It has no `close`. `exclusive` returns its body's result.
- Waiting is `Condition`, from `lock.newCondition()`. `await` may return without a signal.
- `AtomicInt` holds an `Int` and replaces the two width-specific classes. `compareAndSet` returns a `Boolean`. `addAndGet` raises on overflow and leaves the cell unchanged. `Atomic<T>` accepts any `T` and compares with `identical`.
- Cancellation is checked at `Thread.checkCancelled`, `sleep`, `join`, `Task.result` and `Condition.await`. `Lock.lock` is not such a point.
- An exception that leaves a thread's body is written to the error stream and kept for `uncaught()`.
- `Scope.call` reports the first failed task in fork order, with the others suppressed on it.
- The result of `Thread.start` and of `fork` may be ignored.

**Chapter 1, source.**

- The words Cleat adds to Java's are keywords in one position each and identifiers elsewhere: `value`, `annotation`, `open`, `sealed`, `permits`, `foreign`, `only`, `in`, `out` and `default`. A method may be named `open`. `inline` and `when` are reserved.
- `1e10` is a decimal literal.
- A program declares no package named `cleat` or beginning `cleat.`.
- A type name is looked up as a type parameter, then in the file, the named imports, the package, the `*` imports and the prelude.
- The library classes this specification does not describe, such as `List` and `Map`, are defined by the prelude's source.

**Chapter 3, visibility.**

- `private(this)` is gone. A type is visible to its file unless it is `package` or `public`.
- `only(A, B)` is written directly after the audience keyword.
- `protected` follows Java's receiver rule and does not include the package.
- An override keeps or widens the audience. A declaration may not expose a type that its own audience cannot name.
- A public annotated member of a type that is not public can be used through its mirror.

**Chapter 11, syntax.**

- `(T)` is not read as a cast before `-`, `++` or `--`.
- `new T[n][]` follows Java, and the first pair of brackets in a type is the outermost array.
- A lambda always writes its parameters in parentheses.
- `@Intrinsic` marks a prelude method whose body the implementation supplies. The declaration ends in `;`.

**The prelude.** Assumed, all of it: the specification names these classes or leaves them to the prelude's source.

- `Console` has `print`, `println`, `error`, which writes a line to the error stream, and `readLine`. `File` has `readText`, `readLines`, `writeText`, and `open` with `readLine` and `close`. A file holds UTF-8. `Process` has `exit`.
- `List<T>` is [programs/List.cleat](programs/List.cleat), and `prelude/List.cleat` is the same text. `Map<K, V>` keeps its keys in the order they were first set, `map[key]` is `@Nullable V`, and `entries()`, `keys()` and `values()` return lists. `Entry<K, V>` is a value class with public fields `key` and `value`. `StringBuilder.append` is `@Discardable`.
- `String` also has `contains`, `trim`, `split`, `toLowerCase`, `toUpperCase` and `fromChars`. `Char` has `isLetter`, `isDigit`, `isWhitespace`, `toLowerCase` and `toUpperCase`. The numeric classes also have `min`, `max`, `abs` and `parse`, and the float classes `isNaN`, `isInfinite` and `toFixed`.
- `Null` declares `equals`, `hashCode` and `toString` as chapter 5 lists them. `hashCode` of `null` is `0`.
- A `Float64` prints its shortest digits that read back as the same value, with `.0` when it has no fraction, and in exponent form from `1e21` up and below `1e-7`. Of two candidates as short, the nearer is taken, and of two as near, the larger, as Rust's library chooses. A `Float32` prints the same way at its own precision. A reference object prints as its class name, `@`, and a number.
- Printing, `toFixed` and `parse` of the float classes are written in the language, once for both widths, in the package class `Floats`. A float with no fraction from 2^53 up, or 2^24 for a `Float32`, prints its shortest digits padded with zeros, `1152921504606847000.0` for 2^60, as the rule above says; the runtime had printed every digit of the exact value. `toFixed` writes an infinity as `toString` does, `Infinity`, where the runtime wrote `inf`. The runtime prints a float field of a value class, and a float in the message of a failed conversion, by asking the float's `toString`.
- `Field`, `Method` and `Parameter` are value classes that hold a class object and a position. Their work is done by `@Intrinsic` methods of `Class` with package audience.
- `Thread`, `Lock` and `Condition` each hold a `Pointer` to an object of the host, which the collector frees with them. A wait that is a cancellation point wakes every 20 milliseconds to look for the request, so `Condition.await` returns within that time whether or not it was signalled, which section 13.3 allows.
- `Task<out T>` keeps its result in an `Atomic<@Nullable Object>`, because a field of type `T` that is assigned would break `out`.
- A method of the prelude is `@Intrinsic` only when it is one machine operation, an operation on an object's identity, class or storage, a table of Unicode, a mirror's reading of a class, or a call of the host. Everything that can be written over those is source. `compiler/tests/prelude_intrinsics.rs` lists each intrinsic method with its reason: 159, where there were 498. No method of `Rational` is intrinsic, nor any `toString`, `toFixed` or `parse` of a numeric class, and the runtime exports a body only for an intrinsic method or for a call the compiler writes.
- `BigInt` is an integer of any size, the numerator and denominator of a `Rational`. The specification does not name it, so it has package audience and is not in `spec/`. A value an `Int` holds is kept as that `Int`. A larger one keeps its sign and the digits of its magnitude in base 2^30, as a chain of value objects rather than an array, so that two equal values are `identical`, which section 2.5 asks of a value whose storage is not observable. Its arithmetic copies the digits into `Natural`, a package class that works in place on an `Int[]`, where each step of a product or of Knuth's long division fits an `Int`.
- `Rational` is a numerator and a denominator of `BigInt` in lowest terms, written in the language, and its results and exceptions are the ones the runtime gave. `parse` reads a decimal numeral, or a fraction of two integers whose digits `_` may separate, as the runtime's reading did. A literal of it is made by `Rational.literal` from the text the compiler writes.
- The integer classes narrower than 64 bits have no arithmetic of their own. Each computes in `Int` and converts back, and the conversion raises when the result does not fit.
- `parse` of an integer class accepts one sign, and reads `-0` as zero for the unsigned classes too.
- `text + x` is the text joined with `x.toString()`. `Boolean` and `Null` are written in the language with no intrinsic method.

**The checker.** Assumed.

- A name the compiler makes up begins with `#`, which no identifier can.
- `x != null` does not narrow a variable whose type is a bare type parameter: `T` minus `null` has no spelling.
- `var` gives a local the type of its initializer without the unknown type of a wildcard, as a member's result is written in section 7.4.
- When an argument is itself a call, it is checked on its own first. If that fails, it is checked again against the parameter's type, so a generic call can take its type arguments from where it stands.
- When `?:`, `??` or a switch expression has no expected type and one arm is `null`, the type is the other arm's with `@Nullable`.
- `x instanceof T` keeps the type `x` already has when that says more than `T`.
- A method reference adds nothing to inference. A lambda does.
- A use of a wildcard's unknown type as part of a larger explicit type argument is reported as not supported. Standing alone, as in `swap(items, 0, 1)`, it is the type argument of the object passed.
- A reserved word used as a name is rejected, and the message may point at the start of the statement.
- Section 4.2 gives an override the same type parameters as the method it overrides, and section 7.8 the same bounds. The checker also asks for the same required tags.
- Section 3.4 lets an override keep "a list drawn from" the `only` list of the method it overrides. The overriding class may also name itself there.
- Where a subclass declares a method with the name and signature of a superclass method it may not name, section 3.4 makes that a new method. A call through the subclass's type reaches the subclass's method, and the superclass's own calls reach its own.

**The compiler.** Assumed. None of this is language.

- `cleatc check <path>...` and `cleatc build <path>... --entry pkg.Type -o out.exe [--link file]...`. `--entry` names the entry class and `--link` the libraries or objects for foreign methods. A deprecated use is printed as a warning.
- A source file given to the compiler that declares the package `cleat` is read as prelude source in place of the prelude's file of that name. That is how `design/programs/List.cleat` is built and run. Any other file that declares `cleat` is rejected, as section 1.4 says.
- The collector is precise, non-moving mark and sweep, in the Rust runtime. It replaces MMTk, which cannot reserve its side-metadata range on Windows: that is the failing test recorded in the first commit of this branch. `compiler/vendor/mmtk` is no longer linked and is still in the tree.
- A machine number, `Boolean`, `Char` and `Pointer` are held unboxed where their static type says what they are. Every other value is a pointer to an object, a value class the program declares included.
- Generic code is compiled once. Type arguments are passed at run time, and an array stores machine numbers unboxed.
- Each thread has 16 MB of stack. A call that finds less than 1 MB left ends the program with status 1.
- A foreign method that passes a value class goes through C source the compiler writes and clang compiles, so the struct follows the platform's own convention. A value that C returns is built from its fields, without running a compact constructor's body. An array of value classes is copied to C structs and back into fresh values.
- The earlier checker, backend and tests covered the `Int32` subset of the old design and are removed. They remain in history.
- The numeric classes and the small regular files of the prelude are written by `compiler/tools/gen_numbers.py` and `compiler/tools/prelude_core.py`.

**Changed in the specification while implementing it.**

- Sections 3.1 and 3.3: the example fields have initializers, because a field with no default must be assigned.
- Section 8.9: the example is two examples, a declaration and a method.
- Section 4.4 names the one case where an ambiguous call is not rejected: section 6.3 chooses the method of a numeric literal's default class.
- Section 5.3 counts four methods of `Object` that take a `@Nullable` receiver, with `identical`, as section 2.5 declares them.
- Section 10.2 defers declaration annotations on locals and type parameters. A qualifier on a local and a tag required by a type parameter are in the language.
- Section 11.6: the grammar derives `name(args)`, a call with no receiver.
- Section 11.6: in the head of a switch arm a `(` does not begin a lambda. Section 12.1 makes a parenthesized constant a constant, and the lambda rule read `case (1) -> x` as a lambda.
- Section 2.6 says that the method a class declares for a static requirement carries `@Override`, as section 8.8 asks of every method that implements an interface method.
