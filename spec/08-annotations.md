# 8. Annotations

## 8.1 Annotations are modifiers

An annotation is a modifier whose name a program can declare. It is written before the thing it modifies, as `final` and `public` are. There are two kinds.

- A qualifier modifies a type, and it changes what the compiler accepts for values of that type ([section 8.3](#83-qualifiers)).
- A declaration annotation marks a type declaration, a field, a method, a constructor or a parameter. The program can read it at run time ([section 8.9](#89-reading-annotations)), and that is how an annotation takes effect when a program or a library declares it. The declaration annotations of the prelude also have rules that the compiler applies ([section 8.8](#88-the-preludes-declaration-annotations)).

A program declares annotations of both kinds. [Section 8.10](#810-the-preludes-annotations) lists the ones the prelude declares.

No annotation runs code while the program is compiled. An annotation cannot add a member to a class or change the body of a method.

## 8.2 Declaring and writing an annotation

```java
@Refines
public annotation Positive;

@Target(Site.METHOD)
public annotation Route(String path, Int priority = 0);
```

An annotation declaration is a type declaration under the file rules of [chapter 1](01-source.md) and the audiences of [chapter 3](03-visibility.md). `@Positive` is legal wherever the name `Positive` is visible.

**Elements.** A declaration may list elements in a header. It has no body and no type parameters. The type of an element is a numeric class, `Boolean`, `Char`, `String`, an enum, `Class`, an annotation, or an array of one of those, with no qualifier. An element may declare a default. A qualifier has no elements.

**The value.** An annotation declaration declares a final value class that implements the prelude interface `Annotation`. Its elements are its fields, and each is public. A program does not construct an instance with `new`. An instance comes only from a use of the annotation, and [section 8.9](#89-reading-annotations) says how a program obtains it. Only annotations implement `Annotation`: a class or an interface that names it as a supertype is rejected.

**Arguments.** A use supplies its arguments in element order, as in `@Route("/users", 2)`, or all of them by element name, as in `@Route(path = "/users")`. An argument is a constant expression of [chapter 12](12-flow.md), which includes an enum constant, or a class literal, or a use of an annotation. For an array element it is a list of those in braces, and a single one may be written without the braces. An omitted element takes its default, and an element with no default must be supplied. A use with no arguments writes no parentheses.

**Position.** An annotation is written in one of two places.

- A declaration annotation is written before the modifiers of the declaration it marks: a class, an interface, an enum or an annotation declaration, a field or an enum constant, a method, a constructor, or a parameter.
- A qualifier is written directly before the type it qualifies: `@Positive Int`, `List<@Positive Int>`. With an array type it qualifies the element type. `Int @Positive []` qualifies the array.

A qualifier may also be written among the modifiers of a field, a method, a parameter or a local. It then means what it would mean directly before that declaration's type, which for a method is its result type. `@Nullable public String find(String key)` and `public @Nullable String find(String key)` are the same declaration.

An annotation is written at most once in one position.

**Targets.** `@Target` on the declaration of a declaration annotation lists the sites where it may be written. A use at any other site is rejected. The sites are the constants of the prelude enum `Site`:

| Site | Declaration |
| --- | --- |
| `TYPE` | A class, an interface or an enum |
| `ANNOTATION` | An annotation |
| `FIELD` | A field or an enum constant |
| `METHOD` | A method |
| `CONSTRUCTOR` | A constructor |
| `PARAMETER` | A parameter of a method or a constructor |

A declaration annotation with no `@Target` may be written at every site. A qualifier is written on types, and `@Target` on its declaration is rejected.

## 8.3 Qualifiers

A qualifier is an annotation declared with exactly one of two meta-annotations.

- `@Refines` declares a refinement. `@Q T` is a subtype of `T`: a value known to have the property may be used wherever a `T` is expected, and a plain `T` may not be used where `@Q T` is expected. `@Refines(R.class)` also places `Q` below the refinement `R`, so `@Q T` is a subtype of `@R T`. Several refinements may be listed. A cycle is rejected.
- `@Widens` declares a widening. `T` is a subtype of `@Q T`, and `@Q T` is not a subtype of `T`: a value that has not been shown to belong in `T` may not be used as one.

A type may carry several qualifiers. A qualifier applied to a type that already has it leaves the type unchanged, and so does a refinement applied to a type that has a refinement below it.

`S` is a subtype of `T` when that holds for the two types with their qualifiers removed, and both of these hold:

- every refinement on `T` is also on `S`, or is above a refinement on `S`;
- every widening on `S` is also on `T`.

Qualifiers are part of the types in a signature. An override repeats them. Parameter types that differ only in qualifiers do not distinguish overloads. `var` infers a type with its qualifiers.

A literal has no refinement, and neither has the result of `new`. A value obtains a refinement only by the narrowing of [section 8.5](#85-narrowing), or from a declaration whose type already has it.

**Type arguments.** Qualifiers are part of a type argument. An invariant type parameter matches two arguments only when they have the same qualifiers. A parameter declared `out` or `in` uses the subtype rule above, and so does the bound of a wildcard. The qualifiers of a type argument are part of the reified argument: `List<Int>` and `List<@Positive Int>` are different classes at run time, and a cast or an `instanceof` tells them apart.

**Values.** A value does not carry its own qualifiers at run time. A cast or an `instanceof` whose type has a qualifier outside every type argument is therefore rejected: `(@Positive Int) n` is rejected, and `(List<@Positive Int>) o` is legal. [Section 8.6](#86-nullable) gives the one exception.

## 8.4 Receivers

A method may declare its receiver as a first parameter named `this`. Its type is the declaring class, with qualifiers:

```java
public Rational unitPrice(@Priced Order this)
```

Here `Priced` is a refinement the program declares, and `unitPrice` may be sent only to an `Order` known to have it. A method with no receiver parameter has the declaring class, without qualifiers, as its receiver type.

A send is legal only when the static type of the receiver is a subtype of the method's receiver type. Three consequences follow from [section 8.3](#83-qualifiers):

- Every method can be sent to a receiver that has a refinement.
- A method whose receiver type has a refinement can be sent only to a receiver known to have it.
- A receiver whose type has a widening accepts only the methods that declare that widening on their receiver.

An override has the same receiver qualifiers as the method it overrides. The exception is an override of `equals`, `hashCode` or `toString`, whose receiver has no qualifier, as [chapter 5](05-null-and-unit.md) explains.

## 8.5 Narrowing

`@Narrows(Q.class)` marks a method whose result is `Boolean`. The subject of the method is its receiver, or its first parameter when the method is static. A `true` result narrows the subject:

- when `Q` is a refinement, the subject gains `Q`;
- when `Q` is a widening, the subject loses `Q`.

A `false` result narrows nothing.

Narrowing follows the path rule of [chapter 12](12-flow.md#124-narrowing). A local or parameter is narrowed at a point when every path to that point passes such a call with the local as its subject and a `true` result, and does not assign the local afterwards. A field, an array element, and any other expression that is evaluated again are not narrowed.

A `@Narrows(Q.class)` method is declared in the package that declares `Q`. That package decides what `Q` means. Code outside it can gain a refinement, or shed a widening, only through those methods.

```java
@Refines
public annotation Positive;

public class Counts {
    @Narrows(Positive.class)
    public static Boolean isPositive(Int n) {
        return n > 0;
    }
}
```

```java
static Rational share(Rational total, @Positive Int people) {
    return total / people;                     // no zero check: the type rules zero out
}
```

```java
if (Counts.isPositive(n)) {
    Console.println("each pays " + share(total, n));   // n is a @Positive Int here
}
```

A qualifier must describe a fact that stays true of a value for as long as the value exists. The language does not check this. The author of a `@Narrows` method is responsible for it, as the author of `equals` is responsible for the obligations of [chapter 2](02-objects.md). A fact about a number, a `String`, or an instance of a value class cannot change, so a qualifier on those types meets the obligation.

## 8.6 `@Nullable`

`@Nullable` is declared in the prelude as a widening:

```java
@Widens
public annotation Nullable;
```

The subtype rules of [chapter 5](05-null-and-unit.md) are those of [section 8.3](#83-qualifiers), and its rule for sends on a nullable receiver is the receiver rule of [section 8.4](#84-receivers).

`@Nullable` differs from a qualifier a program declares in two ways. Both follow from `null` being built into the language.

- **It adds a value.** `@Nullable T` contains `null`. No other qualifier adds a value to a type. For that reason a cast may name `@Nullable` outside a type argument: `(@Nullable String) e` accepts `null`, and `(String) e` rejects it.
- **Its test is built in.** `e == null` and `e != null` narrow without a `@Narrows` method, and they narrow on both outcomes.

## 8.7 Tags

A declaration annotation written on a class, an interface or an enum is a tag of that type. A tag is a claim by the author of the declaration.

```java
@Target(Site.TYPE)
public annotation Frozen;

@Frozen
public value class Bill { }
```

**Requiring a tag.** A type parameter may require a tag:

```java
public class Snapshot<@Frozen T> { }
```

A type argument for `T` must be a type whose declaration carries `Frozen`, or a type parameter that requires `Frozen` itself. `Snapshot<Bill>` is legal. `Snapshot<StringBuilder>` is rejected when `StringBuilder` does not carry the tag. The requirement names the annotation and no arguments, and a use with any arguments meets it. [Chapter 7](07-generics.md) gives the rule for instantiation.

**Inheriting a tag.** A tag declared with `@Inherited` is also carried by every declaration that extends or implements a marked one. A use written on the declaration itself takes the place of an inherited one. A declaration that would inherit two uses with different arguments, from two of its supertypes, is rejected unless it writes the annotation itself. Without `@Inherited`, a subclass carries the tag only when it is written there.

## 8.8 The prelude's declaration annotations

`@Override` marks a method that overrides a superclass method or implements an interface method. A method that does either must carry it. A method that carries it must do one of them. Both failures are rejected.

`@Discardable` marks a method whose result a caller may ignore. A call of it may stand alone as a statement, as [chapter 4](04-methods.md) says. Without it, a result other than `Unit` must be used.

`@Implicit` marks a static method that the compiler may call on its own to convert a value. The method is declared in a value class `T`. It has one parameter, whose type is another value class `S`, the result type `T`, and no type parameters. It declares an implicit conversion from `S` to `T`, which [chapter 4](04-methods.md#45-assignability) applies where an `S` is assigned to a `T` and between the operands of an operator.

```java
public value class Meters {
    public Rational length;

    @Implicit
    public static Meters from(Int whole) {
        return new Meters(whole);
    }
}
```

```java
Meters height = 3;      // Meters.from(3)
```

The author of an `@Implicit` method promises that the conversion is exact: it returns for every argument, never raises, and loses no information. The language does not check the promise. A conversion that can fail or lose information is not marked `@Implicit`. The program calls it by name, and by the convention the numeric classes follow it raises an exception when the value has no exact counterpart.

`@Intrinsic` marks a method or a constructor of the prelude whose body the implementation supplies, such as integer addition or an array access. The declaration ends in `;` where its body would be. `@Intrinsic` is rejected outside the package `cleat`.

`@Deprecated` marks a type, a field, a method or a constructor. An implementation must report each use of a deprecated declaration from outside the compilation unit that declares it, with the message when there is one. The report does not reject the program.

## 8.9 Reading annotations

A program reads a declaration annotation at run time. It starts from the class object of a type, which [chapter 2](02-objects.md#27-static-members-and-class-objects) describes, and reaches the type's annotated fields and methods through mirrors.

```java
public final class Class {
    public String getName()
    public @Nullable Class getSuperclass()
    public Class[] getInterfaces()
    public <A extends Annotation> @Nullable A getAnnotation()
    public <A extends Annotation> Field[] getAnnotatedFields()
    public <A extends Annotation> Method[] getAnnotatedMethods()
}

public value class Field {
    public String getName()
    public Boolean isStatic()
    public <A extends Annotation> @Nullable A getAnnotation()
    public @Nullable Object get(@Nullable Object target)
    public void set(@Nullable Object target, @Nullable Object value)
}

public value class Method {
    public String getName()
    public Boolean isStatic()
    public Parameter[] getParameters()
    public <A extends Annotation> @Nullable A getAnnotation()
    @Discardable
    public @Nullable Object invoke(@Nullable Object target, @Nullable Object... arguments)
}

public value class Parameter {
    public String getName()
    public <A extends Annotation> @Nullable A getAnnotation()
}
```

**Finding annotations.** The type argument `A` names the annotation that a call asks for.

- `getAnnotation` returns the use of `A` on the declaration that the receiver stands for, or `null` when there is none. On a class object it includes a tag the type inherits under [section 8.7](#87-tags). Every instantiation of a generic type returns the same result.
- `getAnnotatedFields` returns a mirror for each field that the type declares with a use of `A`, in source order. Static fields and enum constants are included.
- `getAnnotatedMethods` returns a mirror for each method that the type declares with a use of `A`, in source order.
- `getSuperclass` returns the class object of the superclass. It returns `null` for `Object` and for an interface. `getInterfaces` returns the class objects of the interfaces that the declaration names, in the order written. A reader that wants the annotated members a type inherits follows these.
- `getParameters` returns a mirror for each parameter of the method, in order.

A member that carries no annotation has no mirror. A program reaches a member this way only through an annotation that the member's author wrote on it. A qualifier belongs to a type and is not read this way.

Each call returns a new annotation value, and an array element of it is a new array. A program that modifies such an array changes no later result.

**Using a mirror.** `get` reads the field, and `set` assigns it. `invoke` calls the method with the arguments given, dispatching as [chapter 2](02-objects.md#210-member-lookup) dispatches a call, and returns the result. For a `void` method that is the `Unit` value. `target` is the instance, and it is not used for a static member. An exception that the method raises propagates unchanged.

A mirror checks what the compiler would have checked for a direct use:

| Condition | Raises |
| --- | --- |
| The member is not `public`, or `set` names a final field | `IllegalAccessException` |
| `target` is not an instance of the declaring type, or a value or an argument is not of the type the member declares | `ClassCastException` |
| The number of arguments is not the number of parameters, or the method is generic | `IllegalArgumentException` |

The type test is the test of a cast. No implicit conversion applies, so an `Int32` is not accepted for an `Int` parameter.

```java
@Target(Site.METHOD)
public annotation Check(String name);

static void runChecks(Object suite) {
    for (Method method : suite.getClass().<Check>getAnnotatedMethods()) {
        Check check = (Check) method.<Check>getAnnotation();
        try {
            method.invoke(suite);
            Console.println("pass: " + check.name);
        } catch (Throwable failure) {
            Console.println("FAIL: " + check.name + ": " + failure.message());
        }
    }
}
```

**Limits.** Mirrors read annotations and use the members that carry them. They do not describe the type of a field or a parameter, list a member that has no annotation, construct an object, or enumerate the types of a program.

## 8.10 The prelude's annotations

| Annotation | Written on | Rule |
| --- | --- | --- |
| `@Refines` | An annotation declaration | [Section 8.3](#83-qualifiers) |
| `@Widens` | An annotation declaration | [Section 8.3](#83-qualifiers) |
| `@Target` | An annotation declaration | [Section 8.2](#82-declaring-and-writing-an-annotation) |
| `@Inherited` | An annotation declaration | [Section 8.7](#87-tags) |
| `@Narrows` | A method | [Section 8.5](#85-narrowing) |
| `@Nullable` | A type use | [Section 8.6](#86-nullable) and [chapter 5](05-null-and-unit.md) |
| `@Override` | A method | [Section 8.8](#88-the-preludes-declaration-annotations) |
| `@Discardable` | A method | [Section 8.8](#88-the-preludes-declaration-annotations) |
| `@Implicit` | A static method of a value class | [Section 8.8](#88-the-preludes-declaration-annotations) |
| `@Deprecated` | A type, field, method or constructor | [Section 8.8](#88-the-preludes-declaration-annotations) |
| `@Intrinsic` | A method or constructor of the prelude | [Section 8.8](#88-the-preludes-declaration-annotations) |
| `@Symbol` | A `foreign` method | [Chapter 4](04-methods.md#411-foreign-methods) |

These declarations are ordinary prelude source, and a program reads the declaration annotations among them as it reads its own. Their rules are the compiler's. An annotation that a program declares has the rules of this chapter and no others: where it may be written, whether a type parameter requires it, and whether it is inherited. What it means beyond that is decided by the code that reads it.
