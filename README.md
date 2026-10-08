# Cleat

Cleat is a programming language with Java's surface and one object model. Every value is an instance of a class. A class or method is final unless it is opened. A member is private unless a wider audience is written. A type is non-null unless it is marked `@Nullable`.

A program compiles ahead of time to native code through LLVM. The runtime is a library: a collector, unwind, and class metadata. There is no stable ABI between compilations.

The name is a working title. Source files use the extension `.cleat`. The prelude package is `cleat`.

The specification in `spec/` is normative. This file is the map. The design conversation is not a rule. The text is current as of 2026-10-08.

## Status

Every chapter of the specification has been revised. The decisions behind the revision, and the programs it is tested against, are in [design/](design/foundations.md).

The prelude is Cleat source in `prelude/`. The compiler in `compiler/` implements the specification: it checks a program, emits LLVM IR, and links the result with a runtime library that holds the collector and the bodies of the prelude's `@Intrinsic` methods. [compiler/DESIGN.md](compiler/DESIGN.md) says how it is built.

```
cd compiler
cargo build
cargo run -- check ../design/programs
cargo run -- build ../design/programs/Calc.cleat --entry demo.Calc -o calc.exe
cargo test
```

clang compiles the IR. The compiler looks for it where `CLEAT_CLANG` points, then in `C:\Program Files\LLVM\bin`, then under `%LOCALAPPDATA%\cleat-llvm`, then on the path.

The tests are the specification's own examples, programs the checker must reject, programs with the output they must print, and the nine programs in [design/programs/](design/programs/). Each compiled program runs twice, the second time with the collector forced at every allocation.

| Chapter | Status |
| --- | --- |
| 1. Source | Revised |
| 2. Objects | Revised |
| 3. Visibility | Revised |
| 4. Methods | Revised |
| 5. Null and Unit | Revised |
| 6. Numbers and text | Revised |
| 7. Generics | Revised |
| 8. Annotations | Revised |
| 9. Execution | Revised |
| 10. Omissions | Revised |
| 11. Syntax | Revised |
| 12. Flow | Revised |
| 13. Concurrency | Revised |

## How to read the specification

Chapters 1 through 9, together with chapters 11 through 13, define the language. Chapter 10 records what is absent, what may come later, and which choices an implementation may make. An omission is not permission to invent a rule.

| Chapter | Subject |
| --- | --- |
| [1. Source](spec/01-source.md) | Lexicon, files, packages, imports, scope |
| [2. Objects](spec/02-objects.md) | Values, references, classes, interfaces, enums |
| [3. Visibility](spec/03-visibility.md) | Audiences, `only`, which members have a fixed audience |
| [4. Methods](spec/04-methods.md) | Methods, operators, lambdas, statements, foreign calls |
| [5. Null and Unit](spec/05-null-and-unit.md) | `Null`, `@Nullable`, `Unit`, the `void` form |
| [6. Numbers and text](spec/06-numbers.md) | Numeric classes, literals, `Rational`, `String`, arrays |
| [7. Generics](spec/07-generics.md) | Type parameters, bounds, `in` and `out`, wildcards, inference |
| [8. Annotations](spec/08-annotations.md) | Qualifiers, narrowing, tags, `@Override`, declaring and reading annotations |
| [9. Execution](spec/09-execution.md) | Evaluation, exceptions, construction, initialization, startup |
| [10. Omissions](spec/10-deferred.md) | Absent features, later features, and bounded implementation choices |
| [11. Syntax](spec/11-syntax.md) | Lexical and syntactic grammar |
| [12. Flow](spec/12-flow.md) | Constants, reachability, definite assignment, narrowing |
| [13. Concurrency](spec/13-concurrency.md) | Threads, shared memory, locks, atomic cells, scopes, cancellation |

## Prelude names

The specification fixes these names. They are ordinary prelude classes and methods.

| Name | Role |
| --- | --- |
| `UInt8`, `UInt16`, `UInt32`, `UInt64` | Unsigned fixed widths |
| `wrappingPlus`, `wrappingMinus`, `wrappingTimes` | Wrapping integer arithmetic |
| `floorDiv`, `mod`, `truncatingDiv`, `truncatingRem` | Integer division and remainder, by name |
| `shiftLeft`, `shiftRight`, `complement`, `xor` | Shifts and bitwise operations |
| `Float32.nearest`, `Float64.nearest` | Rounding onto a float |
| `totalOrder` | IEEE 754 total order |
| `ArithmeticException`, `ClassCastException`, `IndexOutOfBoundsException`, `ClassInitializationException` | Failures named by the numeric, cast, index, and initialization rules |
| `IllegalArgumentException`, `IllegalStateException` | A rejected argument, or an object used in the wrong state |
| `IllegalAccessException` | A mirror used on a member that is not public |
| `Annotation`, `Site`, `Field`, `Method`, `Parameter` | Annotation values, their targets, and the mirrors that read them |
| `AssertionException`, `OutOfMemoryException`, `CancellationException` | A failed assertion, a failed allocation, a cancelled task |
