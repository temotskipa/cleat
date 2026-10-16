# 11. Syntax

This chapter is the grammar. <!-- r[syntax.intro.grammar] -->
A program that the grammar does not derive is rejected. <!-- r[syntax.intro.program-grammar-not] -->
The other chapters add restrictions to what the grammar derives, and those restrictions stand. <!-- r[syntax.intro.add-restrictions-grammar] -->

The notation uses `=` for a definition, `|` for an alternative, `[ ]` for an optional part, `{ }` for a part repeated zero or more times, `( )` for grouping, and quotes for the characters of a token. <!-- r[syntax.intro.notation-uses-definition] -->
Words outside quotes name other rules. <!-- r[syntax.intro.words-outside-quotes] -->

## 11.1 Tokens

[Chapter 1](01-source.md) describes the tokens in prose. <!-- r[syntax.tokens.describes-tokens-prose] -->
This is their grammar. <!-- r[syntax.tokens.grammar] -->

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

A `letter` is a character in Unicode category L, and a `number` one in category N. <!-- r[syntax.tokens.letter-character-unicode] -->
A `plain-character` is any scalar other than the closing quote, a backslash and a line break. <!-- r[syntax.tokens.plain-character-scalar] -->
A block comment ends at the first `*/`. <!-- r[syntax.tokens.block-comment-ends] -->
An identifier is not a keyword, a reserved word, or one of the three word literals. <!-- r[syntax.tokens.identifier-not-keyword] -->

The longest token is read, except that `>` is read alone where it closes a type argument list. <!-- r[syntax.tokens.longest-token-read] -->

## 11.2 Types and annotations

```
type            = {annotation} named-type ["?"] {{annotation} "[" "]" ["?"]}
test-type       = {annotation} named-type {{annotation} "[" "]"}
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

An annotation before the named type qualifies it. <!-- r[syntax.types.annotation-before-named] -->
An annotation before a pair of brackets qualifies that array. <!-- r[syntax.types.annotation-before-pair] -->
A `?` after the named type makes it nullable, and a `?` after a pair of brackets makes that array nullable. <!-- r[syntax.types.question-after-named] -->
With several pairs, the first pair is the outermost array: `Int[][]?` is an array whose elements are arrays of `Int` or `null`. <!-- r[syntax.types.several-pairs-first] -->

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

Each chapter says which modifiers a declaration accepts, and a modifier is written at most once. <!-- r[syntax.declarations.says-modifiers-declaration] -->
A method ends in `;` in place of a block when it is abstract, `foreign` or `@Intrinsic`, or when it is an interface method with no body. <!-- r[syntax.declarations.method-ends-place] -->
A constructor ends in `;` only when it is `@Intrinsic`. <!-- r[syntax.declarations.constructor-ends-only] -->
A constructor's identifier is the name of its class. <!-- r[syntax.declarations.constructor-identifier-name] -->

An annotation at the end of `modifiers` could also be read as the first annotation of the `type` that follows. <!-- r[syntax.declarations.annotation-end-modifiers] -->
For a qualifier the two readings mean the same, as [chapter 8](08-annotations.md#82-declaring-and-writing-an-annotation) says. <!-- r[syntax.declarations.qualifier-readings-mean] -->
A declaration annotation belongs to the declaration. <!-- r[syntax.declarations.declaration-annotation-belongs] -->
Chapter 8 places it before the first modifier, so an annotation that follows a modifier is a qualifier. <!-- r[syntax.declarations.places-before-first] -->

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

An `else` belongs to the nearest `if` that has none. <!-- r[syntax.statements.else-belongs-nearest] -->
A statement that can be read as a local declaration is one. <!-- r[syntax.statements.statement-read-local] -->
An arm that can be read as `case type identifier` is a type arm. <!-- r[syntax.statements.arm-read-case] -->

## 11.6 Expressions

The rules run from the loosest binding to the tightest. <!-- r[syntax.expressions.rules-run-loosest] -->
Assignment, `?:` and `??` group to the right. <!-- r[syntax.expressions.assignment-conditional-coalesce] -->
The other binary operators group to the left. <!-- r[syntax.expressions.binary-operators-group] -->

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
relational      = shift {("<" | ">" | "<=" | ">=") shift | "instanceof" test-type}
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

The grammar derives more than is legal, and these rules settle what it leaves open. <!-- r[syntax.expressions.grammar-derives-more] -->

- **Casts.** `(` type `)` begins a cast only when the token after `)` is not `-`, `++` or `--`. <!-- r[syntax.expressions.casts-type-begins] -->
  `(n) - 1` is a subtraction, and a negated operand is cast as `(Int) (-n)`. <!-- r[syntax.expressions.subtraction-negated-operand] -->
- **Lambdas.** A `(` begins a lambda when the token after its matching `)` is `->`. <!-- r[syntax.expressions.lambdas-begins-lambda] -->
  The head of a switch arm is the exception: the `->` after its constants ends the head, so `case (1) -> x` is an arm whose constant is `(1)`. <!-- r[syntax.expressions.head-switch-arm] -->
- **Type arguments in an expression.** A `<` after a name begins type arguments when the tokens up to the matching `>` are type arguments and the next token is `::`, or `.` followed by `class`. <!-- r[syntax.expressions.type-arguments-expression] -->
  Everywhere else it is the comparison operator. <!-- r[syntax.expressions.everywhere-else-comparison] -->
  The type after `new` and after `instanceof`, and the type arguments of a call, which follow its `.`, are never mistaken for one. <!-- r[syntax.expressions.type-after-new] -->
- **Nullable types.** A statement that begins with a type, `?` and an identifier declares a local: `String? s = null;`. <!-- r[syntax.expressions.nullable-types-statement] -->
  A conditional is never a statement, so the reading takes nothing away. <!-- r[syntax.expressions.conditional-never-statement] -->
- **Names.** `a.b.c` is derived as an identifier and two selectors. <!-- r[syntax.expressions.names-derived-identifier] -->
  [Chapter 1](01-source.md#18-scope) says which parts name a package, a type, a field or a variable. <!-- r[syntax.expressions.says-parts-name] -->
- **Assignment.** The left side of an assignment, and the operand of `++` and `--`, is a variable, a field or an indexed element. <!-- r[syntax.expressions.assignment-left-side] -->
- **Statements.** An `expr` that stands as a statement is one that [chapter 4](04-methods.md#49-statements) allows there. <!-- r[syntax.expressions.statements-expr-stands] -->
- **Negative literals.** A `-` directly before a numeric literal is part of the literal for the rules of [chapter 6](06-numbers.md#63-numeric-literals). <!-- r[syntax.expressions.negative-literals-directly] -->
- **Array creation.** `new T[n]` creates an array with `n` elements, and brackets after `[n]` belong to the element type. <!-- r[syntax.expressions.array-creation-new] -->
  `new T[] { a, b }` lists the elements, and the type before the braces is an array type. <!-- r[syntax.expressions.new-lists-elements] -->
- **Method references.** `x::name` is derived by the `selector` rule or by the `type "::"` rule. <!-- r[syntax.expressions.method-references-reference] -->
  It is a reference through a value when `x` names one, and through a type otherwise. <!-- r[syntax.expressions.reference-through-value] -->
