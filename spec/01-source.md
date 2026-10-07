# 1. Source

## 1.1 Status words

"Must" and "is rejected" are requirements on every implementation. "May" is permission. A program that this specification rejects must be diagnosed. A program whose behavior this specification does not define may be rejected, and an implementation must not give it a behavior that contradicts a defined rule.

An example is normative when it says that a program is legal or rejected. Other examples illustrate a rule. Where an example lists the members of a prelude class, it shows their signatures and leaves out their bodies.

[Chapter 11](11-syntax.md) gives the grammar. The other chapters describe syntax in prose and by example.

## 1.2 Source text

A compilation unit is a UTF-8 file whose name ends in `.cleat`. A byte sequence that is not well-formed UTF-8 is rejected.

Whitespace is U+0009, U+000A, U+000B, U+000C, U+000D and U+0020. Whitespace separates tokens and is otherwise ignored. A line break does not end a statement.

A comment is `//` through the end of the line, or `/*` through the next `*/`. A block comment does not nest, and an unclosed one is rejected. A comment separates tokens as whitespace does.

## 1.3 Tokens

The longest token that can be read at each point is the one read, with one exception: `>` is always read as a single character where a type argument list is being closed, so `List<List<Int>>` needs no space.

**Identifiers.** An identifier is a character in Unicode category L, or `_` or `$`, followed by any number of characters in category L or N, or `_` or `$`.

**Keywords.** These words are keywords. A keyword is not an identifier.

```
abstract assert break case catch class continue else enum extends final
finally for if implements import instanceof interface new package private
protected public return static super switch this throw try using var void
while
```

These words are keywords only in one position each, and identifiers everywhere else. A method may be named `open`, and a field `value`.

| Word | Is a keyword |
| --- | --- |
| `open`, `sealed`, `foreign` | Among the modifiers of a type, a method or a constructor |
| `value` | Directly before `class` |
| `annotation` | Where a type declaration may begin, after its modifiers |
| `permits` | After the header of a class or an interface, before its `{` |
| `in`, `out` | Directly after `<` or `,` in the type parameter list of a class or an interface |
| `default` | Where a switch arm may begin |
| `only` | Directly after an audience keyword, before `(` |

These words are reserved. A reserved word is rejected wherever an identifier would be accepted.

```
boolean byte char double float inline int long short when
```

**Literals.** `true`, `false` and `null` are literals and not identifiers.

An integer literal is a decimal numeral, a hexadecimal numeral introduced by `0x` or `0X`, or a binary numeral introduced by `0b` or `0B`. A decimal numeral has no leading `0` unless it is exactly `0`. A decimal literal is a decimal numeral, `.`, a run of decimal digits, and an optional exponent, or a decimal numeral and an exponent. An exponent is `e` or `E`, an optional `+` or `-`, and a run of decimal digits. `_` may be written between two digits. A numeric literal has no suffix: `1L`, `1.0f` and `07` are rejected. Each literal denotes its exact mathematical value, and [chapter 6](06-numbers.md) says which class it takes.

A character literal is one scalar between `'` marks, written directly or as an escape. A string literal is any number of scalars between `"` marks, on one line. The escapes are:

| Escape | Scalar |
| --- | --- |
| `\n`, `\t`, `\r` | U+000A, U+0009, U+000D |
| `\\`, `\"`, `\'` | The character after the backslash |
| `\u` and four hexadecimal digits | The scalar with that number |
| `\u{` one to six hexadecimal digits `}` | The scalar with that number |

An escape that names a surrogate, or a number above 10FFFF, is rejected. Any other character after a backslash is rejected.

A text block is a string literal that spans lines. It opens with `"""` and the end of that line, and it closes with `"""`. The whitespace before the closing `"""` on its line is the indent. Every content line begins with the indent, which is removed, and a line that does not is rejected. Each line break in the content is U+000A, and the break before the closing line is not part of the string. Escapes apply after the indent is removed.

```java
String help = """
    usage: calc <expression>
      prints the exact value
    """;                        // "usage: calc <expression>\n  prints the exact value"
```

**Separators and operators.** The separators are `( ) { } [ ] ; , . @ :: ... ->`. The operators are those of [chapter 4](04-methods.md), with `?`, `:`, `??`, `=` and the compound assignments. `>>>` is rejected.

## 1.4 Compilation units

A compilation unit has an optional package declaration, then its imports, then its type declarations. A type declaration declares a class, an interface, an enum or an annotation.

`package a.b;` names the package of every type the file declares. A file with no package declaration is in the unnamed package, whose types cannot be named from another package.

A file declares at most one `public` type, and its name is then the file's name without `.cleat`. A file may declare no public type. A type is not declared inside another type, inside a method, or without a name.

**The prelude.** The package `cleat` is the prelude. Its public types are in scope in every compilation unit without an import. A program does not declare a package named `cleat` or one whose name begins with `cleat.`.

This specification defines the prelude types that the language depends on. The prelude declares further classes in ordinary source, among them `List`, `Map`, `StringBuilder`, `Console` and `File`, and that source is their definition.

## 1.5 Packages

A package is a namespace for types and for other packages. It is not an object and has no members of its own. `a.b.C` names the type `C` of the package `a.b`.

A package's name says where it sits: `a.b` is inside `a` whether or not any file declares `package a;`. Being inside another package gives no access to it. [Chapter 3](03-visibility.md) treats every package separately.

## 1.6 Imports

`import a.b.C;` lets the file write `C` for the type `a.b.C`. `import a.b.*;` does the same for every public type of the package `a.b`. It does not import the packages inside `a.b`. An import of a type that the file may not name under chapter 3 is rejected.

A simple name used as a type is looked up in this order:

1. a type parameter in scope;
2. a type declared in the same file;
3. a type imported by name;
4. a type of the same package;
5. a type imported by `*`;
6. a type of the prelude.

When two `*` imports provide the name at step 5, a use of the simple name is rejected, and the program writes the qualified name. Two imports by name of the same simple name are rejected.

## 1.7 Names of members

Two fields of one type do not share a name. A field and a method may. Two methods may share a name when [chapter 4](04-methods.md#44-overloading) can tell them apart. An enum constant is a field.

A type name, a member name and a variable name are any identifiers. The prelude begins type names with a capital letter, and the language does not require it.

## 1.8 Scope

| Declared | In scope |
| --- | --- |
| A parameter of a method, a constructor or a lambda | Throughout the body |
| A local | From the end of its declaration to the end of the innermost block around it |
| A local declared in the first clause of a `for` | In the condition, the update and the body |
| The variable of a `for` over a collection | In the body |
| A `catch` parameter | In the `catch` block |
| A `using` binding | In the later bindings and in the block |
| The name in a type arm of a `switch` | In that arm's body |
| A label | In the statement it labels |
| A type parameter | As [chapter 7](07-generics.md#71-type-parameters) says |

A variable does not hide another variable of the same method: a local, a parameter, a lambda parameter, or any other name in this table that is in scope. A label does not hide another label. A variable may have the name of a field, and the field is then written `this.name`, or `Type.name` when it is static.

A field initializer may name a field declared earlier in the type. It does not name the field it initializes or one declared later.

**Simple names in expressions.** A simple name is looked up as a variable in scope, then as a field of the enclosing type, including the fields it inherits, and then as a type by [section 1.6](#16-imports).

**Qualified names.** `a.b` is resolved from the left. When `a` is a variable or a field, `b` is a member of its value. When `a` is a type, `b` is a static member. When `a` is a package, `b` is a type or a package inside it. A name that could be either a variable or a type is the variable, and one that could be either a type or a package is the type.
