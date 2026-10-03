# 9. Execution

## 9.1 Order

Operands are evaluated left to right. A receiver is evaluated before its arguments. Arguments are evaluated left to right. A short-circuit spelling evaluates only the branches its defining method runs, which [chapter 4](04-methods.md) fixes for `then`, `andAlso`, `orElse`, and `whileTrue`.

An exception unwinds to the nearest `catching` whose class accepts it, then runs `ensuring` blocks as this chapter describes. Unwind is a runtime operation. It is not a value the program returns.

## 9.2 Exceptions

Every exception is an instance of `Throwable`, a reference class in the prelude. `raise` is `Throwable`'s method and is inherited. There are no checked exceptions. A method does not declare the exceptions it raises. `throws` is not a keyword.

A `catching` clause whose class is `E` accepts an exception whose runtime class is `E` or a subclass of `E`. The test is `isInstance`.

The prelude declares these subclasses of `Throwable`. The chapters that raise them name them. Further exception classes may be declared by user code.

| Class | Raised when |
| --- | --- |
| `ArithmeticException` | A numeric operation has no result in its type, or a length is negative |
| `ClassCastException` | `as` or a cast fails |
| `IndexOutOfBoundsException` | An index or a string range is outside its bounds |
| `ClassInitializationException` | A later use of a class whose initialization failed |
| `IllegalArgumentException` | `Enum.valueOf` is given an unknown name |
| `IllegalStateException` | A lock or a thread is used outside the state [chapter 13](13-concurrency.md) allows |
| `AssertionException` | An assertion condition is `false` |
| `OutOfMemoryException` | An allocation cannot be satisfied after collection, as [section 9.6](#96-compilation) defines |
| `ClassLoadException` | `Loader.load` rejects an image, as [section 9.6](#96-compilation) defines |
| `CancellationException` | A cancelled `Scope` task reaches a safepoint, as [chapter 13](13-concurrency.md) defines |

```java
public inline <E extends Throwable, T> T catching(
    Class<E> type,
    inline Block<E, T> handler)
public inline <T> T ensuring(inline Block<Unit> cleanup)
```

Both methods are inline, and both are available on `Block<T>` produced by a lambda of the protected region. The source spelling is a `try` statement:

```
try block {catch-clause} [finally-clause]
catch-clause   = "catch" "(" type identifier ")" block
finally-clause = "finally" block
```

The source form is the send of `catching` on a block of the `try` body, then `ensuring` on a block of that whole `catching`. Several `catch` clauses are nested `catching` sends, outer clause first. A clause that is unreachable because an earlier clause's class is a superclass of it is rejected. A statement `try` may discard a `Unit` result. A `try` expression must not discard a non-`Unit` result.

`catching` evaluates its receiver block. On a normal result or a propagated `return`, `break`, or `continue`, it completes the same way. On a `raise` whose class it accepts, it evaluates the handler. Any other exception propagates.

`ensuring` evaluates its receiver block. On a normal result, and on a propagated `return`, `break`, `continue`, or `raise`, it evaluates the cleanup and then completes the same way. The cleanup block must not `return`, `break`, or `continue`. If cleanup raises, the new exception propagates and the original exception, when there was one, is suppressed on it. Suppression is observable through `Throwable.suppressed()`, which returns an array of the suppressed exceptions in the order they were suppressed. A user `inline` method does not acquire this handling by being inline. `ensuring` is the method that defines it.

A type with a `close()` method of no parameters, returning `Unit` or declared `void`, may be used in `using`.

```
using (type name = expr) block
```

The initializer runs, then the block, then `close()` through `ensuring`. Several bindings close in reverse order of completion of their initializers. A binding whose initializer raises does not close, and bindings that completed close in reverse order. `close` raising follows the `ensuring` rule.

## 9.3 Reference construction

A constructor is declared with the class name and no result type. Each constructor is an overload of `new` on the class object. Constructors are not inherited. The audience of `new` is the audience written on the constructor, defaulting to `private`.

The first statement of a constructor may be `this(...)` or `super(...)`. If neither is written, `super()` is inserted and the superclass must have an applicable zero-argument `new`. `Object` has that constructor. A constructor that both delegates with `this` and contains `super` is rejected.

Field initializers and instance initializer blocks of the class run, in source order, immediately after the superclass constructor returns, and before the rest of the constructor body that called `super`. A constructor that delegates with `this` does not run them; the constructor that calls `super` does.

`@Nullable` instance fields start as `null`. Every other instance field is definitely assigned on every path that completes the constructor, by an initializer or by an assignment in the body. A final field is not assigned twice.

Before that definite assignment completes, the constructor and the initializers must not let the instance escape. In that region the following are rejected: passing `this` as an argument, assigning `this` to a location that is not a field of `this` being initialized, returning `this`, and calling an `open` or `abstract` instance method. A final instance method may be called only when every field it reads is already assigned. A static method may be called.

A value must not be read from a field that is not yet assigned. The superclass constructor runs before the subclass fields are assigned and can see only the superclass part.

## 9.4 Value construction

A user value class has one `new`. Its parameter list is the class's own fields in source order and with their types, after any `super(...)` arguments. `super(...)`, when the superclass is a value class other than `Object`, is the first act of the constructor and binds the superclass components. The class's own parameters then bind its fields. The body runs after every component is bound. The body may raise. The body must not assign a component.

`new Unit()` returns the single `Unit` instance. The `Null` instance is the literal `null`.

Two evaluations of `new` on a value class with equal arguments are equal under `equals`. They are not `identical`, because `identical` is rejected on a value static type.

## 9.5 Class-object initialization

A class object's static field initializers and static initializer blocks run once, in source order, on the first of: a call to `new`, a call of a static method, or a read or write of a static field. A class literal does not initialize the class. Initializing a class initializes its superclass's class object first. It does not initialize superinterfaces. Initializing an interface initializes its superinterfaces first.

A cycle, or any exception raised during initialization, fails the class. The exception propagates to the use that started initialization. The class is then marked failed and is not retried. A later use raises `ClassInitializationException`, and the original exception is suppressed on it. A failed initialization does not return a null slot.

## 9.6 Compilation

The initial program is linked ahead of time. The compiler sees every compilation unit in that link and lowers it to native code through LLVM. There is no binary compatibility contract and no stable ABI. A loaded class is typechecked again when it is loaded. An already loaded class does not change its field layout when a subclass is loaded. Existing instances are not rewritten.

```java
public final class Loader {
    public static Class<?> load(UInt8[] image)
    public static @Nullable Class<?> find(String name)
}
```

`load` evaluates `image` once. The array is non-null. An empty image raises `ClassLoadException`. The image is one or more compilation units in an implementation-chosen encoding. The bytes are not a language value a program may interpret. `load` typechecks those units with the same rules as the compiler, against the classes already loaded. A unit that names a type which is neither already loaded nor in the same image is rejected. The image is installed entirely or not at all.

A loaded class may extend an `open` class. It may not extend a `final` class. It may extend a `sealed` class only when that class's `permits` clause names it, so a sealed class cannot gain a subclass that was not named when the sealed class was loaded. It may implement an interface that is already loaded. It may not override a `final` method. It may not replace a class whose qualified name is already loaded. A second image with the same name and the same definition returns the class already loaded and installs nothing new. A second image with the same name and a different definition raises `ClassLoadException`. A rejected image installs no type and does not run a static initializer. `load` does not initialize the new class. Initialization follows [section 9.5](#95-class-object-initialization) on the first active use. `find` returns the loaded class with that qualified name, or `null` when none is loaded. `find` does not initialize.

`load` publishes the new class only after every speculative compilation that the new class invalidates has been discarded. That publication happens-before any thread can invoke the new class or observe it from `load`'s result or from `find`. Two threads that load the same definition receive the same `Class`. Two threads that load conflicting definitions leave one definition installed. The other call raises `ClassLoadException`.

The JIT compiles a method to native code when it runs, including a method of a class that `load` just installed. It may specialize a method on the receiver classes and branch outcomes it has observed, and it may inline a call. Inlining a `final` method is not speculation: a later load cannot override it. Inlining an `open` call is speculation. The speculative code is discarded before a newly loaded class that could be a receiver of that call is published, and before a not-yet-seen receiver class of that call runs. Deoptimization preserves locals, the pins of [section 4.14](04-methods.md#414-checked-foreign-memory), and the locks of [chapter 13](13-concurrency.md). A program must not rely on which methods were specialized. It may rely on a loaded class behaving by the rules above, and on an open call never running in code that assumed a receiver set the load has already extended.

A final method may be lowered to the machine operation it denotes, including `Int32.plus` to an add and a fixed-length `Vector<Int32>` to a machine vector operation. The lowering is correct when the observable result matches a send of that method. A private method that no reachable call names, and that no retained annotation or reachable `has` requires, may be omitted from the native code. A method a later load could name is not omitted. That includes every `public` method, every `protected` method, and every `package` method.

The runtime is a library linked with the program. It provides precise moving collection of reference objects, exception unwind, and the class metadata `getClass`, `isInstance`, and `has` require. A value has no header for the collector to see. A boxed value is a reference and is collected. The collector must not relocate an unboxed value, a pinned object, or foreign memory. It may relocate any other reference object. Heap digits that belong only to a value move only by updating that value's representation. No program-visible address of those digits exists. After a move, every reference slot denotes the same object. `identical` is unchanged. A move is not a user write of a field.

A collection runs at a safepoint between Cleat operations. It does not suspend a thread inside a foreign call in order to move an object that call has pinned. Other threads may collect during that call. The pin is a root recorded before the call.

The heap grows when a collection does not free enough memory for an allocation. It may shrink when the live set is a fraction of the committed heap. The fraction and the growth step are implementation choices. A program may observe that an allocation succeeds when a collection freed space for it, and that an allocation which still cannot be satisfied raises `OutOfMemoryException`. It must not rely on a heap size, a growth curve, or how often collection runs. Every precise moving collector that honors pins, does not relocate unboxed values or foreign memory, and raises `OutOfMemoryException` only in that case is correct. The collector may organize the heap in generations. That organization is correct when these observations hold.

A reference header's bits are chosen by the implementation. The header must let the collector find every reference field of the instance. Cleat code does not read the header. There is no stable ABI between separately compiled Cleat binaries. Foreign calls use [section 4.11](04-methods.md#411-foreign-signatures). Definite assignment and reachability are [chapter 12](12-flow.md).

## 9.7 Startup

The implementation is given one entry type, by a compiler argument. That type declares exactly one of these, all public and static:

```
void main()
void main(String[] args)
Unit main()
Unit main(String[] args)
```

If both a zero-argument `main` and a one-argument `main` are present, the program is rejected. A missing `main` on the entry type is rejected. `args` is a non-null array of non-null strings, in the host's argument order, not including the program name. Zero arguments is a zero-length array.

`main` returning is normal termination and reports status `0`. An exception leaving `main` is abnormal termination, reports status `1`, and the exception's `toString` is written to the host error stream. No other status is defined.
