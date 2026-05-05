# logger: biblioteca Zenith

Biblioteca simples de log para Snask.

## Uso conceitual

```text
import "logger"
logger::info("Servidor iniciado")
logger::warn("Conexao lenta")
logger::error("Falha critica")
logger::debug("Valor: 42")
```

## Funcoes previstas

| Funcao | Descricao |
| --- | --- |
| `logger::info(msg)` | log informativo |
| `logger::warn(msg)` | aviso |
| `logger::error(msg)` | erro |
| `logger::debug(msg)` | debug |
| `logger::raw(level, msg)` | nivel customizado |

## Status

Experimental. Pode depender da resolucao atual de pacotes/imports.
