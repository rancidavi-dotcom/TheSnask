# SNIF Examples

## Minimal object
```
{ name: "Snask", ok: true, }
```

## Array with trailing comma
```
["a", "b", "c",]
```

## Typed literals
```
{
  created_at: @date"2026-02-18T00:00:00Z",
  price: @dec"19.99",
  status: @enum"STATUS_OK",
}
```

## Big integer preservation
```
{ big: 9007199254740993 }
// parses as:
// { big: { "$i64": "9007199254740993" } }
```

## References
```
{
  cfg: &x{ retries: 3, },
  a: *x,
  b: *x,
}
```

## Invalid (bareword value)
```
{ name: snask }
// ERROR: Barewords are not allowed. Use "snask".
```

## Invalid (wrong separator)
```
{ name = "snask" }
// ERROR: Expected ':' after key.
```

