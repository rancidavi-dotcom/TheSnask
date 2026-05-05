# Exemplos SNIF

## Objeto minimo

```snif
{ name: "Snask", ok: true, }
```

## Array com virgula final

```snif
["a", "b", "c",]
```

## Literais tipados

```snif
{
  created_at: @date"2026-02-18T00:00:00Z",
  price: @dec"19.99",
  status: @enum"STATUS_OK",
}
```

## Inteiro grande

```snif
{ big: 9007199254740993 }
```

O parser preserva como valor marcado quando necessario.

## Referencias

```snif
{
  cfg: &x{ retries: 3, },
  a: *x,
  b: *x,
}
```

## Invalidos

```snif
{ name: snask }
```

Bareword como string nao e permitido.

```snif
{ name = "snask" }
```

O separador correto e `:`.
