# 3. Visibility

## 3.1 Audiences

Every type and every member has an audience: the code that may name it. <!-- r[visibility.audiences.type-member-audience] -->
A use from outside the audience is rejected. <!-- r[visibility.audiences.use-outside-audience] -->

| Written | Who may name a member |
| --- | --- |
| `private` | Code in the declaring type <!-- r[visibility.audiences.private-code-declaring] --> |
| `package` | Code in the same package <!-- r[visibility.audiences.package-code] --> |
| `protected` | Code in the declaring type and in its subclasses <!-- r[visibility.audiences.protected-code-declaring] --> |
| `public` | All code that may name the declaring type <!-- r[visibility.audiences.public-code-name] --> |

A member with no audience written is `private`. <!-- r[visibility.audiences.member-no-audience] -->
That covers fields, methods and constructors alike, with the exceptions of [section 3.5](#35-members-with-a-fixed-audience). <!-- r[visibility.audiences.covers-fields-methods] -->

`private` is by type and not by instance. <!-- r[visibility.audiences.private-type-not] -->
A method of `Account` may read a private field of another `Account`, which is what lets `equals` compare two instances. <!-- r[visibility.audiences.method-account-read] -->

`protected` does not include the rest of the package. <!-- r[visibility.audiences.protected-not-include] -->
Code in a subclass `S` uses a protected instance member only on a receiver whose static type is `S` or a subtype of `S`, or through `super`. <!-- r[visibility.audiences.code-subclass-uses] -->
It does not reach into an instance of another subclass. <!-- r[visibility.audiences.not-reach-instance] -->

A field has one audience for reading and assigning. <!-- r[visibility.audiences.field-audience-reading] -->
[Chapter 2](02-objects.md#28-fields) says that no accessor stands between them. <!-- r[visibility.audiences.says-no-accessor] -->
A field that everyone may read and only the class may assign is a private field with a public method that returns it. <!-- r[visibility.audiences.field-everyone-read] -->

```java
public class Account {
    String owner = "";                   // private
    package Rational balance = 0;
    protected void audit() { }
    public Rational balance() { return balance; }
}
```

## 3.2 Types

A type has one of three audiences. <!-- r[visibility.types.type-three-audiences] -->

| Written | Who may name the type |
| --- | --- |
| Nothing | Code in the same file <!-- r[visibility.types.nothing-code-file] --> |
| `package` | Code in the same package <!-- r[visibility.types.package-code] --> |
| `public` | All code <!-- r[visibility.types.public-code] --> |

`private` and `protected` are rejected on a type. <!-- r[visibility.types.private-protected-rejected] -->
[Chapter 1](01-source.md#14-compilation-units) ties a public type to the name of its file. <!-- r[visibility.types.ties-public-type] -->

A member is never visible more widely than its type. <!-- r[visibility.types.member-never-visible] -->
A public method of a type that only its file may name is callable from outside the file only through a supertype that declares the method. <!-- r[visibility.types.public-method-type] -->

A declaration is rejected when its own audience may see it and may not name a type in its signature. <!-- r[visibility.types.declaration-rejected-audience] -->
A public method does not take a parameter of a type that is private to the file. <!-- r[visibility.types.public-method-not] -->

## 3.3 `only`

`only` narrows an audience to a list of types. <!-- r[visibility.only.only-narrows-audience] -->

```java
public sealed class ArrayList<T> permits SubList {
    Int size = 0;
    package only(ArrayListItr, SubList) @Nullable T[] elements = new @Nullable T[8];

    protected only(SubList) T elementAt(Int index) {
        return (T) elements[index];
    }
}
```

`only(A, B)` is written directly after `package`, `protected` or `public`. <!-- r[visibility.only.only-written-directly] -->
The member may then be named by code in the declaring type and in the types listed, and by no other code. <!-- r[visibility.only.member-named-code] -->
A subclass of a listed type is not included. <!-- r[visibility.only.subclass-listed-type] -->

Each listed type must lie inside the audience that `only` narrows: <!-- r[visibility.only.listed-type-lie] -->

- after `package`, a type of the same package; <!-- r[visibility.only.after-package-type] -->
- after `protected`, a subclass of the declaring type; <!-- r[visibility.only.after-protected-subclass] -->
- after `public`, any type the declaring type may name. <!-- r[visibility.only.after-public-type] -->

A type listed twice is rejected. <!-- r[visibility.only.type-listed-twice] -->
`only` is not written on a type or after `private`. <!-- r[visibility.only.only-not-written] -->

In the example, `elements` may be named in `ArrayList`, `ArrayListItr` and `SubList`, and in no other type of the package. <!-- r[visibility.only.example-elements-named] -->
`elementAt` may be named in `ArrayList` and `SubList`. <!-- r[visibility.only.elementat-named-arraylist] -->
A subclass of `SubList` receives neither grant. <!-- r[visibility.only.subclass-sublist-receives] -->

## 3.4 Audiences and overriding

A method can be overridden only by a class that may name it. <!-- r[visibility.overriding.method-overridden-only] -->
An overriding method has the audience of the method it overrides, or a wider one in the order `package`, `protected`, `public`. <!-- r[visibility.overriding.overriding-method-audience] -->
It is never narrower. <!-- r[visibility.overriding.never-narrower] -->

A private method is not overridden. <!-- r[visibility.overriding.private-method-not] -->
A subclass that declares a method with the same name and signature declares a new method. <!-- r[visibility.overriding.subclass-declares-method] -->

When the overridden method has an `only` list, the overriding method has the same audience keyword and a list drawn from that list. <!-- r[visibility.overriding.overridden-method-only] -->
It does not drop `only`. <!-- r[visibility.overriding.not-drop-only] -->

A method that implements an interface method is `public`. <!-- r[visibility.overriding.method-implements-interface] -->

## 3.5 Members with a fixed audience

- The methods of an interface are `public`, and so are its fields, as [chapter 2](02-objects.md#26-interfaces) says. <!-- r[visibility.fixed.methods-interface-public] -->
  An audience written on one of them is rejected unless it is `public`. <!-- r[visibility.fixed.audience-written-rejected] -->
- The constants of an enum are `public`. <!-- r[visibility.fixed.constants-enum-public] -->
  Its constructors are `private`. <!-- r[visibility.fixed.constructors-private] -->
- The elements of an annotation are `public`. <!-- r[visibility.fixed.elements-annotation-public] -->
- A class that declares no constructor has one with the audience of the class, as [chapter 9](09-execution.md#95-constructing-an-object) says. <!-- r[visibility.fixed.class-declares-no] -->
  A written constructor with no audience is `private`. <!-- r[visibility.fixed.written-constructor-no] -->
- `main` of the entry class is `public`. <!-- r[visibility.fixed.main-entry-class] -->

## 3.6 What an audience governs

An audience governs naming. <!-- r[visibility.governs.audience-governs-naming] -->
The check uses the type that declares the member and the place where the name is written. <!-- r[visibility.governs.check-uses-type] -->
It does not depend on the class of an object at run time. <!-- r[visibility.governs.not-depend-class] -->

- A lambda has the access of the method that contains it. <!-- r[visibility.governs.lambda-access-method] -->
- An instance may be used through any supertype whose members the code may name. <!-- r[visibility.governs.instance-used-through] -->
  A private class that implements a public interface is used through that interface from anywhere. <!-- r[visibility.governs.private-class-implements] -->
- The mirrors of [chapter 8](08-annotations.md#89-reading-annotations) use only members declared `public`. <!-- r[visibility.governs.mirrors-use-only] -->
  An annotated public member of a type that is not public can be used through its mirror by code that could not name the type. <!-- r[visibility.governs.annotated-public-member] -->
  That is the one case where a mirror reaches further than a name. <!-- r[visibility.governs.case-mirror-reaches] -->

`sealed` and `permits` restrict which types may extend or implement a type. <!-- r[visibility.governs.sealed-permits-restrict] -->
They are not audiences, and [chapter 2](02-objects.md#24-extension) defines them. <!-- r[visibility.governs.not-audiences-defines] -->
