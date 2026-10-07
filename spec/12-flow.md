# 12. Flow

This chapter defines four things the compiler works out from the text of a method: which expressions are constants, which statements can be reached, which variables are assigned before they are used, and where a narrowed type holds.

## 12.1 Constant expressions

A constant expression has a value that the compiler computes. It is one of:

- a literal other than `null`;
- a constant expression in parentheses;
- an operator of [chapter 4](04-methods.md) applied to constant expressions, when the method it spells belongs to a numeric class, `Boolean`, `Char` or `String`;
- `a && b`, `a || b` or `c ? a : b` whose operands are constant expressions;
- a call of `from` or `nearest` on a numeric class, or of `Char.from`, whose argument is a constant expression;
- the name of a constant field;
- the name of an enum constant.

A constant field is a `static final` field whose type is a numeric class, `Boolean`, `Char` or `String` and whose initializer is a constant expression. Reading one does not initialize its class, as [chapter 9](09-execution.md#97-class-initialization) says.

Numeric literals in a constant expression take their class by the rules of [chapter 6](06-numbers.md), and an implicit conversion of that chapter may apply. `"v" + 2` is a constant `String`. A constant expression calls no other method, reads no other field, and creates no array.

A constant expression whose evaluation would raise an exception is rejected. `Int32 big = 2147483647 + 1;` is rejected, and so is `1.0 / 0`.

Constant expressions are required as the constants of a `switch` arm and as the arguments of an annotation. A condition that is the constant `true` or `false` affects [section 12.2](#122-reachability).

## 12.2 Reachability

Every statement must be reachable. An unreachable statement is rejected.

A statement either can complete normally or cannot. The body of a method, of a constructor, of a lambda and of a static initializer is reachable. A statement in a block is reachable when it is the first statement of a reachable block, or when the statement before it can complete normally.

| Statement | Can complete normally when |
| --- | --- |
| A local declaration, an expression statement, `assert`, the empty statement | Always |
| A block | Its last statement can, or it is empty |
| `if (c) s` | Always |
| `if (c) s else t` | `s` can or `t` can |
| `while (c) s` | `c` is not the constant `true`, or a `break` leaves the loop |
| `for (init; c; update) s` | `c` is written and is not the constant `true`, or a `break` leaves the loop |
| `for (T x : e) s` | Always |
| `switch` statement | Some arm's body can, or the switch is not exhaustive, or a `break` leaves it |
| `try` | The `try` block or some `catch` block can, and the `finally` block can if there is one |
| `using` | Its block can |
| `label: s` | `s` can, or a `break label` leaves it |
| `return`, `throw`, `break`, `continue` | Never |

The body of a loop is reachable when the loop is, unless its condition is the constant `false`. A loop whose condition is the constant `false` is rejected.

The branches of an `if` are reachable when the `if` is, whatever its condition. `if (DEBUG) { ... }` is legal when `DEBUG` is the constant `false`.

Each arm of a switch, each `catch` block and a `finally` block is reachable when its statement is. [Chapter 9](09-execution.md#93-try) rejects a `catch` clause that an earlier clause makes useless, and [chapter 4](04-methods.md#49-statements) rejects a switch arm that cannot match.

**The end of a body.** A method whose result is `Unit` may complete by reaching the end of its body. The body of any other method must not be able to complete normally: every path ends in a `return` with a value or in a `throw`. The same holds for a lambda with a block body.

In a constructor, `return;` is legal only after the `super(...)` or `this(...)` call that the body writes. A constructor that leaves `super()` implicit has no `return`. `return` with a value is rejected in a constructor.

```java
static Int sign(Int n) {
    if (n > 0) {
        return 1;
    } else if (n < 0) {
        return -1;
    }
    return 0;           // without this line the method is rejected
}

while (true) {
    step();
}
cleanup();              // rejected: unreachable
```

## 12.3 Definite assignment

**Locals.** A local is definitely assigned at a point when every path that reaches the point has passed its initializer or an assignment to it. A use of a local that is not definitely assigned is rejected. A parameter, the variable of a `for` over a collection, a `catch` parameter, a `using` binding and the name of a type arm are assigned where they are in scope.

The paths are those of [section 12.2](#122-reachability). A statement that cannot complete normally contributes no path to what follows it.

- After `if (c) s else t`, a local is assigned when it is assigned after `s` and after `t`. A branch that cannot complete normally does not count against it. With no `else`, the local is assigned after the `if` only when it was assigned before it or by `c`.
- After a loop, a local is assigned when it was assigned before the loop, or by the condition when the condition is false. An assignment in the body does not count, because the body may not run.
- After a `switch` statement, a local is assigned when every arm assigns it and the switch is exhaustive.
- After a `try` statement with no `finally`, a local is assigned when it is assigned after the `try` block and after every `catch` block. At the start of a `catch` or a `finally` block, a local is assigned only when it was assigned before the `try` statement. A local that the `finally` block assigns is assigned after the statement.
- In `a && b`, `b` is evaluated with what `a` assigns. After the whole expression, only what `a` assigns is certain. `||` is the same. After `c ? a : b`, a local is assigned when `c` assigns it or both arms do.
- A lambda may use a local only when it is definitely assigned before the lambda expression. [Chapter 4](04-methods.md#48-lambdas-and-method-references) also requires that the local is never assigned after it is initialized.

A `final` local is assigned at most once. An assignment to it is rejected unless the local is definitely unassigned there: no path to the assignment has passed its initializer or another assignment.

```java
Int limit;
if (strict) {
    limit = 10;
} else {
    limit = 100;
}
use(limit);             // assigned on both paths

Int count;
while (more()) {
    count = 1;
}
use(count);             // rejected: the body may not have run
```

**Fields in a constructor.** [Chapter 9](09-execution.md#95-constructing-an-object) has a constructor assign the fields of its class before it calls `super(...)`. The rules for a local apply to each of those fields in that part of the body:

- At the `super(...)` call, written or implicit, every field declared in the class is definitely assigned, by its initializer or by the body. A `@Nullable` field is exempt and starts as `null`.
- A field is read there only where it is definitely assigned.
- A final field is assigned at most once, by its initializer or by the body before the `super(...)` call, and nowhere else.

A constructor that calls `this(...)` assigns no field of its own: the constructor it calls has assigned them all.

```java
public class Span {
    final Int start;
    final Int length;
    @Nullable String label;

    public Span(Int start, Int end) {
        this.start = start;
        this.length = end - this.start;     // start is assigned, so it may be read
    }                                       // label is null; super() runs here
}
```

**Static fields.** At the end of a class's static initializers, in source order, every static field of the class is definitely assigned, by its initializer or by a static initializer block. A `@Nullable` static field is exempt. A static final field is assigned at most once, and only there. A static field that is read while initialization is still running is checked at run time, as [chapter 9](09-execution.md#97-class-initialization) defines.

**Value classes.** The fields of a value class are assigned by its constructor's arguments. Nothing in the class assigns one.

## 12.4 Narrowing

[Chapter 5](05-null-and-unit.md#54-null-tests-and-narrowing) narrows a local or a parameter after a null test or an `instanceof`, and [chapter 8](08-annotations.md#85-narrowing) after a call of a `@Narrows` method. This section says where the narrowed type holds.

A test gives a fact about a variable `x` on one of its outcomes:

| Test | Fact |
| --- | --- |
| `x != null` | When `true`, `x` is not `null` |
| `x == null` | When `false`, `x` is not `null` |
| `x instanceof T` | When `true`, `x` is a `T` |
| A call of a `@Narrows(Q.class)` method with `x` as its subject | When `true`, `x` gains or loses `Q` |

Conditions combine facts:

- `!c` has when `true` the facts `c` has when `false`, and the reverse.
- `a && b` has when `true` the facts of `a` and of `b` when `true`. When `false` it has only the facts that both have when `false`. `b` is checked with the facts `a` has when `true`.
- `a || b` has when `false` the facts of `a` and of `b` when `false`. When `true` it has only the facts that both have when `true`. `b` is checked with the facts `a` has when `false`.
- Parentheses change nothing.

A fact holds at a point when it holds on every path that reaches the point.

- In `if (c) s else t`, the facts of `c` when `true` hold at the start of `s`, and its facts when `false` hold at the start of `t`. After the statement, a fact holds when it holds at the end of each branch that can complete normally.
- In `while (c) s` and in a `for` statement, the facts of `c` when `true` hold at the start of the body, and its facts when `false` hold after the loop unless a `break` leaves it.
- In `c ? a : b`, the facts of `c` when `true` hold in `a`, and its facts when `false` hold in `b`.
- In `a ?? b`, no fact comes from the test of `a`.
- `assert c;` gives the facts of `c` when `true` to what follows.
- A fact that holds before a `try` statement holds in its `catch` and `finally` blocks when the `try` block does not assign the variable.
- A fact about a variable holds inside a lambda that uses the variable. A lambda uses only variables that are never assigned again, so the fact cannot change.

An assignment to `x` ends every fact about `x`. A fact that holds before a loop holds in the loop only when no statement of the loop assigns `x`.

```java
static Int length(@Nullable String text) {
    if (text == null || text.isEmpty()) {     // the right side runs only when text is not null
        return 0;
    }
    return text.length();                     // the null branch returned, so text is a String
}
```
