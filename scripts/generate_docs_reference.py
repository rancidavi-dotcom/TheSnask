#!/usr/bin/env python3
from __future__ import annotations
import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT_ROOT = ROOT / "docs/site"
OUT_FUNCTIONS = OUT_ROOT / "reference/functions"
OUT_LEARN = OUT_ROOT / "learn"

PROFILE_META = {
    "humane": {"icon": "🛡️", "desc": "Gerenciamento automático de memória via zonas. Seguro por padrão."},
    "systems": {"icon": "⚙️", "desc": "Acesso a ponteiros, memória manual e operações de baixo nível. Exige @unsafe."},
}

STATUS_META = {
    "stable": {"class": "stable", "label": "estável"},
    "partial": {"class": "partial", "label": "parcial"},
    "experimental": {"class": "experimental", "label": "experimental"},
}

SAFETY_META = {
    "segura": "✅ Segura — não pode corromper memória ou causar UB.",
    "exige @unsafe": "⚠️ Exige @unsafe — pode causar UB se mal utilizada.",
}

def esc(s: str) -> str:
    return html.escape(s)

def slug(name: str) -> str:
    return name.replace("_", "-")

CATEGORY_ORDER = [
    "collections", "conversion", "core", "filesystem", "gui",
    "io", "json", "math", "memory", "network", "os", "sfs",
    "snaskgui", "string", "systems", "type",
]

CATEGORY_NAMES = {
    "collections": "Coleções", "conversion": "Conversão", "core": "Núcleo",
    "filesystem": "Sistema de Arquivos", "gui": "GUI (GTK)",
    "io": "Entrada e Saída", "json": "JSON", "math": "Matemática",
    "memory": "Memória", "network": "Rede", "os": "Sistema Operacional",
    "sfs": "SFS (Simple File System)", "snaskgui": "SnaskGUI (Framebuffer)",
    "string": "Strings", "systems": "Systems (Baixo Nível)", "type": "Tipo",
}

def parse_existing_pages() -> dict:
    data = {}
    for f in sorted(OUT_FUNCTIONS.glob("*.html")):
        if f.name == "index.html":
            continue
        text = f.read_text()
        slug_name = f.stem

        entry = {"slug": slug_name}

        m = re.search(r'eyebrow">([^<]+)', text)
        entry["category"] = m.group(1) if m else "core"

        m = re.search(r'<h1>([^<]+)', text)
        entry["name"] = m.group(1) if m else slug_name

        m = re.search(r'class="lead">([^<]+)', text)
        entry["summary"] = m.group(1) if m else ""

        m = re.search(r'class="status\s*([^"]*)"', text)
        entry["status"] = m.group(1).strip() if m else "stable"
        if not entry["status"]:
            entry["status"] = "stable"

        m = re.search(r'perfil:\s*([^<]+)', text)
        entry["profile"] = m.group(1).strip() if m else "humane"

        m = re.search(r'seguranca:\s*([^<]+)', text)
        entry["safety"] = m.group(1).strip() if m else "segura"

        m = re.search(r'Assinatura</h2>\s*<pre><code>([^<]+)', text)
        entry["signature"] = m.group(1).strip() if m else ""

        params = re.findall(r'<tr><td><code>([^<]+)</code></td><td>([^<]*)</td></tr>', text)
        entry["params"] = [(p[0], p[1]) for p in params]

        m = re.search(r'Retorno</h2>\s*<p>([^<]+)', text)
        entry["returns"] = m.group(1).strip() if m else ""

        desc_html = ""
        desc_match = re.search(r'Para que serve</h2>\s*((?:<(?:p|section)[^>]*>[^<]*(?:</(?:p|section)>)?\s*)+)', text)
        if desc_match:
            desc_html = desc_match.group(1)
        entry["description_html"] = desc_html

        m = re.search(r'Exemplo</h2>\s*<pre><code>([^<]+)', text)
        entry["example_code"] = m.group(1).strip() if m else ""

        pitfalls_match = re.search(r'Cuidados</h2>\s*<ul>(.*?)</ul>', text, re.DOTALL)
        if pitfalls_match:
            entry["pitfalls"] = re.findall(r'<li>\s*(.*?)\s*</li>', pitfalls_match.group(1), re.DOTALL)
        else:
            entry["pitfalls"] = []

        related_match = re.search(r'[Ff]un[cç][oõ]es\s+relacionadas</h2>\s*<ul>(.*?)</ul>', text, re.DOTALL)
        if related_match:
            related = re.findall(r'<a href="([^"]+)"', related_match.group(1))
            entry["see_also"] = [r.replace(".html", "") for r in related if not r.startswith("http")]
        else:
            entry["see_also"] = []

        m = re.search(r'Teste real</h2><p>([^<]+)', text)
        entry["test_note"] = m.group(1).strip() if m else ""

        data[slug_name] = entry
    return data

