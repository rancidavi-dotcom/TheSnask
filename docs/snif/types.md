# Tipos SNIF

## Tipos base

- `null`
- `bool`
- `number`
- `string`
- `array`
- `object`

## Objetos marcados

SNIF representa semanticas estendidas com chaves reservadas.

```snif
{ "$i64": "9007199254740993" }
{ "$date": "2026-02-18T00:00:00Z" }
{ "$dec": "19.99" }
{ "$bin": "..." }
{ "$enum": "STATUS_OK" }
```

Literal desconhecido:

```snif
@foo"X"
```

vira conceitualmente:

```snif
{ "$type": "foo", "value": "X" }
```
