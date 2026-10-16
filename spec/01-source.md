# 1. Source

## 1.1 Status words

"Must" and "is rejected" are requirements on every implementation. "May" is permission. <!-- r[source.status.rejected-requirements-implementation] -->
A program that this specification rejects must be diagnosed. <!-- r[source.status.program-specification-rejects] -->
A program whose behavior this specification does not define may be rejected, and an implementation must not give it a behavior that contradicts a defined rule. <!-- r[source.status.program-behavior-specification] -->

An example is normative when it says that a program is legal or rejected. <!-- r[source.status.example-normative-says] -->
Other examples illustrate a rule. <!-- r[source.status.examples-illustrate-rule] -->
Where an example lists the members of a prelude class, it shows their signatures and leaves out their bodies. <!-- r[source.status.example-lists-members] -->

[Chapter 11](11-syntax.md) gives the grammar. <!-- r[source.status.gives-grammar] -->
The other chapters describe syntax in prose and by example. <!-- r[source.status.describe-syntax-prose] -->

## 1.2 Source text

A compilation unit is a UTF-8 file whose name ends in `.cleat`. <!-- r[source.text.compilation-unit-utf] -->
A byte sequence that is not well-formed UTF-8 is rejected. <!-- r[source.text.byte-sequence-not] -->

Whitespace is U+0009, U+000A, U+000B, U+000C, U+000D and U+0020. <!-- r[source.text.whitespace] -->
Whitespace separates tokens and is otherwise ignored. <!-- r[source.text.whitespace-separates-tokens] -->
A line break does not end a statement. <!-- r[source.text.line-break-not] -->

A comment is `//` through the end of the line, or `/*` through the next `*/`. <!-- r[source.text.comment-through-end] -->
A block comment does not nest, and an unclosed one is rejected. <!-- r[source.text.block-comment-not] -->
A comment separates tokens as whitespace does. <!-- r[source.text.comment-separates-tokens] -->

## 1.3 Tokens

The longest token that can be read at each point is the one read, with one exception: `>` is always read as a single character where a type argument list is being closed, so `List<List<Int>>` needs no space. <!-- r[source.tokens.longest-token-read] -->

**Identifiers.** An identifier is a character in Unicode category L, or `_` or `$`, followed by any number of characters in category L or N, or `_` or `$`. <!-- r[source.tokens.identifiers-identifier-character] -->

**Keywords.** These words are keywords. <!-- r[source.tokens.keywords-words] -->
A keyword is not an identifier. <!-- r[source.tokens.keyword-not-identifier] -->

```
abstract assert break case catch class continue else enum extends final
finally for if implements import instanceof interface new package private
protected public return static super switch this throw try using var void
when while
```

These words are keywords only in one position each, and identifiers everywhere else. <!-- r[source.tokens.words-keywords-only] -->
A method may be named `open`, and a field `value`. <!-- r[source.tokens.method-named-open] -->

| Word | Is a keyword |
| --- | --- |
| `open`, `sealed`, `foreign` | Among the modifiers of a type, a method or a constructor <!-- r[source.tokens.open-sealed-foreign] --> |
| `value` | Directly before `class` <!-- r[source.tokens.value-directly-before] --> |
| `annotation` | Where a type declaration may begin, after its modifiers <!-- r[source.tokens.annotation-type-declaration] --> |
| `permits` | After the header of a class or an interface, before its `{` <!-- r[source.tokens.permits-after-header] --> |
| `in`, `out` | Directly after `<` or `,` in the type parameter list of a class or an interface <!-- r[source.tokens.in-out-directly] --> |
| `default` | Where a switch arm may begin <!-- r[source.tokens.default-switch-arm] --> |
| `only` | Directly after an audience keyword, before `(` <!-- r[source.tokens.only-directly-after] --> |

These words are reserved. <!-- r[source.tokens.words-reserved] -->
A reserved word is rejected wherever an identifier would be accepted. <!-- r[source.tokens.reserved-word-rejected] -->

```
boolean byte char double float inline int long short
```

**Literals.** `true`, `false` and `null` are literals and not identifiers. <!-- r[source.tokens.literals-true-false] -->

An integer literal is a decimal numeral, a hexadecimal numeral introduced by `0x` or `0X`, or a binary numeral introduced by `0b` or `0B`. <!-- r[source.tokens.integer-literal-decimal] -->
A decimal numeral has no leading `0` unless it is exactly `0`. <!-- r[source.tokens.decimal-numeral-no] -->
A decimal literal is a decimal numeral, `.`, a run of decimal digits, and an optional exponent, or a decimal numeral and an exponent. <!-- r[source.tokens.decimal-literal-numeral] -->
An exponent is `e` or `E`, an optional `+` or `-`, and a run of decimal digits. <!-- r[source.tokens.exponent-optional-run] -->
`_` may be written between two digits. <!-- r[source.tokens.written-between-digits] -->
A numeric literal has no suffix: `1L`, `1.0f` and `07` are rejected. <!-- r[source.tokens.numeric-literal-no] -->
Each literal denotes its exact mathematical value, and [chapter 6](06-numbers.md) says which class it takes. <!-- r[source.tokens.literal-denotes-exact] -->

