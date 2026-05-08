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
    sidebar_chapters = "".join(
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
        {sidebar_chapters}
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
          <h2>Filosofia</h2>
          <p>O Snask nasce de uma constatação: linguagens de alto nível com GC são produtivas mas desperdiçam hardware; linguagens de sistema são eficientes mas perigosas e verbosas. O Snask oferece <strong>Perfis Adaptativos</strong>:</p>
          <ul>
            <li><strong>🛡️ Humane</strong> — padrão. Memória gerenciada automaticamente via zonas. Foco em produtividade e segurança. Ideal para aplicações, scripts, ferramentas.</li>
            <li><strong>⚙️ Systems</strong> — opcional. Acesso a ponteiros, alocação manual, operações de bits e hardware. Exige blocos <code>@unsafe</code>. Ideal para kernels, drivers, emuladores.</li>
          </ul>
          <p>Você começa no perfil Humane e ativa Systems apenas onde precisa. Os dois convivem no mesmo programa.</p>
        </section>

        <section>
          <h2>Instalação</h2>
          <pre><code>curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
export PATH="$HOME/.snask/bin:$PATH"
snask doctor</code></pre>
        </section>

        <section>
          <h2>Primeiro programa</h2>
          <p>Crie um arquivo <code>main.snask</code>:</p>
          <pre><code>class main {
    fun start() {
        print("Olá, Snask!\\n")
    }
}</code></pre>
          <pre><code>snask build main.snask -o Hello && ./Hello</code></pre>
        </section>

        <section>
          <h2>Estrutura de um programa</h2>
          <p>Todo programa Snask precisa de uma classe <code>main</code> com um método <code>start</code> — esse é o ponto de entrada. O runtime chama <code>start</code> após inicializar o gerenciamento de memória e recursos.</p>
          <pre><code>class main {
    // Executado automaticamente pelo runtime
    fun start() {
        // seu código aqui
    }
}</code></pre>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="basics.html" class="path-card"><span class="step">2</span><div><h4>Variáveis e Tipos</h4><p>Entenda o sistema de tipos, imutabilidade e inferência.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "basics",
        "title": "2. Variáveis e Tipos",
        "content": """
        <p class="eyebrow">Capítulo 2</p>
        <h1>Variáveis e Sistema de Tipos</h1>
        <p class="lead">Snask possui tipagem estática, forte e com inferência. Tudo é imutável por padrão.</p>

        <section>
          <h2>Imutabilidade (<code>let</code>)</h2>
          <p>Por padrão, toda variável é imutável. Uma vez atribuída, o valor não pode ser alterado. Isso elimina classes inteiras de bugs.</p>
          <pre><code>let nome = "Snask"
let versao = 0.4
let is_compilada = true

// nome = "Outro"  ← ERRO de compilação!</code></pre>
        </section>

        <section>
          <h2>Mutabilidade (<code>mut</code>)</h2>
          <p>Use <code>mut</code> quando o valor precisar mudar:</p>
          <pre><code>mut contador = 0
contador = contador + 1  // OK
contador = 100           // OK</code></pre>
          <div class="callout">
            <strong>Dica:</strong> quanto menos <code>mut</code>, mais fácil de raciocinar sobre o código. Prefira criar novos valores a modificar existentes.
          </div>
        </section>

        <section>
          <h2>Tipos primitivos</h2>
          <table>
            <thead><tr><th>Tipo</th><th>Descrição</th><th>Exemplo</th></tr></thead>
            <tbody>
              <tr><td><code>float</code></td><td>Número (inteiro ou ponto flutuante, 64 bits)</td><td><code>42</code>, <code>3.14</code></td></tr>
              <tr><td><code>str</code></td><td>String UTF-8</td><td><code>"olá"</code></td></tr>
              <tr><td><code>bool</code></td><td>Booleano</td><td><code>true</code>, <code>false</code></td></tr>
              <tr><td><code>list</code></td><td>Lista dinâmica</td><td><code>[1, 2, 3]</code></td></tr>
              <tr><td><code>any</code></td><td>Valor dinâmico (JSON)</td><td><code>json_parse(...)</code></td></tr>
              <tr><td><code>ptr</code></td><td>Ponteiro bruto (perfil Systems)</td><td><code>mem_alloc(64)</code></td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h2>Inferência de tipos</h2>
          <p>O Snask infere o tipo automaticamente na maioria dos casos. Você só precisa anotar quando o tipo não é óbvio:</p>
          <pre><code>let a = 10          // float (padrão)
let b: float = 10   // explícito
let c: ptr = mem_alloc(64)  // necessário para ptr

// Strings usam aspas duplas
let texto = "Snask"</code></pre>
        </section>

        <section>
          <h2>Conversão entre tipos</h2>
          <p>O Snask não faz conversão implícita. Use funções de cast:</p>
          <pre><code>let num = str_to_num("42")      // str → float
let texto = num_to_str(3.14)    // float → str
let val = json_parse("{\\"a\\":1}") // str → any (JSON)</code></pre>
          <p>No perfil Systems, casts numéricos explícitos:</p>
          <pre><code>let x: float = 255
let byte = as_u8(x)   // float → u8
let word = as_u16(x)  // float → u16</code></pre>
        </section>

        <section>
          <h2>Comentários</h2>
          <pre><code>// Comentário de linha única

/* Comentário
   multi-linha */</code></pre>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="control-flow.html" class="path-card"><span class="step">3</span><div><h4>Controle de Fluxo</h4><p>if, while, match e loops.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "control-flow",
        "title": "3. Controle de Fluxo",
        "content": """
        <p class="eyebrow">Capítulo 3</p>
        <h1>Controle de Fluxo</h1>
        <p class="lead">Estruturas condicionais e de repetição para controlar o fluxo do programa.</p>

        <section>
          <h2>Condicional <code>if</code></h2>
          <p>O <code>if</code> executa um bloco se a condição for verdadeira. Suporta <code>else</code> e <code>else if</code>:</p>
          <pre><code>let nota = 85

if nota >= 90 {
    print("Excelente!\\n")
} else if nota >= 70 {
    print("Aprovado\\n")
} else {
    print("Recuperação\\n")
}</code></pre>
        </section>

        <section>
          <h2>Laço <code>while</code></h2>
          <p>O <code>while</code> repete um bloco enquanto a condição for verdadeira:</p>
          <pre><code>mut i = 0
while i < 5 {
    print("Contagem: {i}\\n")
    i = i + 1
}</code></pre>
        </section>

        <section>
          <h2>Laço <code>for</code> / <code>range</code></h2>
          <p>Use <code>range</code> para iterar sobre sequências numéricas:</p>
          <pre><code>for i in range(5) {
    print("{i}\\n")  // 0, 1, 2, 3, 4
}

// Iterar sobre lista
let itens = [10, 20, 30]
for item in itens {
    print("{item}\\n")
}</code></pre>
        </section>

        <section>
          <h2><code>match</code> (casamento de padrão)</h2>
          <p>O <code>match</code> compara um valor contra múltiplos padrões:</p>
          <pre><code>let cmd = "start"
match cmd {
    "start"  => print("Iniciando...\\n")
    "stop"   => print("Parando...\\n")
    "status" => print("Ativo\\n")
    else     => print("Comando desconhecido: {cmd}\\n")
}</code></pre>
          <p>O <code>else</code> é obrigatório e cobre todos os outros casos.</p>
        </section>

        <section>
          <h2><code>break</code> e <code>continue</code></h2>
          <pre><code>mut i = 0
while i < 10 {
    i = i + 1
    if i == 3 { continue }  // pula o 3
    if i == 7 { break }     // para no 7
    print("{i}\\n")
}
// Saída: 1, 2, 4, 5, 6</code></pre>
        </section>

        <section>
          <h2>Operadores de comparação</h2>
          <table>
            <thead><tr><th>Operador</th><th>Significado</th></tr></thead>
            <tbody>
              <tr><td><code>==</code></td><td>Igual</td></tr>
              <tr><td><code>!=</code></td><td>Diferente</td></tr>
              <tr><td><code>&lt;</code> <code>&gt;</code></td><td>Menor / Maior</td></tr>
              <tr><td><code>&lt;=</code> <code>&gt;=</code></td><td>Menor igual / Maior igual</td></tr>
              <tr><td><code>&amp;&amp;</code></td><td>E lógico</td></tr>
              <tr><td><code>||</code></td><td>Ou lógico</td></tr>
              <tr><td><code>!</code></td><td>Não lógico</td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="functions.html" class="path-card"><span class="step">4</span><div><h4>Funções</h4><p>Funções, módulos e organização de código.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "functions",
        "title": "4. Funções e Módulos",
        "content": """
        <p class="eyebrow">Capítulo 4</p>
        <h1>Funções e Módulos</h1>
        <p class="lead">Organize seu código em funções reutilizáveis e módulos.</p>

        <section>
          <h2>Declarando funções</h2>
          <p>Use <code>fun</code> para declarar uma função. Parâmetros e retorno são tipados:</p>
          <pre><code>fun somar(a: float, b: float) -> float {
    return a + b
}

fun saudacao(nome: str) {
    print("Olá, {nome}!\\n")
}</code></pre>
        </section>

        <section>
          <h2>Funções sem retorno</h2>
          <p>Se a função não retorna nada, omite a seta:</p>
          <pre><code>fun log(mensagem: str) {
    print("[LOG] {mensagem}\\n")
}</code></pre>
        </section>

        <section>
          <h2>Escopo e variáveis locais</h2>
          <p>Variáveis declaradas dentro de uma função são locais a ela:</p>
          <pre><code>fun calcular() -> float {
    let tmp = 42      // local
    let resultado = tmp * 2
    return resultado
}
// tmp não existe aqui</code></pre>
        </section>

        <section>
          <h2>Módulos e <code>import</code></h2>
          <p>Organize código em módulos com <code>import</code>:</p>
          <pre><code>// arquivo: math.snask
fun quadrado(x: float) -> float {
    return x * x
}

// arquivo: main.snask
import "math"

class main {
    fun start() {
        let q = math::quadrado(5) // 25
        print("{q}\\n")
    }
}</code></pre>
          <p>A sintaxe <code>modulo::funcao</code> acessa itens do módulo importado.</p>
        </section>

        <section>
          <h2>Funções como valores</h2>
          <p>Funções podem ser passadas como argumento (callbacks):</p>
          <pre><code>fun executar(f: any) {
    // f é uma referência de função
    f()
}

fun minha_fun() {
    print("Chamado!\\n")
}

executar(minha_fun)</code></pre>
        </section>

        <section>
          <h2>Boas práticas</h2>
          <ul>
            <li>Funções pequenas e com propósito único (SRP).</li>
            <li>Nomes descritivos no padrão <code>snake_case</code>.</li>
            <li>Evite funções com mais de 30 linhas — extraia.</li>
            <li>Use <code>import</code> para separar domínios.</li>
          </ul>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="collections.html" class="path-card"><span class="step">5</span><div><h4>Coleções e Strings</h4><p>Listas, strings, JSON e operações.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "collections",
        "title": "5. Coleções e Strings",
        "content": """
        <p class="eyebrow">Capítulo 5</p>
        <h1>Coleções e Strings</h1>
        <p class="lead">Trabalhe com listas, strings, JSON e estruturas de dados.</p>

        <section>
          <h2>Listas</h2>
          <p>Listas são coleções dinâmicas. Suportam tipos mistos no perfil Humane:</p>
          <pre><code>let vazia = []
let numeros = [1, 2, 3, 4, 5]
let mista = [42, "texto", true]

print(len(numeros))  // 5
print(numeros[0])    // 1</code></pre>

          <h3>Operações com listas</h3>
          <table>
            <thead><tr><th>Função</th><th>Descrição</th></tr></thead>
            <tbody>
              <tr><td><code>len(x)</code></td><td>Retorna o tamanho</td></tr>
              <tr><td><code>sort(x)</code></td><td>Ordena a lista</td></tr>
              <tr><td><code>reverse(x)</code></td><td>Inverte a ordem</td></tr>
              <tr><td><code>flatten(x)</code></td><td>Achata listas aninhadas</td></tr>
              <tr><td><code>contains(x, v)</code></td><td>Verifica se contém valor</td></tr>
              <tr><td><code>join(x, sep)</code></td><td>Junta em string</td></tr>
              <tr><td><code>unique(x)</code></td><td>Remove duplicatas</td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h2>Strings</h2>
          <p>Strings são UTF-8 e suportam interpolação com <code>{}</code>:</p>
          <pre><code>let nome = "Snask"
let versao = 0.4
let msg = "Bem-vindo ao {nome} v{versao}!"
print("{msg}\\n")

// Operações
let tamanho = len("texto")           // 5
let sub = substring("Snask", 0, 2)   // "Sn"
let maiusculo = upper("snask")       // "SNASK"
let minusculo = lower("SNASK")       // "snask"
let tem_prefixo = starts_with("Snask", "Sn")  // true
let tem_sufixo = ends_with("Snask", "sk")     // true
let trim = trim("  espaco  ")        // "espaco"
let partes = split("a,b,c", ",")     // ["a", "b", "c"]
let trocado = replace("a-a-a", "-", "+") // "a+a+a"</code></pre>
        </section>

        <section>
          <h2>JSON</h2>
          <p>O Snask tem suporte nativo a JSON no perfil Humane:</p>
          <pre><code>let texto = '{"nome": "Snask", "ano": 2024}'

// Parse
let obj = json_parse(texto)

// Acessar campos
let nome = json_get(obj, "nome")   // "Snask"
let ano = json_get(obj, "ano")     // 2024.0

// Verificar tipo
let is_str = is_str(nome)   // true
let is_obj = is_obj(obj)    // true

// Serializar
let saida = json_stringify(obj)</code></pre>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="memory-om.html" class="path-card"><span class="step">6</span><div><h4>Memória (OM)</h4><p>Gerenciamento automático com zonas.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "memory-om",
        "title": "6. Memória (OM)",
        "content": """
        <p class="eyebrow">Capítulo 6</p>
        <h1>Gerenciamento de Memória com OM</h1>
        <p class="lead">O OM (Orquestrador de Memória) gerencia o ciclo de vida de alocações e recursos automaticamente através de zonas.</p>

        <section>
          <h2>O problema</h2>
          <p>Em C, você precisa lembrar de chamar <code>free</code> para cada <code>malloc</code>. Um esquecimento vaza memória; um <code>free</code> a mais quebra o programa. Em linguagens com GC, você não se preocupa, mas paga o preço em pausas e uso de memória.</p>
          <p>O OM oferece um terceiro caminho: <strong>alocação regional</strong>. Você declara uma zona e tudo que aloca dentro dela é liberado automaticamente quando a zona fecha.</p>
        </section>

        <section>
          <h2>Zonas</h2>
          <p>Uma zona é um escopo nomeado que agrupa alocações:</p>
          <pre><code>zone "frame" {
    let buffer = read_file("dados.txt")
    processar(buffer)
} // buffer liberado automaticamente</code></pre>
          <p>No perfil Humane, todas as funções que alocam recursos (arquivos, memória, handles) têm seus ciclos de vida atrelados à zona ativa.</p>
        </section>

        <section>
          <h2>Zonas aninhadas</h2>
          <p>A zona interna libera recursos ao fechar, mas a externa mantém os dela:</p>
          <pre><code>zone "request" {
    let req = parse_http(input)
    zone "response" {
        let resp = build_response(req)
        send(resp)
    } // resp liberado, req ainda vivo
} // req liberado</code></pre>
        </section>

        <section>
          <h2>Boas práticas com zonas</h2>
          <ul>
            <li><strong>Zonas curtas:</strong> uma zona deve viver o mínimo necessário. Quanto antes fechar, menos memória retida.</li>
            <li><strong>Zonas nomeadas:</strong> use nomes semânticos como <code>"frame"</code>, <code>"request"</code>, <code>"batch"</code>.</li>
            <li><strong>Aninhamento máximo:</strong> evite mais de 3 níveis. Extraia funções.</li>
            <li><strong>Combinar com funções:</strong> cada função pode ter suas próprias zonas internas.</li>
          </ul>
        </section>

        <section>
          <h2>No perfil Systems</h2>
          <p>No perfil Systems, você tem controle manual da memória com <code>mem_alloc</code>, <code>mem_free</code>, ponteiros e <code>@unsafe</code>. O OM ainda pode ajudar com zonas para agrupar alocações manuais, mas a liberação é responsabilidade sua:</p>
          <pre><code>@unsafe {
    let p: ptr = mem_alloc(1024)
    mem_write_u8(p, 0, 42)
    let val = mem_read_u8(p, 0) // 42
    mem_free(p)
}</code></pre>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="io-networking.html" class="path-card"><span class="step">7</span><div><h4>IO e Rede</h4><p>Entrada/saída, arquivos, HTTP.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "io-networking",
        "title": "7. IO e Rede",
        "content": """
        <p class="eyebrow">Capítulo 7</p>
        <h1>Entrada/Saída e Rede</h1>
        <p class="lead">Comunique-se com o mundo externo: console, arquivos e requisições HTTP.</p>

        <section>
          <h2>Saída no console</h2>
          <pre><code>print("texto")        // sem quebra de linha
println()              // apenas quebra de linha

// Interpolação
let nome = "Snask"
print("Olá, {nome}!\\n")

// Múltiplos valores
print("Contagem: "); print(42); print("\\n")</code></pre>
        </section>

        <section>
          <h2>Arquivos</h2>
          <p>O Snask oferece funções para leitura e escrita de arquivos:</p>
          <pre><code>// Leitura completa
let config = read_file("settings.json")
print("Tamanho: {len(config)} bytes\\n")

// Escrita (sobrescreve)
write_file("saida.txt", "conteúdo")

// Acrescentar ao final
append_file("log.txt", "nova entrada\\n")

// Trabalhar com diretórios
let entradas = read_dir(".")
for item in entradas {
    if is_file(item) { print("Arquivo: {item}\\n") }
    if is_dir(item)  { print("Diretório: {item}\\n") }
}

// Testar existência
if exists("config.json") {
    print("Arquivo existe\\n")
}</code></pre>
        </section>

        <section>
          <h2>SFS (Simple File System)</h2>
          <p>O SFS oferece operações adicionais de arquivo:</p>
          <pre><code>sfs_copy("origem.txt", "destino.txt")
sfs_move("antigo.txt", "novo.txt")
sfs_delete("temporario.txt")
sfs_mkdir("pasta/nova")
sfs_rmdir("pasta/antiga")
sfs_size("arquivo.bin")    // tamanho em bytes
sfs_mtime("arquivo.txt")   // timestamp de modificação</code></pre>
        </section>

        <section>
          <h2>Requisições HTTP</h2>
          <pre><code>let res = http_get("https://api.github.com/zen")
print("Status: {json_get(res, \\"status\\")}\\n")
print("Body: {json_get(res, \\"body\\")}\\n")

// http_post para enviar dados
// let res = http_post("https://api.exemplo.com/dados", corpo)</code></pre>
          <div class="callout warn">
            <strong>Nota:</strong> requisições HTTP são bloqueantes. O runtime precisa ter suporte a rede e certificados SSL.
          </div>
        </section>

        <section>
          <h2>Sistema</h2>
          <pre><code>let agora = time()           // timestamp atual
let so = platform()           // nome do SO
let args = args()             // argumentos da linha de comando
let vars = env()              // variáveis de ambiente
let dir = cwd()               // diretório atual
set_env("PATH", "/usr/bin")   // definir variável de ambiente
exit(0)                       // sair do programa
sleep(1000)                   // pausar por 1 segundo</code></pre>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="gui.html" class="path-card"><span class="step">8</span><div><h4>GUI</h4><p>Interfaces gráficas com GTK e framebuffer.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "gui",
        "title": "8. GUI (Gráficos)",
        "content": """
        <p class="eyebrow">Capítulo 8</p>
        <h1>Interface Gráfica</h1>
        <p class="lead">Crie interfaces gráficas com widgets GTK (<code>gui_*</code>) ou renderização pixel a pixel (<code>snaskgui_*</code>).</p>

        <section>
          <h2>GUI com Widgets (GTK)</h2>
          <p>Use o sistema <code>gui_*</code> para criar interfaces com botões, campos de texto, labels e containers:</p>
          <pre><code>class main {
    fun start() {
        gui_init()   // inicializa o runtime GTK

        let win = gui_window("Minha Janela", 400, 300)
        let vbox = gui_vbox()
        let label = gui_label("Clique no botão:")
        let btn = gui_button("OK")

        gui_on_click(btn, fun() {
            print("Clicou!\\n")
        })

        gui_add(vbox, label)
        gui_add(vbox, btn)
        gui_set_child(win, vbox)
        gui_show_all(win)
        gui_run()    // entra no loop principal
    }
}</code></pre>

          <h3>Widgets disponíveis</h3>
          <table>
            <thead><tr><th>Função</th><th>Descrição</th></tr></thead>
            <tbody>
              <tr><td><code>gui_window</code></td><td>Cria janela</td></tr>
              <tr><td><code>gui_button</code></td><td>Botão</td></tr>
              <tr><td><code>gui_label</code></td><td>Texto informativo</td></tr>
              <tr><td><code>gui_entry</code></td><td>Campo de texto</td></tr>
              <tr><td><code>gui_textview</code></td><td>Área de texto multi-linha</td></tr>
              <tr><td><code>gui_hbox</code></td><td>Container horizontal</td></tr>
              <tr><td><code>gui_vbox</code></td><td>Container vertical</td></tr>
              <tr><td><code>gui_get_text</code></td><td>Lê texto de widget</td></tr>
              <tr><td><code>gui_set_text</code></td><td>Define texto de widget</td></tr>
              <tr><td><code>gui_on_click</code></td><td>Conecta evento de clique</td></tr>
              <tr><td><code>gui_add</code></td><td>Adiciona filho a container</td></tr>
              <tr><td><code>gui_set_child</code></td><td>Define filho único</td></tr>
              <tr><td><code>gui_show_all</code></td><td>Mostra widget e filhos</td></tr>
            </tbody>
          </table>
        </section>

        <section>
          <h2>SnaskGUI (Framebuffer)</h2>
          <p>O sistema <code>snaskgui_*</code> cria janelas de pixels para jogos, emuladores e visualizações. Você controla cada pixel individualmente:</p>
          <pre><code>class main {
    fun start() {
        snaskgui::init()
        let win = snaskgui::window("Jogo", 320, 200, 2) // 640x400 físico

        while snaskgui::should_close(win) == false {
            snaskgui::poll(win)
            // Desenhar pixel a pixel
            // snaskgui::present_rgba(win, buffer)
            snaskgui::delay(16)  // ~60 FPS
        }
        snaskgui::close(win)
    }
}</code></pre>

          <table>
            <thead><tr><th>Função</th><th>Descrição</th></tr></thead>
            <tbody>
              <tr><td><code>snaskgui::init</code></td><td>Inicializa o sistema</td></tr>
              <tr><td><code>snaskgui::window</code></td><td>Cria janela (largura, altura, escala)</td></tr>
              <tr><td><code>snaskgui::poll</code></td><td>Processa eventos</td></tr>
              <tr><td><code>snaskgui::present_rgba</code></td><td>Envia buffer de pixels</td></tr>
              <tr><td><code>snaskgui::should_close</code></td><td>Verifica se deve fechar</td></tr>
              <tr><td><code>snaskgui::delay</code></td><td>Espera milissegundos</td></tr>
              <tr><td><code>snaskgui::key_down</code></td><td>Verifica se tecla está pressionada</td></tr>
              <tr><td><code>snaskgui::close</code></td><td>Fecha a janela</td></tr>
            </tbody>
          </table>

          <div class="callout warn">
            <strong>Perfil:</strong> SnaskGUI é do perfil Systems (<code>@unsafe</code> quando usar ponteiros). Widgets GTK são perfil Humane.
          </div>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="systems-profile.html" class="path-card"><span class="step">9</span><div><h4>Perfil Systems</h4><p>Baixo nível: ponteiros, bits, unsafe.</p></div></a>
          </div>
        </section>
        """,
    },
    {
        "slug": "systems-profile",
        "title": "9. Perfil Systems",
        "content": """
        <p class="eyebrow">Capítulo 9</p>
        <h1>Perfil Systems e Baixo Nível</h1>
        <p class="lead">Quando você precisa de controle total sobre memória, bits e hardware, o perfil Systems oferece as primitivas necessárias.</p>

        <section>
          <h2>Bloco <code>@unsafe</code></h2>
          <p>Toda operação de baixo nível deve estar dentro de um bloco <code>@unsafe</code>. Isso isola o código perigoso e documenta visualmente onde a segurança manual é necessária:</p>
          <pre><code>@unsafe {
    // Operações que podem causar UB se mal utilizadas
    let p: ptr = mem_alloc(64)
    mem_free(p)
}</code></pre>
        </section>

        <section>
          <h2>Memória manual</h2>
          <pre><code>@unsafe {
    // Alocar
    let buf: ptr = mem_alloc(1024)
    let zerado: ptr = mem_alloc_zero(512)

    // Ler/escrever bytes
    mem_write_u8(buf, 0, 255)
    let byte = mem_read_u8(buf, 0)   // 255

    // Ler/escrever words (16 bits)
    mem_write_u16(buf, 4, 0xAABB)
    let word = mem_read_u16(buf, 4)

    // Ler/escrever dwords (32 bits)
    mem_write_u32(buf, 8, 0xDEADBEEF)

    // Preencher bloco
    mem_fill_u8(buf, 0, 1024, 0)     // zera tudo

    // Copiar entre blocos
    mem_copy(dest, src, tamanho)

    // Liberar
    mem_free(buf)
    mem_free(zerado)
}</code></pre>
          <div class="callout danger">
            <strong>Perigo:</strong> acesso fora dos limites (out-of-bounds) causa falha de segmentação. Sempre verifique os tamanhos.
          </div>
        </section>

        <section>
          <h2>Aritmética wrapping</h2>
          <p>Operações wrapping não estouram — em vez disso, "viram" (wrap around):</p>
          <pre><code>let max: float = 255
let byte = as_u8(max)

// Wrapping: 255 + 1 = 0 (em u8)
let wrap = wrapping_add(byte, 1)  // 0

// Wrapping: 0 - 1 = 255
let wrap2 = wrapping_dec(byte)    // 254

// Sub, mul, inc
let r1 = wrapping_sub(100, 50)
let r2 = wrapping_mul(16, 16)
let r3 = wrapping_inc(99)</code></pre>
        </section>

        <section>
          <h2>Aritmética saturating</h2>
          <p>Saturating limita ao valor máximo/mínimo em vez de wrap:</p>
          <pre><code>let byte = as_u8(200)
let sat = saturating_add(byte, 100)  // 255 (máximo de u8)</code></pre>
        </section>

        <section>
          <h2>Aritmética com carry/borrow</h2>
          <pre><code>// Carry de soma de 8 bits (simula ADC de hardware)
let carry = carry_add_u8(255, 1, 0)  // 1 (houve carry)

// Borrow de subtração de 8 bits (simula SBB)
let borrow = borrow_sub_u8(0, 1, 0)  // 0xFF (borrow)</code></pre>
        </section>

        <section>
          <h2>Operações de bits</h2>
          <pre><code>let flags: float = 0b00001111

// Testar bit
let tem_bit0 = bit_test(flags, 0)  // true
let tem_bit4 = bit_test(flags, 4)  // false

// Definir/limpar
let com_bit5 = bit_set(flags, 5)    // 0b00101111
let sem_bit0 = bit_clear(flags, 0)  // 0b00001110

// Alternar
let invertido = bit_toggle(flags, 3) // 0b00000111

// Escrever valor em bit
let novo = bit_write(flags, 0, false) // 0b00001110</code></pre>
        </section>

        <section>
          <h2>Operações com sinal (overflow)</h2>
          <pre><code>// Overflow detectável em i8
let resultado = overflow_add_i8(120, 10)  // estoura o range i8
let resultado2 = overflow_sub_i8(-120, 10) // estoura o range i8</code></pre>
        </section>

        <section>
          <h2>Ponteiros e aritmética</h2>
          <pre><code>@unsafe {
    let buf: ptr = mem_alloc(64)

    // Avançar ponteiro
    let p2 = ptr_add(buf, 4)
    mem_write_u8(p2, 0, 42)

    // Ler de volta
    let val = mem_read_u8(buf, 4)  // 42
    mem_free(buf)
}</code></pre>
        </section>

        <section>
          <h2>Quando usar Systems?</h2>
          <ul>
            <li><strong>Emuladores</strong> — acesso direto a memória mapeada da CPU emulada.</li>
            <li><strong>Drivers</strong> — manipular registradores de hardware.</li>
            <li><strong>Processamento de áudio/vídeo</strong> — buffers raw de pixels/amostras.</li>
            <li><strong>Serialização binária</strong> — ler/escrever estruturas byte a byte.</li>
            <li><strong>Interop com C</strong> — chamar bibliotecas que exigem ponteiros.</li>
          </ul>
        </section>

        <section>
          <h2>Próximos passos</h2>
          <div class="learning-path">
            <a href="../tooling/installation.html" class="path-card"><span class="step">10</span><div><h4>Tooling</h4><p>CLI, build, testes, LSP e Neovim.</p></div></a>
          </div>
        </section>
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
        redirect_html = f"""<!doctype html>
<html lang="pt-BR"><head><meta charset="utf-8"/>
<title>{esc(fn['name'])} - Snask Docs</title>
<meta http-equiv="refresh" content="0;url=functions/{slug_name}"/>
<script>location.href="functions/{slug_name}"</script>
</head><body><a href="functions/{slug_name}">Redirecionando...</a></body></html>"""
        (OUT_ROOT / "reference" / f"{slug_name}.html").write_text(redirect_html, encoding="utf-8")

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