def enrich_bare_functions(data: dict) -> None:
    enrichments = {
        "flatten": {
            "description_html": (
                "<p><code>flatten</code> transforma uma lista que contém sublistas em uma única lista plana.</p>"
                "<p>É amplamente usada em processamento de dados para normalizar estruturas aninhadas, "
                "como resultados de consultas, grids ou agrupamentos.</p>"
            ),
            "pitfalls": [
                "API parcial: pode ter limites com tipos muito aninhados.",
                "Achata apenas um nível de profundidade (shallow flatten).",
            ],
            "returns": "Uma nova lista com todos os elementos das sublistas concatenados em ordem.",
        },
        "calc_eval": {
            "description_html": (
                "<p>Avalia uma expressão matemática simples fornecida como string.</p>"
                "<p>É útil para cenários onde a expressão vem de entrada do usuário, "
                "arquivos de configuração ou scripting dinâmico.</p>"
            ),
        },
        "pow": {
            "description_html": (
                "<p>Calcula a potência de uma base elevada a um expoente.</p>"
                "<p>Equivalente ao <code>pow(x, y)</code> da matemática clássica.</p>"
            ),
        },
        "len": {
            "description_html": (
                "<p>Retorna o tamanho de uma string, lista, objeto dinâmico ou recurso suportado.</p>"
                "<p>Para listas, retorna o número de elementos. Para strings, retorna a quantidade de caracteres. "
                "Para objetos JSON/dinâmicos, retorna o número de chaves de alto nível.</p>"
            ),
        },
        "json_parse": {
            "description_html": (
                "<p>Converte uma string contendo JSON em um valor dinâmico do Snask.</p>"
                "<p>O valor retornado pode ser navegado com <code>json_get</code>, <code>json_index</code> "
                "e testado com <code>is_obj</code>, <code>is_str</code>, etc.</p>"
            ),
        },
    }
    for name, extra in enrichments.items():
        slug_name = slug(name)
        if slug_name in data:
            for k, v in extra.items():
                data[slug_name][k] = v

def load_functions() -> list[dict]:
    data = parse_existing_pages()
    enrich_bare_functions(data)
    funcs = sorted(data.values(), key=lambda f: f.get("name", ""))
    return funcs

def status_badge(status: str) -> str:
    s = STATUS_META.get(status, STATUS_META["partial"])
    return f'<span class="status {s["class"]}">{s["label"]}</span>'

def profile_badge(profile: str) -> str:
    meta = PROFILE_META.get(profile, PROFILE_META["humane"])
    return f'<span class="meta-pill">{meta["icon"]} {profile}</span>'

def safety_badge(safety: str) -> str:
    note = SAFETY_META.get(safety)
    extra = f' <span class="meta-pill-note">{note}</span>' if note else ""
    return f'<span class="meta-pill seguranca">{safety}{extra}</span>'

