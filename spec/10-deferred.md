# 10. Omissions

These are closed. An implementation must not invent a meaning for an omission and present it as Cleat. The behavior of chapters 1 through 9 and 11 through 13 is the language.

## 10.1 Not in the language

- There is no power operator. Power is `pow`, as [chapter 6](06-numbers.md) defines.
- There is no `BigInt` class. An integer that does not fit in a machine width is a `Number` with heap digits.
- There is no raw string. Text blocks are [chapter 11](11-syntax.md), and escapes apply.
- There is no `>>>` operator.
- There is no prelude `List`, `Map`, or other collection above `Iterable`, `Iterator`, and arrays. A program that wants one declares it. Examples that write `List` use a name the program may declare.
- There is no reflective invocation of an arbitrary method. `Class` has the methods in [chapter 7](07-generics.md).
- There is no per-object monitor, no `synchronized`, and no checked exception.
- There is no binary compatibility contract and no stable ABI. A new class is added by `Loader.load`, as [section 9.6](09-execution.md#96-compilation) defines. A change to a class that is already loaded is a new process, not a redefinition.
- The only machine shapes are `Int32x4` and `Float64x4`, as [chapter 6](06-numbers.md) defines. No other shape is declared by the prelude, and a program does not invent one by syntax.

## 10.2 Left to the implementation

These choices are bounded. They do not add a language rule.

- The bits of a reference header, provided the collector can find every reference field. Cleat code does not read the header.
- The heap's growth step, shrink fraction, and generation layout. The collector is a precise moving collector with the pin, safepoint, and `OutOfMemoryException` rules of [section 9.6](09-execution.md#96-compilation). A program may observe those rules and must not observe a particular heap size. Every collector that meets them is correct.
- The bytes of a `Loader` image, and which methods the JIT has specialized. A program may observe that `load` either installs classes that obey the type rules or raises `ClassLoadException`, and that an open call does not run under a receiver set a load has already extended. It must not observe a byte layout or a specialization choice. Every loader and JIT that meet [section 9.6](09-execution.md#96-compilation) are correct.
- Which registers the platform C ABI uses. The type and layout mapping is [section 4.11](04-methods.md#411-foreign-signatures).
- The compiler argument that names the entry type. The `main` signatures and the status codes are [chapter 9](09-execution.md).
- The characters of a reference class's default `toString`. A program may observe that the string is stable for the lifetime of that instance. A value which does not override `toString` returns a string determined by its class and by `toString` of its components in declaration order, superclass components first, so equal values have equal results. Storing that value in a reference slot does not change the string. Every implementation that meets those observations is correct. An implementation that omits the declaration order or the superclass components is not correct. The rule is [chapter 2](02-objects.md).
- The bits of `Null.hashCode`. A program may observe that every call returns the same `Int32`, including when the instance is stored in `Object`, and that the code agrees with `equals`. Every implementation that returns one unchanging `Int32` is correct. The rule is [chapter 5](05-null-and-unit.md).
- The stored form of a non-integer `Number`. The value is an exact rational. A program may observe the mathematical value, including that `1 / 2` is one half and that the literal `0.1` is one tenth. It may not observe the stored digits. The integer representation remains the unsigned machine width for a non-negative value and the signed machine width for a negative value, as [chapter 6](06-numbers.md) defines, then heap digits. Every exact non-integer representation that preserves those values is correct. The rule is [chapter 6](06-numbers.md).

## 10.3 Where the former open list went

Box identity is [chapter 2](02-objects.md). Two boxes of equal values are not required to be identical, and they are not required to be distinct. Concurrency, structured `Scope`, and atomic cells are [chapter 13](13-concurrency.md). The grammar is [chapter 11](11-syntax.md). Constants, reachability, and definite assignment are [chapter 12](12-flow.md). Foreign layout, `Pointer`, `Span`, `Pin`, and `ForeignBuffer` are chapter 4. The `String` methods are chapter 6. The expander builder is [chapter 8](08-annotations.md). `switch`, `enum`, `assert`, `++`, `--`, varargs, labels, the three-clause `for`, and text blocks are chapters 2, 4, and 11.
