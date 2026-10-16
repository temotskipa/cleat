# 4. Methods

This chapter defines methods, the expressions that call them, and the statements of a method body. <!-- r[methods.intro.defines-methods-expressions] -->
[Chapter 11](11-syntax.md) gives the syntax, including operator precedence. <!-- r[methods.intro.gives-syntax-including] -->

## 4.1 Declaring a method

A method declaration has modifiers, optional type parameters, a result, a name, a parameter list, and a body. <!-- r[methods.declaring.method-declaration-modifiers] -->

- The result is a type, or `void`, which is how the result `Unit` is written, as [chapter 5](05-null-and-unit.md) defines. <!-- r[methods.declaring.result-type-void] -->
- Parameter types and the result type are written. <!-- r[methods.declaring.parameter-types-result] -->
  `var` is rejected in those positions. <!-- r[methods.declaring.var-rejected-positions] -->
  A parameter is a local of the method, and is assignable unless it is declared `final`. <!-- r[methods.declaring.parameter-local-method] -->
- The modifiers are an audience from [chapter 3](03-visibility.md), `static`, `final`, `open`, `abstract` and `foreign`. <!-- r[methods.declaring.modifiers-audience-static] -->
- A generic method writes its type parameters before the result, as [chapter 7](07-generics.md) defines. <!-- r[methods.declaring.generic-method-writes] -->

A method of a class is final unless it is declared `open`: no subclass overrides it. <!-- r[methods.declaring.method-class-final] -->
`final` may be written to state that default. <!-- r[methods.declaring.final-written-state] -->
An `abstract` method has no body and appears only in an abstract class. <!-- r[methods.declaring.abstract-method-no] -->
`open final`, `abstract final` and `abstract static` are rejected. <!-- r[methods.declaring.open-final-abstract] -->
The methods of an interface are described in [chapter 2](02-objects.md). <!-- r[methods.declaring.methods-interface-described] -->

A static method has no receiver. <!-- r[methods.declaring.static-method-no] -->
`this` and `super` are rejected in it. <!-- r[methods.declaring.this-super-rejected] -->
A static method and an instance method may share a name. <!-- r[methods.declaring.static-method-instance] -->
A call is static when its receiver is a type name, and is an instance call when its receiver is a value. <!-- r[methods.declaring.call-static-receiver] -->

**The receiver parameter.** An instance method may declare its receiver as a first parameter named `this`. <!-- r[methods.declaring.receiver-parameter-instance] -->
Its type is the declaring class or interface, with qualifiers: <!-- r[methods.declaring.type-declaring-class] -->

```java
public open Boolean equals(Object? this, Object? other)
```

The receiver parameter is not an argument position, and a call supplies no argument for it. <!-- r[methods.declaring.receiver-parameter-not] -->
[Chapter 8](08-annotations.md) says what its qualifiers mean. <!-- r[methods.declaring.says-qualifiers-mean] -->
Without the declaration, the receiver's type is the declaring class with no qualifier. <!-- r[methods.declaring.without-declaration-receiver] -->
A static method and a constructor do not declare a receiver. <!-- r[methods.declaring.static-method-constructor] -->

## 4.2 Overriding

A method of a class overrides a method of its superclass, or implements a method of an interface, when the two have the same name and the same signature: the same type parameters and the same parameter types, with the same qualifiers. <!-- r[methods.overriding.method-class-overrides] -->
Its result type is the result type of the method it overrides, or a subtype of it, so `Circle copy()` may override `Shape copy()`. <!-- r[methods.overriding.result-type-method] -->

- Only an `open` method, an `abstract` method, or an interface method not declared `final` can be overridden. <!-- r[methods.overriding.only-open-method] -->
  Declaring a method with the name and signature of a visible final method of a superclass is rejected. <!-- r[methods.overriding.declaring-method-name] -->
- The overriding method carries `@Override`, as [chapter 8](08-annotations.md) requires. <!-- r[methods.overriding.overriding-method-carries] -->
- The overriding method is itself final unless it is declared `open`. <!-- r[methods.overriding.overriding-method-final] -->
- Its audience follows [chapter 3](03-visibility.md). <!-- r[methods.overriding.audience-follows] -->
- Its receiver has the same qualifiers as the receiver it overrides. <!-- r[methods.overriding.receiver-qualifiers-overrides] -->
  The exception is an override of `equals`, `hashCode` or `toString`, whose receiver has none. <!-- r[methods.overriding.exception-override-equals] -->
- It carries `@Discardable` exactly when the method it overrides does. <!-- r[methods.overriding.carries-discardable-exactly] -->

