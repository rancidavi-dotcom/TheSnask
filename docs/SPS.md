# SPS (Snask Project System) — v1 (MVP)

O SPS é o sistema oficial de **manifesto de projeto** do Snask.

Nesta fase (MVP), ele resolve:
- `snask.toml` como manifesto do projeto
- `snask build` e `snask run` **sem passar arquivo** (usa o `entry`)
- `snask init` para criar o projeto padrão
- `snask add/remove` (dependências no manifesto)
- `snask.lock` determinístico (versões + sha256)
- `scripts` (atalhos) via `snask run <script>`

---

## 1) Criar um projeto

No diretório do seu projeto:

```bash
snask init
```

Ou definindo nome explicitamente:
```bash
snask init --name meu_app
```

Isso cria:
- `snask.toml`
- `main.snask` (entry default)

---

## 2) Manifesto `snask.toml` (v1)

Exemplo:

```toml
[package]
name = "meu_app"
version = "0.1.0"
entry = "main.snask"

[dependencies]

[build]
opt_level = 2
```

Campos:
- `[package].name` (obrigatório)
- `[package].version` (obrigatório)
- `[package].entry` (default: `main.snask`)
- `[build].opt_level` (0..3, default: 2)

`[dependencies]` existe no MVP mas ainda não é resolvido automaticamente (próxima fase).

---

## 3) Build/Run sem arquivo

Com `snask.toml` presente:

```bash
snask build
snask run
```

O Snask usa o `entry` do manifesto.

Você ainda pode compilar um arquivo direto:
```bash
snask build outro.snask
snask run outro.snask
```

---

## 4) Dependências (v1 simples)

Adicione dependências no `snask.toml`:
```toml
[dependencies]
json = "*"
os = "*"
```

Ou use o CLI:
```bash
snask add json
snask remove json
```

No `build`, o Snask garante que as dependências estejam instaladas em `~/.snask/packages/`.

---

## 5) Lockfile (`snask.lock`)

Ao rodar `snask build` dentro de um projeto SPS, ele escreve `snask.lock` com:
- `version` (do registry)
- `sha256` do arquivo `.snask` instalado

Isso vira a base para builds reproduzíveis e resolver de versões melhor no futuro.

---

## 6) Scripts

No `snask.toml`:
```toml
[scripts]
dev = "snask run main.snask"
build = "snask build"
```

Rodar:
```bash
snask run dev
```

---

## 7) Profiles / opt_level

Você pode definir otimização (0..3):
```toml
[build]
opt_level = 2

[profile.release]
opt_level = 3
```

No MVP, o `opt_level` é aplicado como `llc-18 -O{N}`.
