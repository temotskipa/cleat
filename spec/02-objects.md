# 2. Objects

## 2.1 One population of values

Every value is an instance of exactly one class. Every operation the language spells is a method send or the binding of a name. A rule that applied to one class and could not be written as a method of another class is not a rule of this language.

The class of a value is a fact about the value. Storage is not a second fact of that kind. A value may occupy a register, a tagged word, a flat lane, or digits on the heap. Those are representations. Becoming a reference, with a header and an identity, happens when a value is stored in a reference slot: a slot whose static type is `Object` or an interface, and not a value type. A local whose static type is a value type has no header and no identity.

A missing method is a type error. The compiler does not rewrite an operator into a different method, and it does not search the right operand for a method the left operand lacks.

## 2.2 Reference objects and value objects

A class declared without `value` is a reference class. Instances have identity. The default `equals` is identity. The default `hashCode` agrees with that identity.

A class declared with `value` is a value class. Instances have no identity. The default `equals` compares components with `equals`, in declaration order, and the superclass components first. The default `hashCode` agrees with that comparison. Value fields are final. Two value instances with equal components are equal, including two results of separate evaluations of `new`.

```
class-mod   = {class-modifier} ["value"] "class" identifier [type-params]
              ["extends" type] ["implements" type {"," type}]
              ["permits" type {"," type}] class-body
class-modifier = "public" | "package" | "private" | "abstract" | "final"
               | "sealed" | "open"
```

`value` is written on every value class, including a class that extends a value class. A class that extends a value class and omits `value` is rejected. A reference class that extends a value class is rejected.

A value class is final unless it is `sealed`. `open value class` is rejected. A sealed value class's permitted subclasses are value classes. User code cannot extend a sealed value class it is not permitted to extend. The numeric classes of [chapter 6](06-numbers.md) are sealed value classes whose representations are defined there; they are not required to be writable as user value classes with a field per component.

`final` on a class states the default for a concrete reference class. `abstract final` is rejected. `open final` is rejected. `sealed` and `open` together are rejected.

An `abstract` class is a reference class, has no instances created by `new`, and may declare abstract methods. An abstract class may be extended by any class that can see it, unless it is also `sealed`. `abstract` on a value class is rejected.

A concrete class that is not `open` and not `sealed` is final: no other class may extend it. `final` may be written on it. `open` allows any class that can see it to extend it. `sealed` allows only the classes named in `permits`, and those classes must be able to see the sealed type. A sealed type with an empty `permits` clause is rejected. A permitted type that does not extend or implement the sealed type is rejected. Omitting `permits` on a sealed type is rejected.

## 2.3 The root class

`cleat.Object` is a concrete reference class. It is the superclass of every class that does not write `extends`. It declares:

```java
public Boolean equals(Object other)
public Int32 hashCode()
public String toString()
public Class<C> getClass()
public Boolean identical(Object other)
public Boolean isNull()
public <T> T as(Class<T> type)
```

`C` in the `getClass` result is the static type of the receiver with `@Nullable` removed. That dependence on the receiver is defined in [chapter 7](07-generics.md). It is not a covariant override, and no other method acquires it. `as` is a class test. Its result type is the `T` of the class literal.

