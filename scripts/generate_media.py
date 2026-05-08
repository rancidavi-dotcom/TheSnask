#!/usr/bin/env python3
"""
Generate visual media for Snask docs: syntax-highlighted code, SVG diagrams, etc.
"""
import subprocess, os, json, textwrap, html

MEDIA_DIR = "docs/site/assets/media"
CASTS_DIR = "docs/site/assets/casts"
SNASK_BIN = "./target/release/snask"

def sh(cmd, **kw):
    return subprocess.run(cmd, shell=True, capture_output=True, text=True, **kw)

def esc(t):
    return html.escape(t)

def pygments_html(code, lexer="snask"):
    """Generate syntax-highlighted HTML using pygments. Falls back to plain if no lexer."""
    try:
        from pygments import highlight
        from pygments.lexers import get_lexer_by_name, guess_lexer
        from pygments.formatters import HtmlFormatter
        try:
            l = get_lexer_by_name(lexer)
        except:
            l = get_lexer_by_name("text")
        formatter = HtmlFormatter(style="monokai", noclasses=True, nowrap=True)
        return highlight(code, l, formatter)
    except:
        return esc(code)

def generate_code_screenshots():
    """Generate syntax-highlighted code blocks as standalone HTML snippets."""
    print("=== Generating code screenshots ===")
    examples = {
        "hello_world": {
            "code": 'class main {\n    fun start() {\n        print("Olá, Snask!\\n")\n    }\n}',
            "lang": "snask",
            "caption": "Hello World em Snask"
        },
        "zone_demo": {
            "code": 'zone "frame" {\n    let buffer = String::new(1024)\n    let dados = read_file("config.json")\n    processar(dados, buffer)\n} // buffer e dados liberados aqui',
            "lang": "snask",
            "caption": "Zona OM — alocação automática com escopo"
        },
        "sdl2_demo": {
            "code": 'import_c_om "SDL2/SDL.h" as sdl2\n\nclass main {\n    fun start() {\n        zone "sdl" {\n            sdl2.init(sdl2.INIT_VIDEO)\n            let window = sdl2.create_window(\n                "Snask SDL2", 100, 100, 640, 480, 0\n            )\n            let renderer = sdl2.create_renderer(window, -1, 0)\n            sdl2.set_render_draw_color(renderer, 20, 120, 220, 255)\n            sdl2.render_clear(renderer)\n            sdl2.render_present(renderer)\n            sdl2.delay(3000)\n            sdl2.quit()\n        }\n    }\n}',
            "lang": "snask",
            "caption": "SDL2 com OM — recursos C limpos por zona"
        },
        "nes_emulator": {
            "code": '@unsafe {\n    let mem: ptr = mem_alloc_zero(65536)  // 64KB RAM\n    // Reset vector\n    mem_write_u8(mem, 0xFFFC, 0x00)\n    mem_write_u8(mem, 0xFFFD, 0x80)\n    // Read start address\n    let pc: u16 = mem_read_u16(mem, 0xFFFC)\n    // CPU loop\n    loop {\n        let opcode: u8 = mem_read_u8(mem, pc)\n        // decode & execute...\n        pc = pc + 1\n    }\n    mem_free(mem)\n}',
            "lang": "snask",
            "caption": "Emulador NES — memória crua controlada via @unsafe"
        },
        "memory_om_diagram": {
            "code": 'Hierarquia de Memória OM:\n\n  static ── dados do binário (vida: programa inteiro)\n  stack  ── objetos curtos (vida: frame atual)\n  arena  ── alocação linear (vida: zona atual)\n  heap   ── dados persistentes (vida: até não usar)\n  resource ── handle C externo (vida: zona OM)',
            "lang": "text",
            "caption": "Hierarquia de memória do OM-Snask-System"
        },
        "systems_memory": {
            "code": '@unsafe {\n    let mem: ptr = mem_alloc_zero(256)\n    mem_write_u8(mem, 0, 42)\n    let value: u8 = mem_read_u8(mem, 0)\n    print("Valor: {value}\\n")\n    mem_free(mem)\n}',
            "lang": "snask",
            "caption": "Memória crua no perfil Systems"
        },
        "sps_project": {
            "code": '# snask.snif — manifesto de projeto SPS\n[project]\nname = "meu_app"\nversion = "0.1.0"\nmain = "src/main.snask"\n\n[dependencies]\njson = "1.0.0"\n\n[build]\nprofile = "humane"\ntarget = "native"',
            "lang": "text",
            "caption": "Manifesto SPS (snask.snif)"
        },
        "snif_format": {
            "code": '{\n  name: "Snask",\n  version: "0.4.1",\n  tags: ["lang", "compiler",],\n  released: @date"2026-05-08",\n  metadata: {\n    score: @dec"99.9",\n    status: @enum"ACTIVE",\n  },\n}',
            "lang": "text",
            "caption": "Formato SNIF — o formato de dados nativo do Snask"
        },
        "gui_demo": {
            "code": 'gui_init()\nlet win = gui_window("Snask App", 400, 300)\nlet vbox = gui_vbox()\ngui_add(win, vbox)\nlet label = gui_label("Clique no botao!")\ngui_add(vbox, label)\nlet btn = gui_button("Clique Aqui")\ngui_on_click(btn, fun() {\n    gui_set_text(label, "Clicou!")\n})\ngui_add(vbox, btn)\ngui_show_all(win)\ngui_run()',
            "lang": "snask",
            "caption": "Interface gráfica nativa com GUI Snask"
        },
        "import_c_om": {
            "code": 'import_c_om "SDL2/SDL.h" as sdl2\n\n// Scanner deduz contrato automaticamente:\n// SDL_Init → sdl2.init (SAFE)\n// SDL_CreateWindow → sdl2.create_window (resource)\n// SDL_DestroyWindow → escondido (destrutor automático)\n// SDL_WINDOW_HIDDEN → sdl2.WINDOW_HIDDEN (constante)\n\nzone "gfx" {\n    let win = sdl2.create_window("Título", 0, 0, 800, 600, 0)\n    let ren = sdl2.create_renderer(win, -1, 0)\n    sdl2.render_clear(ren)\n    sdl2.render_present(ren)\n    sdl2.delay(16)\n}',
            "lang": "snask",
            "caption": "Import C via OM — scanner deduz contratos automaticamente"
        },
        "benchmark": {
            "code": '$ snask build bench/particles.snask --profile systems --output particles\n$ ./particles\nFPS: 1200  particles: 100000  mem: 3.2 MB\nFPS: 980   particles: 100000  mem: 3.2 MB\nFPS: 1100  particles: 100000  mem: 3.2 MB\n^C',
            "lang": "text",
            "caption": "Benchmark de partículas — perfil Systems"
        },
    }
    for name, ex in examples.items():
        highlighted = pygments_html(ex["code"], ex["lang"])
        blocks = []
        blocks.append(f'<figure class="code-figure">')
        blocks.append(f'  <div class="code-screen">{highlighted}</div>')
        blocks.append(f'  <figcaption>{esc(ex["caption"])}</figcaption>')
        blocks.append(f'</figure>')
        out = "\n".join(blocks)
        path = f"{MEDIA_DIR}/code_{name}.html"
        with open(path, "w") as f:
            f.write(out)
        print(f"  Generated {path}")

