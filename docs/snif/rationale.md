# SNIF Rationale

SNIF exists because JSON is great for interoperability, but weak for configuration and long-lived data modeling.

## Why SNIF is stricter than JSON-like supersets
SNIF intentionally removes common sources of ambiguity:
- Only `:` as member separator (no `=`).
- Only `//` comments (no multiple comment dialects).
- Only `null` (no `nil` alias).
- No bareword strings (prevents silent bugs and future keyword conflicts).

## Why typed literals exist
Typed literals make intent explicit and predictable:
- `@date"..."` expresses a date/time string semantically.
- `@dec"..."` preserves exact decimals.
- `@bin"..."` clearly marks base64 payload.
- `@enum"..."` provides a safe place for symbolic values without barewords.

## Why big integers become tagged objects
JSON’s single `number` type causes silent precision loss across ecosystems.
SNIF keeps exact values by representing big integers as `{ "$i64": "..." }`.

## Why references exist
Real configs often repeat blocks. References:
- reduce duplication,
- reduce copy/paste errors,
- keep configs maintainable.

