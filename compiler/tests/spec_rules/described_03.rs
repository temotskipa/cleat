//! Chapter 3, Visibility: the sentences that define and describe.

use super::{held, rule, stated, Rule};

/// The example of section 3.3, with types that one file may declare.
macro_rules! array_list {
    ($more:literal) => {
        concat!(
            "sealed class ArrayList<T> permits SubList { Int size = 0; package only(ArrayListItr, SubList) @Nullable T[] elements = new @Nullable T[8]; ",
            "protected only(SubList) T elementAt(Int index) { return (T) elements[index]; } Int own() { return elements.length(); } } ",
            "class ArrayListItr { Int f(ArrayList<Int> l) { return l.elements.length(); } } ",
            $more
        )
    };
}

const OUTSIDE: &str = "is outside its audience here";

pub const SENTENCES: &[Rule] = &[
    // ---- 3.1 ----
    rule("03", "Every type and every member has an audience: the code that may name it", &[], &["==== A.cleat\npublic class A { private Int a = 1; package Int b = 2; protected Int c = 3; public Int d = 4; Int e = 5; }"]),
    rule(
        "03",
        "| `private` | Code in the declaring type |",
        &[("class A { private Int x = 1; } class B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")],
        &["class A { private Int x = 1; Int f(A other) { return x + other.x; } }"],
    ),
    rule(
        "03",
        "| `package` | Code in the same package |",
        &[
            ("==== a/A.cleat\npackage a; public class A { package Int x = 1; }\n==== b/B.cleat\npackage b; class B { Int f(a.A v) { return v.x; } }", "the field `x` of `A` is outside its audience here"),
            ("==== a/P.cleat\npackage a; package class P { }\n==== b/B.cleat\npackage b; class B { void f(a.P p) { } }", "`a.P` is not visible here"),
        ],
        &["==== a/A.cleat\npackage a; public class A { package Int x = 1; }\n==== a/P.cleat\npackage a; package class P { Int f(A v) { return v.x; } }\n==== a/Q.cleat\npackage a; class Q { void g(P p) { } }"],
    ),
    rule(
        "03",
        "| `protected` | Code in the declaring type and in its subclasses |",
        &[("==== a/A.cleat\npackage a; public open class A { protected Int x = 1; }\n==== a/B.cleat\npackage a; class B { Int f(A v) { return v.x; } }", "the field `x` of `A` is outside its audience here")],
        &["==== a/A.cleat\npackage a; public open class A { protected Int x = 1; Int own(A v) { return v.x; } }\n==== b/B.cleat\npackage b; class B extends a.A { Int f() { return x; } }"],
    ),
    rule("03", "| `public` | All code that may name the declaring type |", &[], &["==== a/A.cleat\npackage a; public class A { public Int x = 1; public Int f() { return x; } }\n==== b/B.cleat\npackage b; class B { Int g(a.A v) { return v.x + v.f(); } }"]),
    rule("03", "A member with no audience written is `private`", &[("class A { Int x = 1; } class B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")], &["class A { Int x = 1; Int f() { return x; } }"]),
    rule(
        "03",
        "That covers fields, methods and constructors alike",
        &[
            ("class A { Int x = 1; } class B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here"),
            ("class A { Int g() { return 1; } } class B { Int f(A a) { return a.g(); } }", "the method `g` of `A` is outside its audience here"),
            ("class A { A() { } } class B { A f() { return new A(); } }", "the constructor of `A` is outside its audience here"),
        ],
        &[],
    ),
    rule("03", "`private` is by type and not by instance", &[], &["class Account { private Int balance = 0; Boolean richer(Account other) { return balance > other.balance; } }"]),
    rule(
        "03",
        "A method of `Account` may read a private field of another `Account`",
        &[],
        &["class Account { private Int balance = 0; @Override public Boolean equals(@Nullable Object other) { return other instanceof Account && balance == other.balance; } @Override public Int hashCode() { return balance; } }"],
    ),
    rule(
        "03",
        "A field has one audience for reading and assigning",
        &[
            ("class A { private Int x = 1; } class B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here"),
            ("class A { private Int x = 1; } class B { void f(A a) { a.x = 2; } }", "the field `x` of `A` is outside its audience here"),
        ],
        &["class A { public Int x = 1; } class B { Int f(A a) { a.x = 2; return a.x; } }"],
    ),
    stated("03", "says that no accessor stands between them", "It points to section 2.8, whose sentences have their own cases."),
    // ---- 3.2 ----
    rule("03", "A type has one of three audiences", &[], &["==== A.cleat\npublic class A { } package class B { } class C { }"]),
    rule(
        "03",
        "| Nothing | Code in the same file |",
        &[("==== A.cleat\nclass Hidden { }\n==== B.cleat\nclass B { void f(Hidden h) { } }", "there is no type named `Hidden` here")],
        &["==== A.cleat\nclass Hidden { } class Beside { void f(Hidden h) { } }"],
    ),
    rule("03", "| `public` | All code |", &[], &["==== a/A.cleat\npackage a; public class A { }\n==== b/B.cleat\npackage b; class B { void f(a.A v) { } }\n==== C.cleat\nclass C { void f(a.A v) { } }"]),
    rule("03", "ties a public type to the name of its file", &[("==== A.cleat\npublic class B { }", "the public type `B` must be in a file named `B.cleat`")], &["==== B.cleat\npublic class B { }"]),
    // ---- 3.3 ----
    rule(
        "03",
        "`only` narrows an audience to a list of types",
        &[("class A { package only(B) Int x = 1; } class B { } class C { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")],
        &["class A { package only(B) Int x = 1; } class B { Int f(A a) { return a.x; } }"],
    ),
    rule(
        "03",
        "is written directly after `package`, `protected` or `public`",
        &[],
        &["open class A { package only(B) Int w = 0; protected only(B) Int x = 1; public only(C) Int y = 2; } class B extends A { } class C { }"],
    ),
    rule(
        "03",
        "The member may then be named by code in the declaring type and in the types listed, and by no other code",
        &[("class A { public only(B) Int x = 1; } class B { } class C { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")],
        &["class A { public only(B) Int x = 1; Int own(A other) { return x + other.x; } } class B { Int f(A a) { return a.x; } }"],
    ),
    rule(
        "03",
        "A subclass of a listed type is not included",
        &[("class A { package only(B) Int x = 1; } open class B { } class C extends B { Int f(A a) { return a.x; } }", "the field `x` of `A` is outside its audience here")],
        &[],
    ),
    rule("03", "after `package`, a type of the same package;", &[], &["==== a/A.cleat\npackage a; public class A { package only(B) Int x = 1; }\n==== a/B.cleat\npackage a; public class B { Int f(A v) { return v.x; } }"]),
    rule("03", "after `protected`, a subclass of the declaring type;", &[], &["open class A { protected only(B) Int x = 1; } class B extends A { Int f() { return x; } }"]),
    rule(
        "03",
        "after `public`, any type the declaring type may name.",
        &[],
        &["==== a/A.cleat\npackage a; import b.B; public class A { public only(B) Int y = 2; }\n==== b/B.cleat\npackage b; import a.A; public class B { Int f(A v) { return v.y; } }"],
    ),
    rule(
        "03",
        "and in no other type of the package",
        &[(array_list!("open class SubList<T> extends ArrayList<T> { } class Other { Int f(ArrayList<Int> l) { return l.elements.length(); } }"), "the field `elements` of `ArrayList` is outside its audience here")],
        &[array_list!("open class SubList<T> extends ArrayList<T> { Int h() { return elements.length(); } }")],
    ),
    rule(
        "03",
        "`elementAt` may be named in `ArrayList` and `SubList`",
        &[(array_list!("open class SubList<T> extends ArrayList<T> { } class Itr2 { Int f(ArrayList<Int> l) { return l.elementAt(0); } }"), "the method `elementAt` of `ArrayList` is outside its audience here")],
        &[array_list!("open class SubList<T> extends ArrayList<T> { T g() { return elementAt(0); } }")],
    ),
    rule(
        "03",
        "A subclass of `SubList` receives neither grant",
        &[
            (array_list!("open class SubList<T> extends ArrayList<T> { } class Deep<T> extends SubList<T> { T k() { return elementAt(0); } }"), "the method `elementAt` of `ArrayList` is outside its audience here"),
            (array_list!("open class SubList<T> extends ArrayList<T> { } class Deep<T> extends SubList<T> { Int k() { return elements.length(); } }"), "the field `elements` of `ArrayList` is outside its audience here"),
        ],
        &[],
    ),
    // ---- 3.4 ----
    rule(
        "03",
        "or a wider one in the order `package`, `protected`, `public`",
        &[("open class A { protected open Int f() { return 1; } } class B extends A { @Override package Int f() { return 2; } }", "an overriding method has the audience of the method it overrides, or a wider one")],
        &["open class A { package open Int f() { return 1; } protected open Int g() { return 1; } public open Int h() { return 1; } } class B extends A { @Override protected Int f() { return 2; } @Override public Int g() { return 2; } @Override public Int h() { return 2; } }"],
    ),
    rule(
        "03",
        "A private method is not overridden",
        &[("open class A { private Int f() { return 1; } } class B extends A { @Override public Int f() { return 2; } }", "`f` carries `@Override` and overrides nothing")],
        &["open class A { private Int f() { return 1; } } class B extends A { public Int f() { return 2; } }"],
    ),
    held("03", "A subclass that declares a method with the same name and signature declares a new method", "behavior", "a_method_a_class_cannot_name_is_not_overridden"),
    rule(
        "03",
        "the overriding method has the same audience keyword and a list drawn from that list",
        &[
            ("open class A { protected only(B, C) open Int f() { return 1; } } class B extends A { @Override public only(B) Int f() { return 2; } } class C extends A { }", "the overridden method has an `only` list"),
            ("open class A { public only(B, C) open Int f() { return 1; } } class B extends A { @Override public only(D) Int f() { return 2; } } class C { } class D { }", "the overridden method has an `only` list"),
        ],
        &["open class A { public only(B, C) open Int f() { return 1; } } class B extends A { @Override public only(C) Int f() { return 2; } } class C { }"],
    ),
    rule(
        "03",
        "A method that implements an interface method is `public`",
        &[("interface I { Int f(); } class C implements I { @Override protected Int f() { return 1; } }", "an overriding method has the audience of the method it overrides, or a wider one")],
        &["interface I { Int f(); } class C implements I { @Override public Int f() { return 1; } }"],
    ),
    // ---- 3.5 ----
    rule(
        "03",
        "The methods of an interface are `public`, and so are its fields",
        &[],
        &["==== a/I.cleat\npackage a; public interface I { Int f(); Int X = 1; }\n==== b/B.cleat\npackage b; class B { Int g(a.I i) { return i.f() + a.I.X; } }"],
    ),
    rule("03", "The constants of an enum are `public`", &[], &["==== a/E.cleat\npackage a; public enum E { X, Y }\n==== b/B.cleat\npackage b; class B { a.E f() { return a.E.X; } }"]),
    rule("03", "Its constructors are `private`", &[("enum E { A; public E() { } }", "the constructors of an enum are private"), ("enum E { A; E() { } } class C { E f() { return new E(); } }", "no expression creates an instance of an enum")], &["enum E { A; E() { } }"]),
    rule(
        "03",
        "The elements of an annotation are `public`",
        &[],
        &["==== a/Route.cleat\npackage a; public annotation Route(String path, Int priority = 0);\n==== b/B.cleat\npackage b; class B { String f(a.Route r) { return r.path + r.priority; } }"],
    ),
    rule(
        "03",
        "A class that declares no constructor has one with the audience of the class",
        &[],
        &["==== a/P.cleat\npackage a; public class P { }\n==== a/Q.cleat\npackage a; package class Q { }\n==== a/R.cleat\npackage a; class R { Q f() { return new Q(); } }\n==== b/B.cleat\npackage b; class B { a.P f() { return new a.P(); } }"],
    ),
    held("03", "`main` of the entry class is `public`", "spec_rules", "an_entry_class_declares_exactly_one_main"),
    // ---- 3.6 ----
    stated("03", "An audience governs naming", "It introduces the points after it, which have their own cases."),
    rule(
        "03",
        "The check uses the type that declares the member and the place where the name is written",
        &[("open class A { private Int x = 1; } class B extends A { Int f() { return x; } }", OUTSIDE)],
        &["open class A { public Int x = 1; } class B extends A { } class C { Int f(B b) { return b.x; } }"],
    ),
    rule("03", "A lambda has the access of the method that contains it", &[], &["class A { private Int x = 1; private Int g() { return 2; } Function0<Int> f() { return () -> x + g(); } }"]),
    rule(
        "03",
        "An instance may be used through any supertype whose members the code may name",
        &[],
        &["==== Shape.cleat\npublic interface Shape { Int area(); }\n==== Shapes.cleat\nclass Square implements Shape { @Override public Int area() { return 4; } } public class Shapes { public static Shape make() { return new Square(); } }\n==== B.cleat\nclass B { Int f() { Shape s = Shapes.make(); Object o = s; return s.area() + o.hashCode(); } }"],
    ),
    rule(
        "03",
        "A private class that implements a public interface is used through that interface from anywhere",
        &[],
        &["==== a/Shape.cleat\npackage a; public interface Shape { Int area(); }\n==== a/Shapes.cleat\npackage a; class Square implements Shape { @Override public Int area() { return 4; } } public class Shapes { public static Shape make() { return new Square(); } }\n==== b/B.cleat\npackage b; class B { Int f() { return a.Shapes.make().area(); } }"],
    ),
    held("03", "can be used through its mirror by code that could not name the type", "behavior", "a_mirror_reaches_a_public_member_of_a_type_the_code_cannot_name"),
    held("03", "That is the one case where a mirror reaches further than a name", "behavior", "a_mirror_reaches_a_public_member_of_a_type_the_code_cannot_name"),
    rule(
        "03",
        "`sealed` and `permits` restrict which types may extend or implement a type",
        &[("sealed interface S permits P { } class P implements S { } class Q implements S { }", "`S` is sealed and does not permit this type")],
        &["sealed interface S permits P { } class P implements S { }"],
    ),
];
