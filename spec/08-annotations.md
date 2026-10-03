# 8. Annotations

## 8.1 Declaration

An annotation is declared with `annotation`, not with `@interface` and not as an ordinary value class the user writes by hand.

```
annotation-decl = {audience} "annotation" identifier [type-params]
                  ["(" element {"," element} ")"]
                  ["implements" type {"," type}]
                  annotation-body
element         = type identifier ["=" constant]
```

Header elements are the only state. Zero elements is a marker annotation. Methods in the body are ordinary methods. `implements` attaches interfaces that have at least one method, such as a qualifier protocol or `Expander`. An annotation implementing an empty interface is rejected because an empty interface is already rejected.

Every annotation value implements `Annotation`. `Annotation` declares `Class<Annotation> annotationType()`. The prelude supplies the implementation from the declaration. User code does not reimplement it in order to change the class the value reports.

`@Target` is required on every annotation declaration. A use on a target `Target` does not list is rejected.

Arguments of a use are compile-time constants: literals, class literals, arrays of constants, and `new` of a value class whose arguments are constants. `new` of a reference class is rejected. A constant may be `null` only when the element type is `@Nullable`. Named arguments use the element names. An array argument is written `{` constants `}`. Omitted elements take their default. An element without a default and without an argument is rejected.

An annotation may be generic. `annotation Key<T>(String name)` is used as `@Key<String>("id")`. The type arguments are reified.

An annotation is repeatable without a container annotation. Repeated uses on the same target are ordered as written. Reflection that returns them returns that order.

## 8.2 Targets

`Target` is a final value class of the prelude. Programs do not construct it. The canonical instances are `TYPE`, `FIELD`, `METHOD`, `CONSTRUCTOR`, `PARAMETER`, `TYPE_PARAMETER`, `TYPE_USE`, `LOCAL`, `RECEIVER`, and `ANNOTATION`. `@Target` takes an array of those instances.

A type-use annotation and a declaration annotation are different attachments. On a field, a type annotation applies to the slot, the getter result, and the setter parameter together. A target written at the use selects one attachment when the position is ambiguous:

```
@field: @get: @set: @slot: @param: @return: @receiver: @type:
```

`var` infers type annotations and does not infer declaration annotations.

## 8.3 Qualifiers

`@Nullable` is the language's nullability qualifier. Its meta-annotation is `@Nullness`. User qualifiers do not change layout.

A qualifier annotation implements `Qualifier`. The unannotated type is the default rank of a qualifier lattice. `@SubtypeOf` names the qualifier ranks directly above the annotated qualifier. `@GainedWhen` names a method and a qualifier the result gains after a send of that method when the receiver or an argument had the annotated qualifier. The checker owns these facts. They are data on the annotation, not a second type system.

Type qualifiers follow the type through inference, subtyping, and generic instantiation. They do not use the travel modes of [section 8.5](#85-travel-and-retention).

## 8.4 Expansion and statement rewrite

`Expander` and `RewritesStatement` are prelude interfaces. An annotation may implement either.

An `Expander` runs at compile time on the declaration it annotates. It receives a builder and may add a field, a method, or an interface conformance. It must not remove a member written in source, replace a body written in source, or change a signature written in source. Added members are typechecked by the ordinary rules. Expansion repeats in rounds until a round adds nothing. At most 64 rounds run. The 64th round is the last that may add a member. If another round would still add a member, expansion is a compile error. Rounds run in order, and each round sees the members added by earlier rounds. Calls on the builder in one round run in the order the expander writes them. The expander's only effect is the builder. It does not read files, the clock, or the bodies of other compilation units. The limit is not a runtime exception and does not depend on the target machine.

`RewritesStatement` has one method, `Statement rewrite(Statement original)`. The result is typechecked in the original statement's scope. The rewrite may wrap the original statement in a block send.

The builder and the statement tree are prelude types. They are values the compiler passes while it is compiling. They are not a second language.

```java
public interface Expander {
    void expand(Expansion target)
}
public interface Expansion {
    void addField(String name, Class<?> type, Boolean isFinal)
    void addMethod(String name, Class<?> result, Class<?>[] parameters, Statement body)
    void addInterface(Class<?> type)
}
public abstract class Statement {
    public static Statement block(Statement[] statements)
    public static Statement expression(Expression expr)
    public static Statement returnValue(Expression expr)
    public static Statement returnUnit()
}
public abstract class Expression {
    public static Expression name(String identifier)
    public static Expression send(Expression receiver, String name, Expression[] arguments)
    public static Expression literal(Object value)
}
```

`addField` adds a field of the given name, type, and finality, with the audience of the annotated declaration. `addMethod` adds a method with that signature and body. `addInterface` adds a conformance. `literal` accepts a constant expression's value. A tree the ordinary type checker rejects is a compile error in the annotated declaration. These operations are the whole builder. An expander does not obtain the source text of a hand-written body.

## 8.5 Travel and retention

A declaration annotation carries a travel mode:

| Mode | Effect |
| --- | --- |
| `NOWHERE` | Present only on the declaration where it is written |
| `TO_SUBCLASSES` | Also present on subclasses |
| `TO_OVERRIDES` | Also present on overriding methods |
| `TO_IMPLEMENTATIONS` | Also present on implementing methods |

The default mode is `NOWHERE`. Type qualifiers ignore travel and follow the type.

After typechecking, an annotation with no remaining reader may be discarded. A declaration annotation is present to runtime `Class.has` when a reachable `has` call names that annotation class. A program must not assume an unread annotation is visible to `has`. Qualifiers used only by the checker need not be present at runtime.

## 8.6 Tags

A tag that carries no method requirement is an annotation. `@Serializable` on a type parameter is the constraint of [chapter 7](07-generics.md). `@RandomAccess` is the same kind of annotation when a library defines it. Neither is implemented, extended, or cast to. `value.getClass().has(Serializable.class)` is the runtime test.

A behavioral capability is an interface with methods, such as a `copy` method on an interface that declares it. An empty interface is not the way to spell either pattern.
