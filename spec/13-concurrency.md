# 13. Concurrency

## 13.1 Threads

A program starts with one thread, which runs `main`. A thread runs a body to completion. Threads share every object they can reach.

```java
public final class Thread {
    @Discardable
    public static Thread start(Function0<Unit> body)
    public static Thread current()
    public static void sleep(Int milliseconds)
    public static void checkCancelled()
    public void join()
    public Boolean isAlive()
    public @Nullable Throwable uncaught()
}
```

`Thread.start` creates a thread that runs `body.invoke()`, and returns once the thread exists. Its result may be ignored, and so may the result of `fork`. The body is a lambda, and it captures locals as [chapter 4](04-methods.md#48-lambdas-and-method-references) describes. `Thread.current` returns the thread that is running. `sleep` waits for at least that many milliseconds. A negative argument raises `IllegalArgumentException`.

`join` waits until the thread's body has completed. Joining the running thread raises `IllegalStateException`. `isAlive` is `true` from `start` until the body has completed.

An exception that leaves the body ends that thread and no other. Its `toString` is written to the host's error stream, and `uncaught` returns it from then on. `uncaught` returns `null` for a thread that is running or that completed normally.

The program ends when `main` returns, as [chapter 9](09-execution.md#910-startup-and-termination) defines. A thread that is still running does not keep it alive. A program that needs its threads to finish joins them, or starts them in a scope ([section 13.5](#135-scopes)).

```java
var worker = Thread.start(() -> {
    Console.println("working");
});
worker.join();
```

## 13.2 Shared memory

Two threads may read and assign the same field, static field or array element. The language guarantees four things about what they see.

1. **A read returns a value that was assigned.** It returns a value that some assignment stored in that location, or the location's initial value. It never returns part of one value and part of another, even for a value class with several fields, and it never returns a value that no thread stored.
2. **An object is complete before another thread can see it.** No constructor and no method ever sees a field that has not been assigned, as [chapter 9](09-execution.md#95-constructing-an-object) establishes. That holds across threads: a thread that obtains a reference to an object sees, for each field, the value the constructor gave it or a later one.
3. **Synchronization orders what it joins.** Each row of the table below orders everything the first thread did before the first action ahead of everything the second thread does after the second action. A read that is ordered after an assignment, with no other assignment to the location between or unordered with them, returns the value of that assignment.
4. **Otherwise a read may be stale.** When two threads use a location with nothing ordering them, a read returns the value of some assignment that is not ordered after it. It may be an older value than the latest one. Nothing else goes wrong: the program's types hold, and no other location is affected.

| First action | Second action |
| --- | --- |
| A thread releases a `Lock` | A thread next acquires that lock |
| An operation on an atomic cell | A later operation on the same cell |
| `Thread.start` or `Scope.fork` | The first action of the new thread's body |
| The last action of a thread's body | The return of `join`, or of `result` on its task |
| The end of a class's initialization | Any thread's use of that class |

Within one thread, actions are ordered as [chapter 9](09-execution.md#91-order-of-evaluation) evaluates them.

A field update such as `count = count + 1` is a read and a later assignment. Two threads that run it with nothing ordering them may lose an update. A value that must change in one step is held in an atomic cell, or is guarded by a lock.

There is no `volatile` modifier and no per-object monitor.

## 13.3 Locks

```java
public final class Lock {
    public Lock()
    public void lock()
    public void unlock()
    public <T> T exclusive(Function0<T> body)
    public Condition newCondition()
}

public final class Condition {
    public void await()
    public void signal()
    public void signalAll()
}
```

A `Lock` is held by at most one thread at a time. `lock` waits until the running thread can hold it. A thread that holds it may call `lock` again, and it then releases the lock by calling `unlock` as many times. `unlock` by a thread that does not hold the lock raises `IllegalStateException`.

`exclusive` calls `lock`, runs the body, and calls `unlock` however the body completed. It returns the body's result.

```java
class Longest {
    final Lock lock = new Lock();
    String word = "";

    public void offer(String candidate) {
        lock.exclusive(() -> {
            if (candidate.length() > word.length()) {
                word = candidate;
            }
        });
    }
}
```

**Waiting for a condition.** A `Condition` belongs to the lock whose `newCondition` created it. Each of its three methods raises `IllegalStateException` unless the running thread holds that lock.

- `await` releases the lock completely, waits, and holds the lock again as before when it returns. It may return without a signal, so a caller tests what it waits for in a loop.
- `signal` lets one waiting thread return from `await`, if there is one. `signalAll` lets every waiting thread return.

```java
class Mailbox<T extends Object> {
    final Lock lock = new Lock();
    final Condition filled = lock.newCondition();
    @Nullable T item;

    public void put(T value) {
        lock.exclusive(() -> {
            item = value;
            filled.signalAll();
        });
    }

    public T take() {
        return lock.exclusive(() -> {
            while (item == null) {
                filled.await();
            }
            T taken = (T) item;
            item = null;
            return taken;
        });
    }
}
```

## 13.4 Atomic cells

```java
public final class AtomicInt {
    public AtomicInt(Int initial)
    public Int get()
    public void set(Int value)
    public Boolean compareAndSet(Int expected, Int update)
    @Discardable
    public Int addAndGet(Int delta)
    @Discardable
    public Int incrementAndGet()
}

public final class Atomic<T> {
    public Atomic(T initial)
    public T get()
    public void set(T value)
    public Boolean compareAndSet(T expected, T update)
}
```

An atomic cell holds one value. Each method is one step: no other operation on the cell happens between its read and its assignment.

`compareAndSet` stores `update` when the cell holds `expected`, and returns whether it did. `AtomicInt` compares with `equals`. `Atomic<T>` compares with `identical`, so a cell of a reference class compares references.

`addAndGet` adds `delta` to the cell and returns the sum. `incrementAndGet` is `addAndGet(1)`. When the sum is not an `Int`, the method raises `ArithmeticException` and the cell is unchanged.

## 13.5 Scopes

A scope starts tasks and waits for all of them. No task outlives the call that created its scope.

```java
public final class Scope {
    public static <T> T call(Function1<Scope, T> body)
    @Discardable
    public <T> Task<T> fork(Function0<T> body)
    public void cancel()
    public Boolean isCancelled()
}

public final class Task<out T> {
    public T result()
    public Boolean isDone()
}
```

`Scope.call` creates a scope, runs `body` on the running thread with that scope, and returns the body's result. Before it returns, it cancels the scope if the body raised an exception, and it waits for every task of the scope to complete.

`fork` starts a task: a new thread that runs `body.invoke()`. It returns once the thread exists. Only the thread that is running the scope's `call` may fork, and only while the body of that `call` is running. Any other `fork` raises `IllegalStateException`.

`result` waits until the task has completed, and returns what its body returned. If the body raised an exception, `result` raises that exception. `isDone` is `true` once the body has completed in either way.

```java
Int total = Scope.call((scope) -> {
    Task<Int> left = scope.fork(() -> count(firstHalf));
    Task<Int> right = scope.fork(() -> count(secondHalf));
    return left.result() + right.result();
});
```

**Failure.** A task fails when its body raises an exception other than `CancellationException`. A task that fails cancels its scope. When `call` has waited for every task, it completes in the first of these ways that applies:

1. If the body of `call` raised an exception, that exception propagates. The exceptions of failed tasks are recorded as suppressed on it, in the order the tasks were forked, except one that is the same exception.
2. If a task failed, the exception of the first failed task in fork order propagates, with those of the others suppressed on it.
3. Otherwise `call` returns the body's result.

A `CancellationException` from a task is never reported by `call`.

## 13.6 Cancellation

Cancellation is a request. A cancelled task goes on running until it reaches a point that checks for the request.

`cancel` marks the scope as cancelled, and `isCancelled` reports the mark. A scope is also cancelled by a task that fails and by an exception from the body of its `call`. A scope that is cancelled stays cancelled.

A task is cancelled when its scope is. A thread started by `Thread.start` is never cancelled, and neither is the thread that runs `main`.

A cancelled task raises `CancellationException` at these points, and at no others:

- a call of `Thread.checkCancelled`;
- a call of `Thread.sleep`, `Thread.join`, `Task.result` or `Condition.await`, when it is called and while it waits.

`Lock.lock` is not such a point, so code that cleans up may take a lock. A task inside a foreign call finishes the call. No exception arrives between two statements that do not ask for it, so a task's own data is never left half updated by a cancellation.

A long computation in a task calls `Thread.checkCancelled` now and then. On a thread that is not a cancelled task, it does nothing.

```java
Scope.call((scope) -> {
    for (String path : paths) {
        scope.fork(() -> {
            for (String line : File.readLines(path)) {
                Thread.checkCancelled();        // stops here once another task has failed
                index(line);
            }
        });
    }
});
```
