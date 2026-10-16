# 10. Omissions

Chapters 1 through 9 and 11 through 13 are the language. <!-- r[omissions.intro.through-language] -->
This chapter lists what they leave out. <!-- r[omissions.intro.lists-leave-out] -->
An implementation must not give an omitted feature a meaning and present it as Cleat. <!-- r[omissions.intro.implementation-not-give] -->

## 10.1 Not in the language

These are absent by design. <!-- r[omissions.absent.absent-design] -->

- **Primitive types and boxing.** Every value is an instance of a class ([chapter 2](02-objects.md)). <!-- r[omissions.absent.primitive-types-boxing] -->
- **Raw types and erasure.** A generic type is always written with its arguments, and they exist at run time ([chapter 7](07-generics.md)). <!-- r[omissions.absent.raw-types-erasure] -->
- **Checked exceptions.** A method does not declare what it raises ([chapter 9](09-execution.md#92-exceptions)). <!-- r[omissions.absent.checked-exceptions-method] -->
- **Types inside types.** There are no nested, inner, local or anonymous classes. <!-- r[omissions.absent.types-inside-no] -->
  A lambda covers the common use of an anonymous class. <!-- r[omissions.absent.lambda-covers-common] -->
- **Monitors.** There is no `synchronized`, no `wait` or `notify` on `Object`, and no `volatile`. <!-- r[omissions.absent.monitors-no-synchronized] -->
  [Chapter 13](13-concurrency.md) has `Lock`, `Condition` and atomic cells. <!-- r[omissions.absent.lock-condition-atomic] -->
- **Implicit lossy conversion.** No value is rounded, truncated, or converted to a `String` unless the program calls a method that does it ([chapter 4](04-methods.md#45-assignability)). <!-- r[omissions.absent.implicit-lossy-conversion] -->
- **Integer `/`.** Integer division is written by name ([chapter 6](06-numbers.md#64-integer-arithmetic)). <!-- r[omissions.absent.integer-division-written] -->
- **`>>>`, unary `+` and a power operator.** <!-- r[omissions.absent.unsigned-shift-unary-power] -->
- **Switch fallthrough.** <!-- r[omissions.absent.switch-fallthrough] -->
- **Octal literals and literal suffixes.** <!-- r[omissions.absent.octal-literals-literal] -->
- **Finalizers, weak references, `clone`, and a serialization protocol built into the language.** <!-- r[omissions.absent.finalizers-weak-references] -->
- **A 16-bit character.** A `Char` is one Unicode scalar ([chapter 6](06-numbers.md#69-char-and-string)). <!-- r[omissions.absent.bit-character-char] -->
- **`default` on an interface method.** A method with a body is a default ([chapter 2](02-objects.md#26-interfaces)). <!-- r[omissions.absent.default-interface-method] -->
- **Code that runs at compile time.** No annotation adds a member or rewrites a body ([chapter 8](08-annotations.md)). <!-- r[omissions.absent.code-runs-compile] -->
- **Reflection over members that carry no annotation.** A program does not list the members of a class, call a method by name, or construct an object from a class object ([chapter 8](08-annotations.md#89-reading-annotations)). <!-- r[omissions.absent.reflection-over-members] -->
- **Loading code at run time.** The compiler sees the whole program ([chapter 9](09-execution.md#99-the-program)). <!-- r[omissions.absent.loading-code-run] -->

## 10.2 Not in this version

These may be added later. <!-- r[omissions.later.added-later] -->
Nothing in this version depends on them, and a program must not assume any particular form for them. <!-- r[omissions.later.nothing-version-depends] -->

- Loading classes at run time, and compiling while the program runs. <!-- r[omissions.later.loading-classes-run] -->
- `inline` methods whose lambda arguments can `return` from the caller. <!-- r[omissions.later.inline-methods-lambda] -->
  `inline` is reserved for them. <!-- r[omissions.later.inline-reserved] -->
- A null-safe member access such as `?.`, and string interpolation. <!-- r[omissions.later.null-safe-member] -->
- Properties, or separate audiences for reading and assigning a field. <!-- r[omissions.later.properties-separate-audiences] -->
- Mirrors that describe the types of fields and parameters, construct objects, or enumerate the types of a program. <!-- r[omissions.later.mirrors-describe-types] -->
- Repeatable annotations, and declaration annotations on locals and type parameters. <!-- r[omissions.later.repeatable-annotations-declaration] -->
  A qualifier among the modifiers of a local, and a tag that a type parameter requires, are in the language ([chapter 8](08-annotations.md)). <!-- r[omissions.later.qualifier-among-modifiers] -->
- Pinning an object for longer than one foreign call, and reading memory through a `Pointer`. <!-- r[omissions.later.pinning-object-longer] -->
- Callbacks from C into Cleat. <!-- r[omissions.later.callbacks-cleat] -->
- Vector, complex and matrix classes. <!-- r[omissions.later.vector-complex-matrix] -->
  They are library classes over `Numeric<T>`. <!-- r[omissions.later.library-classes-over] -->

## 10.3 Left to the implementation

An implementation chooses each of the following. <!-- r[omissions.implementation.implementation-chooses-following] -->
A program may rely on what the chapter cited defines and on nothing more. <!-- r[omissions.implementation.program-rely-cited] -->

- How an instance is stored: in a register, inside another object, or on the heap ([chapter 2](02-objects.md#21-one-population-of-values)). <!-- r[omissions.implementation.how-instance-stored] -->
- How instantiations of a generic type are compiled: with shared code or with a copy for each ([chapter 7](07-generics.md#77-type-arguments-at-run-time)). <!-- r[omissions.implementation.how-instantiations-generic] -->
- How storage is reclaimed, and how much there is ([chapter 9](09-execution.md#98-storage)). <!-- r[omissions.implementation.how-storage-reclaimed] -->
- The characters of the default `toString` of a reference class, and of a lambda ([chapter 2](02-objects.md#25-the-root-class)). <!-- r[omissions.implementation.characters-default-tostring] -->
- The value of a default `hashCode`, and of `hashCode` for `null`. <!-- r[omissions.implementation.value-default-hashcode] -->
  It agrees with `equals` and does not change while the program runs. <!-- r[omissions.implementation.agrees-equals-not] -->
- How a `Rational` is stored ([chapter 6](06-numbers.md#66-rational)). <!-- r[omissions.implementation.how-rational-stored] -->
- What is recorded about where an exception was created ([chapter 9](09-execution.md#92-exceptions)). <!-- r[omissions.implementation.recorded-about-exception] -->
- How threads are scheduled, and how many run at once ([chapter 13](13-concurrency.md)). <!-- r[omissions.implementation.how-threads-scheduled] -->
- How the entry class and the libraries for foreign methods are named when a program is built ([chapter 9](09-execution.md#910-startup-and-termination), [chapter 4](04-methods.md#411-foreign-methods)). <!-- r[omissions.implementation.how-entry-class] -->
- The platform's C calling convention ([chapter 4](04-methods.md#411-foreign-methods)). <!-- r[omissions.implementation.platform-calling-convention] -->
- The limit on the depth of calls. <!-- r[omissions.implementation.limit-depth-calls] -->
  Exceeding it ends the program ([chapter 9](09-execution.md#910-startup-and-termination)). <!-- r[omissions.implementation.exceeding-ends-program] -->
