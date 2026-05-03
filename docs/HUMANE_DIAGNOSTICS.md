# Snask Humane Diagnostics

Snask Humane Diagnostics is the diagnostic system for parser, semantic and toolchain errors.

The goal is simple:

- errors should be short by default;
- every user-facing diagnostic should have a stable short code;
- the compiler should point at the exact source line;
- help should be actionable;
- internal Rust debug dumps should never be the default user experience;
- deeper explanations should live behind `snask explain`.

## Shape of an Error

The default format is:

```text
error[S1002]: missing closing `)`
 --> hello.snask:3:22
  |
3 |         print("Hello"
  |                      ^ expected `)` here
note: unclosed '(' opened at 3:14
help: You probably missed a closing ')'.
```

The important pieces are:

- `error`: severity;
- `S1002`: stable public code;
- `missing closing ')'`: short message;
- `hello.snask:3:22`: location;
- source snippet;
- caret annotation;
- optional notes;
- optional help.

## Error Code Ranges

Current public ranges:

| Range | Area |
| --- | --- |
| `S1000`-`S1999` | Parser and syntax errors |
| `S2000`-`S2999` | Semantic/type errors |
| `S9000`-`S9999` | Build modes and toolchain policy |

Examples:

| Code | Meaning |
| --- | --- |
| `S1002` | Missing closing `)` |
| `S1003` | Missing closing `]` |
| `S1004` | Missing closing `}` |
| `S1005` | Expected an indented block |
| `S1010` | Expected an expression |
| `S2002` | Variable not found |
| `S2010` | Type mismatch |
| `S2012` | Immutable assignment |

## `snask explain`

Use `snask explain <code>` for a longer explanation:

```bash
snask explain S1002
```

Output:

```text
S1002: missing closing `)`

Snask found a function call or grouped expression that started with `(` but did not find the matching `)`.

Example:
  print("Hello"

Fix:
  print("Hello")
```

The command also accepts legacy internal codes:

```bash
snask explain SNASK-PARSE-MISSING-RPAREN
```

## Parser Diagnostics

Parser diagnostics should:

- report only the first few errors after recovery;
- prefer the earliest root cause;
- avoid cascading walls of text;
- include delimiter notes when useful;
- use public codes.

Example:

```snask
class main
    fun start()
        print("Hello"
```

Diagnostic:

```text
error[S1002]: missing closing `)`
 --> hello.snask:3:22
  |
3 |         print("Hello"
  |                      ^ expected `)` here
note: unclosed '(' opened at 3:14
help: You probably missed a closing ')'.
```

## Semantic Diagnostics

Semantic diagnostics should use public Snask type names, not Rust enum debug names.

Good:

```text
error[S2010]: expected `int`, found `str`
```

Bad:

```text
SemanticError { kind: TypeMismatch { expected: Int, found: String }, ... }
```

Example:

```snask
class main
    fun start()
        let age: int = "18"
```

Diagnostic:

```text
error[S2010]: expected `int`, found `str`
 --> types.snask:3:9
  |
3 |         let age: int = "18"
  |         ^^^^^^^^^^^^^^^^^^^ type mismatch here
```

## Suggestions

Name errors can include spelling suggestions:

```snask
class main
    fun start()
        let message = "Hello"
        print(mesage)
```

Diagnostic:

```text
error[S2002]: variable `mesage` was not found
 --> name.snask:4:15
  |
4 |         print(mesage)
  |               ^^^^^^ unknown name
help: Did you mean 'message'?
```

Suggestions should be high confidence by default. A bad suggestion is worse than no suggestion.

## Implementation Notes

Core files:

- `src/diagnostics.rs`: generic diagnostic structs, source rendering and public code mapping.
- `src/compiler.rs`: parser/semantic diagnostic conversion.
- `src/semantic_analyzer.rs`: semantic messages and name suggestions.
- `src/explain.rs`: long-form explanations used by `snask explain`.
- `src/main.rs`: CLI command wiring and clean diagnostic output.

## Testing

Focused tests should assert:

- public code appears;
- source snippet appears;
- caret appears;
- help appears when expected;
- internal debug structs do not appear.

Useful commands:

```bash
cargo test humane -- --nocapture
cargo test explain -- --nocapture
cargo check
```

Manual CLI checks:

```bash
snask explain S1002
snask explain S2002
snask build bad_file.snask
```

## Roadmap

Next improvements:

- `snask build --explain` for expanded inline diagnostics;
- JSON diagnostic output for LSP/tooling;
- richer secondary spans;
- fix-it suggestions that can be applied automatically;
- better indentation diagnostics with before/after snippets;
- property/function suggestions from namespaces;
- OM-specific public diagnostic codes;
- linker diagnostics with missing package hints.
