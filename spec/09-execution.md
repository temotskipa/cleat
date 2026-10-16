# 9. Execution

## 9.1 Order of evaluation

Operands are evaluated from left to right. <!-- r[execution.order.operands-evaluated-left] -->
A receiver is evaluated before its arguments, and the arguments from left to right. <!-- r[execution.order.receiver-evaluated-before] -->
Every argument is evaluated before the method runs. <!-- r[execution.order.argument-evaluated-before] -->
`&&`, `||` and `?:` evaluate only the operands that [chapter 4](04-methods.md) says they do. <!-- r[execution.order.and-or-conditional] -->

If evaluating part of an expression raises an exception, the rest of the expression is not evaluated. <!-- r[execution.order.evaluating-part-expression] -->

## 9.2 Exceptions

An exception is an instance of `Throwable` or of a subclass of it. <!-- r[execution.exceptions.exception-instance-throwable] -->

```java
public open class Throwable {
    public Throwable()
    public Throwable(String message)
    public Throwable(String message, Throwable cause)
    public final String message()
    public final @Nullable Throwable cause()
    public final Throwable[] suppressed()
}
```

`message()` is the message given at construction, or the empty string. <!-- r[execution.exceptions.message-given-construction] -->
`cause()` is the exception given at construction as the reason for this one, or `null`. <!-- r[execution.exceptions.cause-exception-given] -->
`suppressed()` returns a new array of the exceptions that [section 9.3](#93-try) and [section 9.4](#94-using) recorded on this one, in the order they were recorded. <!-- r[execution.exceptions.suppressed-returns-new] -->
The default `toString` of an exception is the name of its class and, when the message is not empty, `: ` and the message. <!-- r[execution.exceptions.default-tostring-exception] -->

`throw e` raises an exception, as chapter 4 describes. <!-- r[execution.exceptions.throw-raises-exception] -->
A raised exception ends the statement that raised it, and then each enclosing statement in turn, until a `try` statement handles it. <!-- r[execution.exceptions.raised-exception-ends] -->
When no statement of the method handles it, the call of the method raises it in the caller. <!-- r[execution.exceptions.no-statement-method] -->
An exception that no caller handles ends the thread, as [chapter 13](13-concurrency.md) describes, or ends the program, as [section 9.10](#910-startup-and-termination) describes. <!-- r[execution.exceptions.exception-no-caller] -->

There are no checked exceptions. <!-- r[execution.exceptions.no-checked-exceptions] -->
A method does not declare what it raises, and `throws` is not a keyword. <!-- r[execution.exceptions.method-not-declare] -->

The prelude declares these subclasses of `Throwable`. <!-- r[execution.exceptions.prelude-declares-subclasses] -->
Each is an open class, and a program may declare more. <!-- r[execution.exceptions.open-class-program] -->

| Class | Raised when <!-- r[execution.exceptions.table] --> |
| --- | --- |
| `ArithmeticException` | A numeric operation has no result in its class, or an array length is negative |
| `ClassCastException` | A cast fails |
| `IndexOutOfBoundsException` | An index or a range is outside its bounds |
| `IllegalArgumentException` | A method cannot accept an argument it was given |
| `IllegalStateException` | An object is used in a state that does not allow the operation |
| `IllegalAccessException` | A mirror of [chapter 8](08-annotations.md#89-reading-annotations) is used on a member that is not public, or to assign a final field |
| `AssertionException` | An assertion is `false` |
| `ClassInitializationException` | A static field is read before it is assigned, or a class whose initialization failed is used |
| `OutOfMemoryException` | Storage cannot be found for an allocation |
| `CancellationException` | A task is cancelled, as chapter 13 defines |

An implementation may record where an exception was created, and may include that in the report of section 9.10. <!-- r[execution.exceptions.implementation-record-exception] -->
A program cannot read it. <!-- r[execution.exceptions.program-read] -->

## 9.3 `try`

```java
try {
    Expr expr = new Parser(text).parse();
    Console.println(text + " = " + eval(expr));
} catch (ParseException e) {
    Console.error(text + ": " + e.message());
} catch (ArithmeticException e) {
    Console.error(text + ": division by zero");
} finally {
    attempts++;
}
```

A `try` statement has a block, any number of `catch` clauses, and an optional `finally` block. <!-- r[execution.try.try-statement-block] -->
It has at least one `catch` clause or a `finally` block. <!-- r[execution.try.least-catch-clause] -->

The `try` block runs first. <!-- r[execution.try.try-block-runs] -->
If it raises an exception, the `catch` clauses are examined in source order. <!-- r[execution.try.raises-exception-catch] -->
The first clause whose type the exception is an instance of handles it: its parameter is bound to the exception and its block runs. <!-- r[execution.try.first-clause-type] -->
If no clause accepts the exception, it goes on propagating. <!-- r[execution.try.no-clause-accepts] -->

- The type of a clause is `Throwable` or a subclass of it. <!-- r[execution.try.type-clause-throwable] -->
  The parameter is a local of the clause's block. <!-- r[execution.try.parameter-local-clause] -->
- An exception raised inside a `catch` block is not handled by the other clauses of the same statement. <!-- r[execution.try.exception-raised-inside] -->
- A clause that can never run, because an earlier clause names its type or a supertype of its type, is rejected. <!-- r[execution.try.clause-never-run] -->

The `finally` block runs last, however the `try` block and the `catch` block completed: normally, by `return`, `break` or `continue`, or by an exception. <!-- r[execution.try.finally-block-runs] -->
The statement then completes in that same way. <!-- r[execution.try.statement-completes-way] -->

`return` is rejected in a `finally` block, and so are a `break` and a `continue` that would leave it. <!-- r[execution.try.return-rejected-finally] -->
If a `finally` block raises an exception while another is propagating, the new exception propagates and the earlier one is recorded as suppressed on it. <!-- r[execution.try.finally-block-raises] -->

## 9.4 `using`

```java
using (File input = File.open(path)) {
    process(input);
}
```

`using (T name = e) block` evaluates `e`, binds `name` to its value, runs the block, and then calls `name.close()`, however the block completed. <!-- r[execution.using.using-name-block] -->
The static type of `e` must have a method `close()` with no parameters and the result `Unit`, and must not be `@Nullable`. <!-- r[execution.using.static-type-method] -->
`name` is not assignable. <!-- r[execution.using.name-not-assignable] -->
`var` may be written for `T`. <!-- r[execution.using.var-written] -->

If the block raised an exception and `close` raises another, the exception from the block propagates and the one from `close` is recorded as suppressed on it. <!-- r[execution.using.block-raised-exception] -->
If the block completed in any other way and `close` raises, that exception propagates. <!-- r[execution.using.block-completed-way] -->

Several resources may be written, separated by commas. <!-- r[execution.using.several-resources-written] -->
Each is evaluated and bound in order, and they are closed in the reverse order. <!-- r[execution.using.evaluated-bound-order] -->
If an initializer raises, the resources already bound are closed, and the one being initialized is not. <!-- r[execution.using.initializer-raises-resources] -->

## 9.5 Constructing an object

A constructor of a reference class is declared with the name of the class, a parameter list and a body. <!-- r[execution.objects.constructor-reference-class] -->
It has no result. <!-- r[execution.objects.no-result] -->
A class may declare several, and `new` chooses among them as chapter 4 chooses a method. <!-- r[execution.objects.class-declare-several] -->
Constructors are not inherited. <!-- r[execution.objects.constructors-not-inherited] -->
A constructor's audience is written like a member's and is `private` when it is not written. <!-- r[execution.objects.constructor-audience-written] -->

A class that declares no constructor has one, with no parameters, an empty body, and the audience of the class. <!-- r[execution.objects.class-declares-no] -->

**The two calls.** `super(args)` calls a constructor of the superclass. <!-- r[execution.objects.calls-super-args] -->
`this(args)` calls another constructor of the same class. <!-- r[execution.objects.this-args-calls] -->
A constructor body contains at most one such call, written as a statement of the body itself and not inside another statement. <!-- r[execution.objects.constructor-body-contains] -->
`this(args)`, when written, is the first statement. <!-- r[execution.objects.this-args-written] -->
A body that writes neither call ends with an implicit `super()`. <!-- r[execution.objects.body-writes-call] -->

**Before the call.** Until `super(...)` is called, the constructor is filling in the fields of its own class. <!-- r[execution.objects.before-call-until] -->
In that part of the body, `this` may be used only to assign a field declared in the class, written `this.name = e` or `name = e`, and to read such a field once it is assigned. <!-- r[execution.objects.part-body-this] -->
No method is called on `this`. <!-- r[execution.objects.no-method-called] -->
`this` is not passed as an argument, stored, returned, or captured by a lambda. <!-- r[execution.objects.this-not-passed] -->
No inherited instance member is used. <!-- r[execution.objects.no-inherited-instance] -->
The arguments of the `super(...)` or `this(...)` call are under the same restriction. <!-- r[execution.objects.arguments-super-this] -->

Field initializers run first, in source order, on entry to a constructor that does not call `this(...)`. <!-- r[execution.objects.field-initializers-run] -->
An initializer is under the same restriction. <!-- r[execution.objects.initializer-under-restriction] -->

At the `super(...)` call, every field declared in the class must be assigned, under the rules of [chapter 12](12-flow.md). <!-- r[execution.objects.super-call-field] -->
A `@Nullable` field is the exception: it starts as `null`. <!-- r[execution.objects.nullable-field-exception] -->

**After the call.** When `super(...)` or `this(...)` returns, the object is complete: every field of the class, of its superclasses and of its subclasses is assigned. <!-- r[execution.objects.after-call-super] -->
From there `this` may be used without restriction. <!-- r[execution.objects.this-used-without] -->
A final field is not assigned there. <!-- r[execution.objects.final-field-not] -->

```java
public class Account {
    final String owner;
    Rational balance;

    public Account(String owner) {
        this.owner = owner;
        this.balance = 0;
    }                                // super() runs here

    public Account(String owner, Ledger ledger) {
        this.owner = owner;
        this.balance = 0;
        super();
        ledger.register(this);       // the object is complete
    }
}
```

A subclass assigns its fields before it calls `super`, so the object is complete before any code can call a method on it. <!-- r[execution.objects.subclass-assigns-fields] -->
No constructor and no method ever sees a field that has not been assigned. <!-- r[execution.objects.no-constructor-method] -->

## 9.6 Constructing a value

A value class has exactly one constructor. <!-- r[execution.values.value-class-exactly] -->
Its parameters are the fields of the class, in declaration order, with their types. <!-- r[execution.values.parameters-fields-class] -->
`new Vec3(1, 2, 3)` evaluates the arguments from left to right and produces the value with those fields. <!-- r[execution.values.new-vec3-evaluates] -->
A field of a value class has no initializer. <!-- r[execution.values.field-value-class] -->

A value class may write that constructor in a compact form, to give it an audience or a check: <!-- r[execution.values.value-class-write] -->

```java
public value class Range {
    public Int low;
    public Int high;

    public Range {
        if (low > high) {
            throw new IllegalArgumentException("low is above high");
        }
    }
}
```

The compact form has no parameter list. <!-- r[execution.values.compact-form-no] -->
Its body runs after the fields are bound. <!-- r[execution.values.body-runs-after] -->
The body may use `this` freely and may raise an exception. <!-- r[execution.values.body-use-this] -->
It does not assign a field. <!-- r[execution.values.not-assign-field] -->
Its audience is `private` when it is not written. <!-- r[execution.values.audience-private-not] -->

Without a compact form, the constructor has the audience of the class and no body. <!-- r[execution.values.without-compact-form] -->

Every value of a value class is produced by its constructor, whatever the constructor's audience. <!-- r[execution.values.value-class-produced] -->
Besides `new`, two things construct a value: `new T[n]` makes the default of a value class under [section 6.10](06-numbers.md#610-arrays), and a foreign call makes a value from a struct that C gives back under [section 4.11](04-methods.md#411-foreign-methods). <!-- r[execution.values.besides-new-things] -->
In both, a field that holds a value of a value class is constructed before the value that holds it. <!-- r[execution.values.field-holds-value] -->

Two evaluations of `new` on a value class with equal arguments produce equal values. <!-- r[execution.values.evaluations-new-value] -->
A value class with no fields has one value: `new Unit()` always produces the same one. <!-- r[execution.values.value-class-no] -->

## 9.7 Class initialization

The static state of a class is initialized once, the first time the program uses the class in one of these ways: <!-- r[execution.initialization.static-state-class] -->

- `new` on the class; <!-- r[execution.initialization.new-class] -->
- a call of one of its static methods; <!-- r[execution.initialization.call-static-methods] -->
- a read or an assignment of one of its static fields, other than a read of a constant field of [chapter 12](12-flow.md). <!-- r[execution.initialization.read-assignment-static] -->

A class literal does not initialize the class. <!-- r[execution.initialization.class-literal-not] -->

Initialization runs the static field initializers and the static initializer blocks of the class, in source order. <!-- r[execution.initialization.initialization-runs-static] -->
An enum creates its constants first. <!-- r[execution.initialization.enum-creates-constants] -->
The superclass is initialized before the class. <!-- r[execution.initialization.superclass-initialized-before] -->
The interfaces a class implements are not initialized with it. <!-- r[execution.initialization.interfaces-class-implements] -->
An interface is initialized by its own first use. <!-- r[execution.initialization.interface-initialized-first] -->

If two threads use a class for the first time, one of them runs the initialization and the other waits until it is finished. <!-- r[execution.initialization.threads-use-class] -->

While a class is being initialized, the thread that is initializing it may use the class. <!-- r[execution.initialization.while-class-initialized] -->
A read of a static field that has not yet been assigned raises `ClassInitializationException`. <!-- r[execution.initialization.read-static-field] -->
That can happen only during initialization: when an initializer calls a method that reads a later field, or when two classes use each other while both are initializing. <!-- r[execution.initialization.happen-only-during] -->

An exception raised during initialization propagates to the use that started it, and the class is then failed. <!-- r[execution.initialization.exception-raised-during] -->
Every later use of a failed class raises `ClassInitializationException`, with the original exception as its cause. <!-- r[execution.initialization.later-use-failed] -->

## 9.8 Storage

An object exists from its creation for as long as the program can reach it. <!-- r[execution.storage.object-exists-creation] -->
The implementation reclaims the storage of objects the program can no longer reach. <!-- r[execution.storage.implementation-reclaims-storage] -->

Reclaiming is not observable. <!-- r[execution.storage.reclaiming-not-observable] -->
There is no finalizer, no weak reference, and no way to learn where an object is stored. <!-- r[execution.storage.no-finalizer-weak] -->
`identical` is not affected by anything the implementation does with storage. <!-- r[execution.storage.identical-not-affected] -->

When storage cannot be found for an allocation, even after reclaiming, the allocation raises `OutOfMemoryException`. <!-- r[execution.storage.storage-found-allocation] -->
A program must not rely on how much storage there is, on when reclaiming happens, or on how often. <!-- r[execution.storage.program-not-rely] -->

## 9.9 The program

A program is a set of compilation units that are compiled together, ahead of time, to native code. <!-- r[execution.program.program-set-compilation] -->
The compiler sees every unit of the program. <!-- r[execution.program.compiler-sees-unit] -->
No class is added to a program while it runs. <!-- r[execution.program.no-class-added] -->

There is no stable binary form. <!-- r[execution.program.no-stable-binary] -->
Nothing compiled in one compilation is promised to work with anything compiled in another. <!-- r[execution.program.nothing-compiled-compilation] -->
Calls to C follow the rules of [chapter 4](04-methods.md#411-foreign-methods). <!-- r[execution.program.calls-follow-rules] -->

An implementation may compile a program in any way that preserves the behavior this specification defines. <!-- r[execution.program.implementation-compile-program] -->

## 9.10 Startup and termination

The implementation is told one entry class. <!-- r[execution.startup.implementation-told-entry] -->
That class declares exactly one of these methods: <!-- r[execution.startup.class-declares-exactly] -->

```java
public static void main()
public static void main(String[] args)
```

A class that declares both, or neither, is rejected as an entry class. <!-- r[execution.startup.class-declares-rejected] -->
`args` holds the arguments the host passed, in order, without the program's own name. <!-- r[execution.startup.args-holds-arguments] -->
It is never `null`, and no element is `null`. <!-- r[execution.startup.never-null-no] -->
With no arguments it has length zero. <!-- r[execution.startup.no-arguments-length] -->

A program ends in one of four ways: <!-- r[execution.startup.program-ends-four] -->

| How | Status |
| --- | --- |
| `main` returns | `0` <!-- r[execution.startup.main-returns] --> |
| An exception leaves `main` | `1`. The exception's `toString` is written to the host's error stream <!-- r[execution.startup.exception-leaves-main] --> |
| `Process.exit(status)` is called | `status` <!-- r[execution.startup.process-exit-status] --> |
| A call cannot be given stack space | `1`. A message is written to the host's error stream <!-- r[execution.startup.call-given-stack] --> |

Threads that are still running do not keep the program alive. <!-- r[execution.startup.threads-still-running] -->

`Process.exit` is declared `public static void exit(Int status)`. <!-- r[execution.startup.process-exit-declared] -->
It ends the program at once: no `finally` block and no `close` runs after it. <!-- r[execution.startup.ends-program-once] -->
A status outside `0` through `255` raises `IllegalArgumentException`. <!-- r[execution.startup.status-outside-through] -->

Running out of stack is not an exception, and a program cannot catch it. <!-- r[execution.startup.running-out-stack] -->