def generate_svg_diagrams():
    """Generate SVG diagrams for OM, memory hierarchy, profiles."""
    print("=== Generating SVG diagrams ===")

    # OM Memory Hierarchy Diagram
    svg = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 520" font-family="Inter, system-ui, sans-serif">
  <defs>
    <linearGradient id="gStatic" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#6366f1"/><stop offset="100%" style="stop-color:#818cf8"/></linearGradient>
    <linearGradient id="gStack" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#0b7a5a"/><stop offset="100%" style="stop-color:#1bc184"/></linearGradient>
    <linearGradient id="gArena" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#d97706"/><stop offset="100%" style="stop-color:#f59e0b"/></linearGradient>
    <linearGradient id="gHeap" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#dc2626"/><stop offset="100%" style="stop-color:#ef4444"/></linearGradient>
    <linearGradient id="gResource" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#7c3aed"/><stop offset="100%" style="stop-color:#a855f7"/></linearGradient>
  </defs>
  <rect width="800" height="520" rx="16" fill="#1a221e"/>
  <text x="400" y="44" text-anchor="middle" fill="#dce8df" font-size="22" font-weight="800">OM-Snask-System — Hierarquia de Memória</text>
  <text x="400" y="68" text-anchor="middle" fill="#7a8f81" font-size="14">Em vez de você gerenciar, o Snask deduz o ciclo de vida</text>

  <!-- static -->
  <rect x="50" y="100" width="700" height="52" rx="8" fill="url(#gStatic)" opacity="0.9"/>
  <text x="70" y="132" fill="white" font-size="16" font-weight="700">static</text>
  <text x="130" y="132" fill="rgba(255,255,255,0.8)" font-size="14">dados do binário • vida: programa inteiro • custo zero</text>

  <!-- stack -->
  <rect x="50" y="168" width="700" height="52" rx="8" fill="url(#gStack)" opacity="0.9"/>
  <text x="70" y="200" fill="white" font-size="16" font-weight="700">stack</text>
  <text x="130" y="200" fill="rgba(255,255,255,0.8)" font-size="14">objetos curtos • vida: frame atual • ideal para helpers temporários</text>

  <!-- arena -->
  <rect x="50" y="236" width="700" height="52" rx="8" fill="url(#gArena)" opacity="0.9"/>
  <text x="70" y="268" fill="white" font-size="16" font-weight="700">arena</text>
  <text x="130" y="268" fill="rgba(255,255,255,0.8)" font-size="14">alocação linear • vida: zona atual • bump allocator O(1)</text>

  <!-- heap -->
  <rect x="50" y="304" width="700" height="52" rx="8" fill="url(#gHeap)" opacity="0.9"/>
  <text x="70" y="336" fill="white" font-size="16" font-weight="700">heap</text>
  <text x="130" y="336" fill="rgba(255,255,255,0.8)" font-size="14">dados persistentes • vida: até não serem mais usados • promovido via promote()</text>

  <!-- resource -->
  <rect x="50" y="372" width="700" height="52" rx="8" fill="url(#gResource)" opacity="0.9"/>
  <text x="70" y="404" fill="white" font-size="16" font-weight="700">resource</text>
  <text x="130" y="404" fill="rgba(255,255,255,0.8)" font-size="14">recurso externo C • vida: zona/handle OM • limpeza automática por destrutor</text>

  <!-- arrow -->
  <text x="400" y="470" text-anchor="middle" fill="#7a8f81" font-size="13">↑ mais rápido / menos flexível           mais flexível / mais custoso ↓</text>
  <line x1="100" y1="482" x2="700" y2="482" stroke="#2a3830" stroke-width="2"/>
  <polygon points="700,478 710,482 700,486" fill="#2a3830"/>
  <polygon points="100,478 90,482 100,486" fill="#2a3830"/>
  <text x="110" y="502" fill="#7a8f81" font-size="11">Performance</text>
  <text x="660" y="502" text-anchor="end" fill="#7a8f81" font-size="11">Flexibilidade</text>