def render_fn_page(fn: dict, depth: int = 2) -> str:
    name = esc(fn["name"])
    category = esc(fn.get("category", "core"))
    summary = esc(fn.get("summary", ""))
    signature = esc(fn.get("signature", ""))
    status = fn.get("status", "partial")
    profile = fn.get("profile", "humane")
    safety = fn.get("safety", "segura")
    returns = esc(fn.get("returns", ""))
    example_code = fn.get("example_code", "")

    desc_html = fn.get("description_html", f"<p>{summary}</p>")

    params_html = ""
    for pname, pdesc in fn.get("params", []):
        params_html += f"<tr><td><code>{esc(pname)}</code></td><td>{esc(pdesc)}</td></tr>"
    if not params_html:
        params_html = '<tr><td colspan="2">Nenhum parâmetro.</td></tr>'

    pitfalls_html = ""
    for p in fn.get("pitfalls", []):
        pitfalls_html += f"<li>{p}</li>"
    if not pitfalls_html:
        pitfalls_html = "<li>Nenhum cuidado especial documentado.</li>"

    see_also_html = ""
    for ref in fn.get("see_also", []):
        ref_name = ref.replace("-", "_")
        see_also_html += f'<li><a href="{esc(ref)}.html"><code>{esc(ref_name)}</code></a></li>'
    if not see_also_html:
        see_also_html = "<li>Nenhuma função relacionada documentada.</li>"

    example_section = ""
    if example_code:
        example_section = f"""
        <section>
          <h2>Exemplo</h2>
          <pre><code>{esc(example_code)}</code></pre>
        </section>"""

    test_note = fn.get("test_note", "")
    test_section = ""
    if test_note:
        test_section = f'<h2>Teste real</h2><p>{esc(test_note)}</p>'

    profile_note = PROFILE_META.get(profile, PROFILE_META["humane"])
    safety_note = SAFETY_META.get(safety, "")

    content = f"""
    <p class="eyebrow">{category}</p>
    <h1>{name}</h1>
    <p class="lead">{summary}</p>

    <p class="badge-row">
      {status_badge(status)}
      {profile_badge(profile)}
      {safety_badge(safety)}
    </p>

    <section>
      <h2>Assinatura</h2>
      <pre><code>{signature}</code></pre>
    </section>

    <section>
      <h2>Para que serve</h2>
      {desc_html}
    </section>

    <section>
      <h2>Perfil</h2>
      <p><strong>{profile_note['icon']} {profile}</strong> — {profile_note['desc']}</p>
      {f'<p><strong>Segurança:</strong> {safety_note}</p>' if safety_note else ''}
    </section>

    <section>
      <h2>Parâmetros</h2>
      <div class="table-wrap">
        <table>
          <thead><tr><th>Nome</th><th>Descrição</th></tr></thead>
          <tbody>{params_html}</tbody>
        </table>
      </div>
    </section>

    <section>
      <h2>Retorno</h2>
      <p>{returns}</p>
    </section>

    {example_section}

    <section>
      <h2>Cuidados e Edge Cases</h2>
      <ul>{pitfalls_html}</ul>
    </section>

    <section>
      <h2>Funções relacionadas</h2>
      <ul>{see_also_html}</ul>
    </section>

    {test_section}

    <section>
      <h2>Notas</h2>
      <p>
        Esta página é gerada a partir de <code>scripts/generate_docs_reference.py</code>.
        Se a assinatura mudar no compilador, atualize a fonte de dados e regenere a referência.
      </p>
    </section>
    """
    return render_layout(name, content, depth)