A character literal is one scalar between `'` marks, written directly or as an escape. <!-- r[source.tokens.character-literal-scalar] -->
A string literal is any number of scalars between `"` marks, on one line. <!-- r[source.tokens.string-literal-number] -->
The escapes are: <!-- r[source.tokens.escapes] -->

| Escape | Scalar |
| --- | --- |
| `\n`, `\t`, `\r` | U+000A, U+0009, U+000D <!-- r[source.tokens.newline-tab-return] --> |
| `\\`, `\"`, `\'` | The character after the backslash <!-- r[source.tokens.character-after-backslash] --> |
| `\u` and four hexadecimal digits | The scalar with that number <!-- r[source.tokens.four-hexadecimal-digits] --> |
| `\u{` one to six hexadecimal digits `}` | The scalar with that number <!-- r[source.tokens.six-hexadecimal-digits] --> |

An escape that names a surrogate, or a number above 10FFFF, is rejected. <!-- r[source.tokens.escape-names-surrogate] -->
Any other character after a backslash is rejected, except the `{` that begins an embedded expression. <!-- r[source.tokens.character-after-backslash-rejected] -->

A text block is a string literal that spans lines. <!-- r[source.tokens.text-block-string] -->
It opens with `"""` and the end of that line, and it closes with `"""`. <!-- r[source.tokens.opens-end-line] -->
The whitespace before the closing `"""` on its line is the indent. <!-- r[source.tokens.whitespace-before-closing] -->
Every content line begins with the indent, which is removed, and a line that does not is rejected. <!-- r[source.tokens.content-line-begins] -->
Each line break in the content is U+000A, and the break before the closing line is not part of the string. <!-- r[source.tokens.line-break-content] -->
Escapes apply after the indent is removed. <!-- r[source.tokens.escapes-apply-after] -->

```java
String help = """
    usage: calc <expression>
      prints the exact value
    """;                        // "usage: calc <expression>\n  prints the exact value"
```

