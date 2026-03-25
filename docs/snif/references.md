# SNIF References

SNIF references provide a way to reuse the same value without duplication.

## Syntax
- Define: `&name <value>`
- Use: `*name`

Example:
```
{
  shared: &cfg{ retries: 3, timeout_ms: 1500, },
  service_a: { config: *cfg, },
  service_b: { config: *cfg, },
}
```

## Rules
- `*name` must refer to a name previously defined with `&name` (no forward refs).
- Reference names are identifiers.
- Implementations should reject cycles.
- Reference count should be limited (implementation-defined) to prevent memory abuse.