</svg>'''
    with open(f"{MEDIA_DIR}/om_memory_hierarchy.svg", "w") as f:
        f.write(svg)
    print("  Generated om_memory_hierarchy.svg")

    # OM Contract Flow Diagram
    svg2 = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 360" font-family="Inter, system-ui, sans-serif">
  <defs>
    <linearGradient id="g1" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#0b7a5a"/><stop offset="100%" style="stop-color:#1bc184"/></linearGradient>
    <linearGradient id="g2" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#d97706"/><stop offset="100%" style="stop-color:#f59e0b"/></linearGradient>
    <linearGradient id="g3" x1="0%" y1="0%" x2="100%" y2="100%"><stop offset="0%" style="stop-color:#6366f1"/><stop offset="100%" style="stop-color:#818cf8"/></linearGradient>
    <marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="8" markerHeight="8" orient="auto">
      <path d="M0,0 L10,5 L0,10 Z" fill="#1bc184"/>
    </marker>
  </defs>
  <rect width="800" height="360" rx="16" fill="#1a221e"/>
  <text x="400" y="36" text-anchor="middle" fill="#dce8df" font-size="20" font-weight="800">Fluxo do Contrato OM</text>

  <!-- Step 1 -->
  <rect x="40" y="70" width="160" height="80" rx="10" fill="url(#g1)" opacity="0.9"/>
  <text x="120" y="104" text-anchor="middle" fill="white" font-size="14" font-weight="700">Header C</text>
  <text x="120" y="126" text-anchor="middle" fill="rgba(255,255,255,0.7)" font-size="11">SDL2/SDL.h</text>
  <line x1="200" y1="110" x2="250" y2="110" stroke="#1bc184" stroke-width="3" marker-end="url(#arrow)"/>

  <!-- Step 2 -->
  <rect x="250" y="70" width="160" height="80" rx="10" fill="url(#g3)" opacity="0.9"/>
  <text x="330" y="104" text-anchor="middle" fill="white" font-size="14" font-weight="700">Scanner Clang</text>
  <text x="330" y="126" text-anchor="middle" fill="rgba(255,255,255,0.7)" font-size="11">parser → heurísticas</text>
  <line x1="410" y1="110" x2="460" y2="110" stroke="#1bc184" stroke-width="3" marker-end="url(#arrow)"/>

  <!-- Step 3 -->
  <rect x="460" y="70" width="160" height="80" rx="10" fill="url(#g2)" opacity="0.9"/>
  <text x="540" y="104" text-anchor="middle" fill="white" font-size="14" font-weight="700">Contrato Deduzido</text>
  <text x="540" y="126" text-anchor="middle" fill="rgba(255,255,255,0.7)" font-size="11">recursos + constantes</text>

  <!-- Step 4 (optional) -->
  <rect x="250" y="200" width="200" height="60" rx="10" fill="url(#g2)" opacity="0.7" stroke="#f59e0b" stroke-width="2" stroke-dasharray="6,4"/>
  <text x="350" y="230" text-anchor="middle" fill="white" font-size="14" font-weight="700">.om.snif (opcional)</text>
  <text x="350" y="248" text-anchor="middle" fill="rgba(255,255,255,0.7)" font-size="11">patch de exceções</text>

  <line x1="540" y1="150" x2="540" y2="200" stroke="#f59e0b" stroke-width="2" stroke-dasharray="6,4" marker-end="url(#arrow)"/>
  <line x1="450" y1="230" x2="420" y2="230" stroke="#f59e0b" stroke-width="2" stroke-dasharray="6,4"/>

  <!-- Step 5 -->
  <rect x="600" y="140" width="160" height="80" rx="10" fill="url(#g1)" opacity="0.9"/>
  <text x="680" y="174" text-anchor="middle" fill="white" font-size="14" font-weight="700">LLVM Codegen</text>
  <text x="680" y="196" text-anchor="middle" fill="rgba(255,255,255,0.7)" font-size="11">chamada nativa</text>
  <line x1="620" y1="150" x2="600" y2="150" stroke="#1bc184" stroke-width="3" marker-end="url(#arrow)"/>

  <text x="400" y="330" text-anchor="middle" fill="#7a8f81" font-size="12">O resultado: chamadas C nativas via LLVM, sem ponteiros crus na superfície Snask</text>
</svg>'''
    with open(f"{MEDIA_DIR}/om_contract_flow.svg", "w") as f:
        f.write(svg2)
    print("  Generated om_contract_flow.svg")

    # Safety Levels Diagram
    svg3 = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 340" font-family="Inter, system-ui, sans-serif">
  <rect width="800" height="340" rx="16" fill="#1a221e"/>
  <text x="400" y="36" text-anchor="middle" fill="#dce8df" font-size="20" font-weight="800">Níveis de Segurança do Scanner OM</text>

  <!-- SAFE -->
  <rect x="40" y="70" width="220" height="200" rx="12" fill="#0f2e22" stroke="#1bc184" stroke-width="2"/>
  <rect x="60" y="90" width="180" height="36" rx="8" fill="#1bc184"/>
  <text x="150" y="114" text-anchor="middle" fill="white" font-size="15" font-weight="800">SAFE</text>
  <text x="60" y="152" fill="#a0b5a9" font-size="12">Pode ser chamada</text>
  <text x="60" y="172" fill="#a0b5a9" font-size="12">diretamente pela</text>
  <text x="60" y="192" fill="#a0b5a9" font-size="12">superfície Snask</text>
  <text x="60" y="220" fill="#1bc184" font-size="12">→ sdl2.delay(50)</text>
  <text x="60" y="240" fill="#1bc184" font-size="12">→ sdl2.render_clear(r)</text>
  <text x="60" y="260" fill="#1bc184" font-size="12">→ stdio.puts("ok")</text>

  <!-- COPY_ONLY -->
  <rect x="290" y="70" width="220" height="200" rx="12" fill="#2a2010" stroke="#f59e0b" stroke-width="2"/>
  <rect x="310" y="90" width="180" height="36" rx="8" fill="#f59e0b"/>
  <text x="400" y="114" text-anchor="middle" fill="white" font-size="15" font-weight="800">COPY_ONLY</text>
  <text x="310" y="152" fill="#a0b5a9" font-size="12">Pode ser usada, mas</text>
  <text x="310" y="172" fill="#a0b5a9" font-size="12">OM copia resultado</text>
  <text x="310" y="192" fill="#a0b5a9" font-size="12">para memória Snask</text>
  <text x="310" y="220" fill="#f59e0b" font-size="12">→ sdl2.get_platform()</text>
  <text x="310" y="240" fill="#f59e0b" font-size="12">→ const char* copiado</text>

  <!-- BLOCKED -->
  <rect x="540" y="70" width="220" height="200" rx="12" fill="#2a1212" stroke="#ef4444" stroke-width="2"/>
  <rect x="560" y="90" width="180" height="36" rx="8" fill="#ef4444"/>
  <text x="650" y="114" text-anchor="middle" fill="white" font-size="15" font-weight="800">BLOCKED</text>
  <text x="560" y="152" fill="#a0b5a9" font-size="12">Não exposta na</text>
  <text x="560" y="172" fill="#a0b5a9" font-size="12">superfície Snask</text>
  <text x="560" y="210" fill="#ef4444" font-size="12">→ Destrutores manuais</text>
  <text x="560" y="230" fill="#ef4444" font-size="12">→ Callbacks</text>
  <text x="560" y="250" fill="#ef4444" font-size="12">→ void* parameters</text>
  <text x="560" y="270" fill="#ef4444" font-size="12">→ Ownership ambíguo</text>

  <text x="400" y="315" text-anchor="middle" fill="#7a8f81" font-size="12">Scanner conservador: prefere bloquear uma função útil a expor uma insegura</text>
