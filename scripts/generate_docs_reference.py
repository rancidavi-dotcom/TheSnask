#!/usr/bin/env python3
from __future__ import annotations

import html
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs/site/reference/functions"


def slug(name: str) -> str:
    return name.replace("_", "-")


def code(s: str) -> str:
    return html.escape(s)


FUNCTIONS = [
    {
        "name": "print",
        "category": "io",
        "signature": "print(value: any) -> void",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Escreve um valor na saida padrao sem adicionar quebra de linha automaticamente.",
        "params": [("value", "Valor a imprimir. Strings, numeros, booleanos e nil tem formatacao direta.")],
        "returns": "Nada.",
        "example": 'class main {\n    fun start() {\n        print("Ola, Snask!\\n")\n    }\n}',
        "test": "docs/examples/reference/io_hello.snask",
    },
    {
        "name": "println",
        "category": "io",
        "signature": "println() -> void",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Emite uma quebra de linha pelo runtime. Algumas superficies antigas usam `print(\"\\n\")`.",
        "params": [],
        "returns": "Nada.",
        "example": 'class main {\n    fun start() {\n        print("linha")\n        println()\n    }\n}',
    },
    {
        "name": "abs",
        "category": "math",
        "signature": "abs(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna o valor absoluto de um numero.",
        "params": [("x", "Numero de entrada.")],
        "returns": "Numero sem sinal negativo.",
        "example": "let distance = abs(-42)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "floor",
        "category": "math",
        "signature": "floor(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Arredonda para baixo.",
        "params": [("x", "Numero de entrada.")],
        "returns": "Maior inteiro menor ou igual a `x`, representado como float.",
        "example": "let n = floor(3.9)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "ceil",
        "category": "math",
        "signature": "ceil(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Arredonda para cima.",
        "params": [("x", "Numero de entrada.")],
        "returns": "Menor inteiro maior ou igual a `x`, representado como float.",
        "example": "let n = ceil(3.1)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "round",
        "category": "math",
        "signature": "round(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Arredonda para o inteiro mais proximo.",
        "params": [("x", "Numero de entrada.")],
        "returns": "Valor arredondado, representado como float.",
        "example": "let n = round(3.5)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "pow",
        "category": "math",
        "signature": "pow(base: float, exp: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Eleva `base` a potencia `exp`.",
        "params": [("base", "Base."), ("exp", "Expoente.")],
        "returns": "Resultado numerico.",
        "example": "let square = pow(8, 2)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "sqrt",
        "category": "math",
        "signature": "sqrt(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Calcula raiz quadrada.",
        "params": [("x", "Numero nao negativo para resultado real.")],
        "returns": "Raiz quadrada de `x`.",
        "example": "let hyp = sqrt(144)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "min",
        "category": "math",
        "signature": "min(...values: any) -> any",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Retorna o menor valor entre os argumentos.",
        "params": [("values", "Lista variadica de valores comparaveis.")],
        "returns": "Menor valor encontrado.",
        "example": "let lowest = min(10, 4, 7)",
    },
    {
        "name": "max",
        "category": "math",
        "signature": "max(...values: any) -> any",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Retorna o maior valor entre os argumentos.",
        "params": [("values", "Lista variadica de valores comparaveis.")],
        "returns": "Maior valor encontrado.",
        "example": "let highest = max(10, 4, 7)",
    },
    {
        "name": "sin",
        "category": "math",
        "signature": "sin(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Seno de `x` em radianos.",
        "params": [("x", "Angulo em radianos.")],
        "returns": "Seno do angulo.",
        "example": "let y = sin(PI / 2)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "cos",
        "category": "math",
        "signature": "cos(x: float) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Cosseno de `x` em radianos.",
        "params": [("x", "Angulo em radianos.")],
        "returns": "Cosseno do angulo.",
        "example": "let y = cos(0)",
        "test": "docs/examples/reference/math_core.snask",
    },
    {
        "name": "len",
        "category": "core",
        "signature": "len(value: any) -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna o tamanho de uma string, lista, objeto dinamico ou recurso suportado.",
        "params": [("value", "Valor cujo tamanho sera consultado.")],
        "returns": "Tamanho como numero.",
        "example": 'let n = len("snask")',
        "test": "docs/examples/reference/string_core.snask",
    },
    {
        "name": "upper",
        "category": "string",
        "signature": "upper(text: str) -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Converte texto para maiusculas.",
        "params": [("text", "String de entrada.")],
        "returns": "String convertida.",
        "example": 'let name = upper("snask")',
        "test": "docs/examples/reference/string_core.snask",
    },
    {
        "name": "lower",
        "category": "string",
        "signature": "lower(text: str) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Converte texto para minusculas.",
        "params": [("text", "String de entrada.")],
        "returns": "String convertida.",
        "example": 'let name = lower("SNASK")',
    },
    {
        "name": "trim",
        "category": "string",
        "signature": "trim(text: str) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Remove espacos no inicio e fim da string.",
        "params": [("text", "String de entrada.")],
        "returns": "String sem bordas de whitespace.",
        "example": 'let clean = trim("  ok  ")',
    },
    {
        "name": "substring",
        "category": "string",
        "signature": "substring(text: str, start: float, length: float) -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Extrai parte de uma string por indice inicial e tamanho.",
        "params": [("text", "String original."), ("start", "Indice inicial."), ("length", "Quantidade de caracteres.")],
        "returns": "Trecho extraido.",
        "example": 'let prefix = substring("snask", 0, 2)',
        "test": "docs/examples/reference/string_core.snask",
    },
    {
        "name": "contains",
        "category": "string",
        "signature": "contains(text: str, needle: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se uma string contem outra.",
        "params": [("text", "String completa."), ("needle", "Trecho procurado.")],
        "returns": "`true` se encontrar.",
        "example": 'let ok = contains("snask", "ask")',
    },
    {
        "name": "starts_with",
        "category": "string",
        "signature": "starts_with(text: str, prefix: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa prefixo de string.",
        "params": [("text", "String completa."), ("prefix", "Prefixo esperado.")],
        "returns": "`true` se a string comeca com o prefixo.",
        "example": 'let ok = starts_with("snask", "sn")',
    },
    {
        "name": "ends_with",
        "category": "string",
        "signature": "ends_with(text: str, suffix: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa sufixo de string.",
        "params": [("text", "String completa."), ("suffix", "Sufixo esperado.")],
        "returns": "`true` se a string termina com o sufixo.",
        "example": 'let ok = ends_with("snask", "ask")',
    },
    {
        "name": "split",
        "category": "string",
        "signature": "split(text: str, sep: str) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Divide uma string usando separador.",
        "params": [("text", "String original."), ("sep", "Separador.")],
        "returns": "Lista de partes.",
        "example": 'let parts = split("a,b,c", ",")',
    },
    {
        "name": "join",
        "category": "string",
        "signature": "join(parts: list, sep: str) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Junta lista de strings usando separador.",
        "params": [("parts", "Lista de strings."), ("sep", "Separador.")],
        "returns": "String resultante.",
        "example": 'let text = join(["a", "b"], ",")',
    },
    {
        "name": "replace",
        "category": "string",
        "signature": "replace(text: str, old: str, new: str) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Substitui ocorrencias de `old` por `new`.",
        "params": [("text", "String original."), ("old", "Trecho antigo."), ("new", "Trecho novo.")],
        "returns": "String alterada.",
        "example": 'let text = replace("snask", "ask", "ake")',
    },
    {
        "name": "chars",
        "category": "string",
        "signature": "chars(text: str) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Converte string em lista de caracteres.",
        "params": [("text", "String de entrada.")],
        "returns": "Lista de caracteres como strings.",
        "example": 'let letters = chars("abc")',
    },
    {
        "name": "format",
        "category": "string",
        "signature": "format(template: str, ...values: any) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Formata string com valores variadicos.",
        "params": [("template", "Template de formatacao."), ("values", "Valores usados no template.")],
        "returns": "String formatada.",
        "example": 'let text = format("score: {}", 42)',
    },
    {
        "name": "range",
        "category": "collections",
        "signature": "range(end: float) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Cria uma lista numerica de 0 ate `end - 1`.",
        "params": [("end", "Limite exclusivo.")],
        "returns": "Lista de numeros.",
        "example": "let xs = range(4)",
    },
    {
        "name": "sort",
        "category": "collections",
        "signature": "sort(values: list) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Ordena uma lista.",
        "params": [("values", "Lista original.")],
        "returns": "Lista ordenada.",
        "example": "let xs = sort([3, 1, 2])",
    },
    {
        "name": "reverse",
        "category": "collections",
        "signature": "reverse(values: list) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Inverte a ordem de uma lista.",
        "params": [("values", "Lista original.")],
        "returns": "Lista invertida.",
        "example": "let xs = reverse([1, 2, 3])",
    },
    {
        "name": "unique",
        "category": "collections",
        "signature": "unique(values: list) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Remove duplicatas de uma lista.",
        "params": [("values", "Lista original.")],
        "returns": "Lista sem duplicatas.",
        "example": "let xs = unique([1, 1, 2])",
    },
    {
        "name": "flatten",
        "category": "collections",
        "signature": "flatten(values: list) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Achata uma lista de listas.",
        "params": [("values", "Lista com sublistas.")],
        "returns": "Lista achatada.",
        "example": "let xs = flatten([[1], [2]])",
    },
    {
        "name": "is_nil",
        "category": "type",
        "signature": "is_nil(value: any) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se um valor e nil.",
        "params": [("value", "Valor testado.")],
        "returns": "`true` quando o valor e nil.",
        "example": "let ok = is_nil(nil)",
    },
    {
        "name": "is_str",
        "category": "type",
        "signature": "is_str(value: any) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se um valor e string.",
        "params": [("value", "Valor testado.")],
        "returns": "`true` quando o valor e string.",
        "example": 'let ok = is_str("snask")',
    },
    {
        "name": "is_obj",
        "category": "type",
        "signature": "is_obj(value: any) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se um valor dinamico e objeto.",
        "params": [("value", "Valor testado.")],
        "returns": "`true` quando o valor representa objeto.",
        "example": "let ok = is_obj(value)",
    },
    {
        "name": "read_file",
        "category": "filesystem",
        "signature": "read_file(path: str) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Le um arquivo texto inteiro.",
        "params": [("path", "Caminho do arquivo.")],
        "returns": "Conteudo como string.",
        "example": 'let data = read_file("notes.txt")',
    },
    {
        "name": "write_file",
        "category": "filesystem",
        "signature": "write_file(path: str, content: str) -> void",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Escreve texto em arquivo, substituindo conteudo anterior.",
        "params": [("path", "Caminho."), ("content", "Conteudo novo.")],
        "returns": "Nada.",
        "example": 'write_file("/tmp/snask.txt", "ok")',
    },
    {
        "name": "append_file",
        "category": "filesystem",
        "signature": "append_file(path: str, content: str) -> void",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Acrescenta texto ao fim de um arquivo.",
        "params": [("path", "Caminho."), ("content", "Texto a anexar.")],
        "returns": "Nada.",
        "example": 'append_file("/tmp/snask.log", "linha\\n")',
    },
    {
        "name": "exists",
        "category": "filesystem",
        "signature": "exists(path: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa existencia de arquivo ou diretorio.",
        "params": [("path", "Caminho.")],
        "returns": "`true` se existe.",
        "example": 'let ok = exists("/tmp/snask.txt")',
    },
    {
        "name": "delete",
        "category": "filesystem",
        "signature": "delete(path: str) -> void",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Remove arquivo.",
        "params": [("path", "Caminho do arquivo.")],
        "returns": "Nada.",
        "example": 'delete("/tmp/snask.txt")',
    },
    {
        "name": "read_dir",
        "category": "filesystem",
        "signature": "read_dir(path: str) -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Lista entradas de um diretorio.",
        "params": [("path", "Diretorio.")],
        "returns": "Lista de nomes/caminhos.",
        "example": 'let files = read_dir(".")',
    },
    {
        "name": "is_file",
        "category": "filesystem",
        "signature": "is_file(path: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se o caminho aponta para arquivo.",
        "params": [("path", "Caminho.")],
        "returns": "`true` para arquivo.",
        "example": 'let ok = is_file("main.snask")',
    },
    {
        "name": "is_dir",
        "category": "filesystem",
        "signature": "is_dir(path: str) -> bool",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Testa se o caminho aponta para diretorio.",
        "params": [("path", "Caminho.")],
        "returns": "`true` para diretorio.",
        "example": 'let ok = is_dir(".")',
    },
    {
        "name": "create_dir",
        "category": "filesystem",
        "signature": "create_dir(path: str) -> void",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Cria diretorio.",
        "params": [("path", "Diretorio a criar.")],
        "returns": "Nada.",
        "example": 'create_dir("/tmp/snask-docs")',
    },
    {
        "name": "http_get",
        "category": "network",
        "signature": "http_get(url: str) -> dict",
        "profile": "humane",
        "status": "experimental",
        "safety": "segura",
        "summary": "Executa requisicao HTTP GET pelo runtime.",
        "params": [("url", "URL absoluta.")],
        "returns": "Dicionario com dados da resposta, conforme runtime atual.",
        "example": 'let res = http_get("https://example.com")',
    },
    {
        "name": "http_post",
        "category": "network",
        "signature": "http_post(url: str, body: str) -> void",
        "profile": "humane",
        "status": "experimental",
        "safety": "segura",
        "summary": "Executa requisicao HTTP POST pelo runtime.",
        "params": [("url", "URL absoluta."), ("body", "Corpo textual.")],
        "returns": "Nada na superficie atual.",
        "example": 'http_post("https://example.com", "ok")',
    },
    {
        "name": "time",
        "category": "os",
        "signature": "time() -> float",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna tempo atual do runtime.",
        "params": [],
        "returns": "Timestamp/tempo como float.",
        "example": "let started = time()",
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "sleep",
        "category": "os",
        "signature": "sleep(seconds: float) -> void",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Pausa a execucao por alguns segundos.",
        "params": [("seconds", "Duracao da pausa.")],
        "returns": "Nada.",
        "example": "sleep(0.1)",
    },
    {
        "name": "exit",
        "category": "os",
        "signature": "exit(code: float) -> void",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Encerra o processo com codigo de saida.",
        "params": [("code", "Codigo de saida.")],
        "returns": "Nao retorna.",
        "example": "exit(0)",
    },
    {
        "name": "args",
        "category": "os",
        "signature": "args() -> list",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Retorna argumentos da linha de comando.",
        "params": [],
        "returns": "Lista de strings.",
        "example": "let argv = args()",
    },
    {
        "name": "env",
        "category": "os",
        "signature": "env(name: str) -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Le variavel de ambiente.",
        "params": [("name", "Nome da variavel.")],
        "returns": "Valor, ou string vazia quando ausente.",
        "example": 'let home = env("HOME")',
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "set_env",
        "category": "os",
        "signature": "set_env(name: str, value: str) -> void",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Define variavel de ambiente no processo atual.",
        "params": [("name", "Nome."), ("value", "Valor.")],
        "returns": "Nada.",
        "example": 'set_env("SNASK_MODE", "docs")',
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "cwd",
        "category": "os",
        "signature": "cwd() -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna diretorio atual.",
        "params": [],
        "returns": "Caminho do diretorio atual.",
        "example": "let here = cwd()",
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "platform",
        "category": "os",
        "signature": "platform() -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna nome do sistema operacional.",
        "params": [],
        "returns": "String de plataforma.",
        "example": "let os = platform()",
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "arch",
        "category": "os",
        "signature": "arch() -> str",
        "profile": "humane",
        "status": "estavel",
        "safety": "segura",
        "summary": "Retorna arquitetura da maquina.",
        "params": [],
        "returns": "String de arquitetura.",
        "example": "let cpu = arch()",
        "test": "docs/examples/reference/os_core.snask",
    },
    {
        "name": "str_to_num",
        "category": "conversion",
        "signature": "str_to_num(text: str) -> float",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Converte string numerica para numero.",
        "params": [("text", "Texto numerico.")],
        "returns": "Numero convertido.",
        "example": 'let n = str_to_num("42")',
    },
    {
        "name": "num_to_str",
        "category": "conversion",
        "signature": "num_to_str(value: float) -> str",
        "profile": "humane",
        "status": "parcial",
        "safety": "segura",
        "summary": "Converte numero para string.",
        "params": [("value", "Numero.")],
        "returns": "String textual.",
        "example": "let text = num_to_str(42)",
    },
    {
        "name": "calc_eval",
        "category": "conversion",
        "signature": "calc_eval(expr: str) -> float",
        "profile": "humane",
        "status": "experimental",
        "safety": "segura",
        "summary": "Avalia expressao numerica simples pelo runtime.",
        "params": [("expr", "Expressao textual.")],
        "returns": "Resultado numerico.",
        "example": 'let n = calc_eval("(3 + 4) * 2")',
    },
    {
        "name": "wrapping_add",
        "category": "systems",
        "signature": "wrapping_add(a: any, b: any) -> any",
        "profile": "systems",
        "status": "estavel",
        "safety": "segura",
        "summary": "Soma com overflow por mascara do tipo, util para CPUs reais.",
        "params": [("a", "Operando."), ("b", "Operando.")],
        "returns": "Resultado com wrap.",
        "example": "let x: u8 = wrapping_add(255, 1)",
        "test": "docs/examples/reference/systems_bits.snask",
    },
    {
        "name": "wrapping_sub",
        "category": "systems",
        "signature": "wrapping_sub(a: any, b: any) -> any",
        "profile": "systems",
        "status": "estavel",
        "safety": "segura",
        "summary": "Subtrai com overflow por mascara do tipo.",
        "params": [("a", "Operando."), ("b", "Operando.")],
        "returns": "Resultado com wrap.",
        "example": "let x: u8 = wrapping_sub(0, 1)",
        "test": "docs/examples/reference/systems_bits.snask",
    },
    {
        "name": "wrapping_mul",
        "category": "systems",
        "signature": "wrapping_mul(a: any, b: any) -> any",
        "profile": "systems",
        "status": "estavel",
        "safety": "segura",
        "summary": "Multiplica com overflow por mascara do tipo.",
        "params": [("a", "Operando."), ("b", "Operando.")],
        "returns": "Resultado com wrap.",
        "example": "let x: u8 = wrapping_mul(32, 8)",
        "test": "docs/examples/reference/systems_bits.snask",
    },
    {
        "name": "saturating_add",
        "category": "systems",
        "signature": "saturating_add(a: any, b: any) -> any",
        "profile": "systems",
        "status": "estavel",
        "safety": "segura",
        "summary": "Soma saturando no limite do tipo.",
        "params": [("a", "Operando."), ("b", "Operando.")],
        "returns": "Resultado saturado.",
        "example": "let x: u8 = saturating_add(250, 20)",
        "test": "docs/examples/reference/systems_bits.snask",
    },
]

