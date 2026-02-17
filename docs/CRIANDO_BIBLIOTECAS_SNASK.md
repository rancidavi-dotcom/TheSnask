# 🧩 Criando Bibliotecas em Snask (sem alterar o compilador)

Este guia explica como criar e distribuir bibliotecas **100% em Snask** (arquivos `.snask`), usando apenas o sistema de `import` e o namespace `modulo::funcao()` — **sem mexer no código-fonte do compilador**.

> Observação: bibliotecas que dependem de novas *builtins* (funções nativas do runtime/LLVM) ainda exigem mudanças no compilador/runtime. A proposta aqui é: **tudo que for possível em Snask puro vira biblioteca**, e o compilador fica estável.

Exemplos no repositório:
- Bibliotecas Snask puro: `utils.snask`, `requests.snask`
- Bibliotecas que usam builtins do runtime: `json.snask`, `os.snask`, `blaze.snask`, `blaze_auth.snask`

---

## 1) O que é uma “biblioteca” no Snask?

Uma biblioteca é só um **módulo**: um arquivo `nome.snask` com funções/classes reutilizáveis.

Quando você faz:

```snask
import "minha_lib"
```

o compilador:

1. procura `minha_lib.snask` no diretório do projeto;
2. se não achar, procura em `~/.snask/packages/minha_lib.snask`;
3. e automaticamente aplica namespace: funções viram `minha_lib::minha_funcao()`.

---

## 2) Estrutura mínima de uma biblioteca

Crie um arquivo `math_extra.snask`:

```snask
fun dobro(x)
    return x * 2;

fun soma3(a, b, c)
    return a + b + c;
```

Uso no seu app:

```snask
import "math_extra"

class main
    fun start()
        print(math_extra::dobro(21));
        print(math_extra::soma3(1, 2, 3));
```

Regra prática:
- **defina funções top-level** (`fun nome(...)`) para a API pública;
- se quiser, use `class` internamente, mas exponha funções simples quando possível.

---

## 3) Convenções recomendadas (para evitar dor)

### 3.1 Namespace sempre
Prefira sempre o padrão:
- `minha_lib::funcao()`

Isso evita colisão de nomes com outras libs e com o seu código.

### 3.2 Uma biblioteca = um arquivo
No modelo atual, o `import` carrega **um arquivo por vez**. Então, se sua lib for grande:
- crie arquivos separados (ex.: `http_client.snask`, `http_json.snask`)
- e importe os dois no projeto.

### 3.3 Sem estado global “mágico”
Evite depender de variáveis globais compartilhadas entre arquivos. Prefira:
- funções puras (entrada → saída)
- receber valores como parâmetros

---

## 4) Onde colocar a biblioteca

### Opção A: Local (por projeto)
Coloque `minha_lib.snask` na raiz do projeto (ou ajuste o path no import).

Exemplo:
- `./minha_lib.snask`
- `import "minha_lib"`

### Opção B: Global (na sua máquina)
Coloque o arquivo em:
- `~/.snask/packages/minha_lib.snask`

Agora qualquer projeto pode:
- `import "minha_lib"`

---

## 5) Publicando para outras pessoas (sistema de pacotes)

O Snask já tem um mecanismo de “registry” e instalação via `snask install ...`.
No formato atual, o fluxo típico é:

1. disponibilizar `minha_lib.snask` em um repositório/URL
2. cadastrar no registry (JSON) com nome/descrição/url
3. o usuário instala com `snask install minha_lib`

Se você já usa um registry interno, mantenha o padrão dele. Se quiser, eu posso:
- revisar o formato do seu `registry.json` e
- sugerir um template de entrada para novas libs.

---

## 5.1) Ferramentas oficiais (CLI)

O Snask tem comandos para **criar** e **publicar** bibliotecas no registry oficial (SnaskPackages) sem mexer no compilador.

### Criar template

```bash
snask lib init minha_lib --version 0.1.0 --description "Minha lib de exemplo"
```

Isso gera no diretório atual:
- `minha_lib.snask`
- `minha_lib_README.md`

### Publicar no registry (SnaskPackages)

Pré-requisito: o registry precisa estar clonado em `~/.snask/registry`.
Se ainda não estiver, rode uma vez:

```bash
snask search json
```

Depois publique:

```bash
snask lib publish minha_lib --version 0.1.0 --description "Minha lib de exemplo" --push
```

O publish:
- copia `minha_lib.snask` para `~/.snask/registry/packages/minha_lib.snask`
- cria/atualiza `~/.snask/registry/index/m/minha_lib.json`
- faz `git commit` e (se `--push`) `git push origin main`

---

## 6) Template pronto (copiar e começar)

Crie `minha_lib.snask`:

```snask
// API pública
fun version()
    return "0.1.0";

fun hello(nome)
    return "Olá, " + nome;

// Implementação interna (por convenção: prefixo _)
fun _clamp(n, a, b)
    if n < a
        return a;
    if n > b
        return b;
    return n;
```

Uso:

```snask
import "minha_lib"

class main
    fun start()
        print("Versão:", minha_lib::version());
        print(minha_lib::hello("dev"));
```

---

## 7) Checklist rápido de “pronto pra distribuir”

- O arquivo tem nome simples: `minha_lib.snask`
- Funções públicas têm nomes claros
- Você testou import local e global
- Você evitou dependências de runtime/builtins novas
- Você colocou exemplos mínimos de uso

---

## 8) Limitações atuais (importante)

Algumas coisas ainda não existem como “biblioteca pura” sem tocar no compilador:
- adicionar novas funções nativas (ex.: criptografia “de verdade”, sockets, etc.)
- adicionar novos tipos/representações no backend LLVM

Nesses casos, a alternativa é:
- expor o que der via Snask puro agora
- e só quando for *essencial*, propor uma builtin nova (mudança de runtime/compilador).
