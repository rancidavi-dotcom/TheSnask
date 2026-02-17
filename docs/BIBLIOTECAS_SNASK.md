# 📚 Guia de Bibliotecas Snask (v0.2.1)

O Snask utiliza um sistema de módulos com **Namespaces Obrigatórios**. Ao importar uma biblioteca, você deve acessar suas funções usando o prefixo `nome_da_lib::`.

---

## 1. Biblioteca: `requests` 🌐
Biblioteca completa para comunicação em rede e integração com APIs Web.

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
Módulo de alto desempenho para manipulação de arquivos e diretórios.

### Funções Disponíveis:
*   `sfs::read(path)`: Lê o conteúdo de um arquivo.
*   `sfs::write(path, content)`: Escreve dados (auto-flush garantido).
*   `sfs::exists(path)`: Verifica se o arquivo está no disco.
*   `sfs::delete(path)`: Remove o arquivo permanentemente.

### Exemplo de Uso:
```snask
import "sfs"

sfs::write("logs.txt", "Evento registrado!");
if sfs::exists("logs.txt") {
    print("Log carregado:", sfs::read("logs.txt"));
}
```

---

## 3. Biblioteca: `utils` 🛠️
Utilitários matemáticos e funções auxiliares.

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

💡 **Regra de Ouro:** No Snask v0.2.1, a sintaxe `modulo::funcao()` é o padrão para qualquer código importado. Isso garante que seu código seja modular e livre de conflitos!
