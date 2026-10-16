# 12. Flow

This chapter defines four things the compiler works out from the text of a method: which expressions are constants, which statements can be reached, which variables are assigned before they are used, and where a narrowed type holds. <!-- r[flow.intro.defines-four-things] -->

## 12.1 Constant expressions

A constant expression has a value that the compiler computes. <!-- r[flow.constants.constant-expression-value] -->
It is one of: <!-- r[flow.constants.rule] -->

- a literal other than `null`; <!-- r[flow.constants.literal-null] -->
- a constant expression in parentheses; <!-- r[flow.constants.constant-expression-parentheses] -->
- an operator of [chapter 4](04-methods.md) applied to constant expressions, when the method it spells belongs to a numeric class, `Boolean`, `Char` or `String`; <!-- r[flow.constants.operator-applied-constant] -->
- `a && b`, `a || b` or `c ? a : b` whose operands are constant expressions; <!-- r[flow.constants.and-or-operands] -->
- a call of `from` or `nearest` on a numeric class, or of `Char.from`, whose argument is a constant expression; <!-- r[flow.constants.call-from-nearest] -->
- the name of a constant field; <!-- r[flow.constants.name-constant-field] -->
- the name of an enum constant. <!-- r[flow.constants.name-enum-constant] -->

A constant field is a `static final` field whose type is a numeric class, `Boolean`, `Char` or `String` and whose initializer is a constant expression. <!-- r[flow.constants.constant-field-static] -->
Reading one does not initialize its class, as [chapter 9](09-execution.md#97-class-initialization) says. <!-- r[flow.constants.reading-not-initialize] -->

Numeric literals in a constant expression take their class by the rules of [chapter 6](06-numbers.md), and an implicit conversion of that chapter may apply. <!-- r[flow.constants.numeric-literals-constant] -->
`"v" + 2` is a constant `String`. <!-- r[flow.constants.constant-string] -->
A constant expression calls no other method, reads no other field, and creates no array. <!-- r[flow.constants.constant-expression-calls] -->

A constant expression whose evaluation would raise an exception is rejected. <!-- r[flow.constants.constant-expression-evaluation] -->
`Int32 big = 2147483647 + 1;` is rejected, and so is `1.0 / 0`. <!-- r[flow.constants.int32-big-rejected] -->

Constant expressions are required as the constants of a `switch` arm and as the arguments of an annotation. <!-- r[flow.constants.constant-expressions-required] -->
A condition that is the constant `true` or `false` affects [section 12.2](#122-reachability). <!-- r[flow.constants.condition-constant-true] -->

## 12.2 Reachability

Every statement must be reachable. <!-- r[flow.reachability.statement-reachable] -->
An unreachable statement is rejected. <!-- r[flow.reachability.unreachable-statement-rejected] -->

A statement either can complete normally or cannot. <!-- r[flow.reachability.statement-complete-normally] -->
The body of a method, of a constructor, of a lambda and of a static initializer is reachable. <!-- r[flow.reachability.body-method-constructor] -->
A statement in a block is reachable when it is the first statement of a reachable block, or when the statement before it can complete normally. <!-- r[flow.reachability.statement-block-reachable] -->

| Statement | Can complete normally when |
| --- | --- |
| A local declaration, an expression statement, `assert`, the empty statement | Always <!-- r[flow.reachability.local-declaration-expression] --> |
| A block | Its last statement can, or it is empty <!-- r[flow.reachability.block-last-statement] --> |
| `if (c) s` | Always <!-- r[flow.reachability.if-always] --> |
| `if (c) s else t` | `s` can or `t` can <!-- r[flow.reachability.if-else] --> |
| `while (c) s` | `c` is not the constant `true`, or a `break` leaves the loop <!-- r[flow.reachability.while-not-constant] --> |
| `for (init; c; update) s` | `c` is written and is not the constant `true`, or a `break` leaves the loop <!-- r[flow.reachability.for-init-update] --> |
| `for (T x : e) s` | Always <!-- r[flow.reachability.for-always] --> |
| `switch` statement | Some arm's body can, or the switch is not exhaustive, or a `break` leaves it <!-- r[flow.reachability.switch-statement-arm] --> |
| `try` | The `try` block or some `catch` block can, and the `finally` block can if there is one <!-- r[flow.reachability.try-block-catch] --> |
| `using` | Its block can <!-- r[flow.reachability.using-block] --> |
| `label: s` | `s` can, or a `break label` leaves it <!-- r[flow.reachability.label-break-leaves] --> |
| `return`, `throw`, `break`, `continue` | Never <!-- r[flow.reachability.return-throw-break] --> |

The body of a loop is reachable when the loop is, unless its condition is the constant `false`. <!-- r[flow.reachability.body-loop-reachable] -->
A loop whose condition is the constant `false` is rejected. <!-- r[flow.reachability.loop-condition-constant] -->

The branches of an `if` are reachable when the `if` is, whatever its condition. <!-- r[flow.reachability.branches-if-reachable] -->
`if (DEBUG) { ... }` is legal when `DEBUG` is the constant `false`. <!-- r[flow.reachability.if-debug-legal] -->

Each arm of a switch, each `catch` block and a `finally` block is reachable when its statement is. <!-- r[flow.reachability.arm-switch-catch] -->
[Chapter 9](09-execution.md#93-try) rejects a `catch` clause that an earlier clause makes useless, and [chapter 4](04-methods.md#49-statements) rejects a switch arm that cannot match. <!-- r[flow.reachability.rejects-catch-clause] -->

**The end of a body.** A method whose result is `Unit` may complete by reaching the end of its body. <!-- r[flow.reachability.end-body-method] -->
The body of any other method must not be able to complete normally: every path ends in a `return` with a value or in a `throw`. <!-- r[flow.reachability.body-method-not] -->
The same holds for a lambda with a block body. <!-- r[flow.reachability.holds-lambda-block] -->

In a constructor, `return;` is legal only after the `super(...)` or `this(...)` call that the body writes. <!-- r[flow.reachability.constructor-return-legal] -->
A constructor that leaves `super()` implicit has no `return`. <!-- r[flow.reachability.constructor-leaves-super] -->
`return` with a value is rejected in a constructor. <!-- r[flow.reachability.return-value-rejected] -->

```java
static Int sign(Int n) {
    if (n > 0) {
        return 1;
    } else if (n < 0) {
        return -1;
    }
    return 0;           // without this line the method is rejected
}
```

```java
while (true) {
    step();
}
cleanup();              // rejected: unreachable
```

## 12.3 Definite assignment

**Locals.** A local is definitely assigned at a point when every path that reaches the point has passed its initializer or an assignment to it. <!-- r[flow.assignment.locals-local-definitely] -->
A use of a local that is not definitely assigned is rejected. <!-- r[flow.assignment.use-local-not] -->
A parameter, the variable of a `for` over a collection, a `catch` parameter, a `using` binding and the name of a type arm are assigned where they are in scope. <!-- r[flow.assignment.parameter-variable-for] -->

The paths are those of [section 12.2](#122-reachability). <!-- r[flow.assignment.paths] -->
A statement that cannot complete normally contributes no path to what follows it. <!-- r[flow.assignment.statement-complete-normally] -->

- After `if (c) s else t`, a local is assigned when it is assigned after `s` and after `t`. <!-- r[flow.assignment.after-if-else] -->
  A branch that cannot complete normally does not count against it. <!-- r[flow.assignment.branch-complete-normally] -->
  With no `else`, the local is assigned after the `if` only when it was assigned before it or by `c`. <!-- r[flow.assignment.no-else-local] -->
- After a loop, a local is assigned when it was assigned before the loop, or by the condition when the condition is false. <!-- r[flow.assignment.after-loop-local] -->
  An assignment in the body does not count, because the body may not run. <!-- r[flow.assignment.assignment-body-not] -->
- After a `switch` statement, a local is assigned when every arm assigns it and the switch is exhaustive. <!-- r[flow.assignment.after-switch-statement] -->
- After a `try` statement with no `finally`, a local is assigned when it is assigned after the `try` block and after every `catch` block. <!-- r[flow.assignment.after-try-statement] -->
  At the start of a `catch` or a `finally` block, a local is assigned only when it was assigned before the `try` statement. <!-- r[flow.assignment.start-catch-finally] -->
  A local that the `finally` block assigns is assigned after the statement. <!-- r[flow.assignment.local-finally-block] -->
- In `a && b`, `b` is evaluated with what `a` assigns. <!-- r[flow.assignment.and-evaluated-assigns] -->
  After the whole expression, only what `a` assigns is certain. <!-- r[flow.assignment.after-whole-expression] -->
  `||` is the same. <!-- r[flow.assignment.or] -->
  After `c ? a : b`, a local is assigned when `c` assigns it or both arms do. <!-- r[flow.assignment.after-local-assigned] -->
- A lambda may use a local only when it is definitely assigned before the lambda expression. <!-- r[flow.assignment.lambda-use-local] -->
  [Chapter 4](04-methods.md#48-lambdas-and-method-references) also requires that the local is never assigned after it is initialized. <!-- r[flow.assignment.requires-local-never] -->

A `final` local is assigned at most once. <!-- r[flow.assignment.final-local-assigned] -->
An assignment to it is rejected unless the local is definitely unassigned there: no path to the assignment has passed its initializer or another assignment. <!-- r[flow.assignment.assignment-rejected-unless] -->

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

**Fields in a constructor.** [Chapter 9](09-execution.md#95-constructing-an-object) has a constructor assign the fields of its class before it calls `super(...)`. <!-- r[flow.assignment.fields-constructor-assign] -->
The rules for a local apply to each of those fields in that part of the body: <!-- r[flow.assignment.rules-local-apply] -->

- At the `super(...)` call, written or implicit, every field declared in the class is definitely assigned, by its initializer or by the body. <!-- r[flow.assignment.super-call-written] -->
  A nullable field is exempt and starts as `null`. <!-- r[flow.assignment.nullable-field-exempt] -->
- A field is read there only where it is definitely assigned. <!-- r[flow.assignment.field-read-only] -->
- A final field is assigned at most once, by its initializer or by the body before the `super(...)` call, and nowhere else. <!-- r[flow.assignment.final-field-assigned] -->

A constructor that calls `this(...)` assigns no field of its own: the constructor it calls has assigned them all. <!-- r[flow.assignment.constructor-calls-this] -->

```java
public class Span {
    final Int start;
    final Int length;
    String? label;

    public Span(Int start, Int end) {
        this.start = start;
        this.length = end - this.start;     // start is assigned, so it may be read
    }                                       // label is null; super() runs here
}
```

**Static fields.** At the end of a class's static initializers, in source order, every static field of the class is definitely assigned, by its initializer or by a static initializer block. <!-- r[flow.assignment.static-fields-end] -->
A nullable static field is exempt. <!-- r[flow.assignment.nullable-static-field] -->
A static final field is assigned at most once, and only there. <!-- r[flow.assignment.static-final-field] -->
A static field that is read while initialization is still running is checked at run time, as [chapter 9](09-execution.md#97-class-initialization) defines. <!-- r[flow.assignment.static-field-read] -->

**Value classes.** The fields of a value class are assigned by its constructor's arguments. <!-- r[flow.assignment.value-classes-fields] -->
Nothing in the class assigns one. <!-- r[flow.assignment.nothing-class-assigns] -->

## 12.4 Narrowing

[Chapter 5](05-null-and-unit.md#54-null-tests-and-narrowing) narrows a local or a parameter after a null test or an `instanceof`, and [chapter 8](08-annotations.md#85-narrowing) after a call of a `@Narrows` method. <!-- r[flow.narrowing.narrows-local-parameter] -->
This section says where the narrowed type holds. <!-- r[flow.narrowing.says-narrowed-type] -->

A test gives a fact about a variable `x` on one of its outcomes: <!-- r[flow.narrowing.test-gives-fact] -->

| Test | Fact |
| --- | --- |
| `x != null` | When `true`, `x` is not `null` <!-- r[flow.narrowing.not-equals-null-true] --> |
| `x == null` | When `false`, `x` is not `null` <!-- r[flow.narrowing.equals-null-false] --> |
| `x instanceof T` | When `true`, `x` is a `T` <!-- r[flow.narrowing.instanceof-true] --> |
| A call of a `@Narrows(Q.class)` method with `x` as its subject | When `true`, `x` gains or loses `Q` <!-- r[flow.narrowing.call-narrows-class] --> |

Conditions combine facts: <!-- r[flow.narrowing.conditions-combine-facts] -->

- `!c` has when `true` the facts `c` has when `false`, and the reverse. <!-- r[flow.narrowing.true-facts-false] -->
- `a && b` has when `true` the facts of `a` and of `b` when `true`. <!-- r[flow.narrowing.and-true-facts] -->
  When `false` it has only the facts that both have when `false`. <!-- r[flow.narrowing.false-only-facts] -->
  `b` is checked with the facts `a` has when `true`. <!-- r[flow.narrowing.checked-facts-true] -->
- `a || b` has when `false` the facts of `a` and of `b` when `false`. <!-- r[flow.narrowing.or-false-facts] -->
  When `true` it has only the facts that both have when `true`. <!-- r[flow.narrowing.true-only-facts] -->
  `b` is checked with the facts `a` has when `false`. <!-- r[flow.narrowing.checked-facts-false] -->
- Parentheses change nothing. <!-- r[flow.narrowing.parentheses-change-nothing] -->

A fact holds at a point when it holds on every path that reaches the point. <!-- r[flow.narrowing.fact-holds-point] -->

- In `if (c) s else t`, the facts of `c` when `true` hold at the start of `s`, and its facts when `false` hold at the start of `t`. <!-- r[flow.narrowing.if-else-facts] -->
  After the statement, a fact holds when it holds at the end of each branch that can complete normally. <!-- r[flow.narrowing.after-statement-fact] -->
- In `while (c) s` and in a `for` statement, the facts of `c` when `true` hold at the start of the body, and its facts when `false` hold after the loop unless a `break` leaves it. <!-- r[flow.narrowing.while-for-statement] -->
- In `c ? a : b`, the facts of `c` when `true` hold in `a`, and its facts when `false` hold in `b`. <!-- r[flow.narrowing.facts-true-hold] -->
- In `a ?? b`, no fact comes from the test of `a`. <!-- r[flow.narrowing.coalesce-no-fact] -->
- `assert c;` gives the facts of `c` when `true` to what follows. <!-- r[flow.narrowing.assert-gives-facts] -->
- A fact that holds before a `try` statement holds in its `catch` and `finally` blocks when the `try` block does not assign the variable. <!-- r[flow.narrowing.fact-holds-before-try] -->
- A fact about a variable holds inside a lambda that uses the variable. <!-- r[flow.narrowing.fact-about-variable] -->
  A lambda uses only variables that are never assigned again, so the fact cannot change. <!-- r[flow.narrowing.lambda-uses-only] -->

An assignment to `x` ends every fact about `x`. <!-- r[flow.narrowing.assignment-ends-fact] -->
A fact that holds before a loop holds in the loop only when no statement of the loop assigns `x`. <!-- r[flow.narrowing.fact-holds-before-loop] -->

```java
static Int length(String? text) {
    if (text == null || text.isEmpty()) {     // the right side runs only when text is not null
        return 0;
    }
    return text.length();                     // the null branch returned, so text is a String
}
```
