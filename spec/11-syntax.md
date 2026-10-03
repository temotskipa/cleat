# 11. Syntax

This chapter is the syntax. A program the grammar rejects is rejected. Precedence is the order of the expression layers below. It is the same order [chapter 4](04-methods.md) states in prose. Where a chapter gives a semantic restriction the grammar accepts, the restriction stands.

## 11.1 Lexical grammar

```
input        = {input-element}
input-element = whitespace | comment | token
whitespace   = "\t" | "\n" | "\u000B" | "\f" | "\r" | " "
comment      = "//" {no-line-break} | "/*" {not-end-of-comment} "*/"
token        = identifier | keyword | literal | separator | operator
```

A comment does not nest. An unclosed comment is rejected. Whitespace and comments separate tokens and do not occur inside a token, except inside a literal.

The longest token at each point is chosen. `++` and `--` are operators. `>>>` is one token and is rejected. `>>` is one token. Two `>` characters that close nested type arguments are two tokens.

```
identifier   = ident-start {ident-part}
ident-start  = unicode-L | "_" | "$"
ident-part   = unicode-L | unicode-N | "_" | "$"
keyword      = one of the words in [chapter 1](01-source.md)
literal      = integer-literal | rational-literal | string-literal
             | text-block | char-literal | "true" | "false" | "null"
separator    = "(" | ")" | "{" | "}" | "[" | "]" | ";" | "," | "." | "@" | "::"
operator     = "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "~" | "!"
             | "<<" | ">>" | "<" | ">" | "<=" | ">=" | "==" | "!="
             | "&&" | "||" | "?" | ":" | "="
             | "+=" | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^="
             | "<<=" | ">>=" | "++" | "--"
```

`?` begins a wildcard in a type argument. In an expression it is the conditional operator and must be followed, later, by `:`.

An integer literal is a decimal numeral, `0x` or `0X` and hex digits, or `0b` or `0B` and binary digits. A decimal integer literal has no leading `0` unless it is exactly `0`. A rational literal is digits, `.`, digits, and an optional exponent `e` or `E`, an optional sign, and digits. `_` may separate two digits and may not begin or end a digit run. A numeric literal has no type suffix.

A character literal is `'` one scalar `'`. A string literal is `"` {string-char} `"`. A string character is a scalar other than `"` and `\`, a line break, or an escape, or an escape. An ordinary string literal contains no line break.

```
escape = "\n" | "\t" | "\r" | "\\" | "\"" | "\'"
       | "\u" hex hex hex hex
       | "\u{" hex {hex} "}"