def render_layout(title: str, content: str, depth: int = 0, is_index: bool = False) -> str:
    prefix = "../" * depth

    nav_links = "".join(
        f'<a href="{prefix}learn/{p["slug"]}.html">{p["title"]}</a>'
        for p in LEARN_PAGES
    )

    search_header = ""
    if is_index:
        search_header = """
        <div class="search-container">
          <input type="text" id="functionSearch" placeholder="Pesquisar entre 150 funções..." />
        </div>
        """

    return f"""<!doctype html>
<html lang="pt-BR">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>{esc(title)} - Snask Docs</title>
    <link rel="stylesheet" href="{prefix}assets/site.css" />
  </head>
  <body>
    <header class="topbar">
      <a class="brand" href="{prefix}index.html"><span class="mark">S</span> Snask Docs</a>
      <button class="hamburger" aria-label="Menu" onclick="document.querySelector('.nav').classList.toggle('open')">
        <span></span><span></span><span></span>
      </button>
      <nav class="nav">
        <a href="{prefix}index.html">Início</a>
        <a href="{prefix}learn/getting-started.html">Aprender</a>
        <a aria-current="page" href="{prefix}reference/functions/index.html">Funções</a>
        <a href="{prefix}systems/om.html">OM</a>
        <a href="{prefix}tooling/installation.html">Tooling</a>
      </nav>
    </header>
    <div class="shell">
      <aside class="sidebar">
        <h2>Aprender</h2>
        {nav_links}
        <h2>Referência</h2>
        <a href="{prefix}reference/functions/index.html">Índice de Funções</a>
        <a href="{prefix}reference/language.html">Linguagem</a>
        <a href="{prefix}reference/types.html">Tipos</a>
        <a href="{prefix}reference/runtime.html">Runtime</a>
        <a href="{prefix}reference/diagnostics.html">Diagnósticos</a>
      </aside>
      <main class="content doc-page">
        {search_header}
        {content}
      </main>
    </div>
    <script src="{prefix}assets/site.js"></script>
    <script>
      const input = document.getElementById('functionSearch');
      if(input) {{
        input.addEventListener('input', e => {{
          const term = e.target.value.toLowerCase();
          document.querySelectorAll('.function-card').forEach(card => {{
            card.style.display = card.innerText.toLowerCase().includes(term) ? 'block' : 'none';
          }});
        }});
      }}
    </script>
  </body>
</html>"""

LEARN_PAGES = [
    {
        "slug": "getting-started",
        "title": "1. Introdução",
        "content": """
        <p class="eyebrow">Capítulo 1</p>
        <h1>Introdução ao Snask</h1>
        <p class="lead">Snask é uma linguagem moderna, compilada via LLVM, projetada para ser <strong>"Humana por padrão, Systems quando necessário"</strong>.</p>
        <section>
          <h2>A Filosofia Snask</h2>
          <p>Diferente de linguagens que forçam uma escolha entre produtividade com GC ou controle manual perigoso, o Snask introduz o conceito de <strong>Perfis Adaptativos</strong>.</p>
          <ul>
            <li><strong>Perfil Humane:</strong> Focado na experiência do desenvolvedor. Memória gerenciada automaticamente via zonas.</li>
            <li><strong>Perfil Systems:</strong> Ativa primitivas de baixo nível, ponteiros, bits e alocação manual.</li>
          </ul>
        </section>
        <section>
          <h2>Instalação</h2>
          <pre><code>curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
export PATH="$HOME/.snask/bin:$PATH"
snask doctor</code></pre>
        </section>
        """,
    },
    {
        "slug": "basics",
        "title": "2. Variáveis e Tipos",
        "content": """
        <p class="eyebrow">Capítulo 2</p>
        <h1>Variáveis e Sistema de Tipos</h1>
        <p class="lead">Snask possui tipagem estática com inferência poderosa e imutabilidade por padrão.</p>
        <section>
          <h2>Imutabilidade (let)</h2>
          <p>Tudo é imutável por padrão, evitando efeitos colaterais indesejados.</p>
          <pre><code>let pi = 3.1415</code></pre>
        </section>
        <section>
          <h2>Mutabilidade (mut)</h2>
          <p>Use <code>mut</code> apenas quando o estado precisar variar.</p>
          <pre><code>mut contador = 0
contador = contador + 1</code></pre>
        </section>
        """,
    },
    {
        "slug": "control-flow",
        "title": "3. Controle de Fluxo",
        "content": """
        <p class="eyebrow">Capítulo 3</p>
        <h1>Controle de Fluxo</h1>
        <p class="lead">Estruturas de decisão e iteração seguras.</p>
        <pre><code>if score >= 70 {
    print("Aprovado\\n")
}

mut i = 0
while i < 10 {
    i = i + 1
}</code></pre>
        """,
    },
    {
        "slug": "functions",
        "title": "4. Funções",
        "content": """
        <h1>Funções e Modularidade</h1>
        <p>Funções no Snask são tipadas e podem ser organizadas em módulos reutilizáveis.</p>
        <pre><code>fun somar(a: float, b: float) -> float {
    return a + b
}</code></pre>
        """,
    },
    {
        "slug": "memory-om",
        "title": "5. Memória (OM)",
        "content": """
        <h1>Gerenciamento de Memória (OM)</h1>
        <p>O Snask utiliza o <strong>OM-Snask-System</strong> baseado em <strong>Zonas</strong> para gerenciamento automático de memória.</p>
        <pre><code>zone "request" {
    let dados = carregar()
} // Limpeza instantânea!</code></pre>
        """,
    },
    {
        "slug": "systems-profile",
        "title": "6. Perfil Systems",
        "content": """
        <h1>Perfil Systems e Baixo Nível</h1>
        <p>Acesso direto à memória e hardware via <code>@unsafe</code>.</p>
        <pre><code>@unsafe {
    let p: ptr = mem_alloc(1024)
    mem_free(p)
}</code></pre>
        """,
    },
]