**Embedded expressions.** In a string literal or a text block, `\{` begins an embedded expression, and the matching `}` ends it. <!-- r[source.tokens.embedded-expression-begins] -->
The expression is read as tokens of its own, so it may hold braces and string literals of its own. <!-- r[source.tokens.embedded-read-tokens] -->
It ends on the line where it begins, and it is not empty. <!-- r[source.tokens.embedded-ends-line] -->
[Section 6.9](06-numbers.md#69-char-and-string) says what such a literal is. <!-- r[source.tokens.embedded-section-says] -->

```java
String line = "\{name} has \{count} items";
```

**Separators and operators.** The separators are `( ) { } [ ] ; , . @ :: ... ->`. <!-- r[source.tokens.separators-operators-reference] -->
The operators are those of [chapter 4](04-methods.md), with `?`, `:`, `??`, `=` and the compound assignments. <!-- r[source.tokens.operators-coalesce-compound] -->
`>>>` is rejected. <!-- r[source.tokens.unsigned-shift-rejected] -->

## 1.4 Compilation units

A compilation unit has an optional package declaration, then its imports, then its type declarations. <!-- r[source.units.compilation-unit-optional] -->
A type declaration declares a class, an interface, an enum or an annotation. <!-- r[source.units.type-declaration-declares] -->

`package a.b;` names the package of every type the file declares. <!-- r[source.units.package-names-type] -->
A file with no package declaration is in the unnamed package, whose types cannot be named from another package. <!-- r[source.units.file-no-package] -->

A file declares at most one `public` type, and its name is then the file's name without `.cleat`. <!-- r[source.units.file-declares-most] -->
A file may declare no public type. <!-- r[source.units.file-declare-no] -->
A type is not declared inside another type, inside a method, or without a name. <!-- r[source.units.type-not-declared] -->

**The prelude.** The package `cleat` is the prelude. <!-- r[source.units.prelude-package-cleat] -->
Its public types are in scope in every compilation unit without an import. <!-- r[source.units.public-types-scope] -->
A program does not declare a package named `cleat` or one whose name begins with `cleat.`. <!-- r[source.units.program-not-declare] -->

This specification defines the prelude types that the language depends on. <!-- r[source.units.specification-defines-prelude] -->
The prelude declares further classes in ordinary source, among them `List`, `Map`, `StringBuilder`, `Console` and `File`, and that source is their definition. <!-- r[source.units.prelude-declares-further] -->

## 1.5 Packages

A package is a namespace for types and for other packages. <!-- r[source.packages.package-namespace-types] -->
It is not an object and has no members of its own. <!-- r[source.packages.not-object-no] -->
`a.b.C` names the type `C` of the package `a.b`. <!-- r[source.packages.names-type-package] -->

A package's name says where it sits: `a.b` is inside `a` whether or not any file declares `package a;`. <!-- r[source.packages.package-name-says] -->
Being inside another package gives no access to it. <!-- r[source.packages.inside-package-gives] -->
[Chapter 3](03-visibility.md) treats every package separately. <!-- r[source.packages.treats-package-separately] -->

## 1.6 Imports

`import a.b.C;` lets the file write `C` for the type `a.b.C`. <!-- r[source.imports.import-lets-file] -->
`import a.b.*;` does the same for every public type of the package `a.b`. <!-- r[source.imports.import-public-type] -->
It does not import the packages inside `a.b`. <!-- r[source.imports.not-import-packages] -->
An import of a type that the file may not name under chapter 3 is rejected. <!-- r[source.imports.import-type-file] -->

A simple name used as a type is looked up in this order: <!-- r[source.imports.simple-name-used] -->

1. a type parameter in scope; <!-- r[source.imports.type-parameter-scope] -->
2. a type declared in the same file; <!-- r[source.imports.type-declared-file] -->
3. a type imported by name; <!-- r[source.imports.type-imported-name] -->
4. a type of the same package; <!-- r[source.imports.type-package] -->
5. a type imported by `*`; <!-- r[source.imports.type-imported] -->
6. a type of the prelude. <!-- r[source.imports.type-prelude] -->

When two `*` imports provide the name at step 5, a use of the simple name is rejected, and the program writes the qualified name. <!-- r[source.imports.imports-provide-name] -->
Two imports by name of the same simple name are rejected. <!-- r[source.imports.imports-name-simple] -->

## 1.7 Names of members

Two fields of one type do not share a name. <!-- r[source.member-names.fields-type-not] -->
A field and a method may. <!-- r[source.member-names.field-method] -->
Two methods may share a name when [chapter 4](04-methods.md#44-overloading) can tell them apart. <!-- r[source.member-names.methods-share-name] -->
An enum constant is a field. <!-- r[source.member-names.enum-constant-field] -->

A type name, a member name and a variable name are any identifiers. <!-- r[source.member-names.type-name-member] -->
The prelude begins type names with a capital letter, and the language does not require it. <!-- r[source.member-names.prelude-begins-type] -->

## 1.8 Scope

| Declared | In scope |
| --- | --- |
| A parameter of a method, a constructor or a lambda | Throughout the body <!-- r[source.scope.parameter-method-constructor] --> |
| A local | From the end of its declaration to the end of the innermost block around it <!-- r[source.scope.local-end-declaration] --> |
| A local declared in the first clause of a `for` | In the condition, the update and the body <!-- r[source.scope.local-declared-first] --> |
| The variable of a `for` over a collection | In the body <!-- r[source.scope.variable-for-over] --> |
| A `catch` parameter | In the `catch` block <!-- r[source.scope.catch-parameter-block] --> |
| A `using` binding | In the later bindings and in the block <!-- r[source.scope.using-binding-later] --> |
| The name in a type arm of a `switch` | In that arm's body <!-- r[source.scope.name-type-arm] --> |
| A label | In the statement it labels <!-- r[source.scope.label-statement-labels] --> |
| A type parameter | As [chapter 7](07-generics.md#71-type-parameters) says <!-- r[source.scope.type-parameter-says] --> |

A variable does not hide another variable of the same method: a local, a parameter, a lambda parameter, or any other name in this table that is in scope. <!-- r[source.scope.variable-not-hide] -->
A label does not hide another label. <!-- r[source.scope.label-not-hide] -->
A variable may have the name of a field, and the field is then written `this.name`, or `Type.name` when it is static. <!-- r[source.scope.variable-name-field] -->

A field initializer may name a field declared earlier in the type. <!-- r[source.scope.field-initializer-name] -->
It does not name the field it initializes or one declared later. <!-- r[source.scope.not-name-field] -->

**Simple names in expressions.** A simple name is looked up as a variable in scope, then as a field of the enclosing type, including the fields it inherits, and then as a type by [section 1.6](#16-imports). <!-- r[source.scope.simple-names-expressions] -->

**Qualified names.** `a.b` is resolved from the left. <!-- r[source.scope.qualified-names-resolved] -->
When `a` is a variable or a field, `b` is a member of its value. <!-- r[source.scope.variable-field-member] -->
When `a` is a type, `b` is a static member. <!-- r[source.scope.type-static-member] -->
When `a` is a package, `b` is a type or a package inside it. <!-- r[source.scope.package-type-inside] -->
A name that could be either a variable or a type is the variable, and one that could be either a type or a package is the type. <!-- r[source.scope.name-variable-type] -->
