# Backend Skia experimental

Snask possui `snask_skia` com fallback Cairo quando GTK esta disponivel no runtime. Um backend Skia real exige runtime compilado com `SNASK_SKIA`.

## Status

Experimental. Cairo e o caminho padrao; Skia real depende de instalacao externa e `pkg-config`.

## Instalacao opcional

```bash
snask install-optional skia
```

Esse comando e uma direcao de ferramenta. Se ele nao estiver disponivel no build local, instale Skia manualmente e forneca `skia.pc`.

## Deteccao

`snask setup` tenta usar:

```bash
pkg-config --cflags skia
pkg-config --libs skia
```

Se funcionar, o runtime pode ser compilado com `-DSNASK_SKIA`. Se falhar, Snask continua com Cairo.

## Uso conceitual

```text
import "snask_skia"
const USE_SKIA = 1
```

A API de desenho ainda e experimental.
