# 12. Constants, reachability, and definite assignment

## 12.1 Constant expressions

A constant expression is evaluated at compile time. It is one of:

- a literal
- a parenthesized constant expression
- a cast of a constant expression, which is the class test of [chapter 4](04-methods.md) and [chapter 2](02-objects.md). The cast does not change a numeric width and does not convert `Char`. It is a constant only when the test succeeds. A constant cast that would raise `ClassCastException` is rejected. `(Int32) 1` is rejected when `1` has type `Number`. `(Char) 65` is rejected. The width change is `Int32.from`, or an expected-type bare literal such as `Int32 i = 1`. The scalar conversion is `Char.from`
- a prelude `from` whose receiver is a fixed numeric class or `Char`, whose argument is a constant expression, and which does not raise. A `from` that would raise `ArithmeticException` is rejected
- a unary or binary operator of [chapter 4](04-methods.md) whose operands are constant expressions and whose result is `Boolean`, `String`, `Char`, or a numeric type
- a `final` field of one of those types, initialized by a constant expression, on a class or interface
- an enum constant
- a conditional `c ? a : b` whose three operands are constant expressions

A constant expression does not send a user method, does not read a non-constant field, and does not allocate an array. `String` concatenation in a constant expression is `plus` on the constant operands and is itself constant. A constant expression that would raise at run time is rejected at compile time.

The Boolean constants are the constant expressions whose value is `true` or `false`.

## 12.2 Reachability

A statement is reachable or unreachable. An unreachable statement is rejected. The first statement of a method, of a constructor, and of an instance or static initializer is reachable. Each following rule says when a completion makes the next statement reachable.

A statement completes normally, or it completes abruptly by `return`, `throw`, `break`, or `continue`. A `return` or `throw` always completes abruptly. A `break` or `continue` completes abruptly when its target is outside the statement being considered.

- An empty statement, an expression statement, an assert, and a local declaration complete normally when they are reachable. A declaration whose initializer throws completes abruptly only at run time. The following statement is reachable.
- A block completes normally when its last statement completes normally, or when it is empty. A statement in a block is reachable when it is the first and the block is reachable, or when the preceding statement completes normally.
- `if (c) a else b` completes normally when at least one branch completes normally. When both branches complete abruptly, the statement after the `if` is unreachable. A missing `else` completes normally.
- When `c` is a constant `true`, the else branch is unreachable. When `c` is a constant `false`, the then branch is unreachable.
- `while (c) body` completes normally when `c` is not a constant `true`, and when a `break` targets the loop. The body is reachable when `c` is not a constant `false`. The statement after the loop is unreachable when `c` is a constant `true` and no `break` targets the loop.
- A three-clause `for` uses the same rule on its condition. A missing condition is the constant `true`.
- A for-each completes normally. Its body is reachable.
- `try` completes normally when the try block or some catch completes normally, and the finally, if present, completes normally. If the finally completes abruptly, the try completes abruptly for that reason and the statement after it is unreachable. A catch whose parameter type cannot be thrown by the try, because an earlier catch's type is a supertype, is rejected. That is the unreachable-catch rule of [chapter 9](09-execution.md).
- A switch statement completes normally when some reachable arm completes normally, or when it is not exhaustive. An arm is reachable when the switch is reachable. The statement after a switch expression used as a statement follows the expression-statement rule.
- `break` and `continue` do not complete normally. `return` and `throw` do not complete normally.
- A labeled statement completes as the statement it labels, except that a `break` of that label completes the labeled statement normally.

A method declared `void` may fall off its last reachable statement. Every other method and every constructor must not. On every path that reaches the end of a non-void method, a `return` of a value has already completed the path. A constructor must not `return` a value. `return;` in a constructor completes the constructor normally.

## 12.3 Definite assignment

A local variable is definitely assigned before a use when every path that reaches the use has executed an initializer or an assignment to that local. A use that is not definitely assigned is rejected. A `final` local is assigned on exactly one path execution: a second assignment is rejected, including an assignment the flow analysis sees on some path after an earlier assignment on every path.

The rules assign a status before and after each statement: definitely assigned, or not.

- The status before the method body has every parameter assigned and every other local unassigned.
- An initializer of a local assigns it after the declaration. A local declared without an initializer is unassigned after the declaration.
- An assignment `name = e` assigns `name` after the statement when `e` completes normally. A compound assignment and an increment do the same.
- After `if`/`else`, a local is assigned when it is assigned after both branches. With no else, it is assigned after the if only when it was assigned before the if.
- A local assigned in a loop body is not assigned after the loop unless it was assigned before the loop. A condition may use a local assigned before the loop.
- `&&` assigns on its right what the left assigns when the left is true, and after the whole expression what both sides assign. `||` assigns on its right what the left assigns when the left is false. After `c ? a : b`, a local is assigned when both arms assign it.
- After `try`/`catch`, a local is assigned when it is assigned after the try and after every catch. A finally that completes normally passes that status through. A finally that assigns a local assigns it after the try.
- A switch arm does not assign a local for the code after the switch unless every arm assigns it and the switch is exhaustive.
- A local declared in a for-init is assigned in the condition, body, and update when its declaration initializes it.

A blank `final` field of a reference class is definitely assigned at the end of every constructor that does not complete by `this(...)`. A constructor that completes by `this(...)` does not assign the blank finals itself. A `final` field with an initializer is assigned. A second assignment to a final field is rejected. A use of a blank final in the constructor before its assignment is rejected. An instance initializer is treated as assignment text inserted after `super(...)` returns, in source order, as [chapter 9](09-execution.md) already orders those initializers.

A static `final` field is definitely assigned at the end of class-object initialization, by its initializer or by a static initializer. The same path rule applies. A cycle in static initialization is the failure [chapter 9](09-execution.md) defines, not a definite-assignment success.

A value-class component is assigned by `new` before the body, and the body must not assign it.
