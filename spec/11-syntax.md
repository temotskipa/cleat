# 11. Syntax

This chapter is the grammar. A program that the grammar does not derive is rejected. The other chapters add restrictions to what the grammar derives, and those restrictions stand.

The notation uses `=` for a definition, `|` for an alternative, `[ ]` for an optional part, `{ }` for a part repeated zero or more times, `( )` for grouping, and quotes for the characters of a token. Words outside quotes name other rules.

## 11.1 Tokens

[Chapter 1](01-source.md) describes the tokens in prose. This is their grammar.

```
input           = {whitespace | comment | token}
comment         = "//" {character other than a line break}
                | "/*" {character} "*/"
token           = identifier | keyword | literal | separator | operator

identifier      = ident-start {ident-part}
ident-start     = letter | "_" | "$"
ident-part      = letter | number | "_" | "$"

literal         = integer-literal | decimal-literal | char-literal
                | string-literal | text-block | "true" | "false" | "null"
integer-literal = decimal-numeral
                | ("0x" | "0X") hex-digit {["_"] hex-digit}
                | ("0b" | "0B") binary-digit {["_"] binary-digit}
decimal-numeral = "0" | nonzero-digit {["_"] digit}
decimal-literal = decimal-numeral "." digits [exponent]
                | decimal-numeral exponent
exponent        = ("e" | "E") ["+" | "-"] digits
digits          = digit {["_"] digit}

char-literal    = "'" (plain-character | escape) "'"
string-literal  = '"' {plain-character | escape} '"'
text-block      = '"""' {" " | tab} line-break {character | escape} '"""'
escape          = "\n" | "\t" | "\r" | "\\" | '\"' | "\'"
                | "\u" hex-digit hex-digit hex-digit hex-digit
                | "\u{" hex-digit {hex-digit} "}"

separator       = "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | "@"
                | "::" | "..." | "->"
operator        = "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "~" | "!"
                | "<<" | ">>" | "<" | ">" | "<=" | ">=" | "==" | "!="
                | "&&" | "||" | "??" | "?" | ":" | "++" | "--" | assign-op
assign-op       = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^="
                | "<<=" | ">>="
```

A `letter` is a character in Unicode category L, and a `number` one in category N. A `plain-character` is any scalar other than the closing quote, a backslash and a line break. A block comment ends at the first `*/`. An identifier is not a keyword, a reserved word, or one of the three word literals.

The longest token is read, except that `>` is read alone where it closes a type argument list.

## 11.2 Types and annotations

```
type            = {annotation} named-type {{annotation} "[" "]"}
named-type      = qualified-name [type-args]
qualified-name  = identifier {"." identifier}
type-args       = "<" type-arg {"," type-arg} ">"
type-arg        = type | "?" [("extends" | "super") type]
type-list       = type {"," type}

type-params     = "<" type-param {"," type-param} ">"
type-param      = ["in" | "out"] {annotation} identifier
                  ["extends" type {"&" type}]

annotation      = "@" qualified-name ["(" annotation-args ")"]
annotation-args = annotation-value {"," annotation-value}
                | identifier "=" annotation-value
                  {"," identifier "=" annotation-value}
annotation-value = expr | annotation
                | "{" [annotation-value {"," annotation-value}] "}"
```

An annotation before the named type qualifies it. An annotation before a pair of brackets qualifies that array. With several pairs, the first pair is the outermost array: `Int[] @Nullable []` is an array whose elements are arrays of `Int` or `null`.

## 11.3 Compilation units

```
unit            = [package-decl] {import-decl} {type-decl}
package-decl    = "package" qualified-name ";"
import-decl     = "import" qualified-name ["." "*"] ";"
type-decl       = class-decl | interface-decl | enum-decl | annotation-decl
```

## 11.4 Declarations

```
modifiers       = {annotation | modifier}
modifier        = audience | "static" | "final" | "open" | "abstract"
                | "sealed" | "foreign"
audience        = "private"
                | ("package" | "protected" | "public") ["only" "(" type-list ")"]

class-decl      = modifiers ["value"] "class" identifier [type-params]
                  ["extends" type] ["implements" type-list]
                  ["permits" type-list] "{" {class-member} "}"
class-member    = field | method | constructor | compact-constructor
                | static-init
field           = modifiers type identifier ["=" expr] ";"
method          = modifiers [type-params] result identifier
                  "(" [receiver ["," params] | params] ")" (block | ";")
result          = "void" | type
receiver        = type "this"
params          = param {"," param}
param           = modifiers type ["..."] identifier
constructor     = modifiers identifier "(" [params] ")" (block | ";")
compact-constructor = modifiers identifier block
static-init     = "static" block

interface-decl  = modifiers "interface" identifier [type-params]
                  ["extends" type-list] ["permits" type-list]
                  "{" {field | method} "}"

enum-decl       = modifiers "enum" identifier ["implements" type-list]
                  "{" [enum-constant {"," enum-constant}]
                  [";" {class-member}] "}"
enum-constant   = {annotation} identifier ["(" [args] ")"]

annotation-decl = modifiers "annotation" identifier
                  ["(" element {"," element} ")"] ";"
element         = type identifier ["=" annotation-value]
```

