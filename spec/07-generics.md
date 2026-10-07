# 7. Generics

## 7.1 Type parameters

A class, an interface and a method may declare type parameters. An enum and an annotation may not.

```java
public class Box<T> { }
public interface Ordered<in T> { }
```

```java
public static <T> T[] build(Int length, Function1<Int, T> element)
```

A use of a generic class or interface writes one type argument for each parameter, as in `Box<String>` and `new Box<String>(s)`. A use with no arguments is rejected. The one exception is the class name that qualifies a static member, as [chapter 2](02-objects.md) describes. There are no raw types.

A type argument is a type: a class or interface type, an array type, or a type parameter that is in scope, each with its qualifiers. It must satisfy the bound of the parameter it is given for. In some positions a type argument may instead be a wildcard, as [section 7.4](#74-wildcards) defines.

The type parameters of a class are in scope in its `extends` and `implements` clauses, its instance members and its constructors. They are not in scope in its static members. The type parameters of a method are in scope in its signature and its body. A type parameter must not have the name of another type parameter that is in scope.

Type arguments are not erased. [Section 7.7](#77-type-arguments-at-run-time) says what that means at run time.

## 7.2 Bounds

A type parameter may declare a bound, or several bounds joined by `&`:

```java
static <T extends Numeric<T> & Ordered<T>> T largest(Iterable<T> items)
```

A type argument must be a subtype of each bound, after the argument is substituted for the parameter in the bound. At most one bound is a class. The others are interfaces.

A type parameter with no bound has the bound `@Nullable Object`, so every type is an argument for it. A bound written without `@Nullable` excludes `@Nullable` arguments: `T extends Object` admits exactly the types that do not contain `null`.

Inside the declaration, a value of type `T` has the members of the bounds of `T`. With no bound, those are the methods of `Object` that take a `@Nullable` receiver.

A tag written before the name of a type parameter is a further requirement, as [chapter 8](08-annotations.md) defines. `class Snapshot<@Frozen T>` accepts only a type argument whose declaration carries `Frozen`, or a type parameter that makes the same requirement.

## 7.3 Variance

A type parameter of a class or an interface is invariant unless it is declared `out` or `in`. The type parameters of a method have no variance.

- `out T` promises that the type only produces values of `T`. `T` may be the result type of a method and the type of a final field.
- `in T` promises that the type only consumes values of `T`. `T` may be the type of a method parameter.
- An invariant parameter may be written in either kind of position.

When `T` is written as a type argument, its position is judged through the parameter that receives it. An `out` parameter keeps the direction of the position. An `in` parameter reverses it. An invariant parameter needs both directions, so only an invariant `T` may be written there. A declaration that breaks these rules is rejected. The parameters of a constructor are not restricted.

For a generic type `G`, `G<A>` is a subtype of `G<B>` when this holds for each type parameter:

- for an invariant parameter, `A` and `B` are the same type, with the same qualifiers, or `B` is a wildcard that contains `A` under [section 7.4](#74-wildcards);
- for an `out` parameter, `A` is a subtype of `B`;
- for an `in` parameter, `B` is a subtype of `A`.

An instantiation is also a subtype of the supertypes its declaration names, with the arguments substituted. When `List<T>` implements `Iterable<T>`, `List<String>` is a subtype of `Iterable<String>`, and so of `Iterable<Object>`, because `Iterable` declares `out T`. `List<String>` is not a subtype of `List<Object>`.

## 7.4 Wildcards

A type argument may be a wildcard. A wildcard stands for a type that the program does not name.

```java
static Rational totalArea(List<? extends Shape> shapes)
static void addUnitCircles(List<? super Circle> sink, Int count)
static Int size(List<?> items)
```

- `? extends U` stands for some subtype of `U`.
- `? super L` stands for some supertype of `L`.
- `?` stands for some type that the parameter accepts.

A wildcard stands only for a type that could be written as the argument: one within the parameter's bound that carries each tag the parameter requires. `U` and `L` are types within that bound.

`List<? extends Shape>` is the type of every `List<X>` whose `X` is a subtype of `Shape`. A type that has a wildcard among its arguments is a wildcard type. It is a type for variables and parameters and not a class: the class of an object always has a type for each parameter.

**Where a wildcard is written.** A wildcard may be written wherever a type argument is written, with four exceptions that need a type:

- the arguments of the class in `new`;
- the explicit type arguments of a call;
- the arguments of a type that an `extends` or `implements` clause names;
- the arguments of a class literal.

`new List<?>()` is rejected. A wildcard inside one of those arguments is legal: `new List<List<?>>()` creates a list whose elements are lists of any kind.

**Declared variance.** A parameter declared `out` or `in` already varies in one direction, so a bounded wildcard adds nothing to it.

- For an `out` parameter, `G<? extends U>` is the same type as `G<U>`, and `? super` is rejected.
- For an `in` parameter, `G<? super L>` is the same type as `G<L>`, and `? extends` is rejected.

`?` may be written for any parameter. For an invariant or an `out` parameter with the bound `B`, `?` is the same argument as `? extends B`, so `Iterable<?>` is `Iterable<@Nullable Object>`. For an `in` parameter, `Ordered<?>` is the type of every `Ordered`, which no other spelling gives.

**Subtyping.** [Section 7.3](#73-variance) compares the arguments of an invariant parameter for sameness. With wildcards the rule is containment: `G<A>` is a subtype of `G<B>` when, for each invariant parameter, `B` contains `A`.

- A type contains only itself.
- `? extends U` contains a type `S`, and the wildcard `? extends S`, when `S` is a subtype of `U`.
- `? super L` contains a type `S`, and the wildcard `? super S`, when `L` is a subtype of `S`.
- `?` contains every argument.

`List<Circle>` is a subtype of `List<?>`, of `List<? extends Shape>` and of `List<? super Circle>`. It is not a subtype of `List<Shape>`. `List<? extends Circle>` is a subtype of `List<? extends Shape>`. `List<List<Circle>>` is not a subtype of `List<List<?>>`, because `List<?>` is there a type and contains only itself. It is a subtype of `List<? extends List<?>>`.

A wildcard type has the supertypes that its declaration names, read as the result type of a member is read below. `List<? extends Shape>` is a subtype of `Iterable<Shape>`, so a `for` statement visits its elements as shapes.

`List<? extends Object>` accepts no list of a `@Nullable` type, by the rule of [section 7.2](#72-bounds). `List<?>` accepts every list.

**Members.** When a member is used through a wildcard type, each wildcard stands for one unknown type, called `X` here. The checker knows the bounds of `X` and nothing else: `X` is a subtype of `U` for `? extends U`, a supertype of `L` for `? super L`, and within the parameter's bound in every case. The member's signature is read with `X` in place of the type parameter.

- An argument must be assignable to its parameter as read. Where the parameter's type is `X`, a subtype of `L` is assignable when the wildcard is `? super L`. Nothing is assignable when the wildcard is `? extends U` or `?`.
- The result has the result type as read. The program has no name for `X`, so the type of the expression is written without it. `X` on its own, or as the argument of an `out` parameter, becomes its upper bound: `U`, or the parameter's bound. `X` as the argument of an invariant parameter becomes the wildcard it came from. `X` as the argument of an `in` parameter becomes `L` when it has that lower bound, and `?` otherwise.

A field is read and assigned by the same reading.

```java
List<? extends Shape> shapes = circles;
Shape first = shapes[0];              // get returns X, which is a Shape
shapes.add(new Circle(1));            // rejected: a Circle is not known to be an X

List<? super Circle> sink = shapes2;
sink.add(new Circle(1));              // a Circle is an X, whatever X is
@Nullable Object item = sink[0];      // X is known only to be within the bound of List's parameter
```

**Naming the unknown.** Each evaluation of an expression has an unknown of its own. For a `List<?> items`, `items[0] = items[1]` is rejected, because nothing says that the two uses of `items` have one element type. A generic method says it:

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

An argument of a wildcard type may be given for a parameter whose type has a type parameter of the method in that position, as `List<T>` has. The type parameter is then the unknown type of that argument. At run time it is the type argument of the object that was passed. Two arguments never supply the same unknown: `static <T> void copy(List<T> from, List<T> to)` rejects two arguments of type `List<?>`. The result type of such a call is written without `X`, as a member's is.

## 7.5 Generic methods and inference

A call of a generic method writes all of its type arguments or none of them. Written arguments follow the `.` of the call: `Array.<String>build(n, f)`. `new` always writes the type arguments of a generic class: `new Box<Int>(1)`.

When a call writes none, they are inferred. Each type parameter `P` of the method collects constraints from the call:

- An argument of static type `A` for a parameter whose type is `P`, with or without qualifiers, makes `A` a lower bound of `P`.
- An argument for a parameter whose type has `P` as a type argument, such as `List<P>`, is matched against that type, through the supertype of the argument's type that instantiates the same generic type. The type argument found opposite `P` fixes `P` when the position is invariant. It is a lower bound of `P` when the position is `out`, and an upper bound when it is `in`. When `P` is nested more deeply, the positions on the way combine as [section 7.3](#73-variance) combines them.
- A parameter type that writes `? extends P` puts `P` in an `out` position, and one that writes `? super P` puts it in an `in` position. When the argument's own type has a wildcard opposite `P`, the wildcard's upper bound is what an `out` position finds, and its lower bound is what an `in` position finds. A wildcard with no bound on that side gives no constraint. In an invariant position the wildcard fixes `P` to the unknown type of that argument, as [section 7.4](#74-wildcards) describes.
- The expected type of the call, when there is one, constrains `P` through the method's result type. It is an upper bound when the result type is `P`, and it is matched as an argument is when the result type has `P` as a type argument.
- A lambda argument is considered after the other arguments. Its parameter types must be known by then, from the constraints so far or because the lambda writes them. A parameter type that the lambda writes fixes the type parameter in that position. The type of the lambda's result is then a lower bound for the type parameter in the result position.
- A numeric literal argument for a parameter of type `P` is considered last. If `P` is already determined, the literal is checked against that type, which is its expected type under [chapter 6](06-numbers.md). Otherwise the literal's default class is a lower bound of `P`.

Each `P` is then determined by the first of these rules that applies:

1. If the constraints fix `P`, `P` is that type. All of them must fix it to the same type.
2. If `P` has lower bounds, `P` is the lower bound that every other lower bound is a subtype of. When no lower bound is such a type, `P` is the upper bound that the expected type gave, if it gave one.
3. If `P` has an upper bound from an argument or from the expected type, `P` is that bound. When it has several, `P` is the one that is a subtype of all the others.

If no rule determines `P`, the method is not applicable to the call. It is also not applicable when the type determined for `P` is not a supertype of one of its lower bounds, not a subtype of one of its upper bounds, or not within its declared bound. The program then writes the type arguments.

An implicit conversion plays no part in determining `P`. Once `P` is determined, each argument must be assignable to its parameter under [chapter 4](04-methods.md#45-assignability), and there an argument may be converted.

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

A type is inferred with its qualifiers. Inference never produces a type that the program could not have written, apart from the unknown type of a wildcard argument.

## 7.6 Static requirements

When a bound of `P` is an interface that requires a static method, as [chapter 2](02-objects.md) describes, `P.name(args)` calls that method. The method that runs is the one the class of the type argument has.

```java
static <T extends Numeric<T>> T sum(Iterable<T> items) {
    T total = T.zero();
    for (T item : items) {
        total += item;
    }
    return total;
}
```

`sum` of an `Iterable<Rational>` runs `Rational.zero()`. No other static member is reached through a type parameter.

## 7.7 Type arguments at run time

Type arguments exist at run time.

- Each instantiation of a generic class is a class of its own. `List<String>` and `List<Int>` have different class objects, and `getClass()` on their instances returns different results. The qualifiers of a type argument are part of it, as [chapter 8](08-annotations.md) says.
- A cast and an `instanceof` test type arguments by the subtype rules of [section 7.3](#73-variance) and [section 7.4](#74-wildcards). `(List<String>) o` fails for a `List<Int>`. `o instanceof Iterable<Object>` is `true` for a `List<String>`. `o instanceof List<?>` is `true` for every list, and `o instanceof List<? extends Shape>` is `true` for a `List<Circle>`.
- Inside a generic class or method, a type parameter stands for its argument. `(T) e` and `e instanceof T` test against that argument, and `new @Nullable T[n]` creates an array of it.
- `T[]` is `Array<T>`, an invariant generic class like any other. `o instanceof String[]` is `false` for an `Object[]`. An array of some subtype of `Shape` is written `Array<? extends Shape>`.
- A wildcard type may be a type argument, and it is then part of the class as any argument is. `List<List<?>>` and `List<List<String>>` are different classes.

`getName()` of an instantiation is the name of the generic type, then `<`, the names of the type arguments separated by `, `, and `>`. The name of a type argument is the `getName()` of its class, preceded by each qualifier it carries, written as `@` and the qualifier's qualified name and ordered by that name. A `List` of nullable strings is `cleat.List<@cleat.Nullable cleat.String>`.

A wildcard inside a type argument is named `?`, or `? extends ` or `? super ` followed by the name of its bound. Two spellings of one type have one name: a wildcard that [section 7.4](#74-wildcards) makes the same as a type is named as that type, and `? extends` the parameter's own bound is named `?`.

How an implementation represents instantiations, with shared code or with a copy for each, is not observable.

## 7.8 Signatures

A signature keeps its type arguments. `void accept(List<String> items)` and `void accept(List<Int> items)` are different signatures, and one class may declare both. There is no bridge method.

A class implements a method of a generic interface by declaring it with the interface's type arguments substituted. A class that implements `Ordered<Money>` declares `Int compare(Money other)`.

A class implements a given generic interface with one set of type arguments. `implements Ordered<A>, Ordered<B>` is rejected.

A superclass method and an interface method with the same signature are one method, and the class implements it once.

A generic method is overridden only by a generic method with the same number of type parameters and the same bounds.
