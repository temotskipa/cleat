//! Chapter 9, Execution: the sentences that define and describe.

use super::{FINAL_LOCAL, Rule, held, ran, rule, stated};

const RUN: &str = "Sentences09";
const PERMISSION: &str =
    "It says what an implementation may do, and no run shows that it does not.";
const BEFORE_SUPER: &str = "before `super(...)` is called, `this` is used only to assign a field of the class and to read one that is assigned";
const NOT_ASSIGNED_FIELD: &str = "is not assigned on every path to the `super(...)` call";

pub const SENTENCES: &[Rule] = &[
    // ---- 9.1 ----
    ran(
        "09",
        "Operands are evaluated from left to right",
        RUN,
        "operands: left right",
    ),
    ran(
        "09",
        "A receiver is evaluated before its arguments, and the arguments from left to right",
        RUN,
        "call: receiver first second",
    ),
    ran(
        "09",
        "Every argument is evaluated before the method runs",
        RUN,
        "call: receiver first second method",
    ),
    ran(
        "09",
        "the rest of the expression is not evaluated",
        RUN,
        "raised in an expression: before",
    ),
    // ---- 9.2 ----
    rule(
        "09",
        "An exception is an instance of `Throwable` or of a subclass of it",
        &[
            (
                "class A { void f() { throw new Object(); } }",
                "`throw` raises a `Throwable`, and this is a `Object`",
            ),
            (
                "class A { void f() { throw \"text\"; } }",
                "`throw` raises a `Throwable`, and this is a `String`",
            ),
        ],
        &[
            "class E extends Throwable { } class A { void f() { throw new E(); } void g() { throw new Throwable(\"x\"); } }",
        ],
    ),
    ran(
        "09",
        "`message()` is the message given at construction, or the empty string",
        RUN,
        "message: [why] []",
    ),
    ran(
        "09",
        "`cause()` is the exception given at construction as the reason for this one, or `null`",
        RUN,
        "cause: true true",
    ),
    ran(
        "09",
        "`suppressed()` returns a new array of the exceptions",
        RUN,
        "suppressed: 2 in order, close of q and close of p, and a new array true",
    ),
    ran(
        "09",
        "The default `toString` of an exception is the name of its class",
        RUN,
        "toString: t.MyError | t.MyError: why",
    ),
    ran(
        "09",
        "` and the message.",
        RUN,
        "toString: t.MyError | t.MyError: why",
    ),
    ran(
        "09",
        "`throw e` raises an exception, as chapter 4 describes",
        RUN,
        "propagates: loop-0 after-0 loop-1 caught out",
    ),
    ran(
        "09",
        "A raised exception ends the statement that raised it, and then each enclosing statement in turn",
        RUN,
        "propagates: loop-0 after-0 loop-1 caught out",
    ),
    ran(
        "09",
        "the call of the method raises it in the caller",
        RUN,
        "from a call: from the callee",
    ),
    held(
        "09",
        "An exception that no caller handles ends the thread",
        "host",
        "an_uncaught_exception_ends_the_program_with_status_one",
    ),
    rule(
        "09",
        "There are no checked exceptions",
        &[(
            "class A { void f() throws Throwable { } }",
            "expected `{`, found `throws`",
        )],
        &[
            "class E extends Throwable { } class A { void f() { throw new E(); } void g() { f(); } }",
        ],
    ),
    held(
        "09",
        "The prelude declares these subclasses of `Throwable`",
        "spec_tables",
        "the_prelude_declares_the_exceptions_of_section_9_2",
    ),
    rule(
        "09",
        "Each is an open class, and a program may declare more",
        &[],
        &[
            "class A extends ArithmeticException { } class B extends ClassCastException { } class C extends IndexOutOfBoundsException { } class D extends IllegalArgumentException { } class E extends IllegalStateException { } class F extends IllegalAccessException { } class G extends AssertionException { } class H extends ClassInitializationException { } class I extends OutOfMemoryException { } class J extends CancellationException { } class K extends Throwable { }",
        ],
    ),
    ran(
        "09",
        "| `ClassCastException` | A cast fails |",
        RUN,
        "a cast fails: ClassCastException",
    ),
    ran(
        "09",
        "| `IndexOutOfBoundsException` | An index or a range is outside its bounds |",
        RUN,
        "outside its bounds: index range",
    ),
    ran(
        "09",
        "| `IllegalAccessException` | A mirror of",
        RUN,
        "a mirror is refused: not public, final",
    ),
    ran(
        "09",
        "| `AssertionException` | An assertion is `false` |",
        RUN,
        "an assertion is false: AssertionException",
    ),
    ran(
        "09",
        "| `ClassInitializationException` | A static field is read before it is assigned",
        RUN,
        "read too early: raises",
    ),
    ran(
        "09",
        "| `CancellationException` | A task is cancelled, as chapter 13 defines |",
        "Sentences13",
        "points: raised",
    ),
    stated(
        "09",
        "An implementation may record where an exception was created",
        PERMISSION,
    ),
    // ---- 9.3 ----
    rule(
        "09",
        "A `try` statement has a block, any number of `catch` clauses, and an optional `finally` block",
        &[],
        &[
            "class A { void f() { try { } finally { } try { } catch (Throwable e) { } try { } catch (IllegalStateException e) { } catch (ArithmeticException e) { } catch (Throwable e) { } finally { } } }",
        ],
    ),
    rule(
        "09",
        "It has at least one `catch` clause or a `finally` block",
        &[(
            "class A { void f() { try { } } }",
            "a `try` has at least one `catch` clause or a `finally` block",
        )],
        &["class A { void f() { try { } finally { } } }"],
    ),
    ran(
        "09",
        "The `try` block runs first",
        RUN,
        "try: try clause for thrown finally",
    ),
    ran(
        "09",
        "the `catch` clauses are examined in source order",
        RUN,
        "try: try clause for thrown",
    ),
    ran(
        "09",
        "The first clause whose type the exception is an instance of handles it",
        RUN,
        "try: try clause for thrown",
    ),
    ran(
        "09",
        "If no clause accepts the exception, it goes on propagating",
        RUN,
        "no clause: goes on to the outer, inner",
    ),
    rule(
        "09",
        "The type of a clause is `Throwable` or a subclass of it",
        &[
            (
                "class A { void f() { try { } catch (String e) { } } }",
                "a `catch` clause names `Throwable` or a subclass of it, and this is `String`",
            ),
            (
                "class A { void f() { try { } catch (Object e) { } } }",
                "a `catch` clause names `Throwable` or a subclass of it, and this is `Object`",
            ),
        ],
        &[
            "class E extends IllegalStateException { } class A { void f() { try { } catch (E e) { } catch (Throwable e) { } } }",
        ],
    ),
    rule(
        "09",
        "The parameter is a local of the clause's block",
        &[
            (
                "class A { void f() { try { } catch (Throwable e) { } var m = e.message(); } }",
                "`e` is not a variable, a field or a type in scope",
            ),
            (
                "class A { void f() { try { } catch (Throwable e) { var e = 1; } } }",
                "`e` is already a variable of this method",
            ),
        ],
        &[
            "class A { String f() { try { return \"\"; } catch (Throwable e) { var m = e.message(); return m; } } }",
        ],
    ),
    ran(
        "09",
        "An exception raised inside a `catch` block is not handled by the other clauses of the same statement",
        RUN,
        "in a catch block: raised in a catch block []",
    ),
    ran(
        "09",
        "The `finally` block runs last, however the `try` block and the `catch` block completed",
        RUN,
        "finally after return: returned finally",
    ),
    ran(
        "09",
        "The statement then completes in that same way",
        RUN,
        "finally after an exception: raised finally",
    ),
    ran(
        "09",
        "the new exception propagates and the earlier one is recorded as suppressed on it",
        RUN,
        "finally raises: from finally, suppressed 1: earlier",
    ),
    // ---- 9.4 ----
    ran(
        "09",
        "evaluates `e`, binds `name` to its value, runs the block, and then calls `name.close()`",
        RUN,
        "using: open-a block close-a",
    ),
    rule(
        "09",
        "`name` is not assignable",
        &[(
            "class R { public void close() { } } class A { void f() { using (R r = new R()) { r = new R(); } } }",
            FINAL_LOCAL,
        )],
        &[
            "class R { public void close() { } } class A { void f() { using (R r = new R()) { var s = r; } } }",
        ],
    ),
    rule(
        "09",
        "`var` may be written for `T`",
        &[],
        &[
            "class R { public void close() { } } class A { void f() { using (var r = new R()) { } using (R a = new R(), var b = new R()) { } } }",
        ],
    ),
    ran(
        "09",
        "the exception from the block propagates and the one from `close` is recorded as suppressed on it",
        RUN,
        "block and close raise: from the block, suppressed 1: close of b",
    ),
    ran(
        "09",
        "If the block completed in any other way and `close` raises, that exception propagates",
        RUN,
        "close raises: close of c",
    ),
    ran(
        "09",
        "Several resources may be written, separated by commas",
        RUN,
        "two resources: open-1 open-2",
    ),
    ran(
        "09",
        "Each is evaluated and bound in order, and they are closed in the reverse order",
        RUN,
        "two resources: open-1 open-2 block close-2 close-1",
    ),
    ran(
        "09",
        "the resources already bound are closed, and the one being initialized is not",
        RUN,
        "an initializer raises: no 2 [open-1 close-1 ]",
    ),
    // ---- 9.5 ----
    rule(
        "09",
        "A constructor of a reference class is declared with the name of the class, a parameter list and a body",
        &[("class A { public B() { } }", "expected a name, found `(`")],
        &["class A { Int x; public A(Int x) { this.x = x; } }"],
    ),
    rule(
        "09",
        "A class may declare several, and `new` chooses among them as chapter 4 chooses a method",
        &[(
            "class A { Int x; public A() { this.x = 0; } public A(Int x) { this.x = x; } static void f() { var a = new A(true); } }",
            "no constructor `A` accepts the arguments (Boolean)",
        )],
        &[
            "class A { Int x; public A() { this.x = 0; } public A(Int x) { this.x = x; } public A(String s) { this.x = s.length(); } static void f() { var a = new A(); var b = new A(1); var c = new A(\"s\"); } }",
        ],
    ),
    rule(
        "09",
        "A class that declares no constructor has one, with no parameters, an empty body, and the audience of the class",
        &[(
            "class A { } class B { void f() { var a = new A(1); } }",
            "constructor `A` takes 0 arguments, and 1 is written",
        )],
        &[
            "class A { } class B { void f() { var a = new A(); } }",
            "==== a/A.cleat\npackage a; public class A { }\n==== b/B.cleat\npackage b; import a.A; class B { void f() { var a = new A(); } }",
        ],
    ),
    ran(
        "09",
        "`super(args)` calls a constructor of the superclass",
        RUN,
        "construction: Base sees size 5 Derived done",
    ),
    ran(
        "09",
        "`this(args)` calls another constructor of the same class",
        "Rules",
        "body of Steps(Int)",
    ),
    rule(
        "09",
        "A constructor body contains at most one such call",
        &[
            (
                "open class B { public B(Int x) { } } class D extends B { public D() { super(1); super(2); } }",
                "a constructor body contains at most one `super(...)` or `this(...)` call",
            ),
            (
                "open class B { public B(Int x) { } } class D extends B { public D(Boolean c) { if (c) { super(1); } } }",
                "`super(...)` is written as a statement of a constructor's body itself, not inside another statement",
            ),
        ],
        &["open class B { public B(Int x) { } } class D extends B { public D() { super(1); } }"],
    ),
    rule(
        "09",
        "`this(args)`, when written, is the first statement",
        &[(
            "class A { Int x; public A() { var y = 1; this(y); } public A(Int x) { this.x = x; } }",
            "`this(...)`, when written, is the first statement of the constructor",
        )],
        &["class A { Int x; public A() { this(1); } public A(Int x) { this.x = x; } }"],
    ),
    rule(
        "09",
        "A body that writes neither call ends with an implicit `super()`",
        &[(
            "open class B { public B(Int x) { } } class D extends B { public D() { } }",
            "constructor `B` takes 1 argument, and 0 are written",
        )],
        &["open class B { } class D extends B { Int x; public D() { this.x = 1; } }"],
    ),
    rule(
        "09",
        "the constructor is filling in the fields of its own class",
        &[],
        &[
            "open class B { } class D extends B { Int x; Int y; public D(Int v) { this.x = v; y = x + 1; super(); } }",
        ],
    ),
    rule(
        "09",
        "`this` is not passed as an argument, stored, returned, or captured by a lambda",
        &[
            (
                "class Reg { public static void add(Object o) { } } class A { Int x; public A() { Reg.add(this); this.x = 1; } }",
                BEFORE_SUPER,
            ),
            (
                "class A { Int x; A other; public A() { this.other = this; this.x = 1; } }",
                BEFORE_SUPER,
            ),
            (
                "class A { Int x; public A() { Function0<Int> g = () -> this.x; this.x = 1; } }",
                "a lambda does not capture `this` before `super(...)` is called",
            ),
        ],
        &[],
    ),
    rule(
        "09",
        "No inherited instance member is used",
        &[
            (
                "open class B { public Int inherited = 1; } class D extends B { Int x; public D() { this.x = inherited; super(); } }",
                "the inherited field `inherited` is not used before `super(...)` is called",
            ),
            (
                "open class B { public Int get() { return 1; } } class D extends B { Int x; public D() { this.x = get(); super(); } }",
                "no method is called on `this` before `super(...)` is called, and `get` is an instance method",
            ),
        ],
        &[
            "open class B { public Int inherited = 1; public Int get() { return 1; } } class D extends B { Int x; public D() { this.x = 1; super(); this.x = inherited + get(); } }",
        ],
    ),
    rule(
        "09",
        "The arguments of the `super(...)` or `this(...)` call are under the same restriction",
        &[
            (
                "open class B { public B(Int x) { } public Int inherited = 1; } class D extends B { public D() { super(this.inherited); } }",
                "the inherited field `inherited` is not used before `super(...)` is called",
            ),
            (
                "class A { Int x; public A() { this(helper()); } public A(Int x) { this.x = x; } Int helper() { return 1; } }",
                "no method is called on `this` before `super(...)` is called, and `helper` is an instance method",
            ),
        ],
        &[
            "open class B { public B(Int x) { } } class D extends B { Int y; public D(Int v) { this.y = v; super(y + 1); } }",
        ],
    ),
    rule(
        "09",
        "An initializer is under the same restriction",
        &[(
            "class A { Int x = helper(); Int helper() { return 1; } }",
            "no method is called on `this` before `super(...)` is called, and `helper` is an instance method",
        )],
        &["class A { Int a = 1; Int b = a + 1; }"],
    ),
    ran(
        "09",
        "A `@Nullable` field is the exception",
        RUN,
        "a nullable field: true",
    ),
    ran(
        "09",
        "the object is complete: every field of the class, of its superclasses and of its subclasses is assigned",
        RUN,
        "construction: Base sees size 5",
    ),
    rule(
        "09",
        "From there `this` may be used without restriction",
        &[],
        &[
            "class Reg { public static void add(Object o) { } } class A { Int x; @Nullable A self; public A() { this.x = 1; super(); Reg.add(this); this.self = this; Function0<Int> g = () -> this.x; var y = describe(); } String describe() { return \"a\"; } }",
        ],
    ),
    rule(
        "09",
        "A final field is not assigned there",
        &[(
            "class A { final Int x; public A() { this.x = 1; super(); this.x = 2; } }",
            "the field `x` is not assigned here: it is final",
        )],
        &[
            "class A { final Int x; Int y; public A() { this.x = 1; this.y = 1; super(); this.y = 2; } }",
        ],
    ),
    ran(
        "09",
        "A subclass assigns its fields before it calls `super`",
        RUN,
        "construction: Base sees size 5",
    ),
    rule(
        "09",
        "No constructor and no method ever sees a field that has not been assigned",
        &[
            ("class A { Int x; public A() { } }", NOT_ASSIGNED_FIELD),
            (
                "class A { Int x; Int y; public A() { this.y = x; this.x = 1; } }",
                "the field `x` is read before it is assigned",
            ),
        ],
        &[],
    ),
    // ---- 9.6 ----
    rule(
        "09",
        "A value class has exactly one constructor",
        &[
            (
                "value class V { public Int x; public V { } public V { } }",
                "a value class has exactly one constructor",
            ),
            (
                "value class V { public Int x; public V(Int x) { this.x = x; } }",
                "a value class has one constructor, whose parameters are its fields",
            ),
        ],
        &["value class V { public Int x; public V { } }"],
    ),
    rule(
        "09",
        "Its parameters are the fields of the class, in declaration order, with their types",
        &[
            (
                "value class V { public Int x; public String y; } class A { void f() { var v = new V(1); } }",
                "constructor `V` takes 2 arguments, and 1 is written",
            ),
            (
                "value class V { public Int x; public String y; } class A { void f() { var v = new V(\"a\", 2); } }",
                "`String` is not assignable to `Int`",
            ),
        ],
        &[
            "value class V { public Int x; public String y; } class A { void f() { var v = new V(1, \"a\"); } }",
        ],
    ),
    ran(
        "09",
        "evaluates the arguments from left to right and produces the value with those fields",
        RUN,
        "value constructor: x y z Vec3(1, 2, 3)",
    ),
    rule(
        "09",
        "A value class may write that constructor in a compact form, to give it an audience or a check",
        &[(
            "==== a/V.cleat\npackage a; public value class V { public Int x; package V { } }\n==== b/B.cleat\npackage b; class B { void f() { var v = new a.V(1); } }",
            "the constructor of `V` is outside its audience here",
        )],
        &[
            "value class V { public Int x; public V { if (x < 0) { throw new IllegalArgumentException(\"negative\"); } } }",
        ],
    ),
    ran(
        "09",
        "Its body runs after the fields are bound",
        RUN,
        "compact form: range 2..7",
    ),
    ran(
        "09",
        "The body may use `this` freely and may raise an exception",
        RUN,
        "compact form raises: low is above high",
    ),
    rule(
        "09",
        "Without a compact form, the constructor has the audience of the class and no body",
        &[],
        &[
            "==== a/V.cleat\npackage a; public value class V { public Int x; }\n==== b/B.cleat\npackage b; class B { void f() { var v = new a.V(1); } }",
        ],
    ),
    ran(
        "09",
        "Two evaluations of `new` on a value class with equal arguments produce equal values",
        RUN,
        "equal arguments: true true false",
    ),
    ran(
        "09",
        "A value class with no fields has one value",
        RUN,
        "no fields: true",
    ),
    ran(
        "09",
        "`new Unit()` always produces the same one",
        RUN,
        "no fields: true true",
    ),
    // ---- 9.7 ----
    ran(
        "09",
        "The static state of a class is initialized once, the first time the program uses the class",
        RUN,
        "first uses: ByNew ByCall ByField ByAssign",
    ),
    ran("09", "`new` on the class;", RUN, "first uses: ByNew"),
    ran(
        "09",
        "a call of one of its static methods;",
        RUN,
        "first uses: ByNew ByCall",
    ),
    ran(
        "09",
        "a read or an assignment of one of its static fields, other than a read of a constant field",
        RUN,
        "before any use: []",
    ),
    ran(
        "09",
        "Initialization runs the static field initializers and the static initializer blocks of the class, in source order",
        RUN,
        "order: Above first block second",
    ),
    ran(
        "09",
        "An enum creates its constants first",
        "Sentences02",
        "enum statics: 2",
    ),
    ran(
        "09",
        "The superclass is initialized before the class",
        RUN,
        "order: Above first",
    ),
    ran(
        "09",
        "An interface is initialized by its own first use",
        "Rules",
        "Marked is initialized",
    ),
    ran(
        "09",
        "one of them runs the initialization and the other waits until it is finished",
        RUN,
        "two threads: 1 7 7",
    ),
    ran(
        "09",
        "the thread that is initializing it may use the class",
        RUN,
        "order: Above first block second own-method",
    ),
    ran(
        "09",
        "A read of a static field that has not yet been assigned raises `ClassInitializationException`",
        RUN,
        "read too early: raises",
    ),
    ran(
        "09",
        "An exception raised during initialization propagates to the use that started it, and the class is then failed",
        RUN,
        "failed initialization: the initializer failed",
    ),
    ran(
        "09",
        "Every later use of a failed class raises `ClassInitializationException`, with the original exception as its cause",
        RUN,
        "a failed class: the initializer failed",
    ),
    // ---- 9.8 ----
    ran(
        "09",
        "An object exists from its creation for as long as the program can reach it",
        RUN,
        "storage: kept 7 through 3000 objects that were dropped",
    ),
    stated(
        "09",
        "The implementation reclaims the storage of objects the program can no longer reach",
        "The next sentence says a program cannot observe it. The run tests collect at every allocation, and a program that drops objects goes on.",
    ),
    ran(
        "09",
        "Reclaiming is not observable",
        RUN,
        "storage: kept 7 through 3000 objects that were dropped, and no finalizer ran []",
    ),
    rule(
        "09",
        "There is no finalizer, no weak reference, and no way to learn where an object is stored",
        &[
            (
                "class A { @Override public void finalize() { } }",
                "`finalize` carries `@Override` and overrides nothing",
            ),
            (
                "class A { void f(Object o) { var w = new WeakReference<Object>(o); } }",
                "there is no type named `WeakReference` here",
            ),
            (
                "class A { void f(Object o) { var w = o.address(); } }",
                "`Object` has no method named `address`",
            ),
        ],
        &[],
    ),
    ran(
        "09",
        "`identical` is not affected by anything the implementation does with storage",
        "Rules",
        "identical: true true false false true",
    ),
    // ---- 9.9 ----
    held(
        "09",
        "A program is a set of compilation units that are compiled together, ahead of time, to native code",
        "behavior",
        "a_method_a_class_cannot_name_is_not_overridden",
    ),
    rule(
        "09",
        "The compiler sees every unit of the program",
        &[(
            "==== a/A.cleat\npackage a; public class A { public static Int f() { return b.B.g(); } }",
            "`b` is not a variable, a field or a type in scope",
        )],
        &[
            "==== a/A.cleat\npackage a; public class A { public static Int f() { return b.B.g(); } }\n==== b/B.cleat\npackage b; public class B { public static Int g() { return a.A.f(); } }",
        ],
    ),
    stated(
        "09",
        "There is no stable binary form",
        "It says what the language leaves out, and no program can show a form that is not there.",
    ),
    stated(
        "09",
        "Nothing compiled in one compilation is promised to work with anything compiled in another",
        "It withholds a promise.",
    ),
    held(
        "09",
        "Calls to C follow the rules of",
        "host",
        "foreign_structs",
    ),
    stated(
        "09",
        "An implementation may compile a program in any way that preserves the behavior this specification defines",
        PERMISSION,
    ),
    // ---- 9.10 ----
    held(
        "09",
        "The implementation is told one entry class",
        "host",
        "main_may_take_no_arguments_and_must_be_one",
    ),
    held(
        "09",
        "That class declares exactly one of these methods",
        "spec_rules",
        "an_entry_class_declares_exactly_one_main",
    ),
    held(
        "09",
        "`args` holds the arguments the host passed, in order, without the program's own name",
        "host",
        "main_is_given_the_arguments_the_host_passed",
    ),
    held(
        "09",
        "With no arguments it has length zero",
        "host",
        "main_is_given_the_arguments_the_host_passed",
    ),
    stated(
        "09",
        "A program ends in one of four ways",
        "It introduces the table after it, whose rows have their own cases.",
    ),
    ran("09", "| `main` returns | `0` |", "Rules", "main returns"),
    held(
        "09",
        "| An exception leaves `main` | `1`.",
        "host",
        "an_uncaught_exception_ends_the_program_with_status_one",
    ),
    held(
        "09",
        "The exception's `toString` is written to the host's error stream |",
        "host",
        "an_uncaught_exception_ends_the_program_with_status_one",
    ),
    held(
        "09",
        "| `Process.exit(status)` is called | `status` |",
        "host",
        "exit_ends_the_program_at_once",
    ),
    held(
        "09",
        "A message is written to the host's error stream |",
        "host",
        "a_full_stack_ends_the_program",
    ),
    rule(
        "09",
        "`Process.exit` is declared `public static void exit(Int status)`",
        &[
            (
                "class A { void f() { Process.exit(\"three\"); } }",
                "`String` is not assignable to `Int`",
            ),
            (
                "class A { void f() { Process.exit(); } }",
                "method `exit` takes 1 argument, and 0 are written",
            ),
        ],
        &["class A { void f(Int status) { Process.exit(status); } }"],
    ),
    held(
        "09",
        "It ends the program at once",
        "host",
        "exit_ends_the_program_at_once",
    ),
    ran(
        "09",
        "A status outside `0` through `255` raises `IllegalArgumentException`",
        RUN,
        "exit status: 256 raises",
    ),
];
