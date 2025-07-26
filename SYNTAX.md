# Syntax

Finally some syntax docs!

# Numbers and Floats

Numbers are integers. Floats are floats.<br>
Decimal number and float literals work as normal. Scientific notation is not supported.<br>
Binary, octal and hexadecimal require the prefixes `0b`, `0o` and `0x` respectively.

# Strings

Strings may use double or single quotes.<br>
The current escape sequences are:
- `\'`: `'`
- `\"`: `"`
- `\\`: `\\`
- `\0`: U+0000
- `\t`: U+0009
- `\r`: U+000D
- `\n`: U+000A

# Keywords

- `let`
- `if`
- `true`
- `false`
- `else`
- `unless`
- `out`
- `try`
- `catch`
- `return`
- `throw`
- `loop`
- `while`
- `until`
- `defer`
- `bind`
- `goto`
- `fn`
- `null`
- `debug`
- `release`
- `maybe`
- `for`
- `in`
- `before`
- `again`
- `after`
- `continue`
- `break`
- `possibly`
- `probably`

# Identifiers

Identifiers may not contain any of these characters: `{}+-*/%?=!~<>&|^;@()[].:,"'$#`. Also, they can't start with a decimal digit or be a keyword.

# Comments

```
// single-line

/*
multiline
*/
```

# Labels

Same rules as identifiers, except they require a `#` before them and they can have a digit after the `#`.

# Expressions

```
true;
false;
null;
debug; // true if in debug mode, false otherwise
release; // !debug
maybe; // 50% chance of true, 50% chance of false
possibly; // 2/3 chance of true, 1/3 chance of false
probably; // 75% chance of true, 25% chance of false

expr + expr; // addition
expr - expr; // subtraction
-expr; // negation
expr * expr; // multiplication
expr / expr; // division
expr % expr; // modulus
cond ? expr : expr; // ternary operator
expr1 ?? expr2; // if expr1 is null, result is expr2. otherwise, result is expr1.
expr == expr; // equal to
!expr; // not
expr != expr; // not equal to
expr < expr; // less than
expr <= expr; // less than or equal
expr > expr; // greater than
expr >= expr; // greater than or equal
expr & expr; // and
expr | expr; // or
expr ^ expr; // xor
?expr; // expr != null
expr |> expr; // pipe (explained later)
$; // shorthand (explained later)
(expr);
expr[expr]; // property access
expr.name; // property access
f(args); // function call
(expr, expr, expr); // tuple
```

# Operator Functions

These are the functional counterpart of operators. They are expressed by wrapping the operator in brackets. Here is a list of them:
- `[+]`: addition (2 arguments or more)
- `[-]`: negation (1 argument), subtraction (2 arguments)
- `[*]`: multiplication (2 arguments or more)
- `[/]`: division (2 arguments)
- `[%]`: modulus (2 arguments)
- `[??]`: `??` operator (2 arguments or more)
- `[?:]`: ternary operator (3 arguments)
- `[==]`: equal to (2 arguments)
- `[!]`: not (1 argument)
- `[!=]`: not equal to (2 arguments)
- `[<]`: less than (2 arguments)
- `[<=]`: less than or equal (2 arguments)
- `[>]`: greater than (2 arguments)
- `[>=]`: greater than or equal (2 arguments)
- `[&]`: and (2 arguments or more)
- `[|]`: or (2 arguments or more)
- `[^]`: xor (2 arguments)
- `[.]`: property access (2 arguments or more)
- `[,]`: tuple (at least 1 argument)
- `[|>]`: pipe (2 arguments or more)
- `[?]`: `?` operator (1 argument)

# Alternative Writing

Some keywords and operators have alternative ways to write them.

- `elif` => `else if`
- `elsif` => `else if`
- `fun` => `fn`
- `func` => `fn`
- `function` => `fn`
- `&&` => `&`
- `||` => `|`
- `^^` => `^`

# Pipe Operator and Shorthand

The pipe operator is used to express pipelines more easily. The pipe operator can be used in 2 ways:
- if the right-hand side has any `$`s (the shorthand expression) inside it, these get substituted with the value resulting from the left-hand side. then the result of the right-hand side is the result of the operation.
- otherwise, the right-hand side is assumed to evaluate to a function which is called with 1 single argument: the value of the left-hand side. the result of the call is the result of the operation.

# Statement-expressions

Statement-expressions are expressions that can be turned into statements without putting a semicolon in front of them. They need to be wrapped in parenthesis when used inside normal expressions.

```
// normal if else
if cond {
    // ... (statements)
} else if cond {
    // ... (statements)
} else {
    // ... (statements)
}

// unless is like if but the condition is `not`ed
unless cond {
    // ...
}

// unless can be used everywhere an if can be used

// normal try-catch
try {
    // statements
} catch name {
    // if an error happened
}

// loops

loop {
    // infinite loop
}

loop num {
    // loops num times
}

while cond {
    // loops while cond is true
}

until cond {
    // loops until cond is true
}

for i in iterable {
    // iterates iterable
}

for i in iterable while cond {
    // iterates iterable while cond is true
}

for i in iterable until cond {
    // iterates iterable until cond is true
}

// loop structures

before {
    // executes before the loop is run
} loop {
    // can be any basic loop
} again {
    // executes every time the loop is continued
} after {
    // executes if the loop wasn't broken out of
} else {
    // executes if anything before it was broken out of
}

// functions

fn name(arg1, arg2, arg3) {
    // body
}

// blocks

{
    // ...
}
```

# Statements

All expressions can become statements when a semicolon is put after them. Statement-expressions don't need a semicolon after them.<br>
Other kinds of statements:

```
let name = val; // declares the variable name and assigns it the value val.
let name; // let name = null;
let (name1, name2, name3) = expr; // detuple
let (name1, name2, name3); // let name1; let name2; let name3;
out expr; // similar to tail expressions in rust
return expr; // return
throw expr; // throw
defer {
    // statements here are ran right before the scope the defer is placed ceases to exist
}
defer expr; // defer { expr; }
bind name = val; // creates a new bind: binds are like getter functions
goto #label; // goto
continue; // continue
break; // break
continue expr; // continue; all `$`s in the again block (if any) will be replaced with expr
break expr; // break; all `$`s in the else (loop) block (if any) will be replaced with expr
```