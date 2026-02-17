# 📚 Guia de Bibliotecas Snask (v0.3.0)

O Snask utiliza um sistema de módulos com **Namespaces por padrão**. Ao importar uma biblioteca, você acessa suas funções usando o prefixo `nome_da_lib::`.

Exceção: `prelude` foi feita para ser importada e usada **sem prefixo** (ergonomia).

---

## 0. Biblioteca: `prelude` ✅
Helpers “de sempre”: `println`, `dbg`, `assert`, `expect`, Result-like (`ok/err/unwrap/unwrap_or`), `is_some`, `path_get`.

### Exemplo de Uso:
```snask
import "prelude"

class main
    fun main()
        println("ok");
        assert(1 + 1 == 2, "math");
```

---

## 1. Biblioteca: `requests` 🌐
Biblioteca HTTP simples para comunicação em rede e integração com APIs Web (wrappers das funções nativas `s_http_*`).

### Funções Disponíveis:
*   `requests::get(url)`: Realiza uma requisição GET.
*   `requests::post(url, dados)`: Envia dados via POST (corpo como string).
*   `requests::put(url, dados)`: Atualiza recursos via PUT.
*   `requests::patch(url, dados)`: Modifica recursos via PATCH.
*   `requests::delete(url)`: Remove recursos via DELETE.

### Exemplo de Uso:
```snask
import "requests"

// Buscando o registro oficial de pacotes
let url = "https://raw.githubusercontent.com/rancidavi-dotcom/SnaskPackages/main/registry.json";
let resposta = requests::get(url);

print("Conteúdo do Registry:", resposta);
```

---

## 2. Biblioteca: `sfs` (Snask File System) 📂
Módulo para manipulação de arquivos via runtime nativo `sfs_*`.

### Funções Disponíveis:
*   `sfs::read(path)`: Lê o conteúdo de um arquivo.
*   `sfs::write(path, content)`: Escreve dados (auto-flush garantido).
*   `sfs::exists(path)`: Verifica se o arquivo está no disco.
*   `sfs::delete(path)`: Remove o arquivo permanentemente.

### Exemplo de Uso:
```snask
import "sfs"

sfs::write("logs.txt", "Evento registrado!");
if sfs::exists("logs.txt")
    print("Log carregado:", sfs::read("logs.txt"));
```

---

## 3. Biblioteca: `utils` 🛠️
Utilitários básicos (exemplos simples em Snask puro).

### Funções Disponíveis:
*   `utils::somar(a, b)`: Soma aritmética simples.
*   `utils::calcular_area(raio)`: Área de um círculo (usa constante PI interna).
*   `utils::saudar(nome)`: Imprime uma saudação no terminal.

### Exemplo de Uso:
```snask
import "utils"

utils::saudar("Desenvolvedor");
let area = utils::calcular_area(10);
print("Círculo de raio 10 tem área:", area);
```

---

💡 **Regra de Ouro:** No Snask v0.3.0, a sintaxe `modulo::funcao()` é o padrão para qualquer código importado. Isso garante que seu código seja modular e livre de conflitos!

---

## 4. Biblioteca: `json` 🧩
Utilitários para **serializar** e **parsear** JSON.

### Funções Disponíveis:
*   `json::stringify(valor)`: Converte valores/objetos Snask em JSON.
*   `json::pretty(valor)`: Converte em JSON formatado (indentação 2).
*   `json::parse(texto)`: Faz parse de JSON e retorna um objeto/valor Snask.
*   `json::get(obj, chave)`: Lê um campo por nome (retorna `nil` se não existir).
*   `json::set(obj, chave, valor)`: Define/atualiza um campo (retorna `true/false`).
*   `json::has(obj, chave)`: Verifica se a chave existe (retorna `true/false`).
*   `json::len(obj)`: Quantidade de campos/itens.
*   `json::index(obj, i)`: Acessa item por índice (útil para arrays parseados).

### Exemplo de Uso:
```snask
import "json";

class Pessoa
    let nome = "Davi";
    let idade = 25;

class main
    fun start()
        let p = Pessoa();
        let texto = json::stringify(p);
        print("JSON:", texto);

        let obj = json::parse(texto);
        print("nome:", json::get(obj, "nome"));
        json::set(obj, "cidade", "SP");
        print("pretty:", json::pretty(obj));
```

---

## 4.1 Biblioteca: `sjson` (Sjson) ✅
Camada **mais segura** para JSON, mantendo compatibilidade com `json`.

### Ideia
O `sjson` padroniza operações e oferece versões “safe” que retornam um objeto:
`{ ok: bool, value: any, error: str }`

### Funções Disponíveis (principais)
*   `sjson::decode(text)` / `sjson::encode(value)` / `sjson::encode_pretty(value)`
*   `sjson::decode_safe(text)` (retorna `{ok,value,error}`)
*   `sjson::path_get(root, "a.b.0.c")` (retorna `{ok,value,error}`)
*   Arrays: `sjson::arr()`, `sjson::push(a,v)`, `sjson::at(a,i)`, `sjson::alen(a)`

### Exemplo
```snask
import "sjson";
import "json";

class main
    fun start()
        let r = sjson::decode_safe("[1,2,3]");
        if json::get(r, "ok")
            print("ok len:", sjson::alen(json::get(r, "value")));
        else
            print("erro:", json::get(r, "error"));
```

---

## 4.2 Biblioteca: `gui` 🖼️ (Linux/GTK)
GUI minimalista estilo Tkinter (MVP) para criar apps simples (calculadora, formulários, etc.).

