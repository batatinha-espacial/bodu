# Syntax

```
// LOOP REWORK

// part 1:
loop {
  // runs until you break out of it
}
loop n {
    // runs n times
}
while cond {
  // runs while cond evaluates to true
}
until cond {
  // runs until cond evaluates to true
}
for i in iter {
    // iterates iter
}
for i in iter while cond {
    // iterates iter while cond is true
}
for i in iter until cond {
    // iterates iter until cond is true
}
// part 2:
before { // optional
    // before the loop
} loop n { // can be any loop except for `loop` loops
    // ...
} again { // optional
    // ran everytime it reaches a continue statement
} after { // optional
    // if the loop ended normally
} else { // optional
    // if anything before it was broken out of
}
// note: all these blocks use the same scope
// part 3:
continue; // continues the inner most loop
break; // breaks the inner most loop
continue expr; // makes expr acessible to an again block when using `$$`
break expr; // not allowed in `loop` loops; makes expr acessible to an else block when using `$$`
```

```
debug {
    // only runs in debug mode
}

release {
    // only runs when not in debug mode
}

// these two can be combined
debug {
    // ...
} release {
    // ...
}
```

# Stdlib

- `iter.map`: array.prototype.map (from JS) but for iterators
- `date`: time and date library
- `math.abs`: absolute value
- `iter.enumerate`: same as rust
- `iter.filter`: same as rust