Default `equals` and `hashCode` are those of [section 2.2](#22-reference-objects-and-value-objects). Default `toString` of a reference class is stable for the lifetime of that instance. Default `toString` of a value class is determined by its class and by `toString` of its components in declaration order, superclass components first, so equal values have equal strings. Storing the value in a reference slot does not switch it to the reference rule. The string is still the value's string. A program may rely on that stability and on that equality. It may not rely on the characters of a reference class's default `toString`. Every implementation that meets those observations is correct. [Chapter 10](10-deferred.md) records that bound. The null instance is a `Null` value. Its `toString` is the override in [chapter 5](05-null-and-unit.md), not this default. `getClass` returns the class object of the instance's actual class, and for the null reference that class object is `Null.class`. `isNull` returns `true` exactly when the receiver is the null instance.

`identical` is reference identity. The call is rejected when the static type of the receiver or of the argument is a value type. When both static types are reference types, the result is whether the two references are the same instance. The null reference is the single `Null` instance described in [chapter 5](05-null-and-unit.md); it is identical to itself and to no other reference.

Storing a value in a reference slot boxes it. Two boxes of equal values may be identical or distinct. A program must not depend on which. `equals` is the comparison the language defines for those values.

`as` is a class test on every class, including `Vector` and the other prelude types. It raises `ClassCastException` when the runtime class of the receiver is not the class named by the argument and is not a subclass of that class. For a reified generic class it also requires the type arguments to match, as [chapter 7](07-generics.md) defines. It does not convert a numeric value to a different width, it does not convert `Char`, and it does not copy the lanes of a vector. A failed `as` does not return `null`. The receiver is evaluated once. A constant `as` that would raise is rejected at compile time, as [chapter 12](12-flow.md) defines.

A class may override `equals`, `hashCode`, and `toString`. An override of `equals` must be an equivalence relation on the values it accepts, and equal values must have equal hash codes. The relations supplied by the prelude meet that obligation. `identical`, `getClass`, `isNull`, and `as` are final.

`Object` does not declare `wait`, `notify`, `notifyAll`, `clone`, or `finalize`. There is no per-object monitor. A lock is a `Lock`, specified in [chapter 13](13-concurrency.md). A static type that is a value type is rejected as the receiver of `lock`, `unlock`, or `exclusive`.

## 2.4 Interfaces

```
interface-decl = {iface-modifier} "interface" identifier [type-params]
                 ["extends" type {"," type}]
                 ["permits" type {"," type}] interface-body
iface-modifier = "public" | "package" | "private" | "sealed" | "open"
```

An interface declares one or more methods. An interface with no methods is rejected. A marker used as a tag is an annotation, not an interface.

An interface method without a body is abstract. An interface method with a body is a default implementation and may be overridden. The word `default` is not written on that method. `final` on an interface method with a body forbids override. `final` on an interface method without a body is rejected.

Interface methods are public. The private-by-default rule of classes does not apply to them. An interface has no instance fields. It may declare fields, which are `public static final` slots of its class object. Those three modifiers are implied when omitted. A contradictory modifier is rejected.

A class implements any number of interfaces. It extends one class. An interface extends any number of interfaces. Extension and implementation are nominal. A structural match of method names does not implement an interface.

If two superinterfaces provide a body for the same signature and the class does not override that signature, the program is rejected. If exactly one superinterface provides a body, the class inherits it. A method declared in the class takes precedence over an interface body of the same signature.

An interface may be `sealed` with `permits`. Only the named types may implement or extend it. An interface that is not sealed may be implemented by any type that can see it. `final interface` is rejected.

## 2.5 Class objects

Every class, interface, and annotation has one class object. The class object is created by the implementation. User code does not declare a metaclass.

`static` members are slots and methods of the class object. A static call is resolved on the compile-time type of the receiver class and is not virtual. A subclass static member hides a superclass static member of the same signature; it does not override it.

`C.class` is the class literal. Its type is `Class<C>`. Evaluating a class literal does not initialize the class object.

`new` in source is a call to the class object's `new` method, specified in [chapter 9](09-execution.md).

## 2.6 Fields

A field declaration introduces a slot, a getter, and, when the field is not final, a setter. The getter and setter carry the field's audience and type. Their source spelling is the field's name. A program does not declare them separately and cannot name them as distinct members.

Inside the declaring class, a use of the field reads or writes the slot. Outside the declaring class, a use calls the getter or the setter. A subclass does not have slot access. Generated getters and setters are final.

A subclass field with the same name hides the superclass field. Selection uses the static type of the receiver. There is no virtual field.

A field of a reference class is mutable unless declared `final`. A final field is assigned on every constructor path and is not assigned thereafter. A value-class field is final without writing `final`.

The audience rules, including `private(this)`, are [chapter 3](03-visibility.md).

## 2.7 Enums

```java
public abstract class Enum {
    public String name()
    public Int32 ordinal()
}
```

An `enum` declaration is a final reference class that extends `Enum`. `Enum` permits only those declarations. A hand-written class that extends `Enum` is rejected. The enum is sealed by the language. Its permitted instances are its constants, and user code does not extend the enum. `value enum` is rejected. An enum constant has no class body. Anonymous classes remain rejected.

Each constant is a `public static final` field of the enum type, initialized to a distinct instance before other static initializers of that enum run. `ordinal` is the source position, starting at zero. `name` is the constant's identifier. `equals` is the reference equality inherited from `Object`. `values()` is a public static method that returns a new array of the constants in source order. `valueOf(String name)` returns the constant with that name and raises `IllegalArgumentException` when none has it.

An enum may implement interfaces and may declare fields and methods after the constant list. Those members follow the ordinary rules.

## 2.8 Member lookup

Overload resolution uses the compile-time type of the receiver, as [chapter 4](04-methods.md) defines. After a method is chosen, an `open` instance method is invoked on the runtime class. A `final` instance method, a `static` method, and a field are not dispatched on the runtime class. `super.name` selects the member of the superclass and does not dispatch further. `Type.name` selects a static member of that type.

A subclass method overrides a superclass or interface method only when the signatures match exactly, as chapter 4 requires. A field hides a superclass field of the same name and does not override it. A static method hides a static method of the same signature and does not override it.

If the compile-time type has two applicable methods and neither is more specific, the call is rejected. The program writes a cast or an explicit type argument. It does not rely on the runtime class to choose.

## 2.9 Members inherited from Java that are absent

The following are rejected if written, and the prelude does not provide them: primitive type names, raw types, checked `throws` clauses, nested types, anonymous class bodies, marker interfaces, per-object `synchronized`, switch fallthrough, octal literals, UTF-16 `char` as the definition of characters, `finalize`, `Cloneable`, and a serialization protocol that walks fields by magic. Use-site wildcards are [chapter 7](07-generics.md). `switch` and `enum` are [chapter 11](11-syntax.md) and this chapter. There is no power operator.