def main() -> None:
    OUT_ROOT.mkdir(parents=True, exist_ok=True)
    OUT_FUNCTIONS.mkdir(parents=True, exist_ok=True)
    OUT_LEARN.mkdir(parents=True, exist_ok=True)
    (OUT_ROOT / ".nojekyll").touch()

    (OUT_ROOT / "index.html").write_text(
        render_layout(
            "Snask",
            """
            <div class="hero">
              <p class="eyebrow">Snask Language</p>
              <h1>Performance de Sistemas.<br/>Ergonomia Humana.</h1>
              <p class="lead">Uma linguagem compilada moderna com perfis adaptativos: use o nível de controle que você precisa, nada mais.</p>
              <div class="actions">
                <a href="learn/getting-started.html" class="button primary">Começar Agora</a>
                <a href="reference/functions/index.html" class="button">Ver Funções</a>
              </div>
            </div>
            """,
        ),
        encoding="utf-8",
    )

    for p in LEARN_PAGES:
        (OUT_LEARN / f"{p['slug']}.html").write_text(
            render_layout(p["title"], p["content"], 1), encoding="utf-8"
        )

    funcs = load_functions()

    for fn in funcs:
        slug_name = slug(fn["name"])
        (OUT_FUNCTIONS / f"{slug_name}.html").write_text(
            render_fn_page(fn), encoding="utf-8"
        )

    cat_groups = {}
    for fn in funcs:
        cat = fn.get("category", "core")
        cat_groups.setdefault(cat, []).append(fn)

    index_cards = ""
    for cat in CATEGORY_ORDER:
        if cat not in cat_groups:
            continue
        cat_name = CATEGORY_NAMES.get(cat, cat)
        cards = ""
        for fn in cat_groups[cat]:
            s = esc(fn.get("summary", ""))
            cards += f'<a class="card function-card" href="{slug(fn["name"])}.html"><h3>{esc(fn["name"])}</h3><p>{s}</p></a>'
        if cards:
            index_cards += f'<section class="category-section"><h2>{cat_name}</h2><div class="grid">{cards}</div></section>'

    (OUT_FUNCTIONS / "index.html").write_text(
        render_layout(
            "Índice de Funções",
            f"<div id=\"functionsList\">{index_cards}</div>",
            2,
            True,
        ),
        encoding="utf-8",
    )

    total = len(funcs)
    print(f"Sucesso: {total} páginas de funções e {len(LEARN_PAGES)} capítulos gerados!")

if __name__ == "__main__":
    main()
