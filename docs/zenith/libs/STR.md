# str: biblioteca Zenith

Helpers de string que complementam o runtime Snask.

## Uso conceitual

```text
import "str"
let padded = str::pad_left("42", 5, "0")
let line = str::repeat("-", 30)
let ok = str::is_empty("  ")
let result = str::template("Ola, {name}!", "name", "Davi")
```

## Funcoes previstas

| Funcao | Descricao |
| --- | --- |
| `str::pad_left(s, n, char)` | completa a esquerda |
| `str::pad_right(s, n, char)` | completa a direita |
| `str::repeat(s, n)` | repete string |
| `str::is_empty(s)` | checa vazio/espacos |
| `str::truncate(s, n)` | corta com reticencias |
| `str::slugify(s)` | normaliza para identificador simples |

## Status

Experimental. Exemplos estao como `text` porque a superficie de modulo ainda e parcial.
