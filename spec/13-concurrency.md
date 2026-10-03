# 13. Concurrency

Field accesses, array-element accesses, lock operations, atomic-cell operations, and scope publication execute in one sequentially consistent order. That order respects program order inside each thread. A read sees the latest write to that location in the order, or the location's initial value if no write precedes the read.

An initial value is `null` for a `@Nullable` reference, `zero()` for a numeric field, the scalar 0 for `Char`, and `false` for `Boolean`. A non-null reference field has no initial value. It is assigned before the object becomes visible, by the definite-assignment rule. Sequential consistency then makes that assignment precede a read by a thread that observed the reference.

Every field access and every array-element access is atomic. A read sees a value that was written, of the location's type. It does not see a torn representation, and it does not see a reference the collector does not know.

There is no per-object monitor and no `synchronized` method. Locking a value is rejected because a value has no identity to lock. Mutual exclusion uses `Lock`.

```java
public final class Thread {
    public static Thread start(Block<Unit> body)
    public static Thread current()
    public void join()
    public Boolean isAlive()
    public @Nullable Throwable uncaught()
}
public final class Lock {
    public Lock()
    public void lock()
    public void unlock()
    public void exclusive(inline Block<Unit> body)
    public void close()
}
```

`Thread.current` returns the thread that is running. `Thread.start` creates a thread and arranges for `body.invoke()` to run on it. The body is a non-inline block. `return` inside it leaves the body. Captured locals are the effectively final locals of [chapter 4](04-methods.md), read when `start` is called. `start` returns once the thread exists. The body may interleave with the caller after that return. The caller of `start` is not joined automatically. A thread started this way is not a child of a `Scope`.

`join` waits until the thread's body has completed. `join` on the thread that is running is rejected and raises `IllegalStateException`. After completion, `isAlive` is `false`. An exception that leaves the body is stored and returned by `uncaught`. It does not complete any other thread. A normal completion stores no exception, and `uncaught` returns `null`.

`Lock` is a reference class. `lock` acquires it for the running thread. The same thread may acquire it again. Each `lock` is paired with an `unlock` on that thread. `unlock` by a thread that does not hold it raises `IllegalStateException`. `exclusive` acquires the lock, runs the block, and releases that acquisition if the block completes normally or abruptly, using the `ensuring` rule. `close` releases one acquisition held by the running thread and raises `IllegalStateException` if it holds none. `using` on a `Lock` is legal because `close` has that shape.

A field update `x = x.plus(1)` is a read and a later write. It is not one atomic update. A cell that must change in one step is an atomic cell.

```java
public final class AtomicInt32 {
    public AtomicInt32(Int32 initial)
    public Int32 get()
    public void set(Int32 value)
    public Int32 compareAndSet(Int32 expected, Int32 update)
    public Int32 fetchAndPlus(Int32 delta)
}
public final class AtomicInt64 {
    public AtomicInt64(Int64 initial)
    public Int64 get()
    public void set(Int64 value)
    public Int64 compareAndSet(Int64 expected, Int64 update)
    public Int64 fetchAndPlus(Int64 delta)
}
public final class Atomic<T> {
    public Atomic(T initial)
    public T get()
    public void set(T value)
    public T compareAndSet(T expected, T update)
}
```

`AtomicInt32` and `AtomicInt64` are reference classes. Each cell holds one machine word. `get` and `set` read and write that word. `compareAndSet` stores `update` only when the word is `expected`, and returns the word that was present. `fetchAndPlus` adds with `wrappingPlus` and returns the word that was present. It does not raise on overflow. `Int32.plus` remains the throwing operation. These methods lower to one hardware load, store, compare-and-swap, or fetch-and-add when the platform has that instruction. A lowering is correct when the observable result matches the method.

`Atomic<T>` holds one reference. `T` is a reference type. An instantiation at a value type, at `Int32`, or at `Int64` is rejected. Those words have their own classes. `compareAndSet` uses `identical`, not `equals`. A null initial value is legal only when `T` is `@Nullable`. A null argument to a non-null cell is rejected.

## 13.1 Structured concurrency

A scope joins the tasks forked inside it. A task does not outlive the call that created the scope.

```java
public final class Scope {
    public static <T> T call(inline Block<Scope, T> body)
    public <T> Task<T> fork(Block<T> body)
    public Boolean isCancelled()
    public void cancel()
}
public final class Task<T> {
    public T result()
    public Boolean isDone()
}
```

`call` evaluates the block on the running thread and passes the scope. The block is inline, so `return`, `break`, and `continue` target the caller, as [chapter 4](04-methods.md) defines. `fork` evaluates its block on a new thread. The block is non-inline. Captured locals are read when `fork` is called. `fork` returns a `Task` once the thread exists, and the body may then interleave. `fork` from a thread other than the thread that entered `call` raises `IllegalStateException`. `fork` after the scope has started closing raises `IllegalStateException`. An empty scope forks nothing.

`fork` happens-before the child body. The child's completion happens-before `result` returns and before `call` finishes joining. `result` waits for that task. `result` on the task's own thread raises `IllegalStateException`. `isDone` is `true` after the body has completed.

When the block ends for any reason, `call` cancels every task that is still running and joins every task. A task whose body raises any exception other than `CancellationException` is a failure. After the joins, an exception from the block other than `CancellationException` propagates, and task failures are suppressed on it in fork order. Otherwise the first task failure propagates and later task failures are suppressed on it. Suppression is `Throwable.suppressed()`. `CancellationException` alone does not fail the call. If nothing failed, the block's normal result, `return`, `break`, or `continue` completes as the block did. The joins run before that completion, by the `ensuring` rule.

`cancel` marks the scope cancelled. A running task raises `CancellationException` at its next safepoint. A task inside a foreign call finishes that call first. A pin is not closed by cancellation. `isCancelled` reports the mark. The mark happens-before the `CancellationException` the task observes. A failure recorded by `call` cancels the scope.

`Thread.start` remains the unstructured start. `Scope` is how a set of tasks has one join and one failure.

A data race is still a race on the program's invariants. It is not a torn read, and it is not undefined behavior of the abstract machine. Two threads that share a location without a `Lock`, an atomic cell, or the publishing thread's program order can observe an older write. They cannot observe a value that was never written.

Sequential consistency is the requirement. An implementation may use a weaker schedule only when the observable reads match some sequentially consistent order.
