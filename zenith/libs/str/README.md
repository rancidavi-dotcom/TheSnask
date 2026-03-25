# str — Zenith Library

String helpers that complement Snask's native built-ins.

## Installation

```bash
# The str.snask file is already in ~/.snask/packages/
```

## Usage

```snask
import "str";

let padded = str::pad_left("42", 5, "0");
// "00042"

let line = str::repeat("-", 30);
// "------------------------------"

let ok = str::is_empty("  ");
// true

let result = str::template("Hello, {name}! You have {n} msgs.", "name", "David", "n", "3");
// "Hello, David! You have 3 msgs."
```

## Functions

| Function | Description |
|---|---|
| `str::pad_left(s, n, char)` | Pads left with `char` until length `n`. |
| `str::pad_right(s, n, char)` | Pads right with `char` until length `n`. |
| `str::repeat(s, n)` | Repeats the string `n` times. |
| `str::is_empty(s)` | Returns true if string is empty or only whitespace. |
| `str::truncate(s, n)` | Cuts the string at `n` chars, adds `...`. |
| `str::slugify(s)` | Converts to simple snake_case. |

## Observations (Snask pre-alpha)
- `template()` basic substitution by key/value pairs.
- No native regex support yet.
