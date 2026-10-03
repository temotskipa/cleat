# Cleat

Cleat is a programming language with Java's surface and one object model. Every value is an instance of a class. A class or method is final unless it is opened. A member is private unless a wider audience is written. A type is non-null unless it is marked `@Nullable`.

The initial program compiles ahead of time to native code through LLVM. `Loader` typechecks further classes, and the JIT compiles them. The runtime is a library: a precise moving collector, unwind, and class metadata. Execution is native code. The bytes of a loaded image are not a stable ABI.

The name is a working title. Source files use the extension `.cleat`. The prelude package is `cleat`.

The specification in `spec/` is normative. This file is the map. The design conversation is not a rule. The text is current as of 2026-10-04.

## How to read the specification

Chapters 1 through 9, together with chapters 11 through 13, define the language. Chapter 10 records what is absent and which choices an implementation may make. Where an earlier chapter is silent, chapter 10 says whether the feature is absent or specified somewhere else. An omission is not permission to invent a rule.

| Chapter | Subject |
| --- | --- |
| [1. Source](spec/01-source.md) | Lexicon, files, packages, imports, scope |
| [2. Objects](spec/02-objects.md) | Values, references, classes, interfaces, enums |
| [3. Visibility](spec/03-visibility.md) | Audiences, `only`, `sealed`, `permits` |
| [4. Methods](spec/04-methods.md) | Methods, blocks, control, foreign calls |
| [5. Null and Unit](spec/05-null-and-unit.md) | `Null`, `@Nullable`, `Unit`, the `void` form |
| [6. Numbers](spec/06-numbers.md) | `Number`, fixed widths, vectors, arrays, `String` |
| [7. Generics](spec/07-generics.md) | Reified arguments, `in` and `out`, use-site `?` |
| [8. Annotations](spec/08-annotations.md) | Declarations, qualifiers, compile-time expansion |
| [9. Execution](spec/09-execution.md) | Evaluation, construction, loading, collection |
| [10. Omissions](spec/10-deferred.md) | Absent features, and bounded implementation choices |
| [11. Syntax](spec/11-syntax.md) | Lexical and syntactic grammar |
| [12. Flow](spec/12-flow.md) | Constants, reachability, definite assignment |
| [13. Concurrency](spec/13-concurrency.md) | Threads, `Scope`, locks, atomic cells |

## Prelude names

The specification fixes these names. They are ordinary prelude classes and methods.

| Name | Role |
| --- | --- |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned fixed widths |
| `wrappingPlus`, `wrappingMinus`, `wrappingTimes` | Wrapping integer arithmetic |
| `truncatingDiv`, `truncatingRem` | Division that discards a remainder |
| `shiftLeft`, `shiftRight`, `complement`, `xor` | Shifts and bitwise operations |
| `Float32.round`, `Float64.round` | Rounding onto a float |
| `totalOrder` | IEEE 754 total order |
| `ArithmeticException`, `ClassCastException`, `IndexOutOfBoundsException`, `ClassInitializationException` | Failures named by the numeric, cast, index, and initialization rules |
| `IllegalArgumentException`, `IllegalStateException` | A rejected argument, or an object used in the wrong state |
| `AssertionException`, `OutOfMemoryException`, `ClassLoadException`, `CancellationException` | A failed assertion, a failed allocation, a rejected image, a cancelled task |
