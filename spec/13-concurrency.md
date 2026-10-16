# 13. Concurrency

## 13.1 Threads

A program starts with one thread, which runs `main`. <!-- r[concurrency.threads.program-starts-thread] -->
A thread runs a body to completion. <!-- r[concurrency.threads.thread-runs-body] -->
Threads share every object they can reach. <!-- r[concurrency.threads.threads-share-object] -->

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

`Thread.start` creates a thread that runs `body.invoke()`, and returns once the thread exists. <!-- r[concurrency.threads.thread-start-creates] -->
Its result may be ignored, and so may the result of `fork`. <!-- r[concurrency.threads.result-ignored-fork] -->
The body is a lambda, and it captures locals as [chapter 4](04-methods.md#48-lambdas-and-method-references) describes. <!-- r[concurrency.threads.body-lambda-captures] -->
`Thread.current` returns the thread that is running. <!-- r[concurrency.threads.thread-current-returns] -->
`sleep` waits for at least that many milliseconds. <!-- r[concurrency.threads.sleep-waits-least] -->
A negative argument raises `IllegalArgumentException`. <!-- r[concurrency.threads.negative-argument-raises] -->

`join` waits until the thread's body has completed. <!-- r[concurrency.threads.join-waits-until] -->
Joining the running thread raises `IllegalStateException`. <!-- r[concurrency.threads.joining-running-thread] -->
`isAlive` is `true` from `start` until the body has completed. <!-- r[concurrency.threads.isalive-true-start] -->

An exception that leaves the body ends that thread and no other. <!-- r[concurrency.threads.exception-leaves-body] -->
Its `toString` is written to the host's error stream, and `uncaught` returns it from then on. <!-- r[concurrency.threads.tostring-written-host] -->
`uncaught` returns `null` for a thread that is running or that completed normally. <!-- r[concurrency.threads.uncaught-returns-null] -->

The program ends when `main` returns, as [chapter 9](09-execution.md#910-startup-and-termination) defines. <!-- r[concurrency.threads.program-ends-main] -->
A thread that is still running does not keep it alive. <!-- r[concurrency.threads.thread-still-running] -->
A program that needs its threads to finish joins them, or starts them in a scope ([section 13.5](#135-scopes)). <!-- r[concurrency.threads.program-needs-threads] -->

```java
var worker = Thread.start(() -> {
    Console.println("working");
});
worker.join();
```

## 13.2 Shared memory

Two threads may read and assign the same field, static field or array element. <!-- r[concurrency.memory.threads-read-assign] -->
The language guarantees four things about what they see. <!-- r[concurrency.memory.language-guarantees-four] -->

1. **A read returns a value that was assigned.** It returns a value that some assignment stored in that location, or the location's initial value. <!-- r[concurrency.memory.read-returns-value] -->
   It never returns part of one value and part of another, even for a value class with several fields, and it never returns a value that no thread stored. <!-- r[concurrency.memory.never-returns-part] -->
2. **An object is complete before another thread can see it.** No constructor and no method ever sees a field that has not been assigned, as [chapter 9](09-execution.md#95-constructing-an-object) establishes. <!-- r[concurrency.memory.object-complete-before] -->
   That holds across threads: a thread that obtains a reference to an object sees, for each field, the value the constructor gave it or a later one. <!-- r[concurrency.memory.holds-across-threads] -->
3. **Synchronization orders what it joins.** Each row of the table below orders everything the first thread did before the first action ahead of everything the second thread does after the second action. <!-- r[concurrency.memory.synchronization-orders-joins] -->
   A read that is ordered after an assignment, with no other assignment to the location between or unordered with them, returns the value of that assignment. <!-- r[concurrency.memory.read-ordered-after] -->
4. **Otherwise a read may be stale.** When two threads use a location with nothing ordering them, a read returns the value of some assignment that is not ordered after it. <!-- r[concurrency.memory.otherwise-read-stale] -->
   It may be an older value than the latest one. <!-- r[concurrency.memory.older-value-latest] -->
   Nothing else goes wrong: the program's types hold, and no other location is affected. <!-- r[concurrency.memory.nothing-else-goes] -->

| First action | Second action |
| --- | --- |
| A thread releases a `Lock` | A thread next acquires that lock <!-- r[concurrency.memory.thread-releases-lock] --> |
| An operation on an atomic cell | A later operation on the same cell <!-- r[concurrency.memory.operation-atomic-cell] --> |
| `Thread.start` or `Scope.fork` | The first action of the new thread's body <!-- r[concurrency.memory.thread-start-scope] --> |
| The last action of a thread's body | The return of `join`, or of `result` on its task <!-- r[concurrency.memory.last-action-thread] --> |
| The end of a class's initialization | Any thread's use of that class <!-- r[concurrency.memory.end-class-initialization] --> |

Within one thread, actions are ordered as [chapter 9](09-execution.md#91-order-of-evaluation) evaluates them. <!-- r[concurrency.memory.within-thread-actions] -->

A field update such as `count = count + 1` is a read and a later assignment. <!-- r[concurrency.memory.field-update-count] -->
Two threads that run it with nothing ordering them may lose an update. <!-- r[concurrency.memory.threads-run-nothing] -->
A value that must change in one step is held in an atomic cell, or is guarded by a lock. <!-- r[concurrency.memory.value-change-step] -->

There is no `volatile` modifier and no per-object monitor. <!-- r[concurrency.memory.no-volatile-modifier] -->

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

A `Lock` is held by at most one thread at a time. <!-- r[concurrency.locks.lock-held-most] -->
`lock` waits until the running thread can hold it. <!-- r[concurrency.locks.lock-waits-until] -->
A thread that holds it may call `lock` again, and it then releases the lock by calling `unlock` as many times. <!-- r[concurrency.locks.thread-holds-call] -->
`unlock` by a thread that does not hold the lock raises `IllegalStateException`. <!-- r[concurrency.locks.unlock-thread-not] -->

`exclusive` calls `lock`, runs the body, and calls `unlock` however the body completed. <!-- r[concurrency.locks.exclusive-calls-lock] -->
It returns the body's result. <!-- r[concurrency.locks.returns-body-result] -->

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

**Waiting for a condition.** A `Condition` belongs to the lock whose `newCondition` created it. <!-- r[concurrency.locks.waiting-condition-belongs] -->
Each of its three methods raises `IllegalStateException` unless the running thread holds that lock. <!-- r[concurrency.locks.three-methods-raises] -->

- `await` releases the lock completely, waits, and holds the lock again as before when it returns. <!-- r[concurrency.locks.await-releases-lock] -->
  It may return without a signal, so a caller tests what it waits for in a loop. <!-- r[concurrency.locks.return-without-signal] -->
- `signal` lets one waiting thread return from `await`, if there is one. <!-- r[concurrency.locks.signal-lets-waiting] -->
  `signalAll` lets every waiting thread return. <!-- r[concurrency.locks.signalall-lets-waiting] -->

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

An atomic cell holds one value. <!-- r[concurrency.atomics.atomic-cell-holds] -->
Each method is one step: no other operation on the cell happens between its read and its assignment. <!-- r[concurrency.atomics.method-step-no] -->

`compareAndSet` stores `update` when the cell holds `expected`, and returns whether it did. <!-- r[concurrency.atomics.compareandset-stores-update] -->
`AtomicInt` compares with `equals`. <!-- r[concurrency.atomics.atomicint-compares-equals] -->
`Atomic<T>` compares with `identical`, so a cell of a reference class compares references. <!-- r[concurrency.atomics.atomic-compares-identical] -->

`addAndGet` adds `delta` to the cell and returns the sum. <!-- r[concurrency.atomics.addandget-adds-delta] -->
`incrementAndGet` is `addAndGet(1)`. <!-- r[concurrency.atomics.incrementandget-addandget] -->
When the sum is not an `Int`, the method raises `ArithmeticException` and the cell is unchanged. <!-- r[concurrency.atomics.sum-not-int] -->

## 13.5 Scopes

A scope starts tasks and waits for all of them. <!-- r[concurrency.scopes.scope-starts-tasks] -->
No task outlives the call that created its scope. <!-- r[concurrency.scopes.no-task-outlives] -->

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

`Scope.call` creates a scope, runs `body` on the running thread with that scope, and returns the body's result. <!-- r[concurrency.scopes.scope-call-creates] -->
Before it returns, it cancels the scope if the body raised an exception, and it waits for every task of the scope to complete. <!-- r[concurrency.scopes.before-returns-cancels] -->

`fork` starts a task: a new thread that runs `body.invoke()`. <!-- r[concurrency.scopes.fork-starts-task] -->
It returns once the thread exists. <!-- r[concurrency.scopes.returns-once-thread] -->
Only the thread that is running the scope's `call` may fork, and only while the body of that `call` is running. <!-- r[concurrency.scopes.only-thread-running] -->
Any other `fork` raises `IllegalStateException`. <!-- r[concurrency.scopes.fork-raises-illegalstateexception] -->

`result` waits until the task has completed, and returns what its body returned. <!-- r[concurrency.scopes.result-waits-until] -->
If the body raised an exception, `result` raises that exception. <!-- r[concurrency.scopes.body-raised-exception] -->
`isDone` is `true` once the body has completed in either way. <!-- r[concurrency.scopes.isdone-true-once] -->

```java
Int total = Scope.call((scope) -> {
    Task<Int> left = scope.fork(() -> count(firstHalf));
    Task<Int> right = scope.fork(() -> count(secondHalf));
    return left.result() + right.result();
});
```

**Failure.** A task fails when its body raises an exception other than `CancellationException`. <!-- r[concurrency.scopes.failure-task-fails] -->
A task that fails cancels its scope. <!-- r[concurrency.scopes.task-fails-cancels] -->
When `call` has waited for every task, it completes in the first of these ways that applies: <!-- r[concurrency.scopes.call-waited-task] -->

1. If the body of `call` raised an exception, that exception propagates. <!-- r[concurrency.scopes.body-call-raised] -->
   The exceptions of failed tasks are recorded as suppressed on it, in the order the tasks were forked, except one that is the same exception. <!-- r[concurrency.scopes.exceptions-failed-tasks] -->
2. If a task failed, the exception of the first failed task in fork order propagates, with those of the others suppressed on it. <!-- r[concurrency.scopes.task-failed-exception] -->
3. Otherwise `call` returns the body's result. <!-- r[concurrency.scopes.otherwise-call-returns] -->

A `CancellationException` from a task is never reported by `call`. <!-- r[concurrency.scopes.cancellationexception-task-never] -->

## 13.6 Cancellation

Cancellation is a request. <!-- r[concurrency.cancellation.cancellation-request] -->
A cancelled task goes on running until it reaches a point that checks for the request. <!-- r[concurrency.cancellation.cancelled-task-goes] -->

`cancel` marks the scope as cancelled, and `isCancelled` reports the mark. <!-- r[concurrency.cancellation.cancel-marks-scope] -->
A scope is also cancelled by a task that fails and by an exception from the body of its `call`. <!-- r[concurrency.cancellation.scope-cancelled-task] -->
A scope that is cancelled stays cancelled. <!-- r[concurrency.cancellation.scope-cancelled-stays] -->

A task is cancelled when its scope is. <!-- r[concurrency.cancellation.task-cancelled-scope] -->
A thread started by `Thread.start` is never cancelled, and neither is the thread that runs `main`. <!-- r[concurrency.cancellation.thread-started-start] -->

A cancelled task raises `CancellationException` at these points, and at no others: <!-- r[concurrency.cancellation.cancelled-task-raises] -->

- a call of `Thread.checkCancelled`; <!-- r[concurrency.cancellation.call-thread-checkcancelled] -->
- a call of `Thread.sleep`, `Thread.join`, `Task.result` or `Condition.await`, when it is called and while it waits. <!-- r[concurrency.cancellation.call-thread-sleep] -->

`Lock.lock` is not such a point, so code that cleans up may take a lock. <!-- r[concurrency.cancellation.lock-not-point] -->
A task inside a foreign call finishes the call. <!-- r[concurrency.cancellation.task-inside-foreign] -->
No exception arrives between two statements that do not ask for it, so a task's own data is never left half updated by a cancellation. <!-- r[concurrency.cancellation.no-exception-arrives] -->

A long computation in a task calls `Thread.checkCancelled` now and then. <!-- r[concurrency.cancellation.long-computation-task] -->
On a thread that is not a cancelled task, it does nothing. <!-- r[concurrency.cancellation.thread-not-cancelled] -->

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
