# 3. Visibility

## 3.1 Audiences

Every class member and every type has an audience. When no audience is written on a class member, the audience is `private`. When no audience is written on a type, the audience is `private`. `public` is written to publish a type or a member.

The audiences are:

| Written | Who may name the member |
| --- | --- |
| `private(this)` | Methods of the declaring class, and only when the receiver is the syntactic expression `this` or the bare field name |
| `private` | Methods of the declaring class, on any instance |
| `package` | Code in the same package |
| `protected` | The declaring class and its subclasses |
| `public` | Every type that can see the declaring type |

`protected` does not include the rest of the package. A subclass in another package may use a `protected` member. A non-subclass in the same package may not.

`private(this)` on a static member is rejected. `private only` and `private(this) only` are rejected.

Interface methods are public, as [chapter 2](02-objects.md) requires. The audience of a constructor is the audience of that `new` overload.

A private type may be named only from the compilation unit that declares it. A package type may be named from its package. A public type may be named from anywhere. A member's effective audience is the intersection of its own audience and the audience of its declaring type: a public member of a package type is not visible outside the package.

## 3.2 `only`

`only` narrows an audience. It does not replace the audience keyword, and it is not the sealed-types clause.

```
only-clause = "only" type {"," type}
```

The `only` clause lists types. The member's callers are the declaring class plus the named types. Subclasses of a named type do not inherit the grant. Code in a named type may name the member. Code outside that set may not, even when the base audience would have allowed it.

The named types must sit inside the base audience's domain:

- `package only A` requires each named type to be in the same package.
- `protected only A` requires each named type to be a subclass of the declaring class.
- `public only A` may name a type in another package. The member is still not open to every caller. `public` means the grant may cross packages.

A named type that the declaring type cannot see is rejected. A duplicate name in the list is rejected.

The check is static. It uses the compile-time type of the receiver, not the runtime class.

A lambda has the access of the class whose method contains the lambda expression. It does not have the access of a functional interface it is converted to.

Generated getters and setters carry the field's audience, including its `only` list.

## 3.3 Override and `only`

A subclass that is not in the `only` list cannot override that member and cannot call it. A subclass that is in the list may override it. The override's audience must be a subset of the overridden audience: the same `only` list, or a shorter one whose types are taken from that list, with a base audience that does not add callers. Dropping `only` on the override is rejected, because that widens the set.

A member with no `only` clause uses this widening order: `private`, then `package`, then `protected`, then `public`. An override may keep the audience or move later in that order. It must not move earlier. `private(this)` members are not overridden, because the override would be a different slot. The check is static and uses the compile-time declaring class. It does not depend on the runtime class, on generics, or on nullability.

`final` and the default finality of methods are [chapter 4](04-methods.md). An `only` restriction does not by itself make a method overridable.

## 3.4 `permits`

`permits` appears only on a `sealed` class or interface. It names the types allowed to extend or implement that type. `only` appears only on a member audience. Each keyword in the other's position is rejected.

```java
public sealed class ArrayList<T> permits SubList {
    private(this) Int32 cachedHash;
    private Int32 size;
    package Object elements only ArrayListItr, SubList;
    protected Object elementAt(Int32 index) only SubList {
        return elements;
    }
}
```

`cachedHash` is visible only through the syntactic receiver `this` inside `ArrayList`. `size` is visible to every instance method of `ArrayList`, so `equals` can read the other instance's `size`. `elements` is visible to `ArrayList`, `ArrayListItr`, and `SubList`, and to no other type in the package. `elementAt` is visible to `ArrayList` and `SubList`. A further subclass of `SubList` does not receive either grant. `ArrayListItr` cannot call `elementAt`.

## 3.5 Files and types

The public type of a file, when there is one, is declared `public` and the file name matches it. A type with no modifier is private to its file. A type declared `package` is visible throughout its package. A second `public` type in the same file is rejected.
