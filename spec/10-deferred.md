# 10. Omissions

Chapters 1 through 9 and 11 through 13 are the language. This chapter lists what they leave out. An implementation must not give an omitted feature a meaning and present it as Cleat.

## 10.1 Not in the language

These are absent by design.

- **Primitive types and boxing.** Every value is an instance of a class ([chapter 2](02-objects.md)).
- **Raw types and erasure.** A generic type is always written with its arguments, and they exist at run time ([chapter 7](07-generics.md)).
- **Checked exceptions.** A method does not declare what it raises ([chapter 9](09-execution.md#92-exceptions)).
- **Types inside types.** There are no nested, inner, local or anonymous classes. A lambda covers the common use of an anonymous class.
- **Monitors.** There is no `synchronized`, no `wait` or `notify` on `Object`, and no `volatile`. [Chapter 13](13-concurrency.md) has `Lock`, `Condition` and atomic cells.
- **Implicit lossy conversion.** No value is rounded, truncated, or converted to a `String` unless the program calls a method that does it ([chapter 4](04-methods.md#45-assignability)).
- **Integer `/`.** Integer division is written by name ([chapter 6](06-numbers.md#64-integer-arithmetic)).
- **`>>>`, unary `+` and a power operator.**
- **Switch fallthrough.**
- **Octal literals and literal suffixes.**
- **Finalizers, weak references, `clone`, and a serialization protocol built into the language.**
- **A 16-bit character.** A `Char` is one Unicode scalar ([chapter 6](06-numbers.md#69-char-and-string)).
- **`default` on an interface method.** A method with a body is a default ([chapter 2](02-objects.md#26-interfaces)).
- **Code that runs at compile time.** No annotation adds a member or rewrites a body ([chapter 8](08-annotations.md)).
- **Reflection over members that carry no annotation.** A program does not list the members of a class, call a method by name, or construct an object from a class object ([chapter 8](08-annotations.md#89-reading-annotations)).
- **Loading code at run time.** The compiler sees the whole program ([chapter 9](09-execution.md#99-the-program)).

## 10.2 Not in this version

These may be added later. Nothing in this version depends on them, and a program must not assume any particular form for them.

- Loading classes at run time, and compiling while the program runs.
- `inline` methods whose lambda arguments can `return` from the caller. `inline` is reserved for them.
- A null-safe member access such as `?.`, and string interpolation.
- Properties, or separate audiences for reading and assigning a field.
- Mirrors that describe the types of fields and parameters, construct objects, or enumerate the types of a program.
- Repeatable annotations, and annotations on locals and type parameters.
- Pinning an object for longer than one foreign call, and reading memory through a `Pointer`.
- Callbacks from C into Cleat.
- Vector, complex and matrix classes. They are library classes over `Numeric<T>`.

## 10.3 Left to the implementation

An implementation chooses each of the following. A program may rely on what the chapter cited defines and on nothing more.

- How an instance is stored: in a register, inside another object, or on the heap ([chapter 2](02-objects.md#21-one-population-of-values)).
- How instantiations of a generic type are compiled: with shared code or with a copy for each ([chapter 7](07-generics.md#77-type-arguments-at-run-time)).
- How storage is reclaimed, and how much there is ([chapter 9](09-execution.md#98-storage)).
- The characters of the default `toString` of a reference class, and of a lambda ([chapter 2](02-objects.md#25-the-root-class)).
- The value of a default `hashCode`, and of `hashCode` for `null`. It agrees with `equals` and does not change while the program runs.
- How a `Rational` is stored ([chapter 6](06-numbers.md#66-rational)).
- What is recorded about where an exception was created ([chapter 9](09-execution.md#92-exceptions)).
- How threads are scheduled, and how many run at once ([chapter 13](13-concurrency.md)).
- How the entry class and the libraries for foreign methods are named when a program is built ([chapter 9](09-execution.md#910-startup-and-termination), [chapter 4](04-methods.md#411-foreign-methods)).
- The platform's C calling convention ([chapter 4](04-methods.md#411-foreign-methods)).
- The limit on the depth of calls. Exceeding it ends the program ([chapter 9](09-execution.md#910-startup-and-termination)).
