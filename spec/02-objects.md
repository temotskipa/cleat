# 2. Objects

## 2.1 One population of values

Every value is an instance of exactly one class. There are no primitive types. A number, a character, a truth value, a string, an array and `null` are instances of classes that the prelude declares.

A class is a reference class or a value class, as [section 2.3](#23-reference-classes-and-value-classes) defines. The difference is whether its instances have identity. How an implementation stores an instance is not observable. A value may be held in a register, inside another object, or on the heap, and a program cannot tell which.

An operator is a method of its left operand, as [chapter 4](04-methods.md) defines. A missing method is a type error. The compiler does not rewrite an operator into a different method, and it does not search the right operand for a method the left operand lacks.

## 2.2 What is built in

The prelude is written in Cleat. Its classes are declared under the rules of this specification, and a class that a program declares may do whatever a prelude class does. These are the exceptions:

1. **Literals.** A literal is an instance of a prelude class, and a numeric literal takes its class from context ([chapter 6](06-numbers.md)).
2. **`null`.** `Null` stands outside the class hierarchy, `@Nullable` adds its instance to a type, and the null test is not a method ([chapter 5](05-null-and-unit.md)).
3. **Arrays.** An array is an ordinary object of the prelude class `Array<T>`. The implementation supplies its storage, because no class can declare a number of slots that is chosen at run time. `new T[n]` also has a rule for default elements that belongs to arrays alone ([chapter 6](06-numbers.md)).
4. **Method bodies.** A prelude method may have a body that the implementation supplies, such as integer addition, an array access, or starting a thread. It is marked `@Intrinsic` ([chapter 8](08-annotations.md#88-the-preludes-declaration-annotations)).
5. **Roots.** `Object`, `Null` and `Enum` hold fixed places in the hierarchy, given in this chapter and in chapter 5.
6. **Annotations.** The prelude's annotations have rules that the compiler applies, and the mirrors that read annotations are supplied by the implementation ([chapter 8](08-annotations.md)).

Syntax that is defined by a method or an interface is not an exception, because a program's classes take part on equal terms. An operator is a method call. `for` works on any `Iterable`. `using` works on any class with a `close` method. A lambda becomes an instance of any interface with one method.

## 2.3 Reference classes and value classes

A class declared without `value` is a reference class. Its instances have identity: each evaluation of `new` creates an instance distinct from every other. The default `equals` of a reference class is identity, and the default `hashCode` agrees with it.

A class declared with `value` is a value class. Its instances have no identity. Its fields are final without writing `final`. The default `equals` is `true` when the argument is an instance of the same class and each pair of corresponding fields is equal under `equals`. The default `hashCode` agrees with that comparison. Two evaluations of `new` with equal arguments produce equal instances, and nothing distinguishes them.

A value class is final. It extends no class other than `Object`, and no class extends it. It may implement interfaces. `open`, `sealed` and `abstract` are rejected on a value class.

[Chapter 9](09-execution.md) defines construction for both kinds.

## 2.4 Extension

A class extends one class and implements any number of interfaces. A class with no `extends` clause extends `Object`.

A class is final unless it says otherwise: no class may extend it. `final` may be written to state that default.

- `open` lets any class that can see the class extend it.
- `sealed` lets only the classes named in its `permits` clause extend it. Each named class must be able to see the sealed class and must extend it. A `sealed` class with no `permits` clause, or with an empty one, is rejected.
- `abstract` declares a reference class with no instances of its own. `new` on it is rejected. It may declare abstract methods. Any class that can see it may extend it, unless it is also `sealed`.

`abstract final`, `open final` and `sealed open` are rejected.

## 2.5 The root class

`cleat.Object` is an open reference class. Every class other than `Null` is a subclass of it. It declares:

```java
public open Boolean equals(@Nullable Object this, @Nullable Object other)
public open Int hashCode(@Nullable Object this)
public open String toString(@Nullable Object this)
public final Boolean identical(@Nullable Object this, @Nullable Object other)
public final Class getClass()
```

The first four declare a `@Nullable` receiver, so they may be sent to a receiver that may be `null`. [Chapter 5](05-null-and-unit.md) says what runs for the three open methods when the receiver is `null`. An override of them declares an ordinary receiver.

**Defaults.** The default `equals` and `hashCode` are those of [section 2.3](#23-reference-classes-and-value-classes). The default `toString` of a value class is the simple name of the class, `(`, the `toString` of each field in declaration order separated by `, `, and `)`. A value class `Point` with fields 1 and 2 gives `Point(1, 2)`. The default `toString` of a reference class is chosen by the implementation. It does not change during the lifetime of the instance, and a program must not rely on its characters.

**Overrides.** A class may override `equals`, `hashCode` and `toString`. An override of `equals` must be an equivalence relation, must return `false` for a `null` argument, and must give equal instances equal hash codes. The language does not check these obligations. The overrides in the prelude meet them.

**`identical`.** `a.identical(b)` asks whether anything could tell `a` from `b`.

- Two instances of reference classes are identical when they are the same instance.
- Two instances of a value class are identical when they have the same class and each pair of corresponding fields is identical.
- `null` is identical to `null` and to nothing else.

For a prelude value class whose fields are not written in source, `identical` is `equals`. The float classes are the exception: two floats are identical when `totalOrder` of [chapter 6](06-numbers.md) returns zero.

`identical` is defined for every pair of values. A value class instance seen through a variable of type `Object` is compared by the second rule, so the result never depends on how the implementation stored it.

**`getClass`.** `getClass` returns the class object of the instance's class, as [section 2.7](#27-static-members-and-class-objects) describes.

`Object` does not declare `wait`, `notify`, `notifyAll`, `clone` or `finalize`. There is no per-object monitor. A lock is a `Lock`, specified in [chapter 13](13-concurrency.md).

## 2.6 Interfaces

An interface declares methods. It may declare none: a sealed interface with no methods is the usual root of a fixed set of alternatives.

- A method without a body is abstract. A method with a body is a default implementation. A class that implements the interface inherits it and may override it. `final` on a method with a body forbids the override. `final` on a method without a body is rejected. The word `default` is not written.
- Interface methods are public. The private-by-default rule of classes does not apply to them.
- An interface has no instance fields. A field declared in an interface is `public static final`, whether or not those words are written. A contradictory modifier is rejected.

A class implements any number of interfaces, and an interface extends any number of interfaces. Extension and implementation are nominal. A class whose methods happen to match an interface does not implement it.

If two superinterfaces provide a body for the same signature and the class does not override that signature, the class is rejected. A method declared in the class takes precedence over an interface body of the same signature.

An interface that is not sealed may be implemented by any type that can see it, and `open` may be written to state that. A `sealed` interface names in `permits` the only types that may implement or extend it. `final interface` is rejected.

**Static methods.** A static method with a body belongs to the interface's class object, like a static method of a class. A static method without a body is a requirement on implementers: every class that implements the interface and is not abstract has a public static method with that name and signature. That method carries `@Override`, as [chapter 8](08-annotations.md#88-the-preludes-declaration-annotations) asks of a method that implements an interface method. The signature may use the interface's type parameters, which stand for the implementer's type arguments.

```java
public interface Numeric<T> {
    static T zero();
}
```

`Int` implements `Numeric<Int>`, so it declares `public static Int zero()`. A requirement is called through a type parameter, as [chapter 7](07-generics.md) defines. It is not called on the interface itself.

## 2.7 Static members and class objects

A `static` member belongs to its class and not to any instance. A static field is one slot. A static method has no receiver. A static member is selected by the compile-time type that names it and is not dispatched. A static member of a subclass with the same name and signature hides the superclass's member. It does not override it.

A generic class has one copy of each static member, shared by all of its instantiations. A static member does not use the type parameters of its class, and it is named through the class name without type arguments, as in `Array.build(n, f)`.

Every class, interface, enum and annotation has a class object, which is an instance of `Class`. A generic type has a distinct class object for each set of type arguments, as [chapter 7](07-generics.md) defines.

`C.class` is the class literal. Its type is `Class`. For a generic type it is written with type arguments, as in `List<String>.class`. Evaluating it does not initialize the class.

```java
public final class Class {
    public String getName()
}
```

`getName` returns the qualified name of the class, with its type arguments as chapter 7 writes them. `Class` is a reference class with one instance per class, so `a.getClass() == b.getClass()` asks whether two values have the same class.

`Class` also declares the methods of [chapter 8](08-annotations.md#89-reading-annotations), which read the annotations of a type and reach the members that carry one. There is no other reflection. A program cannot list the members of a class that carry no annotation, call a method by name, or construct an object from a class object.

## 2.8 Fields

A field is a slot of an instance, or of the class object when it is `static`. Code that the field's audience admits, under [chapter 3](03-visibility.md), reads the field by name and assigns it by name. No accessor method stands between them.

A field of a reference class is assignable unless it is declared `final`. A final field is assigned on every constructor path and never afterwards. A field of a value class is final.

A field of a subclass with the same name as a field of a superclass hides it. The static type of the receiver selects between them. No field is dispatched.

## 2.9 Enums

An enum is declared with the keyword `enum`:

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

An enum is a final reference class with a fixed set of instances, its constants. `value enum` is rejected.

The declaration lists the constants first. Each constant is a `public static final` field that holds a distinct instance, created before any other static initializer of the enum runs. A constant may pass arguments to a constructor of the enum. A constant has no class body. An enum's constructors are private, and no other expression creates an instance of the enum.

After its constants, an enum may declare fields, constructors and methods, and it may implement interfaces.

Every enum has these methods:

| Method | Result |
| --- | --- |
| `name()` | The constant's identifier, as a `String` |
| `ordinal()` | The constant's position in the list, counting from zero, as an `Int` |
| `static values()` | A new array of the constants in source order |
| `static valueOf(String name)` | The constant with that name. It raises `IllegalArgumentException` when there is none |

`equals` on an enum is identity.

`name` and `ordinal` are declared by the prelude class `Enum`, which is the superclass of every enum. A program does not write `Enum` to declare an enum, and a class that writes `extends Enum` is rejected. `Enum` exists so that a method can accept any enum, as in `void log(Enum level)`.

## 2.10 Member lookup

A member is selected in two steps.

1. The compile-time types of the receiver and the arguments choose a member, by the overload rules of [chapter 4](04-methods.md).
2. If the chosen member is an `open` method, an `abstract` method or an interface method, the method that runs is the one the receiver's class has for that signature at run time. A final method, a static method and a field are not dispatched.

`super.name(...)` runs the superclass's method and does not dispatch. `Type.name` selects a static member of that type.

A method overrides another only as chapter 4 allows, and it carries `@Override`. A field hides a superclass field of the same name. A static method hides a superclass static method of the same signature.

## 2.11 Absent from Java

[Chapter 10](10-deferred.md#101-not-in-the-language) lists the features of Java that Cleat does not have. Each is rejected if a program writes it.