Each chapter says which modifiers a declaration accepts, and a modifier is written at most once. A method ends in `;` in place of a block when it is abstract, `foreign` or `@Intrinsic`, or when it is an interface method with no body. A constructor ends in `;` only when it is `@Intrinsic`. A constructor's identifier is the name of its class.

An annotation at the end of `modifiers` could also be read as the first annotation of the `type` that follows. For a qualifier the two readings mean the same, as [chapter 8](08-annotations.md#82-declaring-and-writing-an-annotation) says. A declaration annotation belongs to the declaration.

## 11.5 Statements

```
block           = "{" {statement} "}"
statement       = block
                | local ";"
                | expr ";"
                | ";"
                | "if" "(" expr ")" statement ["else" statement]
                | "while" "(" expr ")" statement
                | "for" "(" [local | expr-list] ";" [expr] ";" [expr-list] ")"
                  statement
                | "for" "(" modifiers (type | "var") identifier ":" expr ")"
                  statement
                | "switch" "(" expr ")" "{" {arm-head "->" statement} "}"
                | "try" block {catch-clause} ["finally" block]
                | "using" "(" resource {"," resource} ")" block
                | "return" [expr] ";"
                | "break" [identifier] ";"
                | "continue" [identifier] ";"
                | "throw" expr ";"
                | "assert" expr [":" expr] ";"
                | identifier ":" statement
                | ("super" | "this") "(" [args] ")" ";"

local           = modifiers type identifier ["=" expr]
                | modifiers "var" identifier "=" expr
expr-list       = expr {"," expr}
catch-clause    = "catch" "(" type identifier ")" block
resource        = (type | "var") identifier "=" expr
arm-head        = "case" expr-list
                | "case" type identifier
                | "default"
```

An `else` belongs to the nearest `if` that has none. A statement that can be read as a local declaration is one. An arm that can be read as `case type identifier` is a type arm.

## 11.6 Expressions

The rules run from the loosest binding to the tightest. Assignment, `?:` and `??` group to the right. The other binary operators group to the left.

```
expr            = assignment
assignment      = conditional | unary assign-op assignment
conditional     = coalesce ["?" expr ":" conditional]
coalesce        = or-else ["??" coalesce]
or-else         = and-also {"||" and-also}
and-also        = bit-or {"&&" bit-or}
bit-or          = bit-xor {"|" bit-xor}
bit-xor         = bit-and {"^" bit-and}
bit-and         = equality {"&" equality}
equality        = relational {("==" | "!=") relational}
relational      = shift {("<" | ">" | "<=" | ">=") shift | "instanceof" type}
shift           = additive {("<<" | ">>") additive}
additive        = multiplicative {("+" | "-") multiplicative}
multiplicative  = unary {("*" | "/" | "%") unary}
unary           = ("-" | "~" | "!" | "++" | "--") unary
                | "(" type ")" unary
                | postfix
postfix         = primary {selector} ["++" | "--"]
selector        = "." identifier
                | "." [type-args] identifier "(" [args] ")"
                | "[" expr "]"
                | "::" identifier

primary         = literal
                | identifier
                | identifier "(" [args] ")"
                | "this"
                | "(" expr ")"
                | "super" "." identifier "(" [args] ")"
                | "new" named-type "(" [args] ")"
                | "new" {annotation} named-type "[" expr "]" {{annotation} "[" "]"}
                | "new" type "{" [expr-list] "}"
                | type "." "class"
                | type "::" ("new" | identifier)
                | lambda
                | switch-expr

args            = expr {"," expr}
lambda          = "(" [lambda-param {"," lambda-param}] ")" "->" (expr | block)
lambda-param    = [modifiers type] identifier
switch-expr     = "switch" "(" expr ")" "{" {arm-head "->" arm-value} "}"
arm-value       = expr ";" | "throw" expr ";"
```

The grammar derives more than is legal, and these rules settle what it leaves open.

- **Casts.** `(` type `)` begins a cast only when the token after `)` is not `-`, `++` or `--`. `(n) - 1` is a subtraction, and a negated operand is cast as `(Int) (-n)`.
- **Lambdas.** A `(` begins a lambda when the token after its matching `)` is `->`. The head of a switch arm is the exception: the `->` after its constants ends the head, so `case (1) -> x` is an arm whose constant is `(1)`.
- **Type arguments in an expression.** A `<` after a name begins type arguments when the tokens up to the matching `>` are type arguments and the next token is `::`, or `.` followed by `class`. Everywhere else it is the comparison operator. The type after `new` and after `instanceof`, and the type arguments of a call, which follow its `.`, are never mistaken for one.
- **Names.** `a.b.c` is derived as an identifier and two selectors. [Chapter 1](01-source.md#18-scope) says which parts name a package, a type, a field or a variable.
- **Assignment.** The left side of an assignment, and the operand of `++` and `--`, is a variable, a field or an indexed element.
- **Statements.** An `expr` that stands as a statement is one that [chapter 4](04-methods.md#49-statements) allows there.
- **Negative literals.** A `-` directly before a numeric literal is part of the literal for the rules of [chapter 6](06-numbers.md#63-numeric-literals).
- **Array creation.** `new T[n]` creates an array with `n` elements, and brackets after `[n]` belong to the element type. `new T[] { a, b }` lists the elements, and the type before the braces is an array type.
- **Method references.** `x::name` is derived by the `selector` rule or by the `type "::"` rule. It is a reference through a value when `x` names one, and through a type otherwise.