VALIDATED_EXAMPLES = {
    "docs/examples/reference/io_hello.snask",
    "docs/examples/reference/systems_bits.snask",
    "docs/examples/reference/systems_memory.snask",
    "docs/examples/reference/sfs_basic.snask",
    "docs/examples/reference/json_basic.snask",
    "docs/examples/reference/gui_minimal.snask",
}


for name, typ in [
    ("as_u8", "u8"),
    ("as_u16", "u16"),
    ("as_u32", "u32"),
    ("as_u64", "u64"),
    ("as_i8", "i8"),
    ("as_i16", "i16"),
    ("as_i32", "i32"),
    ("as_i64", "i64"),
    ("as_usize", "usize"),
    ("as_isize", "isize"),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "systems",
            "signature": f"{name}(value: any) -> {typ}",
            "profile": "systems",
            "status": "estavel",
            "safety": "segura",
            "summary": f"Converte valor numerico para `{typ}` com semantica explicita do perfil systems.",
            "params": [("value", "Valor numerico de entrada.")],
            "returns": f"Valor convertido para `{typ}`.",
            "example": f"let n: {typ} = {name}(255)",
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, summary, ret, example in [
    ("lo_u8", "Extrai o byte baixo de uma word.", "u8", "let lo: u8 = lo_u8(0x1234)"),
    ("hi_u8", "Extrai o byte alto de uma word.", "u8", "let hi: u8 = hi_u8(0x1234)"),
    ("make_u16", "Combina byte baixo e alto em word little-endian.", "any", "let pc: u16 = make_u16(0x34, 0x12)"),
    ("is_zero_u8", "Testa flag zero para valor de 8 bits.", "bool", "let z = is_zero_u8(0)"),
    ("is_negative_u8", "Testa bit 7 como flag negativa do 6502.", "bool", "let n = is_negative_u8(0x80)"),
    ("bit_test", "Testa se um bit esta ligado.", "bool", "let carry = bit_test(status, 0)"),
    ("bit_set", "Liga um bit.", "any", "let status = bit_set(0, 7)"),
    ("bit_clear", "Desliga um bit.", "any", "let status = bit_clear(0xFF, 7)"),
    ("bit_toggle", "Inverte um bit.", "any", "let status = bit_toggle(0, 1)"),
    ("bit_write", "Liga ou desliga bit conforme booleano/valor.", "any", "let status = bit_write(0, 1, true)"),
    ("flag_has", "Alias semantico de `bit_test` para registradores de flags.", "bool", "let ok = flag_has(status, 2)"),
    ("flag_set", "Alias semantico de `bit_set` para registradores de flags.", "any", "let status = flag_set(status, 2)"),
    ("flag_clear", "Alias semantico de `bit_clear` para registradores de flags.", "any", "let status = flag_clear(status, 2)"),
    ("flag_write", "Alias semantico de `bit_write` para flags de CPU.", "any", "let status = flag_write(status, 1, true)"),
    ("wrapping_inc", "Incrementa com wrap.", "any", "let x: u8 = wrapping_inc(255, 1)"),
    ("wrapping_dec", "Decrementa com wrap.", "any", "let x: u8 = wrapping_dec(0, 1)"),
]:
    arity = "value: any" if name in ("lo_u8", "hi_u8", "is_zero_u8", "is_negative_u8") else "value: any, bit: any"
    if name in ("bit_write", "flag_write"):
        arity = "value: any, bit: any, enabled: any"
    FUNCTIONS.append(
        {
            "name": name,
            "category": "systems",
            "signature": f"{name}({arity}) -> {ret}",
            "profile": "systems",
            "status": "estavel",
            "safety": "segura",
            "summary": summary,
            "params": [("value", "Valor ou registrador base.")],
            "returns": f"Retorna `{ret}`.",
            "example": example,
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, summary in [
    ("carry_add_u8", "Calcula carry de soma de 8 bits, incluindo carry de entrada."),
    ("borrow_sub_u8", "Calcula borrow de subtracao de 8 bits."),
    ("overflow_add_i8", "Calcula overflow assinado de soma de 8 bits."),
    ("overflow_sub_i8", "Calcula overflow assinado de subtracao de 8 bits."),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "systems",
            "signature": f"{name}(a: any, b: any, carry_or_borrow: any) -> bool",
            "profile": "systems",
            "status": "estavel",
            "safety": "segura",
            "summary": summary,
            "params": [("a", "Operando A."), ("b", "Operando B."), ("carry_or_borrow", "Entrada de carry/borrow.")],
            "returns": "Booleano da condicao.",
            "example": f"let flag = {name}(0xFF, 0x01, 0)",
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, sig, summary, ret, example in [
    ("mem_alloc", "mem_alloc(size: any) -> ptr", "Aloca memoria crua sem zerar.", "Ponteiro para bloco alocado.", "let p: ptr = mem_alloc(256)"),
    ("mem_alloc_zero", "mem_alloc_zero(size: any) -> ptr", "Aloca memoria crua zerada.", "Ponteiro para bloco alocado.", "let p: ptr = mem_alloc_zero(65536)"),
    ("mem_free", "mem_free(ptr: ptr) -> void", "Libera bloco alocado manualmente.", "Nada.", "mem_free(p)"),
    ("ptr_add", "ptr_add(ptr: ptr, offset: any) -> ptr", "Retorna ponteiro deslocado por bytes.", "Novo ponteiro.", "let p2: ptr = ptr_add(p, 16)"),
    ("mem_read_u8", "mem_read_u8(ptr: ptr, offset: any) -> u8", "Le byte em offset.", "Valor `u8`.", "let b: u8 = mem_read_u8(p, 0)"),
    ("mem_read_u16", "mem_read_u16(ptr: ptr, offset: any) -> u16", "Le word little-endian em offset.", "Valor `u16`.", "let w: u16 = mem_read_u16(p, 0)"),
    ("mem_read_u32", "mem_read_u32(ptr: ptr, offset: any) -> u32", "Le dword little-endian em offset.", "Valor `u32`.", "let d: u32 = mem_read_u32(p, 0)"),
    ("mem_write_u8", "mem_write_u8(ptr: ptr, offset: any, value: any) -> void", "Escreve byte em offset.", "Nada.", "mem_write_u8(p, 0, 0xEA)"),
    ("mem_write_u16", "mem_write_u16(ptr: ptr, offset: any, value: any) -> void", "Escreve word little-endian em offset.", "Nada.", "mem_write_u16(p, 0, 0x8000)"),
    ("mem_write_u32", "mem_write_u32(ptr: ptr, offset: any, value: any) -> void", "Escreve dword little-endian em offset.", "Nada.", "mem_write_u32(p, 0, 0x12345678)"),
    ("mem_fill_u8", "mem_fill_u8(ptr: ptr, value: any, size: any) -> void", "Preenche bloco com byte.", "Nada.", "mem_fill_u8(p, 0, 256)"),
    ("mem_copy", "mem_copy(dst: ptr, src: ptr, size: any) -> void", "Copia bytes entre blocos.", "Nada.", "mem_copy(dst, src, 256)"),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "memory",
            "signature": sig,
            "profile": "systems",
            "status": "estavel",
            "safety": "exige @unsafe",
            "summary": summary,
            "params": [("ptr/offset/value", "Argumentos de memoria crua; validar bounds e ownership e responsabilidade do codigo unsafe.")],
            "returns": ret,
            "example": "@unsafe {\n    " + example + "\n}",
            "test": "docs/examples/reference/systems_memory.snask",
        }
    )

for name, sig, summary in [
    ("json_parse", "json_parse(text: str) -> any", "Converte texto JSON em valor dinamico."),
    ("json_stringify", "json_stringify(value: any) -> str", "Serializa valor dinamico para JSON compacto."),
    ("json_stringify_pretty", "json_stringify_pretty(value: any) -> str", "Serializa valor dinamico para JSON formatado."),
    ("json_get", "json_get(value: any, key: str) -> any", "Le campo de objeto JSON."),
    ("json_has", "json_has(value: any, key: str) -> bool", "Testa existencia de campo JSON."),
    ("json_len", "json_len(value: any) -> float", "Retorna tamanho de array/objeto JSON."),
    ("json_index", "json_index(value: any, index: float) -> any", "Le item de array JSON."),
    ("json_set", "json_set(value: any, key: str, item: any) -> bool", "Define campo JSON."),
    ("json_keys", "json_keys(value: any) -> any", "Retorna chaves de objeto JSON."),
    ("json_parse_ex", "json_parse_ex(text: str) -> any", "Parse JSON com superficie extendida do runtime."),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "json",
            "signature": sig,
            "profile": "humane",
            "status": "experimental",
            "safety": "segura",
            "summary": summary,
            "params": [("value/text/key", "Valor JSON dinamico, texto de entrada ou chave.")],
            "returns": "Conforme assinatura.",
            "example": 'import "json"\n\nclass main {\n    fun start() {\n        let obj = json::parse("{\\"name\\":\\"snask\\"}")\n        print(json::get(obj, "name"))\n    }\n}',
            "test": "docs/examples/reference/json_basic.snask",
        }
    )

for name, sig, summary in [
    ("sfs_read", "sfs_read(path: str) -> str", "Le arquivo pela biblioteca SFS."),
    ("sfs_write", "sfs_write(path: str, content: str) -> bool", "Escreve arquivo pela biblioteca SFS."),
    ("sfs_append", "sfs_append(path: str, content: str) -> bool", "Acrescenta texto em arquivo SFS."),
    ("sfs_exists", "sfs_exists(path: str) -> bool", "Testa existencia pelo SFS."),
    ("sfs_delete", "sfs_delete(path: str) -> bool", "Remove arquivo pelo SFS."),
    ("sfs_copy", "sfs_copy(src: str, dst: str) -> bool", "Copia arquivo pelo SFS."),
    ("sfs_move", "sfs_move(src: str, dst: str) -> bool", "Move arquivo pelo SFS."),
    ("sfs_mkdir", "sfs_mkdir(path: str) -> bool", "Cria diretorio pelo SFS."),
    ("sfs_rmdir", "sfs_rmdir(path: str) -> bool", "Remove diretorio pelo SFS."),
    ("sfs_is_file", "sfs_is_file(path: str) -> bool", "Testa arquivo pelo SFS."),
    ("sfs_is_dir", "sfs_is_dir(path: str) -> bool", "Testa diretorio pelo SFS."),
    ("sfs_listdir", "sfs_listdir(path: str) -> any", "Lista diretorio pelo SFS."),
    ("sfs_size", "sfs_size(path: str) -> float", "Tamanho de arquivo."),
    ("sfs_mtime", "sfs_mtime(path: str) -> float", "Tempo de modificacao."),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "sfs",
            "signature": sig,
            "profile": "humane",
            "status": "parcial",
            "safety": "segura",
            "summary": summary,
            "params": [("path", "Caminho de filesystem.")],
            "returns": "Conforme assinatura.",
            "example": 'import "sfs"\n\nclass main {\n    fun start() {\n        let ok = sfs::exists(".")\n        print(ok)\n    }\n}',
            "test": "docs/examples/reference/sfs_basic.snask",
        }
    )

for name, sig, summary in [
    ("gui_init", "gui_init() -> bool", "Inicializa runtime GTK."),
    ("gui_run", "gui_run() -> void", "Entra no loop principal da GUI."),
    ("gui_quit", "gui_quit() -> void", "Sai do loop principal da GUI."),
    ("gui_window", "gui_window(title: str, width: float, height: float) -> any", "Cria janela nativa."),
    ("gui_label", "gui_label(text: str) -> any", "Cria label."),
    ("gui_button", "gui_button(text: str) -> any", "Cria botao."),
    ("gui_entry", "gui_entry() -> any", "Cria input de texto."),
    ("gui_textview", "gui_textview() -> any", "Cria area de texto."),
    ("gui_vbox", "gui_vbox() -> any", "Cria container vertical."),
    ("gui_hbox", "gui_hbox() -> any", "Cria container horizontal."),
    ("gui_add", "gui_add(parent: any, child: any) -> bool", "Adiciona widget a container."),
    ("gui_set_child", "gui_set_child(parent: any, child: any) -> bool", "Define filho unico de container."),
    ("gui_show_all", "gui_show_all(widget: any) -> bool", "Mostra widget e descendentes."),
    ("gui_set_text", "gui_set_text(widget: any, text: str) -> bool", "Define texto de widget suportado."),
    ("gui_get_text", "gui_get_text(widget: any) -> str", "Le texto de widget suportado."),
    ("gui_on_click", "gui_on_click(widget: any, handler: any) -> bool", "Conecta handler de clique."),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "gui",
            "signature": sig,
            "profile": "humane",
            "status": "experimental",
            "safety": "segura",
            "summary": summary,
            "params": [("widget/args", "Handles opacos de widgets e argumentos de UI.")],
            "returns": "Conforme assinatura.",
            "example": 'import "gui"\n\nclass main {\n    fun start() {\n        let ok = gui::init()\n        let win = gui::window("Snask", 320, 180)\n        gui::show_all(win)\n        gui::run()\n    }\n}',
            "test": "docs/examples/reference/gui_minimal.snask",
        }
    )