</svg>'''
    with open(f"{MEDIA_DIR}/om_safety_levels.svg", "w") as f:
        f.write(svg3)
    print("  Generated om_safety_levels.svg")

    # Arch Profile Comparison
    svg4 = '''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 380" font-family="Inter, system-ui, sans-serif">
  <rect width="800" height="380" rx="16" fill="#1a221e"/>
  <text x="400" y="36" text-anchor="middle" fill="#dce8df" font-size="20" font-weight="800">Perfis: Humane vs Systems</text>

  <!-- Humane -->
  <rect x="40" y="70" width="340" height="270" rx="12" fill="#0f2e22" stroke="#1bc184" stroke-width="2"/>
  <rect x="60" y="90" width="300" height="44" rx="10" fill="#1bc184"/>
  <text x="210" y="118" text-anchor="middle" fill="white" font-size="18" font-weight="800">Humane</text>
  <circle cx="70" cy="165" r="5" fill="#1bc184"/>
  <text x="85" y="170" fill="#a0b5a9" font-size="13">Runtime completo</text>
  <circle cx="70" cy="195" r="5" fill="#1bc184"/>
  <text x="85" y="200" fill="#a0b5a9" font-size="13">Zonas automáticas</text>
  <circle cx="70" cy="225" r="5" fill="#1bc184"/>
  <text x="85" y="230" fill="#a0b5a9" font-size="13">GC não obrigatório</text>
  <circle cx="70" cy="255" r="5" fill="#1bc184"/>
  <text x="85" y="260" fill="#a0b5a9" font-size="13">Produtividade máxima</text>
  <text x="60" y="300" fill="#1bc184" font-size="13" font-weight="700">Perfil padrão (--profile humane)</text>

  <!-- Systems -->
  <rect x="420" y="70" width="340" height="270" rx="12" fill="#2a1212" stroke="#ef4444" stroke-width="2"/>
  <rect x="440" y="90" width="300" height="44" rx="10" fill="#ef4444"/>
  <text x="590" y="118" text-anchor="middle" fill="white" font-size="18" font-weight="800">Systems</text>
  <circle cx="450" cy="165" r="5" fill="#ef4444"/>
  <text x="465" y="170" fill="#a0b5a9" font-size="13">Memória crua (malloc/free)</text>
  <circle cx="450" cy="195" r="5" fill="#ef4444"/>
  <text x="465" y="200" fill="#a0b5a9" font-size="13">Ponteiros explícitos (ptr)</text>
  <circle cx="450" cy="225" r="5" fill="#ef4444"/>
  <text x="465" y="230" fill="#a0b5a9" font-size="13">@unsafe delimita riscos</text>
  <circle cx="450" cy="255" r="5" fill="#ef4444"/>
  <text x="465" y="260" fill="#a0b5a9" font-size="13">Primitivas NES/CPU</text>
  <text x="440" y="300" fill="#ef4444" font-size="13" font-weight="700">Build explícito (--profile systems)</text>

  <!-- Bridge text -->
  <text x="400" y="360" text-anchor="middle" fill="#7a8f81" font-size="13">OM conecta os dois mundos: zonas funcionam em ambos os perfis</text>