### Dependências do sistema (Ubuntu/Pop!_OS)
```bash
sudo apt install -y libgtk-3-dev pkg-config
snask setup
```

### Exemplo mínimo
```snask
import "gui";

fun on_click(_btn)
    print("clicou!");

class main
    fun start()
        gui::init();
        let win = gui::window("Snask GUI", 360, 160);
        let box = gui::vbox();
        gui::set_child(win, box);
        let b = gui::button("OK");
        gui::on_click(b, "on_click");
        gui::add(box, b);
        gui::show_all(win);
        gui::run();
```

## 5. Biblioteca: `os` 🖥️
Helpers para sistema/arquivos. Parte é Snask puro, e parte usa funções nativas do runtime (`sfs_*`, `s_time/s_sleep`, etc.).

### Funções Disponíveis:
*   **Info/tempo**: `os::now()`, `os::cwd()`, `os::platform()`, `os::arch()`
*   **Env**: `os::getenv(key)`, `os::setenv(key, value)`
*   **Sleep**: `os::sleep_ms(ms)`, `os::sleep_s(sec)`
*   **Arquivos**: `os::read_file(path)`, `os::write_file(path, content)`, `os::append_file(path, content)`, `os::exists(path)`, `os::delete(path)`, `os::file_size(path)`, `os::mtime(path)`
*   **Diretórios**: `os::mkdir(path)`, `os::rmdir(path)`, `os::is_dir(path)`, `os::listdir(path)`, `os::is_empty_dir(path)`, `os::ensure_dir(path)`
*   **Ops**: `os::copy(src, dst)`, `os::move(src, dst)`, `os::is_file(path)`, `os::touch(path)`
*   **Path**: `os::join(a, b)`, `os::basename(path)`, `os::dirname(path)`, `os::extname(path)`
*   **JSON**: `os::read_json(path)`, `os::write_json(path, obj)`, `os::write_json_pretty(path, obj)`, `os::listdir_json(path)`

### Exemplo de Uso:
```snask
import "os";
import "json";

class main
    fun start()
        os::ensure_dir("tmp");
        let p = os::join("tmp", "a.txt");
        os::touch(p);
        os::append_file(p, "oi\\nmais");
        print("cwd:", os::cwd());
        print("arquivo:", os::basename(p), "size:", os::file_size(p));
        print("dir(json):", os::listdir_json("tmp"));
```

---

## 6. Biblioteca: `blaze` 🔥
Micro-framework estilo Flask (bem simples) para subir um servidor HTTP e responder rotas.

### Como funciona
Você cria um objeto `routes` onde:
- **chave** = path (ex.: `"/"`, `"/ping"`)
- **valor** = `str` (resposta `text/plain`) ou qualquer outro valor/objeto (resposta `application/json`)

### Funções Disponíveis:
*   **Core**: `blaze::new()`, `blaze::run(port, routes)`, `blaze::route(routes, path, value)`
*   **Por método**: `blaze::get/post/put/patch/delete(routes, path, value)`
*   **Responses**: `blaze::resp_text/html/json(...)`, `blaze::resp(status, ct, body)`, `blaze::json_resp(status, value)`, `blaze::redirect(url)`
*   **Atalhos**: `blaze::ok()`, `blaze::not_found()`, `blaze::bad_request()`, `blaze::internal_error()`
*   **Static**: `blaze::file_text/html/json(path)`, `blaze::route_file_text/html/json(routes, path, file_path)`
*   **Handlers (dinâmico)**: `blaze::handler_get/post/put/patch/delete(...)` + parsing `blaze::qs_get(...)` e `blaze::cookie_get(...)`

### Exemplo de Uso:
```snask
import "blaze";
import "json";

class main
    fun start()
        let routes = blaze::new();
        blaze::get(routes, "/", blaze::resp_text("Olá do Blaze!"));
        blaze::get(routes, "/ping", blaze::ok());
        let user = json::new_object();
        json::set(user, "name", "davi");
        blaze::get(routes, "/user", blaze::resp_json(user));

        print("Servidor rodando em http://127.0.0.1:8080");
        blaze::run(8080, routes);
```

---

## 7. Biblioteca: `blaze_auth` 🔐
Primitives de autenticação para apps Blaze: cadastro/login local, sessões e helpers de cookie.

### Funções Disponíveis (principais)
*   `blaze_auth::default_config()` / `blaze_auth::config(db_path, sessions_path)`
*   `blaze_auth::register_local(cfg, user, pass)` / `blaze_auth::verify_local(cfg, user, pass)`
*   `blaze_auth::create_session(cfg, user)` / `blaze_auth::get_session_user(cfg, sid)`
*   `blaze_auth::login_static(cfg, user, pass)` (helper simples que retorna um response-object com cookie)
*   **Nativas (14)**: `auth_hash_password`, `auth_verify_password`, `auth_session_id`, `auth_cookie_session`, etc.
*   **Google OAuth**: existe só como stub por enquanto (`google_*`)

### Exemplo (demo estático)
```snask
import "blaze";
import "blaze_auth";

class main
    fun start()
        let cfg = blaze_auth::default_config();
        blaze_auth::register_local(cfg, "admin", "123");

        let routes = blaze::new();
        blaze::get(routes, "/login", blaze_auth::login_static(cfg, "admin", "123"));
        blaze::get(routes, "/", blaze::resp_text("ok"));

        blaze::run(8080, routes);
```

> Nota: agora o Blaze já expõe **query/cookie/body** via `blaze::handler_*` (veja `blaze_auth_system.snask` para um exemplo “real”).