for name, sig, summary in [
    ("snaskgui_init", "snaskgui_init() -> bool", "Inicializa API framebuffer para jogos/emuladores."),
    ("snaskgui_window", "snaskgui_window(title: str, width: float, height: float, scale: float) -> any", "Cria janela framebuffer."),
    ("snaskgui_present_rgba", "snaskgui_present_rgba(window: any, pixels: ptr, width: float, height: float) -> bool", "Apresenta buffer RGBA."),
    ("snaskgui_poll", "snaskgui_poll(window: any) -> bool", "Processa eventos pendentes."),
    ("snaskgui_key_down", "snaskgui_key_down(window: any, key: float) -> bool", "Consulta estado de tecla."),
    ("snaskgui_should_close", "snaskgui_should_close(window: any) -> bool", "Testa fechamento da janela."),
    ("snaskgui_delay", "snaskgui_delay(ms: float) -> void", "Pausa em milissegundos."),
    ("snaskgui_close", "snaskgui_close(window: any) -> void", "Fecha janela framebuffer."),
]:
    FUNCTIONS.append(
        {
            "name": name,
            "category": "snaskgui",
            "signature": sig,
            "profile": "systems",
            "status": "experimental",
            "safety": "segura; buffers usam @unsafe quando houver ptr",
            "summary": summary,
            "params": [("window/pixels", "Handle de janela ou ponteiro de framebuffer.")],
            "returns": "Conforme assinatura.",
            "example": 'import "snaskgui"\n\nclass main {\n    fun start() {\n        snaskgui::init()\n        let win = snaskgui::window("Pixels", 256, 240, 2)\n        snaskgui::poll(win)\n        snaskgui::close(win)\n    }\n}',
            "test": "docs/examples/reference/snaskgui_minimal.snask",
        }
    )


