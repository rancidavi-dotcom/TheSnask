<p align="center">
  <img src="Snask.png" alt="Snask" width="120" style="border-radius: 24px;" />
</p>

<h1 align="center">Snask v0.4.1-alpha</h1>

<p align="center">
  <strong>Performance de sistemas. Ergonomia humana.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-brightgreen" alt="MIT" />
  <img src="https://img.shields.io/badge/rust-1.85+-orange" alt="Rust" />
  <img src="https://img.shields.io/badge/LLVM-18-blue" alt="LLVM 18" />
  <img src="https://img.shields.io/badge/status-alpha-yellow" alt="Alpha" />
  <img src="https://img.shields.io/badge/platform-linux-lightgrey" alt="Linux" />
</p>

<p align="center">
  <a href="#-instalação">Instalação</a> •
  <a href="#-hello-world">Hello World</a> •
  <a href="#-exemplos">Exemplos</a> •
  <a href="#-perfis">Perfis</a> •
  <a href="#-om-memory-system">OM</a> •
  <a href="#-documentação">Docs</a>
</p>

---

Snask é uma linguagem compilada AOT para binários nativos via **LLVM 18**. O objetivo é unir uma superfície humana para apps, CLI e DX com uma base de sistemas capaz de rodar runtimes, emuladores e interop nativa — sem transformar o usuário em gerenciador manual de memória.

> *Em vez de você escrever o contrato, o Snask deve deduzir o contrato.*

---

## 🚀 Instalação

```bash
# Via instalador universal (Linux)
curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
```

```bash
# Via AUR (Arch Linux)
yay -S snask
```

```bash
# Via .deb (Ubuntu/Debian)
curl -fsSL https://github.com/rancidavi-dotcom/TheSnask/releases/latest/download/snask-amd64.deb -o snask.deb
sudo dpkg -i snask.deb
```

```bash
# Compilação manual
git clone https://github.com/rancidavi-dotcom/TheSnask.git
cd TheSnask
cargo build --release
./target/release/snask doctor
```

---

## 💻 Hello World

```snask
class main {
    fun start() {
        print("Olá, Snask!\n")
    }
}
```

```bash
snask build hello.snask --output hello
./hello
# → Olá, Snask!
```

---

## 🔥 Exemplos

### SDL2 com gerenciamento automático (OM)

```snask
import_c_om "SDL2/SDL.h" as sdl2

class main {
    fun start() {
        zone "sdl" {
            sdl2.init(sdl2.INIT_VIDEO)
            let window = sdl2.create_window(
                "Snask SDL2", 100, 100, 640, 480, 0
            )
            let renderer = sdl2.create_renderer(window, -1, 0)
            sdl2.set_render_draw_color(renderer, 20, 120, 220, 255)
            sdl2.render_clear(renderer)
            sdl2.render_present(renderer)
            sdl2.delay(3000)
            sdl2.quit()
        } // window e renderer destruídos automaticamente
    }
}
```

### Memória crua (perfil Systems)

```snask
class main {
    fun start() {
        @unsafe {
            let mem: ptr = mem_alloc_zero(65536)  // 64KB
            mem_write_u8(mem, 0xFFFC, 0x00)
            mem_write_u8(mem, 0xFFFD, 0x80)
            let pc: u16 = mem_read_u16(mem, 0xFFFC)
            mem_free(mem)
        }
    }
}
```

### Interface gráfica nativa

```snask
gui_init()
let win = gui_window("Snask App", 400, 300)
let vbox = gui_vbox()
let label = gui_label("Clique no botão!")
let btn = gui_button("Clique Aqui")
gui_on_click(btn, fun() { gui_set_text(label, "Clicou!") })
gui_add(vbox, label)
gui_add(vbox, btn)
gui_add(win, vbox)
gui_show_all(win)
gui_run()
```

### Projeto SPS

```snif
[project]
name = "meu_app"
version = "0.1.0"
main = "src/main.snask"

[dependencies]
json = "1.0.0"

[build]
profile = "humane"
target = "native"
```

---

## ⚡ Perfis

Snask tem dois perfis principais que coexistem na mesma linguagem:

