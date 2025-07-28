# Standard Library

Finally some standard library docs. Currently it only documents globals.

# Globals

## `args()`

Returns an `iterator` over the arguments passed to the bodu program.

## `atob(s)`

Decodes `s` (a string) as base64 and returns a `buffer` with the result.

## `async(f)`

Makes a function that runs `f` (a function) on a new thread. The returned function returns a `promise`, which can then be passed to `await` to retrieve the value returned by `f`.

## `await(p)`

Retrieves the value contained in `p` (a `promise` returned by a function returned by `async`), blocking the current thread if the value is not ready.

## `awaitfn(f)`

The inverse of `async`. Takes a function returned by `async` and returns a function that runs `f` (a function returned by `async`) and calls `await` on the result.

## `bin(n)`

Converts `n` (a number) to its string representation in base-2 (binary).

## `boolean(v)`

Converts `v` (any value) to a boolean, throwing an error if not possible.

## `btoa(buf)`

Encodes `buf` (a `buffer`) as base64 and returns a string with the result.

## `chr(n)`

Takes `n` (a number) and returns a string with the character with the Unicode codepoint `n`.

## `eprint(...)`

Takes any number of arguments and writes those arguments joined by tab characters to stderr with a new line.

## `exec(s)`

Takes `s` (a string) and evaluates its contents as bodu code, returning the returned value from the code.

## `float(v)`

Converts `v` (any value) to a float, throwing an error if not possible.

## `from_bin(s)`

Parses `s` (a string) as a binary integer.

## `from_hex(s)`

Parses `s` (a string) as a hexadecimal integer.

## `from_oct(s)`

Parses `s` (a string) as an octal integer.

## `global()`

Returns the global object.

## `hex(n)`

Converts `n` (a number) to its string representation in base-16 (hexadecimal), using lowercase characters.

## `hex_upper(n)`

Converts `n` (a number) to its string representation in base-16 (hexadecimal), using uppercase characters.

## `id(v)`

Returns a string with the id (a string representing the memory address where the value is stored) of `v` (any value).

## `input(...)`

Firstly writes its arguments to stdout in the same way as `print`. Then it reads a line from stdin and returns it.

## `load(s)`

Takes `s` (a string) and converts it into a function that when called runs the strings contents as bodu code.

## `load_here(s)`

Does the same as `load`, but it gives the code access to the current scope.

## `load_lib(path)`

Loads a native bodu library located in `path` (a string).

## `number(v)`

Converts `v` (any value) to a number, throwing an error if not possible.

## `oct(n)`

Converts `n` (a number) to its string representation in base-8 (octal).

## `ord(s)`

Takes `s` (a string) and returns the Unicode codepoint of its first character.

## `print(...)`

Takes any number of arguments and writes those arguments joined by tab characters to stdout with a new line.

## `push_gdefer(f)`

Adds `f` (a function) to a list of functions that will be run when the user hits Ctrl+C.

## `range(stop)`

Same as `range(0, stop, 1)`.

## `range(start, stop)`

Same as `range(start, stop, 1)`.

## `range(start, stop, step)`

Returns an iterator that works similar to Python's `range`.

## `sleep(n)`

Makes the current thread sleep for `n` (a number) milliseconds.

## `stderr(...)`

Takes any number of arguments and writes those arguments joined by tab characters to stderr without a new line.

## `stdout(...)`

Takes any number of arguments and writes those arguments joined by tab characters to stdout without a new line.

## `string(v)`

Converts `v` (any value) to a string, throwing an error if not possible.

## `type(v)`

Returns the type of `v` (any value) as a string.