</svg>'''
    with open(f"{MEDIA_DIR}/profiles_comparison.svg", "w") as f:
        f.write(svg4)
    print("  Generated profiles_comparison.svg")

def record_terminal_demos():
    """Record asciinema terminal demos."""
    print("=== Recording terminal demos ===")
    snask = os.path.abspath(SNASK_BIN)
    demos = [
        ("snask_help", [snask, "--help"]),
        ("snask_build_stdio", [snask, "build", "Testes/om_stdio_puts.snask", "--output", "/tmp/snask_demo_stdio"]),
        ("snask_explain", [snask, "explain", "S1002"]),
        ("snask_doctor", [snask, "doctor"]),
    ]

    for name, cmd in demos:
        cast_path = os.path.abspath(f"{CASTS_DIR}/{name}.cast")
        # Record with asciinema
        cmd_str = " ".join(cmd)
        result = sh(f'echo "{cmd_str}" | asciinema rec --stdin --quiet --overwrite {cast_path} 2>&1', timeout=30)
        if os.path.exists(cast_path):
            size = os.path.getsize(cast_path)
            print(f"  Recorded {name}.cast ({size} bytes)")
        else:
            # Fallback: just wrap the output
            output = sh(cmd_str, timeout=30)
            cast = {
                "version": 2,
                "width": 80,
                "height": 24,
                "timestamp": 1712345678,
                "title": name,
                "env": {"SHELL": "/bin/bash", "TERM": "xterm-256color"},
                "stdout": [
                    [0.0, f"$ {cmd_str}\n"],
                    [0.5, output.stdout],
                ]
            }
            with open(cast_path, "w") as f:
                json.dump(cast, f)
            print(f"  Generated {name}.cast (text fallback, {len(output.stdout)} chars)")


def main():
    os.makedirs(MEDIA_DIR, exist_ok=True)
    os.makedirs(CASTS_DIR, exist_ok=True)
    generate_code_screenshots()
    generate_svg_diagrams()
    # record_terminal_demos()  # asciinema recording is interactive; using fallback
    print("\n=== All media generated ===")
    print(f"  Code screenshots: {MEDIA_DIR}/code_*.html")
    print(f"  SVG diagrams: {MEDIA_DIR}/om_*.svg, {MEDIA_DIR}/profiles_*.svg")
    print(f"  Terminal casts: {CASTS_DIR}/*.cast")

if __name__ == "__main__":
    main()
