# 📄 Arquivos (v0.3.0) — GC, SQLite e Cross-Compilation

Este documento explica como usar as features novas do Snask **v0.3.0** relacionadas a:

- **GC simples** (garbage collection) para strings/buffers do runtime
- **SQLite** nativo no runtime (incluindo API de *statement*)
- **Cross-compilation** (MVP) via `--target`

> Importante: o Snask continua sendo **compilado** (gera binário). O `setup` prepara o runtime nativo que o linker usa no build.

---

## ✅ Pré-requisitos (Linux)

- `cargo` (para compilar o compilador, se você estiver no source)
- `clang-18`, `llc-18`
- `gcc` (para runtime nativo do host)
- `pkg-config`

Opcional:
- **SQLite (host)**: `libsqlite3-dev`
- **GUI (host)**: `libgtk-3-dev`

Depois, rode:

```bash
snask setup
```

Isso cria/atualiza os artefatos em `~/.snask/lib/` (runtime) e instala o binário em `~/.snask/bin/`.

---

## 🧠 GC simples (strings/buffers)

### O que é

No v0.3.0, o runtime passou a rastrear automaticamente diversas strings/buffers alocadas internamente (ex.: concatenação de strings, `num_to_str`, buffers de HTTP/JSON etc.) e libera tudo no final do processo.

### O que isso resolve

- Evita leaks “óbvios” do runtime em programas longos.
- Reduz a necessidade de “dar free” manual dentro do runtime.

### Limitações / notas

- É um **GC simples por rastreamento de ponteiros** (libera no final do processo).
- Ele não é um GC completo com coleta incremental/geracional.
- Se o seu programa cria muitos dados em loop infinito, ainda pode crescer memória (porque a liberação acontece no final).

### Como usar

Você não precisa habilitar nada no Snask. Basta estar no **v0.3.0** e ter rodado:

```bash
snask setup
```

---

## 🗄️ SQLite nativo

### Habilitar SQLite no runtime (host)

Instale as deps e rode `setup`:

```bash
sudo apt install -y libsqlite3-dev pkg-config
snask setup
```

Se o `setup` encontrar SQLite via `pkg-config`, ele compila o runtime com suporte e persiste os link args em:

- `~/.snask/lib/runtime.linkargs`

### Usar no Snask

Você pode usar as builtins diretamente (`sqlite_open`, `sqlite_exec` etc.), mas o fluxo recomendado é importar o wrapper `sqlite.snask` (se estiver no seu projeto/registry).

#### API básica

- `sqlite::open(path)` → handle (string) ou `nil`
- `sqlite::close(db)` → `bool`
- `sqlite::exec(db, sql)` → `bool`
- `sqlite::query(db, sql)` → `any` (estrutura parseada do JSON)

**Dica:** para inspecionar o retorno do `query`, use `json::stringify(...)`.

#### API de *statement* (mais segura)

Para queries parametrizadas e evitar concatenação manual:

- `sqlite::prepare(db, sql)` → stmt handle (string) ou `nil`
- `sqlite::bind_text(stmt, idx1, text)` → `bool` (idx começa em **1**)
- `sqlite::bind_num(stmt, idx1, num)` → `bool`
- `sqlite::bind_null(stmt, idx1)` → `bool`
- `sqlite::step(stmt)` → `bool` (true quando há linha; false em DONE/erro)
- `sqlite::reset(stmt)` → `bool`
- `sqlite::finalize(stmt)` → `bool`
- `sqlite::column_count(stmt)` → `num`
- `sqlite::column_name(stmt, idx0)` → `str` (idx começa em **0**)
- `sqlite::column(stmt, idx0)` → `any`

### Erros comuns

- `sqlite::open(...)` retornando `nil`: caminho inválido, permissão, ou runtime sem SQLite (rode `snask setup` após instalar deps).
- Build falhando na linkagem: runtime linkargs não foram gerados (rode `snask setup` novamente).

---

## 🎯 Cross-compilation (MVP)

### Como funciona

O v0.3.0 introduz um fluxo simples por alvo:

- `snask setup --target <triple>` compila um runtime para o alvo e salva em:
  - `~/.snask/lib/<triple>/runtime.o`
  - `~/.snask/lib/<triple>/runtime.linkargs`
- `snask build arquivo.snask --target <triple>` usa esse runtime e passa `--target=<triple>` para o `clang-18`.

### Exemplo de uso (alvo genérico)

```bash
snask setup --target <triple>
snask build main.snask --target <triple>
```

### Importante: você precisa do toolchain/sysroot do alvo

Se você tentar compilar para Windows/macOS sem toolchain instalado, é normal ver erros como:

- `fatal error: 'stdio.h' file not found`

Isso significa que o `clang` não encontrou os headers/libraries daquele alvo.

### Alvos comuns (exemplos)

- Windows (GNU/mingw): `x86_64-w64-windows-gnu`
- Linux musl (estático): `x86_64-unknown-linux-musl`
- macOS geralmente exige um toolchain específico (ex.: **osxcross**) e SDK.

### Observação sobre SQLite/GTK no cross

No `setup --target`, o Snask **não** tenta habilitar `pkg-config` de GTK/SQLite automaticamente (porque isso costuma ser do host). O runtime cross é gerado com o mínimo (inclui `-pthread`).

Se você quiser SQLite/GTK em cross no futuro, o caminho é configurar um sysroot do alvo e as libs do alvo, e evoluir o `setup` para detectar isso.

---

## ✅ Checklist rápido

- Runtime host OK: `snask setup`
- SQLite host OK: `sudo apt install -y libsqlite3-dev pkg-config` + `snask setup`
- Cross OK (MVP): toolchain do alvo instalado + `snask setup --target <triple>` + `snask build --target <triple>`

