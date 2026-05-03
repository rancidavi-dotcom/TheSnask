# Snask Store

App desktop feito em Snask puro para instalar bibliotecas do `SnaskPackages` via GUI.

## O que faz

- mostra um catalogo visual de libs do ecossistema
- instala e remove pacotes em `~/.snask/packages`
- instala pela GUI, sem `snask install`
- usa o registry local em `~/.snask/registry/packages`
- instala dependencias declaradas no catalogo do app

## Como abrir

```bash
cd apps/snask_store
../../target/release/snask build
./snask_store
```

## Nota tecnica

O app foi mantido 100% em Snask puro.
Ele evita as partes mais instaveis do ecossistema atual, como parser de packages com `;`, namespace `mod::func()` e fluxo HTTP/JSON ainda parcial no runtime.
