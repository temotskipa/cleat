# 2. Objects

## 2.1 One population of values

Every value is an instance of exactly one class. <!-- r[objects.population.value-instance-exactly] -->
There are no primitive types. <!-- r[objects.population.no-primitive-types] -->
A number, a character, a truth value, a string, an array and `null` are instances of classes that the prelude declares. <!-- r[objects.population.number-character-truth] -->

A class is a reference class or a value class, as [section 2.3](#23-reference-classes-and-value-classes) defines. <!-- r[objects.population.class-reference-value] -->
The difference is whether its instances have identity. <!-- r[objects.population.difference-whether-instances] -->
How an implementation stores an instance is not observable. <!-- r[objects.population.how-implementation-stores] -->
A value may be held in a register, inside another object, or on the heap, and a program cannot tell which. <!-- r[objects.population.value-held-register] -->

An operator is a method of its left operand, as [chapter 4](04-methods.md) defines. <!-- r[objects.population.operator-method-left] -->
A missing method is a type error. <!-- r[objects.population.missing-method-type] -->
The compiler does not rewrite an operator into a different method, and it does not search the right operand for a method the left operand lacks. <!-- r[objects.population.compiler-not-rewrite] -->

## 2.2 What is built in

The prelude is written in Cleat. <!-- r[objects.built-in.prelude-written-cleat] -->
Its classes are declared under the rules of this specification, and a class that a program declares may do whatever a prelude class does. <!-- r[objects.built-in.classes-declared-under] -->
These are the exceptions: <!-- r[objects.built-in.exceptions] -->

1. **Literals.** A literal is an instance of a prelude class, and a numeric literal takes its class from context ([chapter 6](06-numbers.md)). <!-- r[objects.built-in.literals-literal-instance] -->
2. **`null`.** `Null` stands outside the class hierarchy, `@Nullable` adds its instance to a type, and the null test is not a method ([chapter 5](05-null-and-unit.md)). <!-- r[objects.built-in.null-stands-outside] -->
3. **Arrays.** An array is an ordinary object of the prelude class `Array<T>`. <!-- r[objects.built-in.arrays-array-ordinary] -->
   The implementation supplies its storage, because no class can declare a number of slots that is chosen at run time. <!-- r[objects.built-in.implementation-supplies-storage] -->
   `new T[n]` also has a rule for default elements that belongs to arrays alone ([chapter 6](06-numbers.md)). <!-- r[objects.built-in.new-rule-default] -->
4. **Method bodies.** A prelude method may have a body that the implementation supplies, such as integer addition, an array access, or starting a thread. <!-- r[objects.built-in.method-bodies-prelude] -->
   It is marked `@Intrinsic` ([chapter 8](08-annotations.md#88-the-preludes-declaration-annotations)). <!-- r[objects.built-in.marked-intrinsic] -->
5. **Roots.** `Object`, `Null` and `Enum` hold fixed places in the hierarchy, given in this chapter and in chapter 5. <!-- r[objects.built-in.roots-object-null] -->
6. **Annotations.** The prelude's annotations have rules that the compiler applies, and the mirrors that read annotations are supplied by the implementation ([chapter 8](08-annotations.md)). <!-- r[objects.built-in.annotations-prelude-rules] -->

Syntax that is defined by a method or an interface is not an exception, because a program's classes take part on equal terms. <!-- r[objects.built-in.syntax-defined-method] -->
An operator is a method call. <!-- r[objects.built-in.operator-method-call] -->
`for` works on any `Iterable`. <!-- r[objects.built-in.for-works-iterable] -->
`using` works on any class with a `close` method. <!-- r[objects.built-in.using-works-class] -->
A lambda becomes an instance of any interface with one method. <!-- r[objects.built-in.lambda-becomes-instance] -->

## 2.3 Reference classes and value classes

A class declared without `value` is a reference class. <!-- r[objects.classes.class-declared-without] -->
Its instances have identity: each evaluation of `new` creates an instance distinct from every other. <!-- r[objects.classes.instances-identity-evaluation] -->
The default `equals` of a reference class is identity, and the default `hashCode` agrees with it. <!-- r[objects.classes.default-equals-reference] -->

A class declared with `value` is a value class. <!-- r[objects.classes.class-declared-value] -->
Its instances have no identity. <!-- r[objects.classes.instances-no-identity] -->
Its fields are final without writing `final`. <!-- r[objects.classes.fields-final-without] -->
The default `equals` is `true` when the argument is an instance of the same class and each pair of corresponding fields is equal under `equals`. <!-- r[objects.classes.default-equals-true] -->
The default `hashCode` agrees with that comparison. <!-- r[objects.classes.default-hashcode-agrees] -->
Two evaluations of `new` with equal arguments produce equal instances, and nothing distinguishes them. <!-- r[objects.classes.evaluations-new-equal] -->

A value class is final. <!-- r[objects.classes.value-class-final] -->
It extends no class other than `Object`, and no class extends it. <!-- r[objects.classes.extends-no-class] -->
It may implement interfaces. <!-- r[objects.classes.implement-interfaces] -->
`open`, `sealed` and `abstract` are rejected on a value class. <!-- r[objects.classes.open-sealed-abstract] -->

[Chapter 9](09-execution.md) defines construction for both kinds. <!-- r[objects.classes.defines-construction-kinds] -->

## 2.4 Extension

A class extends one class and implements any number of interfaces. <!-- r[objects.extension.class-extends-implements] -->
A class with no `extends` clause extends `Object`. <!-- r[objects.extension.class-no-extends] -->

A class is final unless it says otherwise: no class may extend it. <!-- r[objects.extension.class-final-unless] -->
`final` may be written to state that default. <!-- r[objects.extension.final-written-state] -->

- `open` lets any class that can see the class extend it. <!-- r[objects.extension.open-lets-class] -->
- `sealed` lets only the classes named in its `permits` clause extend it. <!-- r[objects.extension.sealed-lets-only] -->
  Each named class must be able to see the sealed class and must extend it. <!-- r[objects.extension.named-class-able] -->
  A `sealed` class with no `permits` clause, or with an empty one, is rejected. <!-- r[objects.extension.sealed-class-no] -->
- `abstract` declares a reference class with no instances of its own. <!-- r[objects.extension.abstract-declares-reference] -->
  `new` on it is rejected. <!-- r[objects.extension.new-rejected] -->
  It may declare abstract methods. <!-- r[objects.extension.declare-abstract-methods] -->
  Any class that can see it may extend it, unless it is also `sealed`. <!-- r[objects.extension.class-see-extend] -->

`abstract final`, `open final` and `sealed open` are rejected. <!-- r[objects.extension.abstract-final-open] -->

## 2.5 The root class

`cleat.Object` is an open reference class. <!-- r[objects.root.cleat-object-open] -->
Every class other than `Null` is a subclass of it. <!-- r[objects.root.class-null-subclass] -->
It declares: <!-- r[objects.root.declares] -->

```java
public open Boolean equals(@Nullable Object this, @Nullable Object other)
public open Int hashCode(@Nullable Object this)
public open String toString(@Nullable Object this)
public final Boolean identical(@Nullable Object this, @Nullable Object other)
public final Class getClass()
```

The first four declare a `@Nullable` receiver, so they may be sent to a receiver that may be `null`. <!-- r[objects.root.first-four-declare] -->
[Chapter 5](05-null-and-unit.md) says what runs for the three open methods when the receiver is `null`. <!-- r[objects.root.says-runs-three] -->
An override of them declares an ordinary receiver. <!-- r[objects.root.override-declares-ordinary] -->

**Defaults.** The default `equals` and `hashCode` are those of [section 2.3](#23-reference-classes-and-value-classes). <!-- r[objects.root.defaults-default-equals] -->
The default `toString` of a value class is the simple name of the class, `(`, the `toString` of each field in declaration order separated by `, `, and `)`. <!-- r[objects.root.default-tostring-value] -->
A value class `Point` with fields 1 and 2 gives `Point(1, 2)`. <!-- r[objects.root.value-class-point] -->
The default `toString` of a reference class is chosen by the implementation. <!-- r[objects.root.default-tostring-reference] -->
It does not change during the lifetime of the instance, and a program must not rely on its characters. <!-- r[objects.root.not-change-during] -->

**Overrides.** A class may override `equals`, `hashCode` and `toString`. <!-- r[objects.root.overrides-class-override] -->
An override of `equals` must be an equivalence relation, must return `false` for a `null` argument, and must give equal instances equal hash codes. <!-- r[objects.root.override-equals-equivalence] -->
The language does not check these obligations. <!-- r[objects.root.language-not-check] -->
The overrides in the prelude meet them. <!-- r[objects.root.overrides-prelude-meet] -->

**`identical`.** `a.identical(b)` asks whether anything could tell `a` from `b`. <!-- r[objects.root.identical-asks-whether] -->

- Two instances of reference classes are identical when they are the same instance. <!-- r[objects.root.instances-reference-classes] -->
- Two instances of a value class are identical when they have the same class and each pair of corresponding fields is identical. <!-- r[objects.root.instances-value-class] -->
- `null` is identical to `null` and to nothing else. <!-- r[objects.root.null-identical-nothing] -->

For a prelude value class whose fields are not written in source, `identical` is `equals`. <!-- r[objects.root.prelude-value-class] -->
The float classes are the exception: two floats are identical when `totalOrder` of [chapter 6](06-numbers.md) returns zero. <!-- r[objects.root.float-classes-exception] -->

`identical` is defined for every pair of values. <!-- r[objects.root.identical-defined-pair] -->
A value class instance seen through a variable of type `Object` is compared by the second rule, so the result never depends on how the implementation stored it. <!-- r[objects.root.value-class-instance] -->

**`getClass`.** `getClass` returns the class object of the instance's class, as [section 2.7](#27-static-members-and-class-objects) describes. <!-- r[objects.root.getclass-returns-class] -->

`Object` does not declare `wait`, `notify`, `notifyAll`, `clone` or `finalize`. <!-- r[objects.root.object-not-declare] -->
There is no per-object monitor. <!-- r[objects.root.no-per-object] -->
A lock is a `Lock`, specified in [chapter 13](13-concurrency.md). <!-- r[objects.root.lock-specified] -->

## 2.6 Interfaces

An interface declares methods. <!-- r[objects.interfaces.interface-declares-methods] -->
It may declare none: a sealed interface with no methods is the usual root of a fixed set of alternatives. <!-- r[objects.interfaces.declare-none-sealed] -->

- A method without a body is abstract. <!-- r[objects.interfaces.method-without-body] -->
  A method with a body is a default implementation. <!-- r[objects.interfaces.method-body-default] -->
  A class that implements the interface inherits it and may override it. <!-- r[objects.interfaces.class-implements-interface] -->
  `final` on a method with a body forbids the override. <!-- r[objects.interfaces.final-method-body] -->
  `final` on a method without a body is rejected. <!-- r[objects.interfaces.final-method-without] -->
  The word `default` is not written. <!-- r[objects.interfaces.word-default-not] -->
- Interface methods are public. <!-- r[objects.interfaces.interface-methods-public] -->
  The private-by-default rule of classes does not apply to them. <!-- r[objects.interfaces.private-default-rule] -->
- An interface has no instance fields. <!-- r[objects.interfaces.interface-no-instance] -->
  A field declared in an interface is `public static final`, whether or not those words are written. <!-- r[objects.interfaces.field-declared-interface] -->
  A contradictory modifier is rejected. <!-- r[objects.interfaces.contradictory-modifier-rejected] -->

A class implements any number of interfaces, and an interface extends any number of interfaces. <!-- r[objects.interfaces.class-implements-number] -->
Extension and implementation are nominal. <!-- r[objects.interfaces.extension-implementation-nominal] -->
A class whose methods happen to match an interface does not implement it. <!-- r[objects.interfaces.class-methods-happen] -->

If two superinterfaces provide a body for the same signature and the class does not override that signature, the class is rejected. <!-- r[objects.interfaces.superinterfaces-provide-body] -->
A method declared in the class takes precedence over an interface body of the same signature. <!-- r[objects.interfaces.method-declared-class] -->

An interface that is not sealed may be implemented by any type that can see it, and `open` may be written to state that. <!-- r[objects.interfaces.interface-not-sealed] -->
A `sealed` interface names in `permits` the only types that may implement or extend it. <!-- r[objects.interfaces.sealed-interface-names] -->
`final interface` is rejected. <!-- r[objects.interfaces.final-interface-rejected] -->

**Static methods.** A static method with a body belongs to the interface's class object, like a static method of a class. <!-- r[objects.interfaces.static-methods-method] -->
A static method without a body is a requirement on implementers: every class that implements the interface and is not abstract has a public static method with that name and signature. <!-- r[objects.interfaces.static-method-without] -->
That method carries `@Override`, as [chapter 8](08-annotations.md#88-the-preludes-declaration-annotations) asks of a method that implements an interface method. <!-- r[objects.interfaces.method-carries-override] -->
The signature may use the interface's type parameters, which stand for the implementer's type arguments. <!-- r[objects.interfaces.signature-use-interface] -->

```java
public interface Numeric<T> {
    static T zero();
}
```

`Int` implements `Numeric<Int>`, so it declares `public static Int zero()`. <!-- r[objects.interfaces.int-implements-numeric] -->
A requirement is called through a type parameter, as [chapter 7](07-generics.md) defines. <!-- r[objects.interfaces.requirement-called-through] -->
It is not called on the interface itself. <!-- r[objects.interfaces.not-called-interface] -->

## 2.7 Static members and class objects

A `static` member belongs to its class and not to any instance. <!-- r[objects.statics.static-member-belongs] -->
A static field is one slot. <!-- r[objects.statics.static-field-slot] -->
A static method has no receiver. <!-- r[objects.statics.static-method-no] -->
A static member is selected by the compile-time type that names it and is not dispatched. <!-- r[objects.statics.static-member-selected] -->
A static member of a subclass with the same name and signature hides the superclass's member. <!-- r[objects.statics.static-member-subclass] -->
It does not override it. <!-- r[objects.statics.not-override] -->

A generic class has one copy of each static member, shared by all of its instantiations. <!-- r[objects.statics.generic-class-copy] -->
A static member does not use the type parameters of its class, and it is named through the class name without type arguments, as in `Array.build(n, f)`. <!-- r[objects.statics.static-member-not] -->

Every class, interface, enum and annotation has a class object, which is an instance of `Class`. <!-- r[objects.statics.class-interface-enum] -->
A generic type has a distinct class object for each set of type arguments, as [chapter 7](07-generics.md) defines. <!-- r[objects.statics.generic-type-distinct] -->

`C.class` is the class literal. <!-- r[objects.statics.class-literal] -->
Its type is `Class`. <!-- r[objects.statics.type-class] -->
For a generic type it is written with type arguments, as in `List<String>.class`. <!-- r[objects.statics.generic-type-written] -->
Evaluating it does not initialize the class. <!-- r[objects.statics.evaluating-not-initialize] -->

```java
public final class Class {
    public String getName()
}
```

`getName` returns the qualified name of the class, with its type arguments as chapter 7 writes them. <!-- r[objects.statics.getname-returns-qualified] -->
`Class` is a reference class with one instance per class, so `a.getClass() == b.getClass()` asks whether two values have the same class. <!-- r[objects.statics.class-reference-instance] -->

`Class` also declares the methods of [chapter 8](08-annotations.md#89-reading-annotations), which read the annotations of a type and reach the members that carry one. <!-- r[objects.statics.class-declares-methods] -->
There is no other reflection. <!-- r[objects.statics.no-reflection] -->
A program cannot list the members of a class that carry no annotation, call a method by name, or construct an object from a class object. <!-- r[objects.statics.program-list-members] -->

## 2.8 Fields

A field is a slot of an instance, or of the class object when it is `static`. <!-- r[objects.fields.field-slot-instance] -->
Code that the field's audience admits, under [chapter 3](03-visibility.md), reads the field by name and assigns it by name. <!-- r[objects.fields.code-field-audience] -->
No accessor method stands between them. <!-- r[objects.fields.no-accessor-method] -->

A field of a reference class is assignable unless it is declared `final`. <!-- r[objects.fields.field-reference-class] -->
A final field is assigned on every constructor path and never afterwards. <!-- r[objects.fields.final-field-assigned] -->
A field of a value class is final. <!-- r[objects.fields.field-value-class] -->

A field of a subclass with the same name as a field of a superclass hides it. <!-- r[objects.fields.field-subclass-name] -->
The static type of the receiver selects between them. <!-- r[objects.fields.static-type-receiver] -->
No field is dispatched. <!-- r[objects.fields.no-field-dispatched] -->

## 2.9 Enums

An enum is declared with the keyword `enum`: <!-- r[objects.enums.enum-declared-keyword] -->

```java
public enum Op { ADD, SUBTRACT, MULTIPLY, DIVIDE }

public enum Planet {
    MERCURY(3.30e23, 2.44e6),
    EARTH(5.97e24, 6.37e6);

    public final Float64 mass;
    public final Float64 radius;

    Planet(Float64 mass, Float64 radius) {
        this.mass = mass;
        this.radius = radius;
    }
}
```

An enum is a final reference class with a fixed set of instances, its constants. <!-- r[objects.enums.enum-final-reference] -->
`value enum` is rejected. <!-- r[objects.enums.value-enum-rejected] -->

The declaration lists the constants first. <!-- r[objects.enums.declaration-lists-constants] -->
Each constant is a `public static final` field that holds a distinct instance, created before any other static initializer of the enum runs. <!-- r[objects.enums.constant-public-static] -->
A constant may pass arguments to a constructor of the enum. <!-- r[objects.enums.constant-pass-arguments] -->
A constant has no class body. <!-- r[objects.enums.constant-no-class] -->
An enum's constructors are private, and no other expression creates an instance of the enum. <!-- r[objects.enums.enum-constructors-private] -->

After its constants, an enum may declare fields, constructors and methods, and it may implement interfaces. <!-- r[objects.enums.after-constants-enum] -->

Every enum has these methods: <!-- r[objects.enums.enum-methods] -->

| Method | Result |
| --- | --- |
| `name()` | The constant's identifier, as a `String` <!-- r[objects.enums.name-constant-identifier] --> |
| `ordinal()` | The constant's position in the list, counting from zero, as an `Int` <!-- r[objects.enums.ordinal-constant-position] --> |
| `static values()` | A new array of the constants in source order <!-- r[objects.enums.static-values-new] --> |
| `static valueOf(String name)` | The constant with that name. It raises `IllegalArgumentException` when there is none <!-- r[objects.enums.static-valueof-string] --> |

`equals` on an enum is identity. <!-- r[objects.enums.equals-enum-identity] -->

`name` and `ordinal` are declared by the prelude class `Enum`, which is the superclass of every enum. <!-- r[objects.enums.name-ordinal-declared] -->
A program does not write `Enum` to declare an enum, and a class that writes `extends Enum` is rejected. <!-- r[objects.enums.program-not-write] -->
`Enum` exists so that a method can accept any enum, as in `void log(Enum level)`. <!-- r[objects.enums.enum-exists-method] -->

## 2.10 Member lookup

A member is selected in two steps. <!-- r[objects.lookup.member-selected-steps] -->

1. The compile-time types of the receiver and the arguments choose a member, by the overload rules of [chapter 4](04-methods.md). <!-- r[objects.lookup.compile-time-types] -->
2. If the chosen member is an `open` method, an `abstract` method or an interface method, the method that runs is the one the receiver's class has for that signature at run time. <!-- r[objects.lookup.chosen-member-open] -->
   A final method, a static method and a field are not dispatched. <!-- r[objects.lookup.final-method-static] -->

`super.name(...)` runs the superclass's method and does not dispatch. <!-- r[objects.lookup.super-name-runs] -->
`Type.name` selects a static member of that type. <!-- r[objects.lookup.type-name-selects] -->

A method overrides another only as chapter 4 allows, and it carries `@Override`. <!-- r[objects.lookup.method-overrides-only] -->
A field hides a superclass field of the same name. <!-- r[objects.lookup.field-hides-superclass] -->
A static method hides a superclass static method of the same signature. <!-- r[objects.lookup.static-method-hides] -->

## 2.11 Absent from Java

[Chapter 10](10-deferred.md#101-not-in-the-language) lists the features of Java that Cleat does not have. <!-- r[objects.absent.lists-features-java] -->
Each is rejected if a program writes it. <!-- r[objects.absent.rejected-program-writes] -->
