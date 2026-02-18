# SNIF Grammar (informal)

Whitespace: spaces, tabs, newlines.
Comments: `//` until end-of-line.

```
document  := value ws EOF

value     := object
          | array
          | string
          | number
          | "true"
          | "false"
          | "null"
          | typed_literal
          | ref_define
          | ref_use

object    := "{" ws (member (ws "," ws member)* ws ","? )? ws "}"
member    := key ws ":" ws value
key       := identifier | string

array     := "[" ws (value (ws "," ws value)* ws ","? )? ws "]"

typed_literal := "@" identifier ws string

ref_define := "&" identifier ws value
ref_use    := "*" identifier

identifier := [A-Za-z_$] [A-Za-z0-9_$-]*
string     := single_quoted | double_quoted
number     := int | float | scientific
```

Notes:
- `:` is the only valid member separator.
- Bareword strings are not allowed.
- Typed literals require a quoted string payload.