A static method overrides nothing. <!-- r[methods.overriding.static-method-overrides] -->
A static method of a class meets a static requirement of an interface, as chapter 2 describes, when it has that name and signature, and it then carries `@Override` too. <!-- r[methods.overriding.static-method-class] -->

Inside an instance method, `super.name(args)` calls the superclass's method without dispatch. <!-- r[methods.overriding.inside-instance-method] -->

## 4.3 Calls

A call is written `receiver.name(args)`, `Type.name(args)`, `super.name(args)` or `name(args)`. <!-- r[methods.calls.call-written-receiver] -->
In the last form the method belongs to the enclosing class or to a class it inherits from. <!-- r[methods.calls.last-form-method] -->
The receiver is `this` for an instance method and the class for a static method. <!-- r[methods.calls.receiver-this-instance] -->
Calling an instance method that way from a static method is rejected. <!-- r[methods.calls.calling-instance-method] -->

A call evaluates the receiver, then the arguments from left to right, and then runs the method with each parameter bound to the value of its argument. <!-- r[methods.calls.call-evaluates-receiver] -->
Assigning a parameter does not change a variable of the caller. <!-- r[methods.calls.assigning-parameter-not] -->

[Section 4.4](#44-overloading) chooses the method at compile time from static types. <!-- r[methods.calls.chooses-method-compile] -->
When the chosen method is `open`, `abstract` or an interface method, the receiver's class at run time selects the body, as [chapter 2](02-objects.md) describes. <!-- r[methods.calls.chosen-method-open] -->

The static type of the receiver must be a subtype of the method's receiver type. <!-- r[methods.calls.static-type-receiver] -->
A receiver whose type is nullable therefore accepts only the methods that declare a nullable receiver, until it is narrowed. <!-- r[methods.calls.receiver-type-nullable] -->

A call is an expression, and its type is the method's result type. <!-- r[methods.calls.call-expression-type] -->
A call whose result is `Unit` may stand alone as a statement. <!-- r[methods.calls.call-result-unit] -->
So may a call of a method marked `@Discardable`, such as an `append` that returns its receiver for chaining. <!-- r[methods.calls.call-method-marked] -->
A call of any other method must be used: as an operand, an argument, an initializer, or the right side of an assignment. <!-- r[methods.calls.call-method-used] -->
A statement that would discard its result is rejected. <!-- r[methods.calls.statement-discard-result] -->

## 4.4 Overloading

Two methods of one class may share a name when their parameter lists differ in length or in the type of a parameter. <!-- r[methods.overloading.methods-class-share] -->
Types that differ only in qualifiers do not count as different, and neither do result types. <!-- r[methods.overloading.types-differ-only] -->

A call chooses among the visible methods of that name that the receiver's static type declares or inherits. <!-- r[methods.overloading.call-chooses-among] -->

1. A method is applicable when the call has as many arguments as the method has parameters and each argument is assignable to its parameter under [section 4.5](#45-assignability), after the inference of [chapter 7](07-generics.md) for a generic method. <!-- r[methods.overloading.method-applicable-call] -->
   [Chapter 6](06-numbers.md) says when a numeric literal is applicable, and [section 4.8](#48-lambdas-and-method-references) says when a lambda is. <!-- r[methods.overloading.says-numeric-literal] -->
2. One applicable method is more specific than another when each of its parameter types is a subtype of the other's corresponding parameter type. <!-- r[methods.overloading.applicable-method-more-specific-parameter] -->
3. If one applicable method is more specific than every other, it is chosen. <!-- r[methods.overloading.applicable-method-more-specific-chosen] -->
   Otherwise the call is rejected, and the program writes a cast or explicit type arguments to choose. <!-- r[methods.overloading.otherwise-call-rejected] -->
   The one exception is a call with a numeric literal argument, where [chapter 6](06-numbers.md#63-numeric-literals) chooses the method of the literal's default class. <!-- r[methods.overloading.exception-call-numeric] -->

A method with a fixed number of parameters is more specific than a varargs method. <!-- r[methods.overloading.method-fixed-number] -->

Resolution first considers only the methods that are applicable without an implicit conversion. <!-- r[methods.overloading.resolution-first-considers] -->
If there is none, it considers the methods that are applicable when an argument may be converted, as [section 4.5](#45-assignability) allows. <!-- r[methods.overloading.none-considers-methods] -->
In that second round, a class counts as more specific than a class it converts to. <!-- r[methods.overloading.second-round-class] -->

## 4.5 Assignability

A value of static type `S` is assignable to a location of type `T` when `S` is a subtype of `T`. <!-- r[methods.assignability.value-static-type] -->
Subtyping is the reflexive, transitive closure of `extends` and `implements`, together with the rules for generic types in [chapter 7](07-generics.md), for nullable types in [chapter 5](05-null-and-unit.md) and for qualifiers in [chapter 8](08-annotations.md). <!-- r[methods.assignability.subtyping-reflexive-transitive] -->

**Implicit conversion.** A value of class `S` is also assignable to a location of type `T` or `T?` when `T` declares an implicit conversion from `S`. <!-- r[methods.assignability.implicit-conversion-value] -->
That is a static method marked `@Implicit`, as [chapter 8](08-annotations.md) defines, and the compiler inserts a call of it. <!-- r[methods.assignability.static-method-marked] -->
The numeric classes declare the implicit conversions that [chapter 6](06-numbers.md#67-conversion) lists, and a program's value classes may declare their own. <!-- r[methods.assignability.numeric-classes-declare] -->

At most one conversion is applied to a value. <!-- r[methods.assignability.most-conversion-applied] -->
Nothing else is converted: not the receiver of a call, not a nullable value, and not a type argument, so an `Int32[]` is never an `Int[]`. <!-- r[methods.assignability.nothing-else-converted] -->

No other method is ever inserted to make a value fit: not to produce a `String`, not to narrow or round a number, not to copy into a superclass. <!-- r[methods.assignability.no-method-ever] -->
A numeric literal takes its class from where it stands, as chapter 6 defines. <!-- r[methods.assignability.numeric-literal-takes] -->

**Expected type.** An expression has an expected type when it is the initializer of a declaration with a written type, the right side of an assignment, an argument for a parameter of a chosen method, the operand of `return`, the expression body of a lambda, or an element of an array creation. <!-- r[methods.assignability.expected-type-expression] -->
The expected type passes through parentheses, through both arms of `?:`, and through the arms of a switch expression. <!-- r[methods.assignability.expected-type-passes] -->
Numeric literals, lambdas, and the inference of chapter 7 use it. <!-- r[methods.assignability.numeric-literals-lambdas] -->

**Disjoint types.** Two types are disjoint when no value can belong to both. <!-- r[methods.assignability.disjoint-types-no] -->
Leaving aside every qualifier: <!-- r[methods.assignability.leaving-aside-qualifier] -->

- two class types are disjoint unless one is a subtype of the other; <!-- r[methods.assignability.class-types-disjoint] -->
- a class type and an interface type are disjoint when the class is final and is not a subtype of the interface; <!-- r[methods.assignability.class-type-interface] -->
- two nullable types are never disjoint, because both contain `null`; <!-- r[methods.assignability.nullable-types-never] -->
- two interface types are never disjoint, and a type parameter is disjoint from nothing. <!-- r[methods.assignability.interface-types-never] -->

`==`, casts and `instanceof` are rejected between disjoint types, as the following sections say. <!-- r[methods.assignability.equals-casts-instanceof] -->

## 4.6 Operators

An operator is a call of a method on its left operand, or on its only operand. <!-- r[methods.operators.operator-call-method] -->
The expression is legal exactly when that call is legal, and it has the call's result type. <!-- r[methods.operators.expression-legal-exactly] -->
Any class may declare these methods. <!-- r[methods.operators.class-declare-methods] -->

| Spelling | Call <!-- r[methods.operators.table] --> |
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

There is no unary `+`, no `>>>`, and no power operator. <!-- r[methods.operators.no-unary-unsigned-shift] -->

When the two operands of a binary operator other than a shift have different classes, and exactly one of them converts implicitly to the class of the other, it is converted first. <!-- r[methods.operators.operands-binary-operator] -->
With an `Int32 i` and an `Int n`, `i + n` is an `Int` addition. <!-- r[methods.operators.int32-int-addition] -->

**Equality.** `a == b` is `a.equals(b)`, for a reference class and a value class alike. <!-- r[methods.operators.equality-equals-reference] -->
`a != b` is `!(a == b)`. <!-- r[methods.operators.not-equals-equals] -->
Two cases differ: <!-- r[methods.operators.cases-differ] -->

- When one operand is the literal `null`, the expression is the null test of [chapter 5](05-null-and-unit.md), not a call. <!-- r[methods.operators.operand-literal-null] -->
- When the static types of the operands are disjoint, the expression is rejected, because it would be `false` for every pair of values. <!-- r[methods.operators.static-types-operands] -->
  Two classes are not rejected when one converts implicitly to the other. <!-- r[methods.operators.classes-not-rejected] -->

Reference identity is `identical`, a method of `Object`. <!-- r[methods.operators.reference-identity-identical] -->

**Compound assignment.** `a += b` is `a = a + b`, except that the location `a` is evaluated once. <!-- r[methods.operators.compound-assignment-except] -->
The same holds for `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=` and `>>=`. <!-- r[methods.operators.holds] -->

**Increment.** `a++` and `++a` are `a = a + 1`, and `a--` and `--a` are `a = a - 1`, with the location evaluated once. <!-- r[methods.operators.increment-location-evaluated] -->
The literal takes its class from `a` by the rule of chapter 6. <!-- r[methods.operators.literal-takes-class] -->
The prefix form yields the updated value, and the postfix form yields the value from before the update. <!-- r[methods.operators.prefix-form-yields] -->
If the method raises, the location is unchanged. <!-- r[methods.operators.method-raises-location] -->

## 4.7 Other expressions

**`&&` and `||`.** Both operands have type `Boolean`. <!-- r[methods.expressions.and-or-operands] -->
`a && b` evaluates `a`. <!-- r[methods.expressions.and-evaluates] -->
If it is `false`, the result is `false` and `b` is not evaluated. <!-- r[methods.expressions.false-result-not] -->
Otherwise the result is the value of `b`. <!-- r[methods.expressions.otherwise-result-value] -->
`a || b` evaluates `a`. <!-- r[methods.expressions.or-evaluates] -->
If it is `true`, the result is `true` and `b` is not evaluated. <!-- r[methods.expressions.true-result-not] -->
Otherwise the result is the value of `b`. <!-- r[methods.expressions.otherwise-result-value-2] -->
These two are not methods. <!-- r[methods.expressions.not-methods] -->
`&` and `|` on `Boolean` are the methods `and` and `or`, and they evaluate both operands. <!-- r[methods.expressions.boolean-methods-and] -->

**`?:`.** In `c ? a : b`, `c` has type `Boolean` and is evaluated once. <!-- r[methods.expressions.conditional-type-boolean] -->
Then exactly one of `a` and `b` is evaluated, and its value is the result. <!-- r[methods.expressions.exactly-evaluated-value] -->
When the expression has an expected type, both arms are checked against it and it is the type of the expression. <!-- r[methods.expressions.expression-expected-type] -->
Otherwise the type is the type of one arm, and the other arm must be assignable to it. <!-- r[methods.expressions.otherwise-type-arm] -->

**`??`.** `a ?? b` is the value of `a` unless that is `null`, and the value of `b` otherwise. <!-- r[methods.expressions.coalesce-value-unless] -->
[Chapter 5](05-null-and-unit.md#54-null-tests-and-narrowing) defines it. <!-- r[methods.expressions.defines] -->

**Casts.** `(T) e` evaluates `e` and tests the class of its value. <!-- r[methods.expressions.casts-evaluates-tests] -->
The test succeeds when that class is a subtype of `T`, including the type arguments of a generic class. <!-- r[methods.expressions.test-succeeds-class] -->
A successful cast yields the same value with static type `T`. <!-- r[methods.expressions.successful-cast-yields] -->
A failed cast raises `ClassCastException`. <!-- r[methods.expressions.failed-cast-raises] -->
[Chapter 5](05-null-and-unit.md) says what a cast does with `null`, and [chapter 8](08-annotations.md) says which qualifiers `T` may carry. <!-- r[methods.expressions.says-cast-null] -->

A cast never converts a value. <!-- r[methods.expressions.cast-never-converts] -->
It does not change the class of a number. <!-- r[methods.expressions.not-change-class] -->
A cast is rejected when `T` and the static type of `e` are disjoint, because the test could not succeed. <!-- r[methods.expressions.cast-rejected-static] -->

**`instanceof`.** `e instanceof T` is `true` when the value of `e` is not `null` and its class is a subtype of `T`. <!-- r[methods.expressions.instanceof-true-value] -->
It is rejected when `T` and the static type of `e` are disjoint. <!-- r[methods.expressions.rejected-static-type] -->
Where it is known `true`, it narrows a local, as chapter 5 describes. <!-- r[methods.expressions.known-true-narrows] -->

**`new`.** `new C(args)` creates an instance of the class `C`, by the construction rules of [chapter 9](09-execution.md). <!-- r[methods.expressions.new-args-creates] -->
The constructor is chosen as [section 4.4](#44-overloading) chooses a method. <!-- r[methods.expressions.constructor-chosen-chooses] -->
A generic class is written with its type arguments: `new List<String>()`. <!-- r[methods.expressions.generic-class-written] -->
Array creation is in [chapter 6](06-numbers.md). <!-- r[methods.expressions.array-creation] -->

**Assignment.** `x = e` evaluates `e` and stores its value in the local or parameter `x`. <!-- r[methods.expressions.assignment-evaluates-stores] -->
`r.f = e` evaluates `r`, then `e`, and stores into the field. <!-- r[methods.expressions.evaluates-stores-field] -->
`a[i] = e` evaluates `a`, `i` and `e` in that order and calls `a.set(i, e)`. <!-- r[methods.expressions.evaluates-order-calls] -->
The value of `e` must be assignable to the location. <!-- r[methods.expressions.value-assignable-location] -->
An assignment is an expression: its value is the value stored, and its type is the type of the location. <!-- r[methods.expressions.assignment-expression-value] -->
It may stand alone as a statement. <!-- r[methods.expressions.stand-alone-statement] -->

## 4.8 Lambdas and method references

A functional interface is an interface with exactly one abstract method and no static requirement. <!-- r[methods.lambdas.functional-interface-exactly] -->
The prelude declares the general ones: <!-- r[methods.lambdas.prelude-declares-general] -->

```java
public interface Function0<out R> { R invoke(); }
public interface Function1<in A, out R> { R invoke(A a); }
public interface Function2<in A, in B, out R> { R invoke(A a, B b); }
```

A lambda is written `(parameters) -> expression` or `(parameters) -> block`. <!-- r[methods.lambdas.lambda-written-parameters] -->
It has no type of its own. <!-- r[methods.lambdas.no-type] -->
It stands where the expected type is a functional interface, and it becomes an instance of that interface whose one method is the lambda. <!-- r[methods.lambdas.stands-expected-type] -->
`var f = () -> 1;` is rejected, because nothing says which interface is meant. <!-- r[methods.lambdas.var-arrow-rejected] -->

- The lambda has as many parameters as the interface's method. <!-- r[methods.lambdas.lambda-many-parameters] -->
  A parameter's type may be omitted, and is then the method's parameter type. <!-- r[methods.lambdas.parameter-type-omitted] -->
  A written type must be that type. <!-- r[methods.lambdas.written-type] -->
- The value of an expression body is the result, and must be assignable to the method's result type. <!-- r[methods.lambdas.value-expression-body] -->
  When that result type is `Unit`, an expression body may instead be any expression that could stand as a statement, and its value is discarded. <!-- r[methods.lambdas.result-type-unit] -->
  A block body returns with `return`, under the rules for a method with that result. <!-- r[methods.lambdas.block-body-returns] -->
  `return` inside a lambda returns from the lambda. <!-- r[methods.lambdas.return-inside-lambda] -->
  `break` and `continue` in a lambda do not reach a statement outside it. <!-- r[methods.lambdas.break-continue-lambda] -->
- Inside a lambda, `this` is the `this` of the enclosing method. <!-- r[methods.lambdas.inside-lambda-this] -->

**Capture.** A lambda may use a local or a parameter of an enclosing method or lambda only if that variable is never assigned after it is initialized. <!-- r[methods.lambdas.capture-lambda-use] -->
Using any other local in a lambda is rejected. <!-- r[methods.lambdas.using-local-lambda] -->
The lambda holds the values those variables had when the lambda expression was evaluated. <!-- r[methods.lambdas.lambda-holds-values] -->
A lambda may read and assign fields through `this`. <!-- r[methods.lambdas.lambda-read-assign] -->

**The instance.** The instance belongs to a final value class that the lambda expression declares implicitly. <!-- r[methods.lambdas.instance-belongs-final] -->
Its fields are the captured values and `this`. <!-- r[methods.lambdas.fields-captured-values] -->
A lambda therefore has no identity, and two evaluations of one lambda expression that capture equal values are equal. <!-- r[methods.lambdas.lambda-therefore-no] -->
Its `toString` is chosen by the implementation. <!-- r[methods.lambdas.tostring-chosen-implementation] -->

**In overload resolution.** A lambda argument is applicable to a parameter whose type is a functional interface whose method has that many parameters, with the same types wherever the lambda writes them. <!-- r[methods.lambdas.overload-resolution-lambda] -->
The body of the lambda plays no part in choosing the method. <!-- r[methods.lambdas.body-lambda-plays] -->
Chapter 7 says how a lambda takes part in inference. <!-- r[methods.lambdas.says-how-lambda] -->

**Method references.** A method reference is a short form of a lambda, and stands in the same places. <!-- r[methods.lambdas.method-references-reference] -->

| Reference | Lambda |
| --- | --- |
| `Type::name`, for a static method | `(args) -> Type.name(args)` <!-- r[methods.lambdas.type-reference-name-static] --> |
| `Type::name`, for an instance method | `(receiver, args) -> receiver.name(args)` <!-- r[methods.lambdas.type-reference-name-instance] --> |
| `expr::name` | `(args) -> v.name(args)`, where `v` is the value of `expr` when the reference is evaluated <!-- r[methods.lambdas.expr-reference-name] --> |
| `Type::new` | `(args) -> new Type(args)` <!-- r[methods.lambdas.type-reference-new] --> |

The parameter types of the interface's method choose among overloads. <!-- r[methods.lambdas.parameter-types-interface] -->

## 4.9 Statements

**Local declarations.** `T name = e;` declares a local of type `T` and initializes it. <!-- r[methods.statements.local-declarations-name] -->
`T name;` declares a local with no value, and [chapter 12](12-flow.md) requires an assignment before any use. <!-- r[methods.statements.name-declares-local] -->
`var name = e;` gives the local the static type of `e`, with its qualifiers. <!-- r[methods.statements.var-name-gives] -->
`var` needs an initializer, and `var name = null;` is rejected, because the type of `null` has no name. <!-- r[methods.statements.var-needs-initializer] -->
A local is assignable unless it is declared `final`. <!-- r[methods.statements.local-assignable-unless] -->
[Chapter 1](01-source.md) gives its scope. <!-- r[methods.statements.gives-scope] -->

**Expression statements.** An assignment, an increment, a decrement, a call whose result is `Unit`, and a call of a `@Discardable` method may each stand as a statement. <!-- r[methods.statements.expression-statements-assignment] -->
Every other expression is rejected as a statement, because its value would be discarded. <!-- r[methods.statements.expression-rejected-statement] -->

**`if` and `while`.** `if (c) s else t` evaluates `c`, which has type `Boolean`, and runs `s` when it is `true` and `t` otherwise. <!-- r[methods.statements.if-while-else] -->
Without `else`, nothing runs when `c` is `false`. <!-- r[methods.statements.without-else-nothing] -->
`while (c) s` evaluates `c` before each iteration, runs `s` while it is `true`, and completes when it is `false`. <!-- r[methods.statements.while-evaluates-before] -->

**`for`.** `for (init; c; update) s` runs `init` once. <!-- r[methods.statements.for-init-update] -->
Before each iteration it evaluates `c`, and completes when `c` is `false`. <!-- r[methods.statements.before-iteration-evaluates] -->
A missing `c` is `true`. <!-- r[methods.statements.missing-true] -->
Then it runs `s`, and then `update`. <!-- r[methods.statements.runs-update] -->
`update` also runs after a `continue` of this loop. <!-- r[methods.statements.update-runs-after] -->
It does not run after `break`, `return` or a raised exception. <!-- r[methods.statements.not-run-after] -->
A local declared in `init` is in scope in `c`, `s` and `update`. <!-- r[methods.statements.local-declared-init] -->

**`for` over a collection.** `for (T x : e) s` requires the static type of `e` to be a subtype of `Iterable<U>`, where `U` is assignable to `T`. <!-- r[methods.statements.for-over-collection] -->
It evaluates `e` and calls `iterator()` once. <!-- r[methods.statements.evaluates-calls-iterator] -->
Then, while `hasNext()` is `true`, it binds `x` to `next()` and runs `s`. <!-- r[methods.statements.while-hasnext-true] -->
`x` is not assignable. <!-- r[methods.statements.not-assignable] -->
`var` may be written for `T`. <!-- r[methods.statements.var-written] -->

```java
public interface Iterator<out T> { Boolean hasNext(); T next(); }
public interface Iterable<out T> { Iterator<T> iterator(); }
```

**`switch`.** `switch (selector) { arms }` evaluates the selector once and runs the first arm that matches. <!-- r[methods.statements.switch-selector-arms] -->
No arm continues into the next. <!-- r[methods.statements.no-arm-continues] -->
An arm is one of three kinds: <!-- r[methods.statements.arm-three-kinds] -->

- `case c1, c2 -> body` matches when the selector equals one of the constants under `equals`. <!-- r[methods.statements.case-c1-c2] -->
  The constants are constant expressions of the selector's type. <!-- r[methods.statements.constants-constant-expressions] -->
  The selector's type is an integer class, `Char`, `Boolean`, `String` or an enum. <!-- r[methods.statements.selector-type-integer] -->
  Writing one constant twice in a switch is rejected. <!-- r[methods.statements.writing-constant-twice] -->
- `case T name -> body` matches when the selector is an instance of `T`, as `instanceof` tests. <!-- r[methods.statements.case-name-arrow] -->
  In the body, `name` is a local of type `T` that holds the selector and is not assignable. <!-- r[methods.statements.body-name-local] -->
  `case T name when guard -> body` matches only when, besides, `guard` is `true`. <!-- r[methods.statements.case-guard-matches] -->
  The guard has type `Boolean`, sees `name`, and is evaluated only after the selector has been found an instance of `T`. <!-- r[methods.statements.guard-boolean-sees] -->
  `T` must not be disjoint from the selector's type. <!-- r[methods.statements.not-disjoint-selector] -->
  An arm that cannot match because an earlier arm without a guard names `T` or a supertype of `T` is rejected. <!-- r[methods.statements.arm-match-because] -->
- `default -> body` matches when no other arm does. <!-- r[methods.statements.default-arrow-body] -->
  A switch has at most one, written last. <!-- r[methods.statements.switch-most-written] -->

One switch uses constant arms or type arms, not both. <!-- r[methods.statements.switch-uses-constant] -->
The selector's type is not nullable. <!-- r[methods.statements.selector-type-not] -->

In a switch statement, each body is a statement. <!-- r[methods.statements.switch-statement-body] -->
If no arm matches, the switch completes normally. <!-- r[methods.statements.no-arm-matches] -->
In a switch expression, each body is an expression or a `throw` statement, and the switch must be exhaustive. <!-- r[methods.statements.switch-expression-body] -->
Its type follows the rule for `?:`, applied across all arms. <!-- r[methods.statements.type-follows-rule] -->

A switch is exhaustive when it has a `default` arm, when its constants are both `true` and `false`, when its constants are every constant of an enum, or when its type arms cover the selector's type. <!-- r[methods.statements.switch-exhaustive-default] -->
Type arms cover a type `S` when one of them that has no guard names `S` or a supertype of `S`. <!-- r[methods.statements.type-arms-cover] -->
They also cover `S` when `S` is a sealed interface or a sealed abstract class and they cover every type in its `permits` clause. <!-- r[methods.statements.cover-sealed-interface] -->

```java
static Rational eval(Expr expr) {
    return switch (expr) {
        case Num n -> n.value;
        case Neg n -> -eval(n.operand);
        case Binary b -> apply(b.op, eval(b.left), eval(b.right));
    };
}
```

A guard narrows an arm to the cases that need their own body, and the arm after it takes the rest: <!-- r[methods.statements.guard-narrows-arm] -->

```java
static String describe(Expr expr) {
    return switch (expr) {
        case Num n when n.value == 0 -> "zero";
        case Num n -> "a number";
        case Neg n -> "a negation";
        case Binary b -> "an operation";
    };
}
```

**Labels, `break` and `continue`.** `break` leaves the innermost enclosing `while`, `for` or switch statement. <!-- r[methods.statements.labels-break-continue] -->
`continue` starts the next iteration of the innermost enclosing loop. <!-- r[methods.statements.continue-starts-next] -->
A labeled statement is `label: statement`. <!-- r[methods.statements.labeled-statement-label] -->
`break label` completes that statement, and `continue label` continues it, which requires it to be a loop. <!-- r[methods.statements.break-label-completes] -->
A label that does not enclose the `break` or `continue` is rejected. <!-- r[methods.statements.label-not-enclose] -->

**`return` and `throw`.** `return e;` returns the value of `e`, which must be assignable to the method's result type. <!-- r[methods.statements.return-throw-returns] -->
`return;` returns from a method whose result is `Unit`. <!-- r[methods.statements.return-returns-method] -->
`throw e;` raises the value of `e`, whose static type is `Throwable` or a subclass of it. <!-- r[methods.statements.throw-raises-value] -->
[Chapter 9](09-execution.md) defines exceptions, `try` and `using`. <!-- r[methods.statements.defines-exceptions-try] -->

**`assert`.** `assert c;` evaluates `c`, which has type `Boolean`, and raises `AssertionException` when it is `false`. <!-- r[methods.statements.assert-evaluates-type] -->
`assert c : detail;` also evaluates `detail` in that case and gives `detail.toString()` to the exception as its message. <!-- r[methods.statements.assert-detail-evaluates] -->
Assertions are always checked. <!-- r[methods.statements.assertions-always-checked] -->
No mode removes them. <!-- r[methods.statements.no-mode-removes] -->

## 4.10 Varargs

A parameter `T... name` is a parameter of type `T[]` and is the last parameter. <!-- r[methods.varargs.parameter-name-type] -->
When a call chooses a varargs method, a single trailing argument that is assignable to `T[]` is passed as that array. <!-- r[methods.varargs.call-chooses-varargs] -->
Otherwise the trailing arguments are evaluated from left to right and stored in a new `T[]`. <!-- r[methods.varargs.otherwise-trailing-arguments] -->
No trailing arguments give a zero-length array. <!-- r[methods.varargs.no-trailing-arguments] -->

## 4.11 Foreign methods

A method declared `foreign` is also declared `static` and has no body. <!-- r[methods.foreign.method-declared-foreign] -->
Its body is a C function. <!-- r[methods.foreign.body-function] -->
Each parameter type and the result type is a machine type: <!-- r[methods.foreign.parameter-type-result] -->

| Cleat type | C type <!-- r[methods.foreign.table] --> |
| --- | --- |
| `Boolean` | `_Bool` |
| `Char` | `uint32_t` |
| `Int8`, `Int16`, `Int32`, `Int` | `int8_t`, `int16_t`, `int32_t`, `int64_t` |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t` |
| `Float32`, `Float64` | `float`, `double` |
| `Pointer` | `void *` |
| A value class whose fields are all machine types | A struct of those fields in source order, with the platform's C layout, passed by value |

A result written `void` is the C result `void`. <!-- r[methods.foreign.result-written-void] -->

A parameter, but not a result, may also be `T[]` where `T` is a machine type. <!-- r[methods.foreign.parameter-not-result] -->
The C function receives a pointer to the first element, with the elements laid out consecutively as C lays out `T`. <!-- r[methods.foreign.function-receives-pointer] -->
It may read and write the array's elements through that pointer until the call returns, and not afterwards. <!-- r[methods.foreign.read-write-array] -->
Its writes are in the array when the call returns. <!-- r[methods.foreign.writes-array-call] -->
The length is not passed. <!-- r[methods.foreign.length-not-passed] -->
A program that needs it passes `a.length()` as another argument. <!-- r[methods.foreign.program-needs-passes] -->
For an empty array the pointer must not be used. <!-- r[methods.foreign.empty-array-pointer] -->

A value of a value class that C returns, and each element of an array of a value class that C was passed, is constructed after the call from the fields in its struct, as [section 9.6](09-execution.md#96-constructing-a-value) says. <!-- r[methods.foreign.value-class-returns] -->
An exception that its constructor raises comes out of the call. <!-- r[methods.foreign.exception-constructor-raises] -->
The arrays are constructed in the order of their parameters, each in index order, and the result after them. <!-- r[methods.foreign.arrays-constructed-order] -->
An element takes its place in its array once it is constructed, so when one raises, it and every element after it in that order keep the values they had before the call. <!-- r[methods.foreign.element-takes-place] -->

Every other type is rejected on a foreign method. <!-- r[methods.foreign.type-rejected-foreign] -->
That includes `String`, `Rational`, a reference class, an interface, a type parameter, and a nullable type. <!-- r[methods.foreign.includes-string-rational] -->
A lambda cannot be passed, so a C function cannot call back into Cleat. <!-- r[methods.foreign.lambda-passed-function] -->

`Pointer` is a final value class that holds an address. <!-- r[methods.foreign.pointer-final-value] -->
`equals` compares addresses, and `Pointer.zero()` is the null address. <!-- r[methods.foreign.equals-compares-addresses] -->
Cleat does not read or write through a `Pointer`. <!-- r[methods.foreign.cleat-not-read] -->
A program passes it back to foreign methods. <!-- r[methods.foreign.program-passes-back] -->
Memory that C code allocates is not collected. <!-- r[methods.foreign.memory-code-allocates] -->

The call uses the platform's C calling convention for those types. <!-- r[methods.foreign.call-uses-platform] -->
The C symbol is the method's name, unless the method carries `@Symbol("name")`, which names another. <!-- r[methods.foreign.symbol-method-name] -->
The implementation is told which libraries supply the symbols when the program is linked. <!-- r[methods.foreign.implementation-told-libraries] -->

C code is outside this specification. <!-- r[methods.foreign.code-outside-specification] -->
A C function that writes outside the memory it was given, keeps a pointer after the call, or unwinds through the call has no defined behavior. <!-- r[methods.foreign.function-writes-outside] -->
