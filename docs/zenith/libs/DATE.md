# date: biblioteca Zenith

Helpers de data/tempo baseados no timestamp Unix.

## Uso conceitual

```text
import "date"
let ts = date::now()
let stamp = date::stamp()
let elapsed = date::since(ts)
```

## Funcoes previstas

| Funcao | Descricao |
| --- | --- |
| `date::now()` | timestamp Unix |
| `date::stamp()` | string `[ts:...]` |
| `date::label()` | rotulo legivel simples |
| `date::since(ts)` | tempo decorrido |
| `date::diff(a, b)` | diferenca em segundos |

## Status

Experimental. Sem timezone completo ainda.
