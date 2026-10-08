# 3. Visibility

## 3.1 Audiences

Every type and every member has an audience: the code that may name it. A use from outside the audience is rejected.

| Written | Who may name a member |
| --- | --- |
| `private` | Code in the declaring type |
| `package` | Code in the same package |
| `protected` | Code in the declaring type and in its subclasses |
| `public` | All code that may name the declaring type |

A member with no audience written is `private`. That covers fields, methods and constructors alike, with the exceptions of [section 3.5](#35-members-with-a-fixed-audience).

`private` is by type and not by instance. A method of `Account` may read a private field of another `Account`, which is what lets `equals` compare two instances.

`protected` does not include the rest of the package. Code in a subclass `S` uses a protected instance member only on a receiver whose static type is `S` or a subtype of `S`, or through `super`. It does not reach into an instance of another subclass.

A field has one audience for reading and assigning. [Chapter 2](02-objects.md#28-fields) says that no accessor stands between them. A field that everyone may read and only the class may assign is a private field with a public method that returns it.

```java
public class Account {
    String owner = "";                   // private
    package Rational balance = 0;
    protected void audit() { }
    public Rational balance() { return balance; }
}
```

## 3.2 Types

A type has one of three audiences.

| Written | Who may name the type |
| --- | --- |
| Nothing | Code in the same file |
| `package` | Code in the same package |
| `public` | All code |

`private` and `protected` are rejected on a type. [Chapter 1](01-source.md#14-compilation-units) ties a public type to the name of its file.

A member is never visible more widely than its type. A public method of a type that only its file may name is callable from outside the file only through a supertype that declares the method.

A declaration is rejected when its own audience may see it and may not name a type in its signature. A public method does not take a parameter of a type that is private to the file.

## 3.3 `only`

`only` narrows an audience to a list of types.

```java
public sealed class ArrayList<T> permits SubList {
    Int size = 0;
    package only(ArrayListItr, SubList) @Nullable T[] elements = new @Nullable T[8];

    protected only(SubList) T elementAt(Int index) {
        return (T) elements[index];
    }
}
```

`only(A, B)` is written directly after `package`, `protected` or `public`. The member may then be named by code in the declaring type and in the types listed, and by no other code. A subclass of a listed type is not included.

Each listed type must lie inside the audience that `only` narrows:

- after `package`, a type of the same package;
- after `protected`, a subclass of the declaring type;
- after `public`, any type the declaring type may name.

A type listed twice is rejected. `only` is not written on a type or after `private`.

In the example, `elements` may be named in `ArrayList`, `ArrayListItr` and `SubList`, and in no other type of the package. `elementAt` may be named in `ArrayList` and `SubList`. A subclass of `SubList` receives neither grant.

## 3.4 Audiences and overriding

A method can be overridden only by a class that may name it. An overriding method has the audience of the method it overrides, or a wider one in the order `package`, `protected`, `public`. It is never narrower.

A private method is not overridden. A subclass that declares a method with the same name and signature declares a new method.

When the overridden method has an `only` list, the overriding method has the same audience keyword and a list drawn from that list. It does not drop `only`.

A method that implements an interface method is `public`.

## 3.5 Members with a fixed audience

- The methods of an interface are `public`, and so are its fields, as [chapter 2](02-objects.md#26-interfaces) says. An audience written on one of them is rejected unless it is `public`.
- The constants of an enum are `public`. Its constructors are `private`.
- The elements of an annotation are `public`.
- A class that declares no constructor has one with the audience of the class, as [chapter 9](09-execution.md#95-constructing-an-object) says. A written constructor with no audience is `private`.
- `main` of the entry class is `public`.

## 3.6 What an audience governs

An audience governs naming. The check uses the type that declares the member and the place where the name is written. It does not depend on the class of an object at run time.

- A lambda has the access of the method that contains it.
- An instance may be used through any supertype whose members the code may name. A private class that implements a public interface is used through that interface from anywhere.
- The mirrors of [chapter 8](08-annotations.md#89-reading-annotations) use only members declared `public`. An annotated public member of a type that is not public can be used through its mirror by code that could not name the type. That is the one case where a mirror reaches further than a name.

`sealed` and `permits` restrict which types may extend or implement a type. They are not audiences, and [chapter 2](02-objects.md#24-extension) defines them.
