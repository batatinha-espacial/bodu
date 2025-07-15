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

- `os` lib: at least detect the OS
- `range`: same as python
- `iter.map`: array.prototype.map (from JS) but for iterators