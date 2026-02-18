# SNIF Types

## Base types
- `null`
- `bool`
- `number`
- `string`
- `array`
- `object`

## Tagged objects (extended semantics)
SNIF represents extended types as objects with reserved keys.

### Exact int64
```
{ "$i64": "9007199254740993" }
```

### Date/time (string payload)
```
{ "$date": "2026-02-18T00:00:00Z" }
```

### Decimal (exact)
```
{ "$dec": "19.99" }
```

### Binary (base64 payload)
```
{ "$bin": "..." }
```

### Enum
```
{ "$enum": "STATUS_OK" }
```

### Unknown typed literal
```
@foo"X" -> { "$type": "foo", "value": "X" }
```

