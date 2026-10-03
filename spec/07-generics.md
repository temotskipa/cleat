# 7. Generics

## 7.1 Reified type arguments

A class, interface, annotation, or method may declare type parameters. A use supplies type arguments or, where this chapter allows, infers them. The arguments are present at runtime. A cast and `instanceof` test the arguments. `List<String>` and `List<Object>` are different classes at runtime. There are no raw types. A use of a generic type that omits the arguments is rejected.

Type parameters are invariant unless declared `out` or `in`.

```
type-params = "<" type-param {"," type-param} ">"
type-param  = ["in" | "out"] {annotation} identifier ["extends" bound]
bound       = type {"&" type}
```

`out T` may appear only in result positions, including a result type, a field type of a final field that is not written from outside, and a type argument that is itself in an `out` position. `in T` may appear only in input positions: parameter types and a type argument that is itself in an `in` position. The compiler rejects a use of the parameter on the wrong side. An invariant parameter may appear on either side.

`Iterator<Int32>` is a subtype of `Iterator<Number>` because `Iterator` declares `out T`. `List<Int32>` is not a subtype of `List<Number>` when `List` is invariant. A use may leave an argument unknown, as [section 7.2](#72-use-site-wildcards) defines. Omitting the argument list is still a raw type and is rejected.

A bound `T extends Number & Iterable<T>` requires the argument to be a subtype of every conjunct. A type parameter with no bound has bound `Object`.

## 7.2 Use-site wildcards

A type argument may be a wildcard. A wildcard is a static unknown over a concrete runtime instantiation. It is not a raw type and not a runtime class of its own.

```
type-arg = type | "?" | "? extends" type | "? super" type
```

In a type argument, `?` begins a wildcard. The conditional spelling `?:` is an expression, not a type argument. `*` is not a type argument. A star projection is rejected.

`List<?>` accepts any instantiation of an invariant `List`. `List<? extends Number>` accepts an instantiation whose argument is a subtype of `Number`. `List<? super Int32>` accepts an instantiation whose argument is a supertype of `Int32`. A bare `?` means `? extends B`, where `B` is that parameter's declared bound. A wildcard whose bounds are outside that declared bound is rejected. `? super` is legal only as a type argument, directly after `?`. Elsewhere `super` is the superclass keyword. `in` and `out` are not written in a type argument.

Subtyping compares arguments in order:

- An invariant parameter requires the expected argument to contain the provided argument. A concrete type contains only itself. `? extends U` contains a type `S`, or `? extends S`, when `S` is a subtype of `U`. `? super U` contains a type `S`, or `? super S`, when `U` is a subtype of `S`. `?` contains every argument its bound allows.
- An `out` parameter also accepts a provided argument that is a subtype of a concrete expected argument. Every type contained in `Iterator<? extends Int32>` is a subtype of `Iterator<Number>`, so that wildcard type is too.
- An `in` parameter reverses the concrete direction. A provided `Consumer<Number>` is a subtype of an expected `Consumer<Int32>` when `Consumer` declares `in T`.

`List<String>` is a subtype of `List<?>`, of `List<? extends String>`, and of `List<? super String>`. It is not a subtype of `List<Object>`. `List<? extends Number>` is not a subtype of `List<Number>`. `List<List<String>>` is not a subtype of `List<List<?>>`. `List<List<String>>` is a subtype of `List<? extends List<?>>`.

`new` and an explicit method type argument require a concrete argument in that position. `new Box<?>()` and `Box.<?>of(1)` are rejected. A wildcard nested inside a concrete argument is legal: `new Box<List<?>>(items)`.

A class literal requires concrete type arguments. `List<String>.class` has type `Class<List<String>>` and denotes that instantiation. `List<?>.class` is rejected. Where a cast or `instanceof` type contains a wildcard, the test is not a class-literal send. The runtime class must be the named class or a subclass, projected onto that class's parameters. A concrete argument matches only the same argument. A wildcard argument matches a reified argument inside its bounds. `(List<String>) items` on a `List<?>` raises `ClassCastException` unless the reified argument is `String`. `(List<?>) items` on a `List<String>` succeeds. `instanceof List<?>` is true for every `List` instantiation.

Capture is the checker's internal name for a wildcard. Using an expression whose type contains a wildcard replaces each wildcard in that use by a fresh type variable with the wildcard's bounds. The variable is not a name in source. One expression produces one capture, shared by that expression's immediate use. A second expression produces a different capture.

The capture is what a send sees. On `List<? extends Number>`, a method that returns the parameter returns the capture, and the result is usable as `Number`. A method that accepts the parameter requires the capture, so a `Number` argument is rejected. On `List<? super Int32>`, a method that accepts the parameter accepts an `Int32`, and a method that returns the parameter returns the capture, usable as the parameter's upper bound.

A generic method gives one capture a source name for its body. `<T> void twice(List<T> items)` accepts a `List<?>`, and `T` is that argument's capture. Inside `twice`, both uses of the parameter have the same `T`. `<T> void pair(List<T> a, List<T> b)` rejects two separate `List<?>` arguments, because each argument captures on its own. The program passes those values through one type parameter when they are the same instantiation.

Inference does not invent a wildcard. It captures an argument's wildcards before solving the method's type parameters. An annotation constraint is not satisfied by a bare wildcard. `Box<?>` is rejected when `Box` declares `<@Serializable T>`. `Box<? extends S>` is legal when `S` already carries that constraint.

## 7.3 Instantiation and inference

A method type parameter is inferred when every parameter is determined by one of these:

- an argument whose type is not a bare numeric literal and which names the parameter in the corresponding parameter type, producing exactly one solution under subtyping
- an expected type of the call that names the parameter in the result type
- a bare numeric literal whose corresponding parameter type is a fixed numeric type, in which case the literal adopts that type when it fits

If a parameter is not determined, or two applicable methods remain after [chapter 4](04-methods.md) resolution, the call is rejected. The program writes the arguments: `Box.<Int32>of(1)`. Because `Class` is `out`, `Class<Int32>` is a subtype of `Class<Number>`. For `Vector<T>.map`, the argument `Int32.class` is a solution for `U=Int32` and for `U=Number`. With no expected type those are two solutions, and `lanes.map(Int32.class)` is rejected. `lanes.<Int32>map(Int32.class)` writes `U`. An expected type `Vector<Int32>` determines `U` as `Int32`. `lanes.map(Number.class)` has the one solution `U=Number`.

Explicit type arguments on a method are written immediately after the `.` of the call, before the method name. Explicit type arguments on a constructor are written on the class: `new Box<Int32>(1)`.

A value-type instantiation whose arguments are all concrete is monomorphized. `Vector<Int32>` has the layout [chapter 6](06-numbers.md) gives it. A wildcard argument is not a layout. The value keeps the representation of its runtime instantiation, and the static type follows [section 7.2](#72-use-site-wildcards). A reference instantiation with concrete arguments shares the reference representation. The static rules do not change with the representation.

A static call whose receiver is a type parameter is legal when the parameter's bound declares that static method. After instantiation it resolves on the class of the type argument, not on the bound. The lane conversion inside `Vector<T>.map` is the send `U.from`. That send is `Int32.from` when `U` is `Int32`, and `Number.from` when `U` is `Number`. `Number.from` returns its argument.

## 7.4 `Class`

```java
public final class Class<out T> {
    public Boolean isInstance(Object value)
    public Boolean has(Class<Annotation> annotation)
    public String getName()
    public @Nullable Class<?> getSuperclass()
    public Class<?>[] getInterfaces()
    public Boolean isValue()
    public Boolean isInterface()
    public Boolean isAnnotation()
}
```

`out` is legal because none of these methods produces a `T`. `Class<String>` is a subtype of `Class<Object>`. `has` accepts `Serializable.class` when `Serializable` implements `Annotation`, because `Class<Serializable>` is then a subtype of `Class<Annotation>`.

For a receiver whose static type is `C` or `@Nullable C`, the result type of `getClass` is `Class<C>`. This is a property of that one method. It is not a general covariant-return rule. The runtime class object is the instance's actual class. A class literal `C.class` has type `Class<C>` and does not initialize `C`.

`isInstance` tests the runtime class and, for a generic class, the reified arguments. A wildcard argument in the tested type matches any reified argument inside that wildcard's bounds. A concrete argument matches only itself. `isInstance` returns `false` for `null` unless the tested class is `Null`.

`getName` returns the binary name: the qualified name, with `.` between packages and the type, and with type arguments rendered in source order inside `<>`. A nested type does not occur. `getSuperclass` returns the class object of the superclass, or `null` when the receiver is `Object.class` or an interface. The result is `Class<?>` because the superclass is not a type parameter of `Class`. `getInterfaces` returns the interfaces the class declares, in source order, as `Class<?>`. `isValue`, `isInterface`, and `isAnnotation` report the kind of declaration. There is no reflective invocation. A program that has a `Class` object and no instance cannot send an arbitrary member through `Class`.

## 7.5 Annotation constraints on type parameters

A declaration annotation on a type parameter is a constraint on the instantiating class. `<@Serializable T>` requires the class of the type argument to carry `Serializable` as [chapter 8](08-annotations.md) defines retention. `<@Serializable T extends Number>` requires both the bound and the annotation. The check is performed at instantiation. A type argument that is itself a type parameter must carry the same annotation constraint.

`value.getClass().has(Serializable.class)` is the runtime form of that test. It does not replace the static check.

## 7.6 Arrays and generics

`T[]` is reified in `T`. `instanceof String[]` is false for an `Object[]`. Generic array creation `new T[n]` inside a generic method is legal because `T` is reified, and it is rejected when [chapter 6](06-numbers.md) rejects `new T[n]` for lack of a default element.

## 7.7 Override and erasure

Signatures are compared with their type arguments, not after erasure. There is no bridge method. A class implements an interface method by declaring the same name, the same type parameters, and the same parameter and result types. Two methods that would have collided after erasure and differ in their arguments are distinct and both exist.

A superclass method and an interface method of the same signature are one method. The class implements it once.
