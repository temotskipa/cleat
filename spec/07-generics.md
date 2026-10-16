# 7. Generics

## 7.1 Type parameters

A class, an interface and a method may declare type parameters. <!-- r[generics.parameters.class-interface-method] -->
An enum and an annotation may not. <!-- r[generics.parameters.enum-annotation-not] -->

```java
public class Box<T> { }
public interface Ordered<in T> { }
```

```java
public static <T> T[] build(Int length, Function1<Int, T> element)
```

A use of a generic class or interface writes one type argument for each parameter, as in `Box<String>` and `new Box<String>(s)`. <!-- r[generics.parameters.use-generic-class] -->
A use with no arguments is rejected. <!-- r[generics.parameters.use-no-arguments] -->
The one exception is the class name that qualifies a static member, as [chapter 2](02-objects.md) describes. <!-- r[generics.parameters.exception-class-name] -->
There are no raw types. <!-- r[generics.parameters.no-raw-types] -->

A type argument is a type: a class or interface type, an array type, or a type parameter that is in scope, each with its qualifiers. <!-- r[generics.parameters.type-argument-class] -->
It must satisfy the bound of the parameter it is given for. <!-- r[generics.parameters.satisfy-bound-parameter] -->
In some positions a type argument may instead be a wildcard, as [section 7.4](#74-wildcards) defines. <!-- r[generics.parameters.positions-type-argument] -->

The type parameters of a class are in scope in its `extends` and `implements` clauses, its instance members and its constructors. <!-- r[generics.parameters.type-parameters-class] -->
They are not in scope in its static members. <!-- r[generics.parameters.not-scope-static] -->
The type parameters of a method are in scope in its signature and its body. <!-- r[generics.parameters.type-parameters-method] -->
A type parameter must not have the name of another type parameter that is in scope. <!-- r[generics.parameters.type-parameter-not] -->

Type arguments are not erased. <!-- r[generics.parameters.type-arguments-not] -->
[Section 7.7](#77-type-arguments-at-run-time) says what that means at run time. <!-- r[generics.parameters.says-means-run] -->

## 7.2 Bounds

A type parameter may declare a bound, or several bounds joined by `&`: <!-- r[generics.bounds.type-parameter-declare] -->

```java
static <T extends Numeric<T> & Ordered<T>> T largest(Iterable<T> items)
```

A type argument must be a subtype of each bound, after the argument is substituted for the parameter in the bound. <!-- r[generics.bounds.type-argument-subtype] -->
At most one bound is a class. <!-- r[generics.bounds.most-bound-class] -->
The others are interfaces. <!-- r[generics.bounds.others-interfaces] -->

A type parameter with no bound has the bound `Object?`, so every type is an argument for it. <!-- r[generics.bounds.type-parameter-no] -->
A bound written without `?` excludes nullable arguments: `T extends Object` admits exactly the types that do not contain `null`. <!-- r[generics.bounds.bound-written-without] -->

Inside the declaration, a value of type `T` has the members of the bounds of `T`. <!-- r[generics.bounds.inside-declaration-value] -->
With no bound, those are the methods of `Object` that take a nullable receiver. <!-- r[generics.bounds.no-bound-methods] -->

A tag written before the name of a type parameter is a further requirement, as [chapter 8](08-annotations.md) defines. <!-- r[generics.bounds.tag-written-before] -->
`class Snapshot<@Frozen T>` accepts only a type argument whose declaration carries `Frozen`, or a type parameter that makes the same requirement. <!-- r[generics.bounds.class-snapshot-frozen] -->

## 7.3 Variance

A type parameter of a class or an interface is invariant unless it is declared `out` or `in`. <!-- r[generics.variance.type-parameter-class] -->
The type parameters of a method have no variance. <!-- r[generics.variance.type-parameters-method] -->

- `out T` promises that the type only produces values of `T`. <!-- r[generics.variance.out-promises-type] -->
  `T` may be the result type of a method and the type of a final field. <!-- r[generics.variance.result-type-method] -->
- `in T` promises that the type only consumes values of `T`. <!-- r[generics.variance.in-promises-type] -->
  `T` may be the type of a method parameter. <!-- r[generics.variance.type-method-parameter] -->
- An invariant parameter may be written in either kind of position. <!-- r[generics.variance.invariant-parameter-written] -->

When `T` is written as a type argument, its position is judged through the parameter that receives it. <!-- r[generics.variance.written-type-argument] -->
An `out` parameter keeps the direction of the position. <!-- r[generics.variance.out-parameter-keeps] -->
An `in` parameter reverses it. <!-- r[generics.variance.in-parameter-reverses] -->
An invariant parameter needs both directions, so only an invariant `T` may be written there. <!-- r[generics.variance.invariant-parameter-needs] -->
A declaration that breaks these rules is rejected. <!-- r[generics.variance.declaration-breaks-rules] -->
The parameters of a constructor are not restricted. <!-- r[generics.variance.parameters-constructor-not] -->

For a generic type `G`, `G<A>` is a subtype of `G<B>` when this holds for each type parameter: <!-- r[generics.variance.generic-type-subtype] -->

- for an invariant parameter, `A` and `B` are the same type, with the same qualifiers, or `B` is a wildcard that contains `A` under [section 7.4](#74-wildcards); <!-- r[generics.variance.invariant-parameter-type] -->
- for an `out` parameter, `A` is a subtype of `B`; <!-- r[generics.variance.out-parameter-subtype] -->
- for an `in` parameter, `B` is a subtype of `A`. <!-- r[generics.variance.in-parameter-subtype] -->

An instantiation is also a subtype of the supertypes its declaration names, with the arguments substituted. <!-- r[generics.variance.instantiation-subtype-supertypes] -->
When `List<T>` implements `Iterable<T>`, `List<String>` is a subtype of `Iterable<String>`, and so of `Iterable<Object>`, because `Iterable` declares `out T`. <!-- r[generics.variance.list-implements-iterable] -->
`List<String>` is not a subtype of `List<Object>`. <!-- r[generics.variance.list-string-not] -->

## 7.4 Wildcards

A type argument may be a wildcard. <!-- r[generics.wildcards.type-argument-wildcard] -->
A wildcard stands for a type that the program does not name. <!-- r[generics.wildcards.wildcard-stands-type] -->

```java
static Rational totalArea(List<? extends Shape> shapes)
static void addUnitCircles(List<? super Circle> sink, Int count)
static Int size(List<?> items)
```

- `? extends U` stands for some subtype of `U`. <!-- r[generics.wildcards.extends-stands-subtype] -->
- `? super L` stands for some supertype of `L`. <!-- r[generics.wildcards.super-stands-supertype] -->
- `?` stands for some type that the parameter accepts. <!-- r[generics.wildcards.stands-type-parameter] -->

A wildcard stands only for a type that could be written as the argument: one within the parameter's bound that carries each tag the parameter requires. <!-- r[generics.wildcards.wildcard-stands-only] -->
`U` and `L` are types within that bound. <!-- r[generics.wildcards.types-within-bound] -->

`List<? extends Shape>` is the type of every `List<X>` whose `X` is a subtype of `Shape`. <!-- r[generics.wildcards.list-extends-shape-type] -->
A type that has a wildcard among its arguments is a wildcard type. <!-- r[generics.wildcards.type-wildcard-among] -->
It is a type for variables and parameters and not a class: the class of an object always has a type for each parameter. <!-- r[generics.wildcards.type-variables-parameters] -->

**Where a wildcard is written.** A wildcard may be written wherever a type argument is written, with four exceptions that need a type: <!-- r[generics.wildcards.wildcard-written-wherever] -->

- the arguments of the class in `new`; <!-- r[generics.wildcards.arguments-class-new] -->
- the explicit type arguments of a call; <!-- r[generics.wildcards.explicit-type-arguments] -->
- the arguments of a type that an `extends` or `implements` clause names; <!-- r[generics.wildcards.arguments-type-extends] -->
- the arguments of a class literal. <!-- r[generics.wildcards.arguments-class-literal] -->

`new List<?>()` is rejected. <!-- r[generics.wildcards.new-list-rejected] -->
A wildcard inside one of those arguments is legal: `new List<List<?>>()` creates a list whose elements are lists of any kind. <!-- r[generics.wildcards.wildcard-inside-arguments] -->

**Declared variance.** A parameter declared `out` or `in` already varies in one direction, so a bounded wildcard adds nothing to it. <!-- r[generics.wildcards.declared-variance-parameter] -->

- For an `out` parameter, `G<? extends U>` is the same type as `G<U>`, and `? super` is rejected. <!-- r[generics.wildcards.out-parameter-extends] -->
- For an `in` parameter, `G<? super L>` is the same type as `G<L>`, and `? extends` is rejected. <!-- r[generics.wildcards.in-parameter-super] -->

`?` may be written for any parameter. <!-- r[generics.wildcards.written-parameter] -->
For an invariant or an `out` parameter with the bound `B`, `?` is the same argument as `? extends B`, so `Iterable<?>` is `Iterable<Object?>`. <!-- r[generics.wildcards.invariant-out-parameter] -->
For an `in` parameter, `Ordered<?>` is the type of every `Ordered`, which no other spelling gives. <!-- r[generics.wildcards.in-parameter-ordered] -->

**Subtyping.** [Section 7.3](#73-variance) compares the arguments of an invariant parameter for sameness. <!-- r[generics.wildcards.subtyping-compares-arguments] -->
With wildcards the rule is containment: `G<A>` is a subtype of `G<B>` when, for each invariant parameter, `B` contains `A`. <!-- r[generics.wildcards.wildcards-rule-containment] -->

- A type contains only itself. <!-- r[generics.wildcards.type-contains-only] -->
- `? extends U` contains a type `S`, and the wildcard `? extends S`, when `S` is a subtype of `U`. <!-- r[generics.wildcards.extends-contains-type] -->
- `? super L` contains a type `S`, and the wildcard `? super S`, when `L` is a subtype of `S`. <!-- r[generics.wildcards.super-contains-type] -->
- `?` contains every argument. <!-- r[generics.wildcards.contains-argument] -->

`List<Circle>` is a subtype of `List<?>`, of `List<? extends Shape>` and of `List<? super Circle>`. <!-- r[generics.wildcards.list-circle-subtype] -->
It is not a subtype of `List<Shape>`. <!-- r[generics.wildcards.not-subtype-list] -->
`List<? extends Circle>` is a subtype of `List<? extends Shape>`. <!-- r[generics.wildcards.list-extends-circle] -->
`List<List<Circle>>` is not a subtype of `List<List<?>>`, because `List<?>` is there a type and contains only itself. <!-- r[generics.wildcards.list-circle-not] -->
It is a subtype of `List<? extends List<?>>`. <!-- r[generics.wildcards.subtype-list-extends] -->

A wildcard type has the supertypes that its declaration names, read as the result type of a member is read below. <!-- r[generics.wildcards.wildcard-type-supertypes] -->
`List<? extends Shape>` is a subtype of `Iterable<Shape>`, so a `for` statement visits its elements as shapes. <!-- r[generics.wildcards.list-extends-shape-subtype] -->

`List<? extends Object>` accepts no list of a nullable type, by the rule of [section 7.2](#72-bounds). <!-- r[generics.wildcards.list-extends-object] -->
`List<?>` accepts every list. <!-- r[generics.wildcards.list-accepts] -->

**Members.** When a member is used through a wildcard type, each wildcard stands for one unknown type, called `X` here. <!-- r[generics.wildcards.members-member-used] -->
The checker knows the bounds of `X` and nothing else: `X` is a subtype of `U` for `? extends U`, a supertype of `L` for `? super L`, and within the parameter's bound in every case. <!-- r[generics.wildcards.checker-knows-bounds] -->
The member's signature is read with `X` in place of the type parameter. <!-- r[generics.wildcards.member-signature-read] -->

- An argument must be assignable to its parameter as read. <!-- r[generics.wildcards.argument-assignable-parameter] -->
  Where the parameter's type is `X`, a subtype of `L` is assignable when the wildcard is `? super L`. <!-- r[generics.wildcards.parameter-type-subtype] -->
  Nothing is assignable when the wildcard is `? extends U` or `?`. <!-- r[generics.wildcards.nothing-assignable-wildcard] -->
- The result has the result type as read. <!-- r[generics.wildcards.result-type-read] -->
  The program has no name for `X`, so the type of the expression is written without it. <!-- r[generics.wildcards.program-no-name] -->
  `X` on its own, or as the argument of an `out` parameter, becomes its upper bound: `U`, or the parameter's bound. <!-- r[generics.wildcards.argument-out-parameter] -->
  `X` as the argument of an invariant parameter becomes the wildcard it came from. <!-- r[generics.wildcards.argument-invariant-parameter] -->
  `X` as the argument of an `in` parameter becomes `L` when it has that lower bound, and `?` otherwise. <!-- r[generics.wildcards.argument-in-parameter] -->

A field is read and assigned by the same reading. <!-- r[generics.wildcards.field-read-assigned] -->

```java
List<? extends Shape> shapes = circles;
Shape first = shapes[0];              // get returns X, which is a Shape
shapes.add(new Circle(1));            // rejected: a Circle is not known to be an X

List<? super Circle> sink = shapes2;
sink.add(new Circle(1));              // a Circle is an X, whatever X is
Object? item = sink[0];      // X is known only to be within the bound of List's parameter
```

**Naming the unknown.** Each evaluation of an expression has an unknown of its own. <!-- r[generics.wildcards.naming-unknown-evaluation] -->
For a `List<?> items`, `items[0] = items[1]` is rejected, because nothing says that the two uses of `items` have one element type. <!-- r[generics.wildcards.list-items-rejected] -->
A generic method says it: <!-- r[generics.wildcards.generic-method-says] -->

```java
static <T> void swap(List<T> items, Int i, Int j) {
    T held = items[i];
    items[i] = items[j];
    items[j] = held;
}

static void swapFirstTwo(List<?> items) {
    swap(items, 0, 1);                // T is the unknown element type of items
}
```

An argument of a wildcard type may be given for a parameter whose type has a type parameter of the method in that position, as `List<T>` has. <!-- r[generics.wildcards.argument-wildcard-type] -->
The type parameter is then the unknown type of that argument. <!-- r[generics.wildcards.type-parameter-unknown] -->
At run time it is the type argument of the object that was passed. <!-- r[generics.wildcards.run-time-type] -->
Two arguments never supply the same unknown: `static <T> void copy(List<T> from, List<T> to)` rejects two arguments of type `List<?>`. <!-- r[generics.wildcards.arguments-never-supply] -->
The result type of such a call is written without `X`, as a member's is. <!-- r[generics.wildcards.result-type-call] -->

## 7.5 Generic methods and inference

A call of a generic method writes all of its type arguments or none of them. <!-- r[generics.inference.call-generic-method] -->
Written arguments follow the `.` of the call: `Array.<String>build(n, f)`. <!-- r[generics.inference.written-arguments-follow] -->
`new` always writes the type arguments of a generic class: `new Box<Int>(1)`. <!-- r[generics.inference.new-always-writes] -->

When a call writes none, they are inferred. <!-- r[generics.inference.call-writes-none] -->
Each type parameter `P` of the method collects constraints from the call: <!-- r[generics.inference.type-parameter-method] -->

- An argument of static type `A` for a parameter whose type is `P`, with or without qualifiers, makes `A` a lower bound of `P`. <!-- r[generics.inference.argument-static-type] -->
- An argument for a parameter whose type has `P` as a type argument, such as `List<P>`, is matched against that type, through the supertype of the argument's type that instantiates the same generic type. <!-- r[generics.inference.argument-parameter-type] -->
  The type argument found opposite `P` fixes `P` when the position is invariant. <!-- r[generics.inference.type-argument-found] -->
  It is a lower bound of `P` when the position is `out`, and an upper bound when it is `in`. <!-- r[generics.inference.lower-bound-position] -->
  When `P` is nested more deeply, the positions on the way combine as [section 7.3](#73-variance) combines them. <!-- r[generics.inference.nested-more-deeply] -->
- A parameter type that writes `? extends P` puts `P` in an `out` position, and one that writes `? super P` puts it in an `in` position. <!-- r[generics.inference.parameter-type-writes] -->
  When the argument's own type has a wildcard opposite `P`, the wildcard's upper bound is what an `out` position finds, and its lower bound is what an `in` position finds. <!-- r[generics.inference.argument-type-wildcard] -->
  A wildcard with no bound on that side gives no constraint. <!-- r[generics.inference.wildcard-no-bound] -->
  In an invariant position the wildcard fixes `P` to the unknown type of that argument, as [section 7.4](#74-wildcards) describes. <!-- r[generics.inference.invariant-position-wildcard] -->
- The expected type of the call, when there is one, constrains `P` through the method's result type. <!-- r[generics.inference.expected-type-call] -->
  It is an upper bound when the result type is `P`, and it is matched as an argument is when the result type has `P` as a type argument. <!-- r[generics.inference.upper-bound-result] -->
- A lambda argument is considered after the other arguments. <!-- r[generics.inference.lambda-argument-considered] -->
  Its parameter types must be known by then, from the constraints so far or because the lambda writes them. <!-- r[generics.inference.parameter-types-known] -->
  A parameter type that the lambda writes fixes the type parameter in that position. <!-- r[generics.inference.parameter-type-lambda] -->
  The type of the lambda's result is then a lower bound for the type parameter in the result position. <!-- r[generics.inference.type-lambda-result] -->
- A numeric literal argument for a parameter of type `P` is considered last. <!-- r[generics.inference.numeric-literal-argument] -->
  If `P` is already determined, the literal is checked against that type, which is its expected type under [chapter 6](06-numbers.md). <!-- r[generics.inference.already-determined-literal] -->
  Otherwise the literal's default class is a lower bound of `P`. <!-- r[generics.inference.otherwise-literal-default] -->

Each `P` is then determined by the first of these rules that applies: <!-- r[generics.inference.determined-first-rules] -->

1. If the constraints fix `P`, `P` is that type. <!-- r[generics.inference.constraints-fix-type] -->
   All of them must fix it to the same type. <!-- r[generics.inference.fix-type] -->
2. If `P` has lower bounds, `P` is the lower bound that every other lower bound is a subtype of. <!-- r[generics.inference.lower-bounds-bound] -->
   When no lower bound is such a type, `P` is the upper bound that the expected type gave, if it gave one. <!-- r[generics.inference.no-lower-bound] -->
3. If `P` has an upper bound from an argument or from the expected type, `P` is that bound. <!-- r[generics.inference.upper-bound-argument] -->
   When it has several, `P` is the one that is a subtype of all the others. <!-- r[generics.inference.several-subtype-others] -->

If no rule determines `P`, the method is not applicable to the call. <!-- r[generics.inference.no-rule-determines] -->
It is also not applicable when the type determined for `P` is not a supertype of one of its lower bounds, not a subtype of one of its upper bounds, or not within its declared bound. <!-- r[generics.inference.not-applicable-type] -->
The program then writes the type arguments. <!-- r[generics.inference.program-writes-type] -->

An implicit conversion plays no part in determining `P`. <!-- r[generics.inference.implicit-conversion-plays] -->
Once `P` is determined, each argument must be assignable to its parameter under [chapter 4](04-methods.md#45-assignability), and there an argument may be converted. <!-- r[generics.inference.once-determined-argument] -->

```java
static <T> T first(List<T> items)
static <T> List<T> pair(T a, T b)
static <A, B> List<B> map(List<A> items, Function1<A, B> f)
static <T> void addAll(List<? super T> sink, Iterable<T> items)
```

```java
var name = first(names);                  // T is String: List<T> fixes it
var both = pair("a", "b");                // T is String
var mixed = pair("a", 'b');               // rejected: neither String nor Char is a supertype of the other
List<Object> xs = pair("a", 'b');         // T is Object: the expected type fixes it
var lengths = map(names, (s) -> s.length());   // A is String, then B is Int from the lambda's result
addAll(shapes, circles);                  // T is Circle: Iterable<T> gives the lower bound Circle,
                                          // and List<Shape> against List<? super T> the upper bound Shape
```

A type is inferred with its qualifiers. <!-- r[generics.inference.type-inferred-qualifiers] -->
Inference never produces a type that the program could not have written, apart from the unknown type of a wildcard argument. <!-- r[generics.inference.inference-never-produces] -->

## 7.6 Static requirements

When a bound of `P` is an interface that requires a static method, as [chapter 2](02-objects.md) describes, `P.name(args)` calls that method. <!-- r[generics.static.bound-interface-requires] -->
The method that runs is the one the class of the type argument has. <!-- r[generics.static.method-runs-class] -->

```java
static <T extends Numeric<T>> T sum(Iterable<T> items) {
    T total = T.zero();
    for (T item : items) {
        total += item;
    }
    return total;
}
```

`sum` of an `Iterable<Rational>` runs `Rational.zero()`. <!-- r[generics.static.sum-iterable-rational] -->
No other static member is reached through a type parameter. <!-- r[generics.static.no-static-member] -->

## 7.7 Type arguments at run time

Type arguments exist at run time. <!-- r[generics.run-time.type-arguments-exist] -->

- Each instantiation of a generic class is a class of its own. <!-- r[generics.run-time.instantiation-generic-class] -->
  `List<String>` and `List<Int>` have different class objects, and `getClass()` on their instances returns different results. <!-- r[generics.run-time.list-string-int] -->
  The qualifiers of a type argument are part of it, as [chapter 8](08-annotations.md) says. <!-- r[generics.run-time.qualifiers-type-argument] -->
- A cast and an `instanceof` test type arguments by the subtype rules of [section 7.3](#73-variance) and [section 7.4](#74-wildcards). <!-- r[generics.run-time.cast-instanceof-test] -->
  `(List<String>) o` fails for a `List<Int>`. <!-- r[generics.run-time.list-string-fails] -->
  `o instanceof Iterable<Object>` is `true` for a `List<String>`. <!-- r[generics.run-time.instanceof-iterable-object] -->
  `o instanceof List<?>` is `true` for every list, and `o instanceof List<? extends Shape>` is `true` for a `List<Circle>`. <!-- r[generics.run-time.instanceof-list-true] -->
- Inside a generic class or method, a type parameter stands for its argument. <!-- r[generics.run-time.inside-generic-class] -->
  `(T) e` and `e instanceof T` test against that argument, and `new T?[n]` creates an array of it. <!-- r[generics.run-time.instanceof-test-against] -->
- `T[]` is `Array<T>`, an invariant generic class like any other. <!-- r[generics.run-time.array-invariant-generic] -->
  `o instanceof String[]` is `false` for an `Object[]`. <!-- r[generics.run-time.instanceof-string-false] -->
  An array of some subtype of `Shape` is written `Array<? extends Shape>`. <!-- r[generics.run-time.array-subtype-shape] -->
- A wildcard type may be a type argument, and it is then part of the class as any argument is. <!-- r[generics.run-time.wildcard-type-argument] -->
  `List<List<?>>` and `List<List<String>>` are different classes. <!-- r[generics.run-time.list-string-different] -->

`getName()` of an instantiation is the name of the generic type, then `<`, the names of the type arguments separated by `, `, and `>`. <!-- r[generics.run-time.getname-instantiation-name] -->
The name of a type argument is the `getName()` of its class, preceded by each qualifier it carries, written as `@` and the qualifier's qualified name and ordered by that name. <!-- r[generics.run-time.name-type-argument] -->
A `List` of nullable strings is `cleat.List<@cleat.Nullable cleat.String>`. <!-- r[generics.run-time.list-nullable-strings] -->

A wildcard inside a type argument is named `?`, or `? extends ` or `? super ` followed by the name of its bound. <!-- r[generics.run-time.wildcard-inside-type] -->
Two spellings of one type have one name: a wildcard that [section 7.4](#74-wildcards) makes the same as a type is named as that type, and `? extends` the parameter's own bound is named `?`. <!-- r[generics.run-time.spellings-type-name] -->

How an implementation represents instantiations, with shared code or with a copy for each, is not observable. <!-- r[generics.run-time.how-implementation-represents] -->

## 7.8 Signatures

A signature keeps its type arguments. <!-- r[generics.signatures.signature-keeps-type] -->
`void accept(List<String> items)` and `void accept(List<Int> items)` are different signatures, and one class may declare both. <!-- r[generics.signatures.void-accept-list] -->
There is no bridge method. <!-- r[generics.signatures.no-bridge-method] -->

A class implements a method of a generic interface by declaring it with the interface's type arguments substituted. <!-- r[generics.signatures.class-implements-method] -->
A class that implements `Ordered<Money>` declares `Int compare(Money other)`. <!-- r[generics.signatures.class-implements-ordered] -->

A class implements a given generic interface with one set of type arguments. <!-- r[generics.signatures.class-implements-given] -->
`implements Ordered<A>, Ordered<B>` is rejected. <!-- r[generics.signatures.implements-ordered-rejected] -->

A superclass method and an interface method with the same signature are one method, and the class implements it once. <!-- r[generics.signatures.superclass-method-interface] -->

A generic method is overridden only by a generic method with the same number of type parameters and the same bounds. <!-- r[generics.signatures.generic-method-overridden] -->