def render_page(fn: dict) -> str:
    params = fn.get("params") or []
    param_rows = "\n".join(
        f"<tr><td><code>{html.escape(name)}</code></td><td>{html.escape(desc)}</td></tr>"
        for name, desc in params
    ) or '<tr><td colspan="2">Sem parametros.</td></tr>'
    test = fn.get("test")
    test_block = (
        f"<h2>Teste real</h2><p>Exemplo versionado: <code>{html.escape(test)}</code></p>"
        f"<pre><code>snask build {html.escape(test)} --output /tmp/snask-doc-{slug(fn['name'])}</code></pre>"
        if test
        else "<h2>Teste real</h2><p>Esta pagina ainda nao tem arquivo de exemplo dedicado. O contrato tipado vem do analisador semantico e a funcao permanece marcada conforme status.</p>"
    )
    status_class = "partial" if fn["status"] == "parcial" else "experimental" if fn["status"] == "experimental" else ""
    return f"""<!doctype html>
<html lang=\"pt-BR\">
  <head>
    <meta charset=\"utf-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
    <title>{html.escape(fn['name'])} - Snask Docs</title>
    <link rel=\"stylesheet\" href=\"../../assets/site.css\" />
  </head>
  <body>
    <header class=\"topbar\">
      <a class=\"brand\" href=\"../../index.html\"><span class=\"mark\">S</span> Snask Docs</a>
      <nav class=\"nav\">
        <a href=\"../../index.html\">Inicio</a>
        <a href=\"../../learn/getting-started.html\">Aprender</a>
        <a aria-current=\"page\" href=\"../language.html\">Referencia</a>
        <a href=\"../../systems/om.html\">OM</a>
        <a href=\"../../tooling/installation.html\">Tooling</a>
      </nav>
    </header>
    <div class=\"shell\">
      <aside class=\"sidebar\">
        <h2>Funcoes</h2>
        <a href=\"index.html\">Indice de funcoes</a>
        <a href=\"../language.html\">Linguagem</a>
        <a href=\"../types.html\">Tipos</a>
        <a href=\"../runtime.html\">Runtime e builtins</a>
        <a href=\"../diagnostics.html\">Diagnosticos</a>
      </aside>
      <main class=\"content doc-page\">
        <p class=\"eyebrow\">{html.escape(fn['category'])}</p>
        <h1>{html.escape(fn['name'])}</h1>
        <p class=\"lead\">{html.escape(fn['summary'])}</p>
        <p>
          <span class=\"status {status_class}\">{html.escape(fn['status'])}</span>
          <span class=\"meta-pill\">perfil: {html.escape(fn['profile'])}</span>
          <span class=\"meta-pill\">seguranca: {html.escape(fn['safety'])}</span>
        </p>
        <h2>Assinatura</h2>
        <pre><code>{code(fn['signature'])}</code></pre>
        <h2>Parametros</h2>
        <table><thead><tr><th>Nome</th><th>Descricao</th></tr></thead><tbody>{param_rows}</tbody></table>
        <h2>Retorno</h2>
        <p>{html.escape(fn['returns'])}</p>
        <h2>Exemplo</h2>
        <pre><code>{code(fn['example'])}</code></pre>
        {test_block}
        <h2>Notas</h2>
        <p>
          Esta pagina e gerada a partir de <code>scripts/generate_docs_reference.py</code>.
          Se a assinatura mudar no compilador, atualize a fonte de dados e regenere a referencia.
        </p>
      </main>
    </div>
    <script src=\"../../assets/site.js\"></script>
  </body>
</html>
"""


