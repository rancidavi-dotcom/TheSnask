# SNIF Specification (v1)

SNIF (Snask Interchange Format) is a human-friendly, safe, deterministic data format designed for the Snask ecosystem.

## Goals
- Human-friendly configs (comments, trailing commas, simple objects/arrays).
- Deterministic parsing with a minimal syntax surface.
- Safer semantics than JSON for real configs (typed literals, references, big-int preservation).
- Easy to implement and fast to parse.

## Non-goals
- Full YAML feature parity.
- Complex schema language built into the format.

## Canonical decisions (MUST)
- Key/value separator is `:` only.
- Comments are `//` line comments only.
- Null is `null` only (`nil` is not a keyword).
- Barewords as string values are not allowed. Strings MUST be quoted.

## Document structure
A SNIF document is a single value: object, array, string, number, boolean, or null.

## Types
Supported base types:
- `null`
- `bool` (`true`, `false`)
- `number` (float-like)
- `string`
- `array`
- `object`

SNIF adds *typed literals* and *safe big integers* using tagged objects.

## Typed literals
Syntax:
- `@type"payload"` or `@type'payload'`

Built-in typed literals:
- `@date"..."`
- `@dec"..."`
- `@bin"..."`
- `@enum"..."`

Decoded representation:
- `@date"X"` becomes `{ "$date": "X" }`
- `@dec"X"` becomes `{ "$dec": "X" }`
- `@bin"X"` becomes `{ "$bin": "X" }`
- `@enum"X"` becomes `{ "$enum": "X" }`

Unknown types:
- `@foo"X"` becomes `{ "$type": "foo", "value": "X" }`

## Numbers and big integers
- Floating numbers and scientific notation parse as `number`.
- Integer tokens outside the safe IEEE-754 range (±(2^53-1)) MUST be preserved exactly as `{ "$i64": "..." }`.

## References
References reduce duplication and enable shared objects.

Syntax:
- `&name <value>` defines a reference `name` pointing to `<value>` and evaluates to `<value>`.
- `*name` evaluates to the previously-defined value of `name`.

Rules:
- Reference names are identifiers.
- Forward references are not allowed (using `*name` before `&name` is an error).
- Cycles are not allowed (if created indirectly, behavior is an error).

## Trailing commas
Trailing commas are allowed in objects and arrays.

## Errors
A parser MUST fail on:
- Unknown identifiers as values (barewords).
- Missing `:` after object keys.
- Unknown references (`*name` without a previous `&name`).
- Excessive depth (implementation-defined limit).

