# 1. Source

## 1.1 Status words

"Must" and "is rejected" are requirements on every implementation. "May" is permission. A program this specification rejects must be diagnosed. A program whose behavior this specification does not define may be rejected; an implementation must not give that program a behavior that contradicts a defined rule.

Examples are normative when they say a program is legal or rejected. Other examples illustrate a rule.

The grammar uses `=` for definition, `|` for alternative, `[ ]` for an optional term, `{ }` for zero or more, `( )` for grouping, and quotes for terminal characters. A space in a grammar is the concatenation of terms. Words written in prose are syntax categories.

## 1.2 Source text

A compilation unit is a UTF-8 file whose name ends in `.cleat`. A byte sequence that is not UTF-8 is rejected. The Unicode scalar values U+D800 through U+DFFF do not occur in well-formed UTF-8 and are rejected inside literals as well.

Whitespace is U+0009, U+000A, U+000B, U+000C, U+000D, and U+0020. Whitespace separates tokens and is otherwise ignored. A line break does not end a statement.

Comments are `//` through the next U+000A or U+000D, and `/*` through the next `*/`. A block comment does not nest. An unclosed block comment is rejected.

## 1.3 Tokens

An identifier is a character in Unicode category L, or `_` or `$`, followed by zero or more characters in category L or N, or `_` or `$`. The compiler may emit names containing `$`. A user identifier may contain `$`.

The following words are keywords. A keyword is not an identifier.

```
abstract annotation assert break case catch class continue else enum
extends final finally for foreign if implements import in inline instanceof
interface new null only open out package permits private protected public
return sealed static super switch this throw try using value var void while
```

`in` and `out` are keywords only where [chapter 7](07-generics.md) writes them, directly after `<` or `,` in a type-parameter list. In every other position they are identifiers. `default` is a keyword only as a switch arm. In every other position it is an identifier. It is not written on an interface method.

The following words are reserved. A reserved word is rejected wherever an identifier or a type name would be accepted.

```
boolean byte char double float int long short when
```

`true` and `false` are literals, not identifiers.

An integer literal is a decimal numeral, a hexadecimal numeral introduced by `0x` or `0X`, or a binary numeral introduced by `0b` or `0B`. A decimal integer literal has no leading `0` unless the literal is exactly `0`. A rational literal is a decimal integer, `.`, a decimal integer, and an optional exponent. An exponent is `e` or `E`, an optional `+` or `-`, and a decimal integer. Digits of either literal may contain `_` between two digits. A numeric literal has no type suffix. `1L`, `1.0f`, `0.1d`, a leading-zero decimal such as `07`, and a hexadecimal floating literal are rejected.

A rational literal denotes the exact decimal value of its digits. It does not denote a binary floating-point approximation. The integer value of an integer literal is exact.

A string literal is `"` … `"`. A character literal is `'` … `'`. A text block is the form in [chapter 11](11-syntax.md). Escapes in each are `\n`, `\t`, `\r`, `\\`, `\"`, `\'`, `\u` followed by exactly four hexadecimal digits, and `\u{` followed by one to six hexadecimal digits and `}`. Each escape denotes one Unicode scalar value. A scalar outside 0..10FFFF, or a surrogate scalar, is rejected. A character literal holds exactly one scalar. A string literal holds the sequence of scalars written in it. There is no raw-string form.

`true`, `false`, and `null` are literals.

Separators and operator characters are the set in [chapter 11](11-syntax.md): `( ) { } [ ] ; , . @ ::` and the operator spellings in [chapter 4](04-methods.md), including `++` and `--`. `>>>` is one token and is rejected. The lexical grammar, including maximal match, is chapter 11.

## 1.4 Compilation units

```
unit        = [package-decl] {import-decl} {type-decl}
package-decl = "package" qualified-name ";"
import-decl  = "import" qualified-name ["." "*"] ";"
qualified-name = identifier {"." identifier}
```

A package declaration names the package that owns every type declared in the file. A file with no package declaration is in the unnamed package. The unnamed package has no children and cannot be named from another package.

A file contains at most one type declared `public`. If it contains a public type, the file name, without `.cleat`, is that type's simple name. Further types in the file are not public. A file may contain no public type. Nested, inner, local, and anonymous type declarations are rejected.

The prelude package `cleat` is in scope in every compilation unit without an import. Every prelude type is public, and every prelude method this specification writes is public unless the method's own chapter gives a different audience. A user package named `cleat` is rejected. Prelude type names may be shadowed by an explicit import or by a type in the same file. A simple name that would denote both a shadowed prelude type and another visible type is rejected; the program writes the qualified name.

## 1.5 Packages

A package is a namespace object. Its slots are the class objects of its types and its child packages. A package is not a class. No method is declared on a package, and a package is not the receiver of a send. The qualified name `a.b.C` selects those slots; evaluation of the name does not send a message.

Package membership is lexical. The package `a.b` is a child of `a` whether or not a file declares `package a`.

## 1.6 Imports

An import ending in a simple name binds that name to one class object. An import ending in `*` binds the public class objects of that package, and does not bind child packages or types that are not public. A single-type import of a type that is not accessible is rejected.

A simple name used as a type is resolved in this order: a type declared in the same file, a single-type import, a type in the same package, a star import, the prelude. If two star imports bind the same simple name and the program uses that simple name, the program is rejected. The qualified name remains available.

An import binds a class object. It does not bind a package method, because a package has none.

## 1.7 Names and declarations

A type name is a class, an interface, or an annotation. Type names use the spelling of class names: an initial capital letter is the convention of the prelude and is not enforced on user types.

A member name introduced in a type must be unique among the methods of one signature, the fields, and the nested names of that type. There are no nested types, so the last set is empty. A field and a method may share a simple name. Two fields may not. Two methods may share a name when [overload resolution](04-methods.md) can separate them.

Names are not values. An expression that names a local, a parameter, or a field denotes the object bound to that name. There is no expression whose value is the name itself.

## 1.8 Scope

A parameter is in scope throughout its method or constructor. A local is in scope from the end of its declaration to the end of the innermost block that contains the declaration. A for-init local is in scope in the condition, the update, and the body. A catch parameter is in scope in its catch block. A using-binding is in scope in the using block. A label is in scope in the statement it labels. Labels in one method are unique.

A local, parameter, or label may not hide another local, parameter, or label of the same method. A local may hide a field. The field is named `this.name` or `Type.name` when it is static. A field initializer may name fields declared earlier in the same type and may not name the field being declared or a field declared later. A parameter may hide a field.

A simple name in a type body, and outside a method, is a type member. Inside a method it is resolved as a local or parameter, then as a member of the enclosing type, then as an inherited member, then as a type by [section 1.6](#16-imports). A name with a `.` is resolved from the left. The left is a type, a package, or a value. A package prefix selects a child package or a type and is not a send. A type prefix selects a static member. A value prefix selects an instance member by the lookup of [chapter 2](02-objects.md).

A name that resolves both as a type in scope and as a package prefix denotes the type when a member or type argument follows, and denotes the package when only further package segments follow. An ambiguous simple name from two star imports is rejected, as [section 1.6](#16-imports) says.