def render_index() -> str:
    groups: dict[str, list[dict]] = {}
    for fn in sorted(FUNCTIONS, key=lambda f: (f["category"], f["name"])):
        groups.setdefault(fn["category"], []).append(fn)
    sections = []
    for category, fns in groups.items():
        links = "\n".join(
            f'<a class="card function-card" href="{slug(fn["name"])}.html"><h3>{html.escape(fn["name"])}</h3><p><code>{html.escape(fn["signature"])}</code></p><p>{html.escape(fn["summary"])}</p></a>'
            for fn in fns
        )
        sections.append(f"<h2>{html.escape(category.title())}</h2><div class=\"grid\">{links}</div>")
    body = "\n".join(sections)
    total = len(FUNCTIONS)
    return f"""<!doctype html>
<html lang=\"pt-BR\">
  <head>
    <meta charset=\"utf-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
    <title>Indice de funcoes - Snask Docs</title>
    <link rel=\"stylesheet\" href=\"../../assets/site.css\" />
  </head>
  <body>
    <header class=\"topbar\">
      <a class=\"brand\" href=\"../../index.html\"><span class=\"mark\">S</span> Snask Docs</a>
      <nav class=\"nav\">
        <a href=\"../../index.html\">Inicio</a>
        <a href=\"../../learn/getting-started.html\">Aprender</a>
        <a aria-current=\"page\" href=\"../language.html\">Referencia</a>
        <a href=\"../../systems/om.html\">OM</a>
        <a href=\"../../tooling/installation.html\">Tooling</a>
      </nav>
    </header>
    <div class=\"shell\">
      <aside class=\"sidebar\">
        <h2>Referencia</h2>
        <a href=\"index.html\">Indice de funcoes</a>
        <a href=\"../language.html\">Linguagem</a>
        <a href=\"../types.html\">Tipos</a>
        <a href=\"../runtime.html\">Runtime e builtins</a>
        <a href=\"../diagnostics.html\">Diagnosticos</a>
        <a href=\"../status.html\">Status real</a>
      </aside>
      <main class=\"content\">
        <p class=\"eyebrow\">Biblioteca padrao</p>
        <h1>Indice de funcoes</h1>
        <p class=\"lead\">Referencia individual de {total} funcoes e builtins da Snask, com assinatura, status, seguranca, exemplos e comandos de teste quando ha exemplo versionado.</p>
        <div class=\"callout\"><strong>Fonte de verdade:</strong> a lista acompanha os builtins registrados pelo analisador semantico e as superficies chamadas pelo runtime LLVM atual. Exemplos com arquivo versionado sao validados por <code>scripts/check_doc_examples.sh</code>.</div>
        {body}
      </main>
    </div>
    <script src=\"../../assets/site.js\"></script>
  </body>
</html>
"""


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for old in OUT.glob("*.html"):
        old.unlink()
    for fn in FUNCTIONS:
        if fn.get("test") not in VALIDATED_EXAMPLES:
            fn.pop("test", None)
        (OUT / f"{slug(fn['name'])}.html").write_text(render_page(fn), encoding="utf-8")
    (OUT / "index.html").write_text(render_index(), encoding="utf-8")
    print(f"generated {len(FUNCTIONS)} function pages in {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
