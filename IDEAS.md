# Syntax

```
loop {
    // infinite
}

loop n {
    // runs n times
}

first {
    // before the loop
} loop n {
    // runs n times
} then {
    // if the loop ended normally
} else {
    // if either first, loop, or then were broken out of
}
```

```
for a in iter {
    // iterates iter
}

for a in iter while cond {
    // iterates iter until iter runs out of elements or cond evaluates to false
}

for a in iter until cond {
    // works similarly to the above
}
```

# Stdlib

- `iter.map`: array.prototype.map (from JS) but for iterators
- `date`: time and date library
- `math.abs`: absolute value
- `bin`: integer to binary string
- `chr`: converts an integer representing an unicode codepoint to a string
- `iter.enumerate`: same as rust
- `iter.filter`: same as rust
- `hex`: integer to hexadecimal string
- `id`: returns the id of a value (ids are memory addresses as strings)
- `oct`: integer to octal string
- `ord`: converts a string to the unicode codepoint of its first character