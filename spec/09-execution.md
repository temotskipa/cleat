# 9. Execution

## 9.1 Order of evaluation

Operands are evaluated from left to right. A receiver is evaluated before its arguments, and the arguments from left to right. Every argument is evaluated before the method runs. `&&`, `||` and `?:` evaluate only the operands that [chapter 4](04-methods.md) says they do.

If evaluating part of an expression raises an exception, the rest of the expression is not evaluated.

## 9.2 Exceptions

An exception is an instance of `Throwable` or of a subclass of it.

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

`message()` is the message given at construction, or the empty string. `cause()` is the exception given at construction as the reason for this one, or `null`. `suppressed()` returns a new array of the exceptions that [section 9.3](#93-try) and [section 9.4](#94-using) recorded on this one, in the order they were recorded. The default `toString` of an exception is the name of its class and, when the message is not empty, `: ` and the message.

`throw e` raises an exception, as chapter 4 describes. A raised exception ends the statement that raised it, and then each enclosing statement in turn, until a `try` statement handles it. When no statement of the method handles it, the call of the method raises it in the caller. An exception that no caller handles ends the thread, as [chapter 13](13-concurrency.md) describes, or ends the program, as [section 9.10](#910-startup-and-termination) describes.

There are no checked exceptions. A method does not declare what it raises, and `throws` is not a keyword.

The prelude declares these subclasses of `Throwable`. Each is an open class, and a program may declare more.

| Class | Raised when |
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

An implementation may record where an exception was created, and may include that in the report of section 9.10. A program cannot read it.

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

A `try` statement has a block, any number of `catch` clauses, and an optional `finally` block. It has at least one `catch` clause or a `finally` block.

The `try` block runs first. If it raises an exception, the `catch` clauses are examined in source order. The first clause whose type the exception is an instance of handles it: its parameter is bound to the exception and its block runs. If no clause accepts the exception, it goes on propagating.

- The type of a clause is `Throwable` or a subclass of it. The parameter is a local of the clause's block.
- An exception raised inside a `catch` block is not handled by the other clauses of the same statement.
- A clause that can never run, because an earlier clause names its type or a supertype of its type, is rejected.

The `finally` block runs last, however the `try` block and the `catch` block completed: normally, by `return`, `break` or `continue`, or by an exception. The statement then completes in that same way.

`return` is rejected in a `finally` block, and so are a `break` and a `continue` that would leave it. If a `finally` block raises an exception while another is propagating, the new exception propagates and the earlier one is recorded as suppressed on it.

## 9.4 `using`

```java
using (File input = File.open(path)) {
    process(input);
}
```

`using (T name = e) block` evaluates `e`, binds `name` to its value, runs the block, and then calls `name.close()`, however the block completed. The static type of `e` must have a method `close()` with no parameters and the result `Unit`, and must not be `@Nullable`. `name` is not assignable. `var` may be written for `T`.

If the block raised an exception and `close` raises another, the exception from the block propagates and the one from `close` is recorded as suppressed on it. If the block completed in any other way and `close` raises, that exception propagates.

Several resources may be written, separated by commas. Each is evaluated and bound in order, and they are closed in the reverse order. If an initializer raises, the resources already bound are closed, and the one being initialized is not.

## 9.5 Constructing an object

A constructor of a reference class is declared with the name of the class, a parameter list and a body. It has no result. A class may declare several, and `new` chooses among them as chapter 4 chooses a method. Constructors are not inherited. A constructor's audience is written like a member's and is `private` when it is not written.

A class that declares no constructor has one, with no parameters, an empty body, and the audience of the class.

**The two calls.** `super(args)` calls a constructor of the superclass. `this(args)` calls another constructor of the same class. A constructor body contains at most one such call, written as a statement of the body itself and not inside another statement. `this(args)`, when written, is the first statement. A body that writes neither call ends with an implicit `super()`.

**Before the call.** Until `super(...)` is called, the constructor is filling in the fields of its own class. In that part of the body, `this` may be used only to assign a field declared in the class, written `this.name = e` or `name = e`, and to read such a field once it is assigned. No method is called on `this`. `this` is not passed as an argument, stored, returned, or captured by a lambda. No inherited instance member is used. The arguments of the `super(...)` or `this(...)` call are under the same restriction.

Field initializers run first, in source order, on entry to a constructor that does not call `this(...)`. An initializer is under the same restriction.

At the `super(...)` call, every field declared in the class must be assigned, under the rules of [chapter 12](12-flow.md). A `@Nullable` field is the exception: it starts as `null`.

**After the call.** When `super(...)` or `this(...)` returns, the object is complete: every field of the class, of its superclasses and of its subclasses is assigned. From there `this` may be used without restriction. A final field is not assigned there.

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

A subclass assigns its fields before it calls `super`, so the object is complete before any code can call a method on it. No constructor and no method ever sees a field that has not been assigned.

## 9.6 Constructing a value

A value class has exactly one constructor. Its parameters are the fields of the class, in declaration order, with their types. `new Vec3(1, 2, 3)` evaluates the arguments from left to right and produces the value with those fields. A field of a value class has no initializer.

A value class may write that constructor in a compact form, to give it an audience or a check:

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

The compact form has no parameter list. Its body runs after the fields are bound. The body may use `this` freely and may raise an exception. It does not assign a field. Its audience is `private` when it is not written.

Without a compact form, the constructor has the audience of the class and no body.

Two evaluations of `new` on a value class with equal arguments produce equal values. A value class with no fields has one value: `new Unit()` always produces the same one.

## 9.7 Class initialization

The static state of a class is initialized once, the first time the program uses the class in one of these ways:

- `new` on the class;
- a call of one of its static methods;
- a read or an assignment of one of its static fields, other than a read of a constant field of [chapter 12](12-flow.md).

A class literal does not initialize the class.

Initialization runs the static field initializers and the static initializer blocks of the class, in source order. An enum creates its constants first. The superclass is initialized before the class. The interfaces a class implements are not initialized with it. An interface is initialized by its own first use.

If two threads use a class for the first time, one of them runs the initialization and the other waits until it is finished.

While a class is being initialized, the thread that is initializing it may use the class. A read of a static field that has not yet been assigned raises `ClassInitializationException`. That can happen only during initialization: when an initializer calls a method that reads a later field, or when two classes use each other while both are initializing.

An exception raised during initialization propagates to the use that started it, and the class is then failed. Every later use of a failed class raises `ClassInitializationException`, with the original exception as its cause.

## 9.8 Storage

An object exists from its creation for as long as the program can reach it. The implementation reclaims the storage of objects the program can no longer reach.

Reclaiming is not observable. There is no finalizer, no weak reference, and no way to learn where an object is stored. `identical` is not affected by anything the implementation does with storage.

When storage cannot be found for an allocation, even after reclaiming, the allocation raises `OutOfMemoryException`. A program must not rely on how much storage there is, on when reclaiming happens, or on how often.

## 9.9 The program

A program is a set of compilation units that are compiled together, ahead of time, to native code. The compiler sees every unit of the program. No class is added to a program while it runs.

There is no stable binary form. Nothing compiled in one compilation is promised to work with anything compiled in another. Calls to C follow the rules of [chapter 4](04-methods.md#411-foreign-methods).

An implementation may compile a program in any way that preserves the behavior this specification defines.

## 9.10 Startup and termination

The implementation is told one entry class. That class declares exactly one of these methods:

```java
public static void main()
public static void main(String[] args)
```

A class that declares both, or neither, is rejected as an entry class. `args` holds the arguments the host passed, in order, without the program's own name. It is never `null`, and no element is `null`. With no arguments it has length zero.

A program ends in one of four ways:

| How | Status |
| --- | --- |
| `main` returns | `0` |
| An exception leaves `main` | `1`. The exception's `toString` is written to the host's error stream |
| `Process.exit(status)` is called | `status` |
| A call cannot be given stack space | `1`. A message is written to the host's error stream |

Threads that are still running do not keep the program alive.

`Process.exit` is declared `public static void exit(Int status)`. It ends the program at once: no `finally` block and no `close` runs after it. A status outside `0` through `255` raises `IllegalArgumentException`.

Running out of stack is not an exception, and a program cannot catch it.
