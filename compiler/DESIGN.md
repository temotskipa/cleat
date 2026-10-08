# How the compiler is built

Notes for whoever works on `compiler/` next. The specification is in `spec/`; this file
says how the implementation meets it, and nothing here is a language rule.

## Pipeline

`lex.rs` → `parse.rs` (`ast.rs`) → `sema/decl.rs` (class table, `sema/program.rs`) →
`sema/check/` (bodies, giving the typed tree of `sema/tir.rs`) → `emit/` (LLVM IR text)
→ `clang -c` → link with the runtime `rt/` (a Rust static library).

The prelude is ordinary source in `../prelude`, compiled with every program. A method
marked `@Intrinsic` has no body: the backend either inlines it or calls the runtime
symbol `cl_<Class>_<method>[_<ParamClass>...]`.

## Values

- A value whose static type is a prelude number class other than `Rational`, or
  `Boolean`, `Char` or `Pointer`, and is not `@Nullable`, is a machine value.
- Every other value is a pointer to an object: any reference class, a value class the
  program declares, `String`, `Rational`, an array, any `@Nullable` type, and any type
  parameter. A machine value is boxed where it meets such a type (`TKind::Coerce`, and
  at call and field boundaries by the declared type of the member).
- `Unit` is no value at all in a result; where one is needed as an object it is the
  runtime's one `Unit` object.

A field has the representation of its declared type, so a field of type `T` is always a
pointer. An array is the exception: `Int[]` stores machine integers, and code that sees
an array through a type parameter (`T[]`) reads and writes it through the runtime, which
looks at the array's own element kind.

## Objects, types and dispatch

- Object: `{ ptr type, i64 gc }` then the fields, superclass first. The type is a
  `TypeDesc`: a class with its type arguments, interned by the runtime, so two types are
  the same exactly when their pointers are.
- The compiler emits one constant `ClassInfo` per class (name, flags, supertypes as type
  expressions over the class's parameters, reference-field offsets, the method table,
  field and annotation data) and constant `TypeExpr` trees. `cl_type_eval(expr, env)`
  turns an expression into a `TypeDesc` given the type arguments in scope.
- Generic code is shared. An instance method finds its class's type arguments from
  `this`. A generic method takes a hidden pointer to its own type arguments.
- A virtual call looks its method up by selector in the receiver's class table. Each
  method that overrides nothing has a selector; `equals`, `hashCode` and `toString` are
  0, 1 and 2. Where an override's machine signature differs from the method it overrides
  (a receiver or parameter that is a machine value, say), the table holds a bridge.
- A lambda is a synthetic value class with the captured values as fields. Its type
  parameters are the type variables in scope where it was written.

## Exceptions

Every compiled function takes the thread's context first. A raised exception is stored
in the context and the function returns; the caller tests the slot after each call and
branches to its handler or returns in turn. `finally` and `using` are compiled once,
with a small integer that says how to continue afterwards.

## Storage

The collector is precise, non-moving mark and sweep, in `rt/src/gc.rs`. Each compiled
function keeps every pointer it holds in a frame of slots linked from the context (a
shadow stack), so the collector sees exactly the live roots. Threads stop for a
collection at allocations, at loop back edges, and around blocking calls. `vendor/mmtk`
is the earlier attempt on MMTk and is not linked.

## Foreign calls

A foreign method whose types are all scalars, or arrays of scalars, is called directly.
One that takes or returns a value class goes through C source the compiler writes
(`emit/mod.rs`, `foreign_shim`) and gives to clang with the module. The C code copies
an object's fields into the struct C expects and back, so the platform's own rules for
passing structs apply.

## Limits

A thread has 16 MB of stack (`rt/src/gc.rs`, `STACK`; `emit/link.rs` asks the linker for
the same). Every compiled function compares its frame's address with a limit in the
context, and a call that finds the stack full ends the program.

## Tests

- `tests/spec_examples.rs`: one test for each fenced example in `spec/`.
- `tests/rejections.rs` and `tests/declarations.rs`: programs the checker must reject.
- `tests/behavior.rs`: programs in `tests/behavior/`, each with the output it must print.
- `tests/host.rs`: foreign structs, the ways a program ends, and warnings.
- `tests/programs.rs`: builds and runs each program in `design/programs/`.

A compiled program runs twice in these tests, the second time with `CLEAT_GC_STRESS=1`,
which collects at every allocation. A pointer the compiled code failed to keep in a
frame slot then shows up as a wrong result or a crash.
