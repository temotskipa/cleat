# 8. Annotations

## 8.1 Annotations are modifiers

An annotation is a modifier whose name a program can declare. <!-- r[annotations.modifiers.annotation-modifier-name] -->
It is written before the thing it modifies, as `final` and `public` are. <!-- r[annotations.modifiers.written-before-thing] -->
There are two kinds. <!-- r[annotations.modifiers.kinds] -->

- A qualifier modifies a type, and it changes what the compiler accepts for values of that type ([section 8.3](#83-qualifiers)). <!-- r[annotations.modifiers.qualifier-modifies-type] -->
- A declaration annotation marks a type declaration, a field, a method, a constructor or a parameter. <!-- r[annotations.modifiers.declaration-annotation-marks] -->
  The program can read it at run time ([section 8.9](#89-reading-annotations)), and that is how an annotation takes effect when a program or a library declares it. <!-- r[annotations.modifiers.program-read-run] -->
  The declaration annotations of the prelude also have rules that the compiler applies ([section 8.8](#88-the-preludes-declaration-annotations)). <!-- r[annotations.modifiers.declaration-annotations-prelude] -->

A program declares annotations of both kinds. <!-- r[annotations.modifiers.program-declares-annotations] -->
[Section 8.10](#810-the-preludes-annotations) lists the ones the prelude declares. <!-- r[annotations.modifiers.lists-ones-prelude] -->

No annotation runs code while the program is compiled. <!-- r[annotations.modifiers.no-annotation-runs] -->
An annotation cannot add a member to a class or change the body of a method. <!-- r[annotations.modifiers.annotation-add-member] -->

## 8.2 Declaring and writing an annotation

```java
@Refines
public annotation Positive;

@Target(Site.METHOD)
public annotation Route(String path, Int priority = 0);
```

An annotation declaration is a type declaration under the file rules of [chapter 1](01-source.md) and the audiences of [chapter 3](03-visibility.md). <!-- r[annotations.declaring.annotation-declaration-type] -->
`@Positive` is legal wherever the name `Positive` is visible. <!-- r[annotations.declaring.positive-legal-wherever] -->

**Elements.** A declaration may list elements in a header. <!-- r[annotations.declaring.elements-declaration-list] -->
It has no body and no type parameters. <!-- r[annotations.declaring.no-body-type] -->
The type of an element is a numeric class, `Boolean`, `Char`, `String`, an enum, `Class`, an annotation, or an array of one of those, with no qualifier. <!-- r[annotations.declaring.type-element-numeric] -->
An element may declare a default. <!-- r[annotations.declaring.element-declare-default] -->
A qualifier has no elements. <!-- r[annotations.declaring.qualifier-no-elements] -->

**The value.** An annotation declaration declares a final value class that implements the prelude interface `Annotation`. <!-- r[annotations.declaring.value-annotation-declaration] -->
Its elements are its fields, and each is public. <!-- r[annotations.declaring.elements-fields-public] -->
A program does not construct an instance with `new`. <!-- r[annotations.declaring.program-not-construct] -->
An instance comes only from a use of the annotation, and [section 8.9](#89-reading-annotations) says how a program obtains it. <!-- r[annotations.declaring.instance-comes-only] -->
Only annotations implement `Annotation`: a class or an interface that names it as a supertype is rejected. <!-- r[annotations.declaring.only-annotations-implement] -->

**Arguments.** A use supplies its arguments in element order, as in `@Route("/users", 2)`, or all of them by element name, as in `@Route(path = "/users")`. <!-- r[annotations.declaring.arguments-use-supplies] -->
An argument is a constant expression of [chapter 12](12-flow.md), which includes an enum constant, or a class literal, or a use of an annotation. <!-- r[annotations.declaring.argument-constant-expression] -->
For an array element it is a list of those in braces, and a single one may be written without the braces. <!-- r[annotations.declaring.array-element-list] -->
An omitted element takes its default, and an element with no default must be supplied. <!-- r[annotations.declaring.omitted-element-takes] -->
A use with no arguments writes no parentheses. <!-- r[annotations.declaring.use-no-arguments] -->

**Position.** An annotation is written in one of two places. <!-- r[annotations.declaring.position-annotation-written] -->

- A declaration annotation is written before the modifiers of the declaration it marks: a class, an interface, an enum or an annotation declaration, a field or an enum constant, a method, a constructor, or a parameter. <!-- r[annotations.declaring.declaration-annotation-written] -->
- A qualifier is written directly before the type it qualifies: `@Positive Int`, `List<@Positive Int>`. <!-- r[annotations.declaring.qualifier-written-directly] -->
  With an array type it qualifies the element type. <!-- r[annotations.declaring.array-type-qualifies] -->
  `Int @Positive []` qualifies the array. <!-- r[annotations.declaring.int-positive-qualifies] -->

A qualifier may also be written among the modifiers of a field, a method, a parameter or a local. <!-- r[annotations.declaring.qualifier-written-among] -->
It then means what it would mean directly before that declaration's type, which for a method is its result type. <!-- r[annotations.declaring.means-mean-directly] -->
`@Nullable public String find(String key)` and `public @Nullable String find(String key)` are the same declaration. <!-- r[annotations.declaring.nullable-public-string] -->

An annotation is written at most once in one position. <!-- r[annotations.declaring.annotation-written-most] -->
A declaration's modifiers and the annotations directly before its type are one position, and so are the annotations directly before any other type or before one pair of brackets. <!-- r[annotations.declaring.declaration-modifiers-annotations] -->

**Targets.** `@Target` on the declaration of a declaration annotation lists the sites where it may be written. <!-- r[annotations.declaring.targets-target-declaration] -->
A use at any other site is rejected. <!-- r[annotations.declaring.use-site-rejected] -->
The sites are the constants of the prelude enum `Site`: <!-- r[annotations.declaring.sites-constants-prelude] -->

| Site | Declaration <!-- r[annotations.declaring.table] --> |
| --- | --- |
| `TYPE` | A class, an interface or an enum |
| `ANNOTATION` | An annotation |
| `FIELD` | A field or an enum constant |
| `METHOD` | A method |
| `CONSTRUCTOR` | A constructor |
| `PARAMETER` | A parameter of a method or a constructor |

A declaration annotation with no `@Target` may be written at every site. <!-- r[annotations.declaring.declaration-annotation-no] -->
A qualifier is written on types, and `@Target` on its declaration is rejected. <!-- r[annotations.declaring.qualifier-written-types] -->

## 8.3 Qualifiers

A qualifier is an annotation declared with exactly one of two meta-annotations. <!-- r[annotations.qualifiers.qualifier-annotation-declared] -->

- `@Refines` declares a refinement. <!-- r[annotations.qualifiers.refines-declares-refinement] -->
  `@Q T` is a subtype of `T`: a value known to have the property may be used wherever a `T` is expected, and a plain `T` may not be used where `@Q T` is expected. <!-- r[annotations.qualifiers.subtype-value-known] -->
  `@Refines(R.class)` also places `Q` below the refinement `R`, so `@Q T` is a subtype of `@R T`. <!-- r[annotations.qualifiers.refines-class-places] -->
  Several refinements may be listed. <!-- r[annotations.qualifiers.several-refinements-listed] -->
  A cycle is rejected. <!-- r[annotations.qualifiers.cycle-rejected] -->
- `@Widens` declares a widening. <!-- r[annotations.qualifiers.widens-declares-widening] -->
  `T` is a subtype of `@Q T`, and `@Q T` is not a subtype of `T`: a value that has not been shown to belong in `T` may not be used as one. <!-- r[annotations.qualifiers.subtype-not-value] -->

A type may carry several qualifiers. <!-- r[annotations.qualifiers.type-carry-several] -->
A qualifier applied to a type that already has it leaves the type unchanged, and so does a refinement applied to a type that has a refinement below it. <!-- r[annotations.qualifiers.qualifier-applied-type] -->

`S` is a subtype of `T` when that holds for the two types with their qualifiers removed, and both of these hold: <!-- r[annotations.qualifiers.subtype-holds-types] -->

- every refinement on `T` is also on `S`, or is above a refinement on `S`; <!-- r[annotations.qualifiers.refinement-above] -->
- every widening on `S` is also on `T`. <!-- r[annotations.qualifiers.widening] -->

Qualifiers are part of the types in a signature. <!-- r[annotations.qualifiers.qualifiers-part-types] -->
An override repeats them. <!-- r[annotations.qualifiers.override-repeats] -->
Parameter types that differ only in qualifiers do not distinguish overloads. <!-- r[annotations.qualifiers.parameter-types-differ] -->
`var` infers a type with its qualifiers. <!-- r[annotations.qualifiers.var-infers-type] -->

A literal has no refinement, and neither has the result of `new`. <!-- r[annotations.qualifiers.literal-no-refinement] -->
A value obtains a refinement only by the narrowing of [section 8.5](#85-narrowing), or from a declaration whose type already has it. <!-- r[annotations.qualifiers.value-obtains-refinement] -->

**Type arguments.** Qualifiers are part of a type argument. <!-- r[annotations.qualifiers.type-arguments-qualifiers] -->
An invariant type parameter matches two arguments only when they have the same qualifiers. <!-- r[annotations.qualifiers.invariant-type-parameter] -->
A parameter declared `out` or `in` uses the subtype rule above, and so does the bound of a wildcard. <!-- r[annotations.qualifiers.parameter-declared-out] -->
The qualifiers of a type argument are part of the reified argument: `List<Int>` and `List<@Positive Int>` are different classes at run time, and a cast or an `instanceof` tells them apart. <!-- r[annotations.qualifiers.qualifiers-type-argument] -->

**Values.** A value does not carry its own qualifiers at run time. <!-- r[annotations.qualifiers.values-value-not] -->
A cast or an `instanceof` whose type has a qualifier outside every type argument is therefore rejected: `(@Positive Int) n` is rejected, and `(List<@Positive Int>) o` is legal. <!-- r[annotations.qualifiers.cast-instanceof-type] -->
[Section 8.6](#86-nullable) gives the one exception. <!-- r[annotations.qualifiers.gives-exception] -->

## 8.4 Receivers

A method may declare its receiver as a first parameter named `this`. <!-- r[annotations.receivers.method-declare-receiver] -->
Its type is the declaring class, with qualifiers: <!-- r[annotations.receivers.type-declaring-class] -->

```java
public Rational unitPrice(@Priced Order this)
```

Here `Priced` is a refinement the program declares, and `unitPrice` may be sent only to an `Order` known to have it. <!-- r[annotations.receivers.priced-refinement-program] -->
A method with no receiver parameter has the declaring class, without qualifiers, as its receiver type. <!-- r[annotations.receivers.method-no-receiver] -->

A send is legal only when the static type of the receiver is a subtype of the method's receiver type. <!-- r[annotations.receivers.send-legal-only] -->
Three consequences follow from [section 8.3](#83-qualifiers): <!-- r[annotations.receivers.three-consequences-follow] -->

- Every method can be sent to a receiver that has a refinement. <!-- r[annotations.receivers.method-sent-receiver] -->
- A method whose receiver type has a refinement can be sent only to a receiver known to have it. <!-- r[annotations.receivers.method-receiver-type] -->
- A receiver whose type has a widening accepts only the methods that declare that widening on their receiver. <!-- r[annotations.receivers.receiver-type-widening] -->

An override has the same receiver qualifiers as the method it overrides. <!-- r[annotations.receivers.override-receiver-qualifiers] -->
The exception is an override of `equals`, `hashCode` or `toString`, whose receiver has no qualifier, as [chapter 5](05-null-and-unit.md) explains. <!-- r[annotations.receivers.exception-override-equals] -->

## 8.5 Narrowing

`@Narrows(Q.class)` marks a method whose result is `Boolean`. <!-- r[annotations.narrowing.narrows-class-marks] -->
The subject of the method is its receiver, or its first parameter when the method is static. <!-- r[annotations.narrowing.subject-method-receiver] -->
A `true` result narrows the subject: <!-- r[annotations.narrowing.true-result-narrows] -->

- when `Q` is a refinement, the subject gains `Q`; <!-- r[annotations.narrowing.refinement-subject-gains] -->
- when `Q` is a widening, the subject loses `Q`. <!-- r[annotations.narrowing.widening-subject-loses] -->

A `false` result narrows nothing. <!-- r[annotations.narrowing.false-result-narrows] -->

Narrowing follows the path rule of [chapter 12](12-flow.md#124-narrowing). <!-- r[annotations.narrowing.narrowing-follows-path] -->
A local or parameter is narrowed at a point when every path to that point passes such a call with the local as its subject and a `true` result, and does not assign the local afterwards. <!-- r[annotations.narrowing.local-parameter-narrowed] -->
A field, an array element, and any other expression that is evaluated again are not narrowed. <!-- r[annotations.narrowing.field-array-element] -->

A `@Narrows(Q.class)` method is declared in the package that declares `Q`. <!-- r[annotations.narrowing.narrows-class-method] -->
That package decides what `Q` means. <!-- r[annotations.narrowing.package-decides-means] -->
Code outside it can gain a refinement, or shed a widening, only through those methods. <!-- r[annotations.narrowing.code-outside-gain] -->

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

A qualifier must describe a fact that stays true of a value for as long as the value exists. <!-- r[annotations.narrowing.qualifier-describe-fact] -->
The language does not check this. <!-- r[annotations.narrowing.language-not-check] -->
The author of a `@Narrows` method is responsible for it, as the author of `equals` is responsible for the obligations of [chapter 2](02-objects.md). <!-- r[annotations.narrowing.author-narrows-method] -->
A fact about a number, a `String`, or an instance of a value class cannot change, so a qualifier on those types meets the obligation. <!-- r[annotations.narrowing.fact-about-number] -->

## 8.6 `@Nullable`

`@Nullable` is declared in the prelude as a widening: <!-- r[annotations.nullable.nullable-declared-prelude] -->

```java
@Widens
public annotation Nullable;
```

The subtype rules of [chapter 5](05-null-and-unit.md) are those of [section 8.3](#83-qualifiers), and its rule for sends on a nullable receiver is the receiver rule of [section 8.4](#84-receivers). <!-- r[annotations.nullable.subtype-rules-rule] -->

`@Nullable` differs from a qualifier a program declares in two ways. <!-- r[annotations.nullable.nullable-differs-qualifier] -->
Both follow from `null` being built into the language. <!-- r[annotations.nullable.follow-null-built] -->

- **It adds a value.** `@Nullable T` contains `null`. <!-- r[annotations.nullable.adds-value-nullable] -->
  No other qualifier adds a value to a type. <!-- r[annotations.nullable.no-qualifier-adds] -->
  For that reason a cast may name `@Nullable` outside a type argument: `(@Nullable String) e` accepts `null`, and `(String) e` rejects it. <!-- r[annotations.nullable.reason-cast-name] -->
- **Its test is built in.** `e == null` and `e != null` narrow without a `@Narrows` method, and they narrow on both outcomes. <!-- r[annotations.nullable.test-built-equals] -->

## 8.7 Tags

A declaration annotation written on a class, an interface or an enum is a tag of that type. <!-- r[annotations.tags.declaration-annotation-written] -->
A tag is a claim by the author of the declaration. <!-- r[annotations.tags.tag-claim-author] -->

```java
@Target(Site.TYPE)
public annotation Frozen;

@Frozen
public value class Bill { }
```

**Requiring a tag.** A type parameter may require a tag: <!-- r[annotations.tags.requiring-tag-type] -->

```java
public class Snapshot<@Frozen T> { }
```

A type argument for `T` must be a type whose declaration carries `Frozen`, or a type parameter that requires `Frozen` itself. <!-- r[annotations.tags.type-argument-declaration] -->
`Snapshot<Bill>` is legal. <!-- r[annotations.tags.snapshot-bill-legal] -->
`Snapshot<StringBuilder>` is rejected when `StringBuilder` does not carry the tag. <!-- r[annotations.tags.snapshot-stringbuilder-rejected] -->
The requirement names the annotation and no arguments, and a use with any arguments meets it. <!-- r[annotations.tags.requirement-names-annotation] -->
[Chapter 7](07-generics.md) gives the rule for instantiation. <!-- r[annotations.tags.gives-rule-instantiation] -->

**Inheriting a tag.** A tag declared with `@Inherited` is also carried by every declaration that extends or implements a marked one. <!-- r[annotations.tags.inheriting-tag-declared] -->
A use written on the declaration itself takes the place of an inherited one. <!-- r[annotations.tags.use-written-declaration] -->
A declaration that would inherit two uses with different arguments, from two of its supertypes, is rejected unless it writes the annotation itself. <!-- r[annotations.tags.declaration-inherit-uses] -->
Without `@Inherited`, a subclass carries the tag only when it is written there. <!-- r[annotations.tags.without-inherited-subclass] -->

## 8.8 The prelude's declaration annotations

`@Override` marks a method that overrides a superclass method or implements an interface method. <!-- r[annotations.declaration.override-marks-method] -->
A method that does either must carry it. <!-- r[annotations.declaration.method-carry] -->
A method that carries it must do one of them. <!-- r[annotations.declaration.method-carries] -->
Both failures are rejected. <!-- r[annotations.declaration.failures-rejected] -->

`@Discardable` marks a method whose result a caller may ignore. <!-- r[annotations.declaration.discardable-marks-method] -->
A call of it may stand alone as a statement, as [chapter 4](04-methods.md) says. <!-- r[annotations.declaration.call-stand-alone] -->
Without it, a result other than `Unit` must be used. <!-- r[annotations.declaration.without-result-unit] -->

`@Implicit` marks a static method that the compiler may call on its own to convert a value. <!-- r[annotations.declaration.implicit-marks-static] -->
The method is declared in a value class `T`. <!-- r[annotations.declaration.method-declared-value] -->
It has one parameter, whose type is another value class `S`, the result type `T`, and no type parameters. <!-- r[annotations.declaration.parameter-type-value] -->
It declares an implicit conversion from `S` to `T`, which [chapter 4](04-methods.md#45-assignability) applies where an `S` is assigned to a `T` and between the operands of an operator. <!-- r[annotations.declaration.declares-implicit-conversion] -->

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

The author of an `@Implicit` method promises that the conversion is exact: it returns for every argument, never raises, and loses no information. <!-- r[annotations.declaration.author-implicit-method] -->
The language does not check the promise. <!-- r[annotations.declaration.language-not-check] -->
A conversion that can fail or lose information is not marked `@Implicit`. <!-- r[annotations.declaration.conversion-fail-lose] -->
The program calls it by name, and by the convention the numeric classes follow it raises an exception when the value has no exact counterpart. <!-- r[annotations.declaration.program-calls-name] -->

`@Intrinsic` marks a method or a constructor of the prelude whose body the implementation supplies, such as integer addition or an array access. <!-- r[annotations.declaration.intrinsic-marks-method] -->
The declaration ends in `;` where its body would be. <!-- r[annotations.declaration.declaration-ends-body] -->
`@Intrinsic` is rejected outside the package `cleat`. <!-- r[annotations.declaration.intrinsic-rejected-outside] -->

`@Deprecated` marks a type, a field, a method or a constructor. <!-- r[annotations.declaration.deprecated-marks-type] -->
An implementation must report each use of a deprecated declaration from outside the compilation unit that declares it, with the message when there is one. <!-- r[annotations.declaration.implementation-report-use] -->
The report does not reject the program. <!-- r[annotations.declaration.report-not-reject] -->

## 8.9 Reading annotations

A program reads a declaration annotation at run time. <!-- r[annotations.reading.program-reads-declaration] -->
It starts from the class object of a type, which [chapter 2](02-objects.md#27-static-members-and-class-objects) describes, and reaches the type's annotated fields and methods through mirrors. <!-- r[annotations.reading.starts-class-object] -->

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

**Finding annotations.** The type argument `A` names the annotation that a call asks for. <!-- r[annotations.reading.finding-annotations-type] -->

- `getAnnotation` returns the use of `A` on the declaration that the receiver stands for, or `null` when there is none. <!-- r[annotations.reading.getannotation-returns-use] -->
  On a class object it includes a tag the type inherits under [section 8.7](#87-tags). <!-- r[annotations.reading.class-object-includes] -->
  Every instantiation of a generic type returns the same result. <!-- r[annotations.reading.instantiation-generic-type] -->
- `getAnnotatedFields` returns a mirror for each field that the type declares with a use of `A`, in source order. <!-- r[annotations.reading.getannotatedfields-returns-mirror] -->
  Static fields and enum constants are included. <!-- r[annotations.reading.static-fields-enum] -->
- `getAnnotatedMethods` returns a mirror for each method that the type declares with a use of `A`, in source order. <!-- r[annotations.reading.getannotatedmethods-returns-mirror] -->
- `getSuperclass` returns the class object of the superclass. <!-- r[annotations.reading.getsuperclass-returns-class] -->
  It returns `null` for `Object` and for an interface. <!-- r[annotations.reading.returns-null-object] -->
  `getInterfaces` returns the class objects of the interfaces that the declaration names, in the order written. <!-- r[annotations.reading.getinterfaces-returns-class] -->
  A reader that wants the annotated members a type inherits follows these. <!-- r[annotations.reading.reader-wants-annotated] -->
- `getParameters` returns a mirror for each parameter of the method, in order. <!-- r[annotations.reading.getparameters-returns-mirror] -->

A member that carries no annotation has no mirror. <!-- r[annotations.reading.member-carries-no] -->
A program reaches a member this way only through an annotation that the member's author wrote on it. <!-- r[annotations.reading.program-reaches-member] -->
A qualifier belongs to a type and is not read this way. <!-- r[annotations.reading.qualifier-belongs-type] -->

Each call returns a new annotation value, and an array element of it is a new array. <!-- r[annotations.reading.call-returns-new] -->
A program that modifies such an array changes no later result. <!-- r[annotations.reading.program-modifies-array] -->

**Using a mirror.** `get` reads the field, and `set` assigns it. <!-- r[annotations.reading.using-mirror-get] -->
`invoke` calls the method with the arguments given, dispatching as [chapter 2](02-objects.md#210-member-lookup) dispatches a call, and returns the result. <!-- r[annotations.reading.invoke-calls-method] -->
For a `void` method that is the `Unit` value. <!-- r[annotations.reading.void-method-unit] -->
`target` is the instance, and it is not used for a static member. <!-- r[annotations.reading.target-instance-not] -->
An exception that the method raises propagates unchanged. <!-- r[annotations.reading.exception-method-raises] -->

A mirror checks what the compiler would have checked for a direct use: <!-- r[annotations.reading.mirror-checks-compiler] -->

| Condition | Raises |
| --- | --- |
| The member is not `public`, or `set` names a final field | `IllegalAccessException` <!-- r[annotations.reading.member-not-public] --> |
| `target` is not an instance of the declaring type, or a value or an argument is not of the type the member declares | `ClassCastException` <!-- r[annotations.reading.target-not-instance] --> |
| The number of arguments is not the number of parameters, or the method is generic | `IllegalArgumentException` <!-- r[annotations.reading.number-arguments-not] --> |

The type test is the test of a cast. <!-- r[annotations.reading.type-test-cast] -->
No implicit conversion applies, so an `Int32` is not accepted for an `Int` parameter. <!-- r[annotations.reading.no-implicit-conversion] -->

```java
@Target(Site.METHOD)
public annotation Check(String name);
```

```java
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

**Limits.** Mirrors read annotations and use the members that carry them. <!-- r[annotations.reading.limits-mirrors-read] -->
They do not describe the type of a field or a parameter, list a member that has no annotation, construct an object, or enumerate the types of a program. <!-- r[annotations.reading.not-describe-type] -->

## 8.10 The prelude's annotations

| Annotation | Written on | Rule <!-- r[annotations.prelude.table] --> |
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

These declarations are ordinary prelude source, and a program reads the declaration annotations among them as it reads its own. <!-- r[annotations.prelude.declarations-ordinary-prelude] -->
Their rules are the compiler's. <!-- r[annotations.prelude.rules-compiler] -->
An annotation that a program declares has the rules of this chapter and no others: where it may be written, whether a type parameter requires it, and whether it is inherited. <!-- r[annotations.prelude.annotation-program-declares] -->
What it means beyond that is decided by the code that reads it. <!-- r[annotations.prelude.means-beyond-decided] -->