```

The braced form has one to six hex digits. The value is one Unicode scalar. A surrogate or a value outside 0..10FFFF is rejected.

A text block is `"""`, whitespace, a line break, content lines, and a closing `"""`. Content starts on the line after the opening break. The closing `"""` may have whitespace before it on its own line. That whitespace is the indent. Every content line must begin with the indent, and that prefix is removed. A content line that does not is rejected. Escapes are applied after the indent is removed. Line breaks in the content are U+000A. There is no raw-string form. A text block is a string literal.

## 11.2 Types

```
type         = {annotation} unann-type
unann-type   = named-type {"[" "]"}
named-type   = qualified-name [type-args]
type-args    = "<" type-arg {"," type-arg} ">"
type-arg     = type | "?" ["extends" type | "super" type]
type-params  = "<" type-param {"," type-param} ">"
type-param   = ["in" | "out"] {annotation} identifier ["extends" bound]
bound        = type {"&" type}
qualified-name = identifier {"." identifier}
annotation   = "@" qualified-name [type-args] ["(" [ann-args] ")"]
ann-args     = value-pair {"," value-pair} | expr {"," expr}
value-pair   = identifier "=" expr
```

`in` and `out` are keywords only in the position `type-param` shows. A type argument list is required on a generic type. An empty argument list is rejected.

## 11.3 Compilation unit

```
unit         = [package-decl] {import-decl} {type-decl}
package-decl = "package" qualified-name ";"
import-decl  = "import" qualified-name ["." "*"] ";"
type-decl    = class-decl | interface-decl | annotation-decl | enum-decl
```

## 11.4 Declarations

```
audience     = "public" | "protected" | "package" | "private" ["(" "this" ")"]
only-clause  = "only" named-type {"," named-type}
class-mod    = audience | "abstract" | "final" | "sealed" | "open" | "value"
class-decl   = {class-mod} "class" identifier [type-params]
               ["extends" type] ["implements" type {"," type}]
               ["permits" type {"," type}] class-body
class-body   = "{" {class-member} "}"
class-member = field | method | constructor | instance-init | static-init
field        = {field-mod} type identifier ["=" expr] ";"
field-mod    = audience | only-clause | "static" | "final"
method       = {method-mod} [type-params] result identifier "(" [params] ")"
               (block | ";")
method-mod   = audience | only-clause | "static" | "final" | "open"
             | "abstract" | "inline" | "foreign"
result       = "void" | type
params       = param {"," param}
param        = {param-mod} type identifier | {param-mod} type "..." identifier
param-mod    = "final" | "inline"
constructor  = {audience} {only-clause} [type-params] identifier "(" [params] ")" block
instance-init = block
static-init  = "static" block
```

A `foreign` or `abstract` method has `;` and no block. Every other method has a block. A varargs parameter is the last parameter. A constructor's identifier is the class name.

```
interface-decl = {iface-mod} "interface" identifier [type-params]
                 ["extends" type {"," type}]
                 ["permits" type {"," type}] interface-body
iface-mod    = audience | "sealed" | "open"
interface-body = "{" {interface-member} "}"
interface-member = method | field
```

An interface field is `public static final` whether or not those words are written. Omitting them does not change that. Writing a contradictory modifier is rejected.

```
annotation-decl = {audience} "annotation" identifier [type-params]
                  ["(" element {"," element} ")"]
                  ["implements" type {"," type}] annotation-body
element      = type identifier ["=" expr] 
enum-decl    = {audience} {only-clause} "enum" identifier
               ["implements" type {"," type}] enum-body
enum-body    = "{" enum-constant {"," enum-constant} [";"] {class-member} "}"
enum-constant = identifier
```

## 11.5 Statements

```
block        = "{" {statement} "}"
statement    = local | statement-expr ";" | if-stmt | while-stmt | for-stmt
             | for-each | return-stmt | break-stmt | continue-stmt
             | throw-stmt | try-stmt | using-stmt | switch-stmt
             | assert-stmt | block | identifier ":" statement | ";"
local        = {annotation} ["final"] type identifier ["=" expr] ";"
             | ["final"] "var" identifier "=" expr ";"
statement-expr = assignment | call | increment
if-stmt      = "if" "(" expr ")" statement ["else" statement]
while-stmt   = "while" "(" expr ")" statement
for-stmt     = "for" "(" [for-init] ";" [expr] ";" [for-update] ")" statement
for-init     = local-no-semi | statement-expr {"," statement-expr}
for-update   = statement-expr {"," statement-expr}
for-each     = "for" "(" ["final"] type identifier ":" expr ")" statement
return-stmt  = "return" [expr] ";"
break-stmt   = "break" [identifier] ";"
continue-stmt = "continue" [identifier] ";"
throw-stmt   = "throw" expr ";"
try-stmt     = "try" block {catch-clause} [finally-clause]
catch-clause = "catch" "(" type identifier ")" block
finally-clause = "finally" block
using-stmt   = "using" "(" resource {"," resource} ")" block
resource     = type identifier "=" expr
switch-stmt  = "switch" "(" expr ")" "{" {switch-arm} "}"
switch-arm   = ("case" constant {"," constant} | "default") "->" statement
assert-stmt  = "assert" expr [":" expr] ";"
assignment   = left "=" expr | left compound-assign expr
left         = name | primary "." identifier | primary "[" expr "]"
increment    = "++" left | "--" left | left "++" | left "--"
```

An empty statement `;` is legal and does nothing. A local in `for-init` is in scope in the condition, the update, and the body.

## 11.6 Expressions

Operators bind in the layers below, tightest first. Every binary layer is left-associative except assignment and the conditional, which are right-associative. Unary operators are right-associative.

```
expr         = assignment-expr
assignment-expr = conditional-expr
             | left ("=" | "+=" | "-=" | "*=" | "/=" | "%="
                    | "&=" | "|=" | "^=" | "<<=" | ">>=") assignment-expr
conditional-expr = or-else-expr ["?" expr ":" conditional-expr]
or-else-expr = and-also-expr {"||" and-also-expr}
and-also-expr = or-expr {"&&" or-expr}
or-expr      = xor-expr {"|" xor-expr}
xor-expr     = and-expr {"^" and-expr}
and-expr     = equality-expr {"&" equality-expr}
equality-expr = relation-expr {("==" | "!=") relation-expr}
relation-expr = shift-expr {("<" | ">" | "<=" | ">=" ) shift-expr}
             | shift-expr "instanceof" type
shift-expr   = add-expr {("<<" | ">>") add-expr}
add-expr     = mul-expr {("+" | "-") mul-expr}
mul-expr     = unary-expr {("*" | "/" | "%") unary-expr}
unary-expr   = ("+" | "-" | "~" | "!" | "++" | "--" | cast | "new") unary-expr
             | postfix
cast         = "(" type ")" unary-expr
postfix      = primary {postfix-op}
postfix-op   = "." identifier [type-args] ["(" [args] ")"]
             | "[" expr "]"
             | "++" | "--"
primary      = literal | name | "this" | "super" "." identifier [type-args] ["(" [args] ")"]
             | "new" named-type ["(" [args] ")"]
             | "new" type array-creator
             | "(" expr ")"
             | lambda
             | method-ref
             | switch-expr
array-creator = "[" expr "]" {"[" "]"} | "[" "]" "{" [expr {"," expr}] "}"
args         = expr {"," expr}
lambda       = "(" [lambda-params] ")" "->" (expr | block)
lambda-params = lambda-param {"," lambda-param}
lambda-param = [type] identifier
method-ref   = type "::" identifier | type "::" "new" | expr "::" identifier
switch-expr  = "switch" "(" expr ")" "{" {switch-arm-expr} "}"
switch-arm-expr = ("case" constant {"," constant} | "default") "->" expr ";"
name         = identifier {"." identifier}
constant     = expr
```

A `constant` in a switch arm is a constant expression, as [chapter 12](12-flow.md) defines. A switch expression's arms are expressions, not statements. A switch statement's arms are statements.

`instanceof` does not continue into a following shift expression. A following `<` is a type argument or a comparison of a larger expression, not part of the type.
