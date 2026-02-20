# Comparação do Snask: Julho/2025 (ZIP) vs 18 Fev 2026 (repo)

Este documento compara dois “snapshots” do projeto:

- **Julho/2025 (ZIP):** `SnaskCode-SnaskInstaller.zip` (pasta `SnaskCode-SnaskInstaller/`)
- **Atual (18 fev 2026):** este repositório (estado atual)

## 1) O que o Snask *era* em Julho/2025 (ZIP)

### Arquitetura
- **Interpretado**: execução via **Python** (`main.py`) usando gramática **Lark** (`grammar.lark`).
- Estilo “bundle”: o ZIP trazia também script de atualização (Windows), um `.vsix` do VS Code e artefatos do runtime Python.

### Superfície da linguagem (como descrito no README do ZIP)
O README do ZIP descreve uma sintaxe mais “divertida/DSL”, com palavras‑chave como:
- Variáveis: `make`, `keep`, `set`, `zap`
- Funções: `craft`, `back`
- Controle de fluxo: `when`, `whenn`, `whenem`, `spin`, `loopy`, `breaky`, `skipit`
- Coleções: `pack`, `box` (+ helpers como `packadd`, `boxput`, etc.)
- Módulos: `to use "module"` (import)
- Arquivos: `readfile`, `writefile`
- GUI: `window`, `button`, `label`, `entry`, `start_gui_loop` (via ponte Python)

### Ecossistema + distribuição no ZIP
- Existia um instalador de pacotes (`snaskpack.py`) que baixava libs via um servidor local (“SnaskHub”).
- Existia um bridge com Flask (`snask_server.py`) para registrar rotas e responder requisições.

## 2) O que o Snask *é* em 18 Fev 2026 (este repo)

### Arquitetura
- **Compilado**: compilador em **Rust** que gera **LLVM IR** e faz link com um **runtime nativo em C**.
- **Runtime em C**: fornece primitives (OS/GUI/SQLite/threads), dependendo das flags do setup.

### Linguagem (direção atual)
- Sintaxe mais convencional, centrada em:
  - `class main` + `fun start()`
  - `let/mut/const`, `if/elif/else`, `while`, `for`, etc.
  - `import "lib"` com namespacing (`lib::fn`)
- **Projeto multi‑arquivo** suportado por:
  - `from / import modulo;`
  - `from dir/subdir import modulo;`

### Tooling de plataforma
- **SPS (Snask Project System)**:
  - Manifest: `snask.snif` (SNIF “strict”)
  - Lockfile: `snask.lock`
  - CLI: `snask init/build/run/add/remove/update/list/search/setup/doctor`
- **Distribuição**:
  - `snask dist` gera binários em `dist/`
  - Linux (best‑effort): `.deb` / `.AppImage`
  - Instalação “app Linux”: `snask dist --linux-user` (instala em `~/.local` + gera `.desktop`)

### GUI
- GUI agora é **GTK3 nativo** no runtime (widgets reais + suporte a CSS via `gui::css()` e `gui::add_class()`).
- Ergonomia é construída em libs (ex.: `apps/snask_vault`).

## 3) Maiores diferenças (alto impacto)

### A) Interpretador → Compilador (+ runtime nativo)
**Julho/2025:** Python + Lark.

**Fev/2026:** pipeline compilado (LLVM) + runtime C + etapa de link.

Impacto:
- Melhor teto de performance.
- Binários “de verdade”.
- Mais responsabilidade: ABI/runtime estável, segurança do runtime, empacotamento, etc.

### B) “Scripts soltos” → ecossistema unificado (SPS + SNIF + lock + registry)
**Julho/2025:** scripts + “hub” local.

**Fev/2026:** workflow coerente:
- `snask.snif` como fonte de verdade do projeto.
- `snask.lock` para reprodutibilidade.
- `snask doctor` para suporte/diagnóstico.

### C) GUI bridge → GTK3 nativo
**Julho/2025:** ponte estilo Tkinter/Python.

**Fev/2026:** GTK3 no runtime + wrappers em libs + CSS.

## 4) Mapeamento conceitual (antigo → novo)

Não é 1:1 (a sintaxe mudou), mas conceitualmente:

- `to use "X"` → `import "X"` (e agora também `from ... import X;`)
- `window/button/label/entry/start_gui_loop` → `gui::window/gui::button/.../gui::run`
- `readfile/writefile` → `sfs::read/sfs::write` (e helpers via `os::...`)

## 5) “Regressões” / coisas do ZIP que não existem igual hoje

- A sintaxe “fun/DSL” do ZIP não é compatível com o compilador atual.
- O ZIP tinha um bridge Flask pronto (`snask_server.py`) para web prototipar rapidamente.
  - Hoje o Snask consegue fazer web (Blaze existiu no roadmap), mas a abordagem é diferente.
- O ZIP era pensado como “bundle” (instalar tudo junto).
  - Hoje a direção é “compilador + runtime + packages + dist”.

## 6) Por que a direção de 2026 é mais forte pro objetivo “platform language”

Ela casa com:
> “Snask is a batteries-included platform language for building fast desktop and tooling applications with a unified ecosystem.”

Porque entrega:
- Pipeline de build estável (compiler + runtime)
- Sistema de projeto (SPS)
- Formato padrão de config/interchange (SNIF)
- História de distribuição (`snask dist`)
- História de GUI (GTK3 + libs)

## 7) Próximos passos recomendados

- **Estabilidade**: reduzir breaking changes em runtime/sintaxe core.
- **DX**: melhorar diagnósticos e templates “golden path”.
- **Packaging**: evoluir `snask dist` com mais metadata (ícone, desktop, versão, licença).
- **Segurança**: se o Vault virar algo real, precisa criptografia séria + keyring do OS.