| Perfil | Uso | Características |
|--------|-----|-----------------|
| `humane` | apps, CLI, aprendizado | Runtime completo, zonas automáticas, produtividade máxima |
| `systems` | emuladores, parsers, memória crua | `@unsafe`, ponteiros, primitivas NES/CPU, controle total |

Build explícito: `snask build --profile systems`

> No perfil **Humane** você nunca vê um `malloc`. No perfil **Systems** você controla cada byte — e o OM continua gerenciando seus recursos C.

---

## 🧠 OM Memory System

O **OM (Orquestrador de Memória)** é o coração do Snask. Ele unifica:

- **Zonas** — escopos nominados com liberação automática
- **Arenas** — alocação linear O(1) para processamento em lote
- **Recursos externos** — handles C com destrutores registrados
- **Contratos C** — bindings deduzidos automaticamente de headers, com patch `.om.snif` opcional

```snask
zone "request" {
    let dados = read_file("config.json")
    let parsed = json_parse(dados)
    processar(parsed)
} // tudo liberado aqui
```

🔗 [Documentação completa do OM](docs/systems/OM_SNASK_SYSTEM.md) • [Tutorial .om.snif](docs/site/systems/om.html#modulo-5-tutorial-om-snif)

---

## 📂 Estrutura do projeto

```
├── src/                  # Compilador (Rust)
│   ├── bin/snask.rs      # CLI principal
│   ├── bin/snask-lsp.rs  # Servidor LSP
│   └── ...               # Parser, semântico, codegen LLVM
├── docs/
│   ├── site/             # Site publicado (GitHub Pages)
│   │   ├── systems/om.html    # OM tutorial completo
│   │   ├── showcase.html      # Galeria visual
│   │   └── learn/             # 9 capítulos do zero ao expert
│   └── systems/          # Documentação técnica
├── apps/
│   ├── nes_emulator/     # Emulador NES em Snask puro
│   ├── snask_store/      # App GUI experimental
│   └── snask_vault/      # App de storage
├── Testes/               # Testes de integração OM
└── scripts/              # Utilitários
```

---

## 📊 Status das features

| Área | Status |
|------|--------|
| Core: `let`, `mut`, `if`, `while`, `for in`, funções tipadas | ✅ estável |
| Classes nominais com herança | ✅ estável |
| Coleções genéricas `list<T>`, `dict<K,V>` | ✅ estável |
| `@unsafe` gate + memória crua | ✅ estável |
| Primitivas NES/CPU (bits, flags, overflow) | ✅ estável |
| OM: zonas, recursos, scanner C | ✅ estável |
| Patch `.om.snif` | ✅ estável |
| C interop universal | 🔶 experimental |
| GUI nativa | 🔶 experimental |
| LSP | 🔶 parcial |
| Borrow checker estático | 🔄 planejado |

---

## 📖 Documentação

- [🌐 Site oficial](docs/site/index.html) — documentação visual publicada
- [🔥 Showcase](docs/site/showcase.html) — galeria de tecnologia
- [📚 Aprender Snask](docs/reference/LEARN_SNASK.md) — do zero ao expert (9 capítulos)
- [📘 Referência da linguagem](docs/reference/LANGUAGE_REFERENCE.md)
- [⚙️ OM-Snask-System](docs/systems/OM_SNASK_SYSTEM.md)
- [🔧 CLI Reference](docs/site/tooling/cli.html)
- [📋 Feature status](docs/reference/FEATURE_STATUS.md)

---

## 🏆 Benchmark & orgulho técnico

O NES emulator em `apps/nes_emulator/` executa ROMs NROM reais em Snask puro — um laboratório vivo do perfil **Systems**.

```bash
cargo build --release
./target/release/snask build apps/nes_emulator/nes_master.snask --profile systems --output nes
./nes
```

Veja benchmarks em [docs/benchmarks/](docs/benchmarks/).

---

## 🤝 Contribuição

Snask está em alpha. Issues, PRs e discussões são bem-vindos.

- Reporte bugs: [GitHub Issues](https://github.com/rancidavi-dotcom/TheSnask/issues)
- Discussões de design: abra uma issue com tag `discussion`
- Documentação: PRs em `docs/` e `docs/site/`

---

<p align="center">
  <img src="Snask.png" alt="Snask" width="48" style="border-radius: 8px;" />
  <br/>
  <strong>Snask</strong> — MIT License
</p>
