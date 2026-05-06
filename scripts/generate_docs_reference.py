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
        "params": [("x", "Numero de entrada (real).")],
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
        "params": [("x", "Numero decimal.")],
        "returns": "Maior inteiro menor ou igual a `x`.",
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
        "params": [("x", "Numero decimal.")],
        "returns": "Menor inteiro maior ou igual a `x`.",
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
        "params": [("x", "Numero decimal.")],
        "returns": "Valor arredondado (0.5 arredonda para cima).",
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
        "params": [("base", "Valor base."), ("exp", "Expoente.")],
        "returns": "Resultado da potencia.",
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
        "params": [("x", "Numero positivo.")],
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
        "params": [("values", "Lista de valores comparaveis (numeros ou strings).")],
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
        "params": [("values", "Lista de valores comparaveis.")],
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
        "returns": "Seno (entre -1 e 1).",
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
        "returns": "Cosseno (entre -1 e 1).",
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
        "params": [("value", "A colecao, string ou objeto cujo tamanho sera consultado.")],
        "returns": "Tamanho como numero (float).",
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
        "returns": "Nova string com caracteres em caixa alta.",
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
        "returns": "Nova string com caracteres em caixa baixa.",
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
        "params": [("text", "String original.")],
        "returns": "String limpa sem whitespace nas extremidades.",
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
        "params": [("text", "String original."), ("start", "Indice inicial (comeca em 0)."), ("length", "Quantidade de caracteres a extrair.")],
        "returns": "O trecho extraido como uma nova string.",
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
        "params": [("text", "String onde a busca sera feita."), ("needle", "O termo procurado.")],
        "returns": "`true` se encontrar o termo, caso contrario `false`.",
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
        "params": [("text", "String completa."), ("prefix", "Prefixo a ser testado.")],
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
        "params": [("text", "String completa."), ("suffix", "Sufixo a ser testado.")],
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
        "params": [("text", "String a ser dividida."), ("sep", "Caractere ou texto usado como delimitador.")],
        "returns": "Lista contendo as partes da string dividida.",
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
        "params": [("parts", "Lista de strings a serem unidas."), ("sep", "Separador a ser inserido entre os itens.")],
        "returns": "Uma única string resultante da uniao.",
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
        "params": [("text", "String original."), ("old", "Trecho a ser substituido."), ("new", "Novo texto a ser inserido.")],
        "returns": "Nova string com as substituicoes realizadas.",
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
        "returns": "Lista contendo cada caractere como uma string individual.",
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
        "params": [("template", "String com espacos reservados (geralmente {}) para substituicao."), ("values", "Valores que preencherao os espacos reservados.")],
        "returns": "String final formatada.",
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
        "params": [("end", "Limite exclusivo da sequencia.")],
        "returns": "Lista de numeros inteiros.",
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
        "params": [("values", "Lista a ser ordenada.")],
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
        "returns": "Lista com os elementos em ordem inversa.",
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
        "params": [("values", "Lista com possiveis duplicatas.")],
        "returns": "Lista contendo apenas elementos únicos.",
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
        "params": [("values", "Lista contendo sublistas.")],
        "returns": "Uma única lista com todos os elementos dos sub-niveis.",
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
        "params": [("value", "Valor a ser testado.")],
        "returns": "`true` se for nil.",
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
        "params": [("value", "Valor a ser testado.")],
        "returns": "`true` se for string.",
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
        "params": [("value", "Valor a ser testado.")],
        "returns": "`true` se for um objeto dinamico.",
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
        "params": [("path", "Caminho do arquivo no sistema.")],
        "returns": "Conteudo do arquivo como string.",
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
        "params": [("path", "Caminho do arquivo."), ("content", "Texto a ser gravado.")],
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
        "params": [("path", "Caminho do arquivo."), ("content", "Texto a ser anexado.")],
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
        "params": [("path", "Caminho a verificar.")],
        "returns": "`true` se o caminho existe.",
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
        "params": [("path", "Caminho do arquivo a ser deletado.")],
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
        "params": [("path", "Diretorio a ser listado.")],
        "returns": "Lista de strings com os nomes dos arquivos/pastas.",
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
        "params": [("path", "Caminho a verificar.")],
        "returns": "`true` se for arquivo.",
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
        "params": [("path", "Caminho a verificar.")],
        "returns": "`true` se for diretorio.",
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
        "params": [("path", "Caminho do novo diretorio.")],
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
        "params": [("url", "URL absoluta do recurso.")],
        "returns": "Dicionario com status, headers e body.",
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
        "params": [("url", "URL absoluta."), ("body", "Conteudo do corpo da requisicao.")],
        "returns": "Nada.",
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
        "returns": "Timestamp (segundos desde a Epoch).",
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
        "params": [("seconds", "Duracao da pausa em segundos.")],
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
        "params": [("code", "Codigo de saida (0 para sucesso, 1+ para erro).")],
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
        "returns": "Lista de strings com os argumentos.",
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
        "params": [("name", "Nome da variavel de ambiente.")],
        "returns": "Valor da variavel ou string vazia.",
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
        "params": [("name", "Nome da variavel."), ("value", "Valor a ser definido.")],
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
        "returns": "Caminho completo do diretorio de trabalho.",
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
        "returns": "Nome da plataforma (linux, windows, darwin, etc).",
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
        "returns": "Nome da arquitetura (x86_64, arm64, etc).",
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
            "params": [("value", "Valor numerico de entrada a ser truncado ou estendido.")],
            "returns": f"Valor convertido para `{typ}`.",
            "example": f"let n: {typ} = {name}(255)",
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, summary, ret, example, params in [
    ("lo_u8", "Extrai o byte baixo de uma word.", "u8", "let lo: u8 = lo_u8(0x1234)", [("value", "Valor de 16 bits ou mais.")]),
    ("hi_u8", "Extrai o byte alto de uma word.", "u8", "let hi: u8 = hi_u8(0x1234)", [("value", "Valor de 16 bits ou mais.")]),
    ("make_u16", "Combina byte baixo e alto em word little-endian.", "any", "let pc: u16 = make_u16(0x34, 0x12)", [("lo", "Byte baixo."), ("hi", "Byte alto.")]),
    ("is_zero_u8", "Testa flag zero para valor de 8 bits.", "bool", "let z = is_zero_u8(0)", [("value", "Valor a testar.")]),
    ("is_negative_u8", "Testa bit 7 como flag negativa do 6502.", "bool", "let n = is_negative_u8(0x80)", [("value", "Valor a testar.")]),
    ("bit_test", "Testa se um bit esta ligado.", "bool", "let carry = bit_test(status, 0)", [("value", "Valor base."), ("bit", "Indice do bit (base-0).")]),
    ("bit_set", "Liga um bit.", "any", "let status = bit_set(0, 7)", [("value", "Valor base."), ("bit", "Indice do bit.")]),
    ("bit_clear", "Desliga um bit.", "any", "let status = bit_clear(0xFF, 7)", [("value", "Valor base."), ("bit", "Indice do bit.")]),
    ("bit_toggle", "Inverte um bit.", "any", "let status = bit_toggle(0, 1)", [("value", "Valor base."), ("bit", "Indice do bit.")]),
    ("bit_write", "Liga ou desliga bit conforme booleano/valor.", "any", "let status = bit_write(0, 1, true)", [("value", "Valor base."), ("bit", "Indice do bit."), ("enabled", "Booleano ou 0/1.")]),
    ("flag_has", "Alias semantico de `bit_test` para registradores de flags.", "bool", "let ok = flag_has(status, 2)", [("value", "Registrador."), ("bit", "Indice da flag.")]),
    ("flag_set", "Alias semantico de `bit_set` para registradores de flags.", "any", "let status = flag_set(status, 2)", [("value", "Registrador."), ("bit", "Indice da flag.")]),
    ("flag_clear", "Alias semantico de `bit_clear` para registradores de flags.", "any", "let status = flag_clear(status, 2)", [("value", "Registrador."), ("bit", "Indice da flag.")]),
    ("flag_write", "Alias semantico de `bit_write` para flags de CPU.", "any", "let status = flag_write(status, 1, true)", [("value", "Registrador."), ("bit", "Indice da flag."), ("enabled", "Estado.")]),
    ("wrapping_inc", "Incrementa com wrap.", "any", "let x: u8 = wrapping_inc(255, 1)", [("value", "Valor base.")]),
    ("wrapping_dec", "Decrementa com wrap.", "any", "let x: u8 = wrapping_dec(0, 1)", [("value", "Valor base.")]),
]:
    arity = "value: any" if name in ("lo_u8", "hi_u8", "is_zero_u8", "is_negative_u8", "wrapping_inc", "wrapping_dec") else "value: any, bit: any"
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
            "params": params,
            "returns": f"Retorna `{ret}`.",
            "example": example,
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, summary, params in [
    ("carry_add_u8", "Calcula carry de soma de 8 bits, incluindo carry de entrada.", [("a", "Operando A."), ("b", "Operando B."), ("carry", "Carry de entrada (0 ou 1).")]),
    ("borrow_sub_u8", "Calcula borrow de subtracao de 8 bits.", [("a", "Minuendo."), ("b", "Subtraendo."), ("borrow", "Borrow de entrada.")]),
    ("overflow_add_i8", "Calcula overflow assinado de soma de 8 bits.", [("a", "Operando A."), ("b", "Operando B."), ("carry", "Carry de entrada.")]),
    ("overflow_sub_i8", "Calcula overflow assinado de subtracao de 8 bits.", [("a", "Minuendo."), ("b", "Subtraendo."), ("borrow", "Borrow de entrada.")]),
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
            "params": params,
            "returns": "Booleano da condicao.",
            "example": f"let flag = {name}(0xFF, 0x01, 0)",
            "test": "docs/examples/reference/systems_bits.snask",
        }
    )

for name, sig, summary, ret, example, params in [
    ("mem_alloc", "mem_alloc(size: any) -> ptr", "Aloca memoria crua sem zerar.", "Ponteiro para bloco alocado.", "let p: ptr = mem_alloc(256)", [("size", "Quantidade de bytes a alocar.")]),
    ("mem_alloc_zero", "mem_alloc_zero(size: any) -> ptr", "Aloca memoria crua zerada.", "Ponteiro para bloco alocado.", "let p: ptr = mem_alloc_zero(65536)", [("size", "Quantidade de bytes a alocar e zerar.")]),
    ("mem_free", "mem_free(ptr: ptr) -> void", "Libera bloco alocado manualmente.", "Nada.", "mem_free(p)", [("ptr", "Ponteiro para o inicio do bloco alocado anteriormente.")]),
    ("ptr_add", "ptr_add(ptr: ptr, offset: any) -> ptr", "Retorna ponteiro deslocado por bytes.", "Novo ponteiro.", "let p2: ptr = ptr_add(p, 16)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes (positivo ou negativo).")]),
    ("mem_read_u8", "mem_read_u8(ptr: ptr, offset: any) -> u8", "Le byte em offset.", "Valor `u8`.", "let b: u8 = mem_read_u8(p, 0)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes.")]),
    ("mem_read_u16", "mem_read_u16(ptr: ptr, offset: any) -> u16", "Le word little-endian em offset.", "Valor `u16`.", "let w: u16 = mem_read_u16(p, 0)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes.")]),
    ("mem_read_u32", "mem_read_u32(ptr: ptr, offset: any) -> u32", "Le dword little-endian em offset.", "Valor `u32`.", "let d: u32 = mem_read_u32(p, 0)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes.")]),
    ("mem_write_u8", "mem_write_u8(ptr: ptr, offset: any, value: any) -> void", "Escreve byte em offset.", "Nada.", "mem_write_u8(p, 0, 0xEA)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes."), ("value", "Valor u8 a escrever.")]),
    ("mem_write_u16", "mem_write_u16(ptr: ptr, offset: any, value: any) -> void", "Escreve word little-endian em offset.", "Nada.", "mem_write_u16(p, 0, 0x8000)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes."), ("value", "Valor u16 a escrever.")]),
    ("mem_write_u32", "mem_write_u32(ptr: ptr, offset: any, value: any) -> void", "Escreve dword little-endian em offset.", "Nada.", "mem_write_u32(p, 0, 0x12345678)", [("ptr", "Ponteiro base."), ("offset", "Deslocamento em bytes."), ("value", "Valor u32 a escrever.")]),
    ("mem_fill_u8", "mem_fill_u8(ptr: ptr, value: any, size: any) -> void", "Preenche bloco com byte.", "Nada.", "mem_fill_u8(p, 0, 256)", [("ptr", "Ponteiro base."), ("value", "Valor byte a repetir."), ("size", "Numero de bytes a preencher.")]),
    ("mem_copy", "mem_copy(dst: ptr, src: ptr, size: any) -> void", "Copia bytes entre blocos.", "Nada.", "mem_copy(dst, src, 256)", [("dst", "Ponteiro de destino."), ("src", "Ponteiro de origem."), ("size", "Numero de bytes a copiar.")]),
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
            "params": params,
            "returns": ret,
            "example": "@unsafe {\n    " + example + "\n}",
            "test": "docs/examples/reference/systems_memory.snask",
        }
    )

for name, sig, summary, params in [
    ("json_parse", "json_parse(text: str) -> any", "Converte texto JSON em valor dinamico.", [("text", "String contendo o JSON válido.")]),
    ("json_stringify", "json_stringify(value: any) -> str", "Serializa valor dinamico para JSON compacto.", [("value", "Objeto, lista ou valor primitivo a ser serializado.")]),
    ("json_stringify_pretty", "json_stringify_pretty(value: any) -> str", "Serializa valor dinamico para JSON formatado.", [("value", "Valor a ser serializado com indentação.")]),
    ("json_get", "json_get(value: any, key: str) -> any", "Le campo de objeto JSON.", [("value", "Objeto JSON."), ("key", "Chave do campo desejado.")]),
    ("json_has", "json_has(value: any, key: str) -> bool", "Testa existencia de campo JSON.", [("value", "Objeto JSON."), ("key", "Chave a verificar.")]),
    ("json_len", "json_len(value: any) -> float", "Retorna tamanho de array/objeto JSON.", [("value", "Objeto ou Array JSON.")]),
    ("json_index", "json_index(value: any, index: float) -> any", "Le item de array JSON.", [("value", "Array JSON."), ("index", "Indice do item (base-0).")]),
    ("json_set", "json_set(value: any, key: str, item: any) -> bool", "Define campo JSON.", [("value", "Objeto JSON."), ("key", "Chave."), ("item", "Novo valor.")]),
    ("json_keys", "json_keys(value: any) -> any", "Retorna chaves de objeto JSON.", [("value", "Objeto JSON.")]),
    ("json_parse_ex", "json_parse_ex(text: str) -> any", "Parse JSON com superficie extendida do runtime.", [("text", "Texto JSON.")]),
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
            "params": params,
            "returns": "Conforme assinatura.",
            "example": 'import "json"\n\nclass main {\n    fun start() {\n        let obj = json::parse("{\\"name\\":\\"snask\\"}")\n        print(json::get(obj, "name"))\n    }\n}',
            "test": "docs/examples/reference/json_basic.snask",
        }
    )

for name, sig, summary, params in [
    ("sfs_read", "sfs_read(path: str) -> str", "Le arquivo pela biblioteca SFS.", [("path", "Caminho do arquivo.")]),
    ("sfs_write", "sfs_write(path: str, content: str) -> bool", "Escreve arquivo pela biblioteca SFS.", [("path", "Caminho."), ("content", "Conteúdo a gravar.")]),
    ("sfs_append", "sfs_append(path: str, content: str) -> bool", "Acrescenta texto em arquivo SFS.", [("path", "Caminho."), ("content", "Texto a anexar.")]),
    ("sfs_exists", "sfs_exists(path: str) -> bool", "Testa existencia pelo SFS.", [("path", "Caminho.")]),
    ("sfs_delete", "sfs_delete(path: str) -> bool", "Remove arquivo pelo SFS.", [("path", "Caminho.")]),
    ("sfs_copy", "sfs_copy(src: str, dst: str) -> bool", "Copia arquivo pelo SFS.", [("src", "Origem."), ("dst", "Destino.")]),
    ("sfs_move", "sfs_move(src: str, dst: str) -> bool", "Move arquivo pelo SFS.", [("src", "Origem."), ("dst", "Destino.")]),
    ("sfs_mkdir", "sfs_mkdir(path: str) -> bool", "Cria diretorio pelo SFS.", [("path", "Novo diretório.")]),
    ("sfs_rmdir", "sfs_rmdir(path: str) -> bool", "Remove diretorio pelo SFS.", [("path", "Diretório a remover.")]),
    ("sfs_is_file", "sfs_is_file(path: str) -> bool", "Testa arquivo pelo SFS.", [("path", "Caminho.")]),
    ("sfs_is_dir", "sfs_is_dir(path: str) -> bool", "Testa diretorio pelo SFS.", [("path", "Caminho.")]),
    ("sfs_listdir", "sfs_listdir(path: str) -> any", "Lista diretorio pelo SFS.", [("path", "Diretório.")]),
    ("sfs_size", "sfs_size(path: str) -> float", "Tamanho de arquivo.", [("path", "Caminho.")]),
    ("sfs_mtime", "sfs_mtime(path: str) -> float", "Tempo de modificacao.", [("path", "Caminho.")]),
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
            "params": params,
            "returns": "Conforme assinatura.",
            "example": 'import "sfs"\n\nclass main {\n    fun start() {\n        let ok = sfs::exists(".")\n        print(ok)\n    }\n}',
            "test": "docs/examples/reference/sfs_basic.snask",
        }
    )

for name, sig, summary, params in [
    ("gui_init", "gui_init() -> bool", "Inicializa runtime GTK.", []),
    ("gui_run", "gui_run() -> void", "Entra no loop principal da GUI.", []),
    ("gui_quit", "gui_quit() -> void", "Sai do loop principal da GUI.", []),
    ("gui_window", "gui_window(title: str, width: float, height: float) -> any", "Cria janela nativa.", [("title", "Título da janela."), ("width", "Largura."), ("height", "Altura.")]),
    ("gui_label", "gui_label(text: str) -> any", "Cria label.", [("text", "Texto inicial do label.")]),
    ("gui_button", "gui_button(text: str) -> any", "Cria botao.", [("text", "Rótulo do botão.")]),
    ("gui_entry", "gui_entry() -> any", "Cria input de texto.", []),
    ("gui_textview", "gui_textview() -> any", "Cria area de texto.", []),
    ("gui_vbox", "gui_vbox() -> any", "Cria container vertical.", []),
    ("gui_hbox", "gui_hbox() -> any", "Cria container horizontal.", []),
    ("gui_add", "gui_add(parent: any, child: any) -> bool", "Adiciona widget a container.", [("parent", "Container (vbox/hbox/window)."), ("child", "Widget filho.")]),
    ("gui_set_child", "gui_set_child(parent: any, child: any) -> bool", "Define filho unico de container.", [("parent", "Widget pai."), ("child", "Widget filho.")]),
    ("gui_show_all", "gui_show_all(widget: any) -> bool", "Mostra widget e descendentes.", [("widget", "Widget base a ser exibido.")]),
    ("gui_set_text", "gui_set_text(widget: any, text: str) -> bool", "Define texto de widget suportado.", [("widget", "Label, Botão ou Entry."), ("text", "Novo texto.")]),
    ("gui_get_text", "gui_get_text(widget: any) -> str", "Le texto de widget suportado.", [("widget", "Widget para leitura.")]),
    ("gui_on_click", "gui_on_click(widget: any, handler: any) -> bool", "Conecta handler de clique.", [("widget", "Botão."), ("handler", "Função a ser chamada no clique.")]),
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
            "params": params,
            "returns": "Conforme assinatura.",
            "example": 'import "gui"\n\nclass main {\n    fun start() {\n        let ok = gui::init()\n        let win = gui::window("Snask", 320, 180)\n        gui::show_all(win)\n        gui::run()\n    }\n}',
            "test": "docs/examples/reference/gui_minimal.snask",
        }
    )

for name, sig, summary, params in [
    ("snaskgui_init", "snaskgui_init() -> bool", "Inicializa API framebuffer para jogos/emuladores.", []),
    ("snaskgui_window", "snaskgui_window(title: str, width: float, height: float, scale: float) -> any", "Cria janela framebuffer.", [("title", "Título da janela."), ("width", "Largura lógica em pixels."), ("height", "Altura lógica em pixels."), ("scale", "Fator de escala de exibição (ex: 2.0 para 2x).")]),
    ("snaskgui_present_rgba", "snaskgui_present_rgba(window: any, pixels: ptr, width: float, height: float) -> bool", "Apresenta buffer RGBA.", [("window", "Handle da janela retornado por snaskgui_window."), ("pixels", "Ponteiro para o buffer de pixels RGBA."), ("width", "Largura do buffer."), ("height", "Altura do buffer.")]),
    ("snaskgui_poll", "snaskgui_poll(window: any) -> bool", "Processa eventos pendentes.", [("window", "Handle da janela.")]),
    ("snaskgui_key_down", "snaskgui_key_down(window: any, key: float) -> bool", "Consulta estado de tecla.", [("window", "Handle da janela."), ("key", "Código numérico da tecla.")]),
    ("snaskgui_should_close", "snaskgui_should_close(window: any) -> bool", "Testa fechamento da janela.", [("window", "Handle da janela.")]),
    ("snaskgui_delay", "snaskgui_delay(ms: float) -> void", "Pausa em milissegundos.", [("ms", "Tempo em milissegundos.")]),
    ("snaskgui_close", "snaskgui_close(window: any) -> void", "Fecha janela framebuffer.", [("window", "Handle da janela.")]),
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
            "params": params,
            "returns": "Conforme assinatura.",
            "example": 'import "snaskgui"\n\nclass main {\n    fun start() {\n        snaskgui::init()\n        let win = snaskgui::window("Pixels", 256, 240, 2)\n        snaskgui::poll(win)\n        snaskgui::close(win)\n    }\n}',
            "test": "docs/examples/reference/snaskgui_minimal.snask",
        }
    )


DEEP_NOTES = {
    "snaskgui_init": {
        "purpose": [
            "Inicializa o subsistema de vídeo e eventos do SnaskGUI. Esta deve ser a primeira função chamada antes de qualquer tentativa de criar janelas ou manipular gráficos no perfil systems.",
            "Ela configura os drivers internos (como SDL2 ou DRM, dependendo da plataforma) e prepara o ambiente para renderização de alta performance.",
        ],
        "example": """import \"snaskgui\"

class main {
    fun start() {
        if snaskgui::init() {
            print(\"Sistema gráfico pronto!\\n\")
            // Prossiga com snaskgui::window
        } else {
            print(\"Erro ao inicializar gráficos.\\n\")
        }
    }
}""",
        "pitfalls": [
            "Chamar qualquer outra função `snaskgui_*` antes desta resultará em comportamento indefinido ou crash.",
            "Em sistemas Linux sem ambiente gráfico (X11/Wayland), esta função pode falhar se o driver de framebuffer não estiver disponível.",
        ],
        "related": ["snaskgui_window", "snaskgui_close"],
    },
    "snaskgui_window": {
        "purpose": [
            "Cria uma janela nativa otimizada para exibição de pixels (framebuffer), ideal para emuladores, jogos retrô e ferramentas de visualização de dados.",
            "Diferente de interfaces baseadas em widgets (botões, inputs), esta janela é uma 'tela em branco' onde você tem controle total sobre cada pixel enviado via `snaskgui_present_rgba`.",
            "O parâmetro `scale` é crucial: ele permite que você renderize em uma resolução baixa (ex: 256x240 do NES) e a janela seja exibida em um tamanho confortável (ex: 768x720 com scale 3) sem perder a nitidez dos pixels.",
        ],
        "example": """import \"snaskgui\"

class main {
    fun start() {
        snaskgui::init()
        // Janela com título \"Snask\", 320x200 pixels, escala 2x (640x400 físico)
        let win = snaskgui::window(\"Snask Game\", 320, 200, 2)

        while snaskgui::should_close(win) == false {
            snaskgui::poll(win)
            // Lógica de desenho aqui
            snaskgui::delay(16)
        }

        snaskgui::close(win)
    }
}""",
        "pitfalls": [
            "O retorno é um handle opaco. Não tente realizar operações aritméticas com ele.",
            "Largura e altura são resoluções *lógicas*. Se você criar uma janela 100x100 com escala 10, ela ocupará 1000x1000 pixels na tela.",
            "Não confunda com `gui_window`, que é para interfaces GTK/Widgets.",
        ],
        "related": ["snaskgui_init", "snaskgui_present_rgba", "snaskgui_poll", "snaskgui_close"],
    },
    "snaskgui_present_rgba": {
        "purpose": [
            "A função principal de renderização. Ela pega um bloco de memória bruta contendo pixels no formato RGBA (4 bytes por pixel) e os 'carimba' na janela.",
            "É extremamente rápida pois utiliza aceleração de hardware (quando disponível) para converter o buffer de memória em uma textura de tela.",
        ],
        "example": """import \"snaskgui\"

class main {
    fun start() {
        snaskgui::init()
        let win = snaskgui::window(\"Video\", 256, 256, 2)
        
        // Exemplo: preencher memória com cor sólida (estático)
        @unsafe {
            let size = 256 * 256 * 4
            let pixels = mem_alloc_zero(size)
            
            // Loop principal
            while snaskgui::should_close(win) == false {
                snaskgui::poll(win)
                snaskgui::present_rgba(win, pixels, 256, 256)
                snaskgui::delay(16)
            }
        }
    }
}""",
        "pitfalls": [
            "Deve ser usada dentro de um bloco `@unsafe` pois lida com ponteiros (`ptr`).",
            "O buffer de pixels deve ter exatamente `width * height * 4` bytes. Se for menor, haverá leitura inválida de memória (Buffer Overflow).",
            "A ordem dos bytes é R, G, B, A (8 bits cada).",
        ],
        "related": ["snaskgui_window", "mem_alloc_zero", "snaskgui_poll"],
    },
    "mem_alloc_zero": {
        "purpose": [
            "Aloca um bloco de memória contígua e inicializa todos os seus bytes com o valor zero. É o equivalente ao `calloc(size, 1)` da linguagem C.",
            "Esta função é preferível a `mem_alloc` quando a segurança e o determinismo são importantes, pois garante que não haverá 'lixo' de memória (dados de processos anteriores) no bloco alocado.",
            "O ponteiro retornado pertence ao gerenciamento manual de memória do perfil systems e não é rastreado pelo coletor de lixo ou pelo sistema de zonas automático fora do seu escopo de criação.",
        ],
        "example": """@unsafe {
    // Aloca 64KB de RAM zerada para uma CPU 6502
    let ram: ptr = mem_alloc_zero(65536)
    
    // O primeiro byte terá o valor 0
    let first = mem_read_u8(ram, 0)
    
    // Sempre libere a memória manualmente
    mem_free(ram)
}""",
        "pitfalls": [
            "Causa vazamento de memória (Memory Leak) se `mem_free` não for chamado.",
            "Pode retornar um ponteiro nulo (nil) se o sistema ficar sem memória disponível.",
            "Acesso fora dos limites (out-of-bounds) do tamanho solicitado resultará em corrupção de memória ou falha de segmentação.",
        ],
        "related": ["mem_alloc", "mem_free", "mem_fill_u8"],
    },
    "mem_free": {
        "purpose": [
            "Libera um bloco de memória previamente alocado por `mem_alloc` ou `mem_alloc_zero`, devolvendo-o ao sistema operacional ou ao heap global.",
            "É uma função crítica para a saúde de aplicações de longa duração (como servidores ou emuladores), pois evita o consumo excessivo de recursos (leaks).",
        ],
        "example": """@unsafe {
    let buffer = mem_alloc(1024)
    // ... uso do buffer ...
    mem_free(buffer)
    // ATENÇÃO: 'buffer' agora é um ponteiro pendente (dangling pointer). 
    // Não tente lê-lo ou escrevê-lo novamente.
}""",
        "pitfalls": [
            "Double Free: Tentar liberar o mesmo ponteiro duas vezes causará um crash imediato da aplicação por segurança.",
            "Invalid Free: Tentar liberar um ponteiro que não foi retornado por uma função de alocação (ou um ponteiro deslocado via `ptr_add`) é um erro fatal.",
        ],
        "related": ["mem_alloc", "mem_alloc_zero"],
    },
    "mem_read_u8": {
        "purpose": [
            "Lê um único byte (8 bits) de um endereço de memória física especificado por um ponteiro base e um deslocamento (offset).",
            "É a primitiva fundamental para implementar barramentos de memória, leitura de cabeçalhos binários e processamento de buffers de rede.",
        ],
        "example": """@unsafe {
    // Lê o byte no offset 0x10 de um buffer
    let status: u8 = mem_read_u8(buffer, 0x10)
    
    if bit_test(status, 7) {
        print(\"Flag ativa!\\n\")
    }
}""",
        "pitfalls": [
            "O offset não é validado em tempo de execução. Se você ler `buffer + size`, lerá dados arbitrários.",
            "Lentidão: Chamadas repetidas em um loop apertado podem ser otimizadas pelo compilador, mas prefira operações de bloco (`mem_copy`) se for mover muitos dados.",
        ],
        "related": ["mem_write_u8", "mem_read_u16", "mem_read_u32"],
    },
    "bit_test": {
        "purpose": [
            "Verifica se um bit específico dentro de um valor inteiro está ligado (1) ou desligado (0).",
            "Extremamente útil para decodificar registros de status de hardware, protocolos binários e sistemas de flags de baixo consumo de memória.",
        ],
        "example": """let flags = 0b10100000
// Testa o bit 7 (o mais significativo em um u8)
if bit_test(flags, 7) {
    print(\"Bit 7 está ON\\n\")
}""",
        "pitfalls": [
            "Os índices de bits são base-0, indo de 0 (bit menos significativo) até 7 (para u8), 15 (u16), etc.",
            "Passar um índice maior que o tamanho do tipo de dado resultará sempre em false ou comportamento indefinido.",
        ],
        "related": ["bit_set", "bit_clear", "bit_toggle"],
    },
    "wrapping_add": {
        "purpose": [
            "Realiza uma soma aritmética que ignora o estouro (overflow). Se o resultado for maior que o valor máximo suportado pelo tipo, ele 'dá a volta' (wrap around) para zero.",
            "Esta é a semântica padrão de CPUs reais e é obrigatória para a implementação correta de lógica de registradores de CPU (como o Program Counter ou o acumulador de um 6502/Z80).",
        ],
        "example": """// Em um u8, o valor máximo é 255.
let a: u8 = 255
let b: u8 = 1
let res = wrapping_add(a, b)
// res será 0, e não um erro de overflow ou 256.
""",
        "pitfalls": [
            "Não use esta função para lógica de negócios (financeira, contagem) onde um estouro deve ser tratado como erro. Use apenas em lógica de sistemas.",
        ],
        "related": ["wrapping_sub", "saturating_add"],
    },
    "len": {
        "purpose": [
            "Retorna a quantidade de elementos ou o tamanho dimensional de um valor.",
            "Para **Strings**, retorna o número de caracteres (atualmente mapeado para bytes na versão v0.4).",
            "Para **Listas**, retorna o número de itens armazenados.",
            "Para **Objetos Dinâmicos/JSON**, retorna o número de chaves de alto nível ou itens em um array.",
        ],
        "example": """let texto = \"Snask\"
let tamanho = len(texto) // 5

let lista = [10, 20, 30]
print(len(lista)) // 3""",
        "pitfalls": [
            "Em strings com caracteres multi-byte (Unicode/UTF-8 complexos), o comportamento atual pode retornar o tamanho em bytes, não em glifos visuais.",
        ],
        "related": ["substring", "chars"],
    },
    "substring": {
        "purpose": [
            "Extrai uma porção de uma string original começando em um índice específico e capturando um comprimento determinado.",
            "É a ferramenta padrão para parsing manual de textos, extração de prefixos ou limpeza de dados.",
        ],
        "example": """let email = \"usuario@snask.org\"
// Extrai \"usuario\" (começa no 0, pega 7 caracteres)
let user = substring(email, 0, 7)""",
        "pitfalls": [
            "Se o índice de início + comprimento ultrapassar o tamanho da string, a função pode lançar um erro ou retornar uma string vazia dependendo do modo de segurança do runtime.",
            "Índices começam em 0.",
        ],
        "related": ["len", "split", "replace"],
    },
    "split": {
        "purpose": [
            "Divide uma string em uma lista de strings menores, utilizando um separador (delimitador) específico.",
            "Muito utilizada para processar arquivos CSV, caminhos de diretórios ou entradas de usuário formatadas.",
        ],
        "example": """let csv = \"maca,banana,uva\"
let frutas = split(csv, \",\")
// Resultado: [\"maca\", \"banana\", \"uva\"]""",
        "pitfalls": [
            "Se o separador for uma string vazia, o comportamento depende da versão do runtime (pode dividir caractere a caractere ou retornar a string original).",
            "Separadores adjacentes resultam em strings vazias na lista resultante.",
        ],
        "related": ["join", "substring"],
    },
    "join": {
        "purpose": [
            "Concatena todos os elementos de uma lista de strings em uma única string, inserindo um separador entre cada elemento.",
            "É a operação inversa do `split`.",
        ],
        "example": """let partes = [\"usr\", \"bin\", \"snask\"]
let caminho = join(partes, \"/\")
// Resultado: \"usr/bin/snask\"""",
        "pitfalls": [
            "Todos os elementos da lista devem ser do tipo string. Se houver números ou nils, a função pode falhar se não houver conversão implícita.",
        ],
        "related": ["split", "format"],
    },
    "str_to_num": {
        "purpose": [
            "Tenta converter uma representação textual (string) em um valor numérico (float).",
            "Suporta notação decimal e, em alguns runtimes, notação científica.",
        ],
        "example": """let preco_texto = \"42.50\"
let valor = str_to_num(preco_texto)
let total = valor * 2 // 85.0""",
        "pitfalls": [
            "Se a string contiver caracteres não numéricos (ex: \"abc\"), a função retornará 0 ou um erro de diagnóstico dependendo do perfil.",
            "Sempre limpe a string com `trim` antes de converter para evitar erros por espaços em branco.",
        ],
        "related": ["num_to_str", "trim"],
    },
    "read_file": {
        "purpose": [
            "Lê o conteúdo completo de um arquivo de texto e o retorna como uma string única.",
            "É a forma mais rápida de carregar configurações, scripts ou pequenos bancos de dados em memória.",
        ],
        "example": """let config = read_file(\"settings.json\")
print(\"Configuração carregada: {len(config)} bytes\\n\")""",
        "pitfalls": [
            "Não recomendado para arquivos gigantes (GBs), pois carrega tudo no heap de uma vez.",
            "Se o arquivo não existir ou o processo não tiver permissão de leitura, a função pode lançar uma exceção ou retornar nil.",
        ],
        "related": ["write_file", "exists", "sfs_read"],
    },
    "write_file": {
        "purpose": [
            "Cria um novo arquivo ou sobrescreve um existente com o conteúdo de texto fornecido.",
            "Se o arquivo já existir, todo o conteúdo anterior será perdido.",
        ],
        "example": """let log = \"Sessão iniciada em {time()}\\n\"
write_file(\"log.txt\", log)""",
        "pitfalls": [
            "Operação destrutiva: não há aviso antes de sobrescrever arquivos.",
            "Certifique-se de que o diretório pai existe antes de chamar esta função.",
        ],
        "related": ["append_file", "read_file", "delete"],
    },
    "time": {
        "purpose": [
            "Retorna o tempo atual do sistema. No perfil humane, geralmente retorna o número de segundos decorridos desde a 'Epoch' (1 de Janeiro de 1970).",
            "É essencial para medir performance de algoritmos, criar timestamps de logs ou controlar o tempo em loops de lógica simples.",
        ],
        "example": """let inicio = time()
// ... executa algo pesado ...
let fim = time()
print(\"Tempo decorrido: {fim - inicio} segundos\\n\")""",
        "pitfalls": [
            "A precisão (milissegundos vs segundos) pode variar entre plataformas e runtimes.",
        ],
        "related": ["sleep", "snaskgui_delay"],
    },
    "sleep": {
        "purpose": [
            "Pausa a execução do thread atual por um período determinado de segundos.",
            "Útil para evitar consumo excessivo de CPU em loops de espera ou para coordenar processos temporizados.",
        ],
        "example": """print(\"Aguardando...\\n\")
sleep(1.5) // pausa por 1 segundo e meio
print(\"Pronto!\\n\")""",
        "pitfalls": [
            "Pausar o thread principal em aplicações GUI fará com que a janela pare de responder (congelamento).",
        ],
        "related": ["time", "snaskgui_delay"],
    },
    "exit": {
        "purpose": [
            "Encerra imediatamente o processo Snask atual com um código de saída específico.",
            "Por convenção, `0` indica sucesso e qualquer outro número indica um erro ou estado específico.",
        ],
        "example": """if exists(\"config.txt\") == false {
    print(\"Erro fatal: arquivo de config ausente!\\n\")
    exit(1)
}""",
        "pitfalls": [
            "Esta função é terminal: nada que venha depois dela no código será executado.",
        ],
        "related": ["args"],
    },
    "sort": {
        "purpose": [
            "Ordena os elementos de uma lista. Para listas de números, a ordem é crescente. Para strings, é lexicográfica.",
            "No Snask v0.4, a ordenação é in-place (modifica a lista original) ou retorna uma nova cópia dependendo do tipo da lista.",
        ],
        "example": """let notas = [9, 5, 10, 2]
let ordenadas = sort(notas)
// [2, 5, 9, 10]""",
        "pitfalls": [
            "Misturar tipos incompatíveis na lista (ex: [1, \"a\"]) pode causar erros de comparação.",
        ],
        "related": ["reverse", "unique"],
    },
    "json_parse": {
        "purpose": [
            "Converte uma string contendo dados no formato JSON em um objeto dinâmico ou lista manipulável pelo Snask.",
            "É a base para integração com APIs Web e leitura de arquivos de configuração modernos.",
        ],
        "example": """import \"json\"
let raw = \"{\\\"id\\\": 1, \\\"active\\\": true}\"
let obj = json::parse(raw)
print(json::get(obj, \"id\")) // 1""",
        "pitfalls": [
            "Se o JSON for inválido, a função pode retornar nil ou lançar um erro de parsing.",
            "Strings JSON devem usar aspas duplas escapadas se definidas dentro de literais Snask.",
        ],
        "related": ["json_stringify", "json_get", "http_get"],
    },
    "json_stringify": {
        "purpose": [
            "Transforma um objeto ou lista Snask em uma string no formato JSON compacto.",
            "Útil para salvar estados de jogos, enviar dados para APIs ou criar logs estruturados.",
        ],
        "example": """import \"json\"
let player = {\"score\": 1500, \"name\": \"Snasker\"}
let json_text = json::stringify(player)
// Resultado: \"{\\\"score\\\":1500,\\\"name\\\":\\\"Snasker\\\"}\" """,
        "pitfalls": [
            "Não suporta a serialização de funções ou handles opacos (como janelas ou ponteiros).",
        ],
        "related": ["json_parse", "json_stringify_pretty"],
    },
    "sfs_read": {
        "purpose": [
            "Lê o conteúdo de um arquivo através da biblioteca SFS (Snask File System), que oferece uma camada de abstração mais robusta que o `read_file` básico.",
            "Geralmente associada a runtimes que suportam sandboxing ou sistemas de arquivos virtuais.",
        ],
        "example": """import \"sfs\"
if sfs::exists(\"data.txt\") {
    let content = sfs::read(\"data.txt\")
    print(content)
}""",
        "pitfalls": [
            "Caminhos podem ser relativos à raiz do projeto ou absolutos, dependendo da configuração do SFS no runtime.",
        ],
        "related": ["sfs_write", "read_file", "sfs_exists"],
    },
    "carry_add_u8": {
        "purpose": [
            "Calcula o bit de carry (transporte) resultante de uma soma de 8 bits. É fundamental para implementar somas de precisão arbitrária (ex: somar dois números de 16 bits usando operações de 8 bits).",
            "A função leva em conta o bit de carry de entrada, simulando exatamente o comportamento da instrução `ADC` (Add with Carry) de processadores como o 6502.",
        ],
        "example": """// Soma 255 + 1 com carry de entrada 0
let carry = carry_add_u8(255, 1, 0)
// carry será true, pois 255 + 1 estoura 8 bits.
""",
        "pitfalls": [
            "O retorno é um booleano (true/false) representando se houve transporte.",
        ],
        "related": ["borrow_sub_u8", "overflow_add_i8", "wrapping_add"],
    },
    "lo_u8": {
        "purpose": [
            "Extrai o byte menos significativo (Low Byte) de um valor de 16 bits ou maior.",
            "Em sistemas Little Endian (como o Snask), este é o primeiro byte da representação em memória.",
        ],
        "example": """let word = 0xABCD
let lo = lo_u8(word) // 0xCD""",
        "pitfalls": [
            "Aplica uma máscara automática `& 0xFF` no valor de entrada.",
        ],
        "related": ["hi_u8", "make_u16"],
    },
    "bit_write": {
        "purpose": [
            "Define o estado de um bit específico (0 ou 1) baseado em um valor booleano ou numérico.",
            "É uma forma declarativa de modificar flags sem precisar lidar manualmente com operadores de máscara (`|` e `&`).",
        ],
        "example": """mut status = 0b00000000
let condicao = true
status = bit_write(status, 7, condicao)
// status agora é 0b10000000""",
        "pitfalls": [
            "O índice do bit deve estar dentro do intervalo do tipo de dado (0-7 para u8, etc).",
        ],
        "related": ["bit_set", "bit_clear", "flag_write"],
    },
    "abs": {
        "purpose": [
            "Retorna o valor absoluto (módulo) de um número real.",
            "Transforma números negativos em positivos, mantendo números positivos ou zero inalterados.",
        ],
        "example": """let delta = -42.5
let magnitude = abs(delta) // 42.5""",
        "pitfalls": [
            "Em sistemas de ponto flutuante, `abs(-0.0)` retorna `0.0`.",
        ],
        "related": ["sqrt", "min", "max"],
    },
    "pow": {
        "purpose": [
            "Calcula o resultado de uma base elevada a um expoente.",
            "Utiliza o algoritmo de exponenciação rápida do runtime LLVM para garantir precisão e velocidade.",
        ],
        "example": """let base = 2
let expoente = 10
let resultado = pow(base, expoente) // 1024.0""",
        "pitfalls": [
            "Resultados podem exceder o valor máximo de um float (Infinity) se o expoente for muito alto.",
            "Base negativa com expoente fracionário pode resultar em NaN (Not a Number).",
        ],
        "related": ["sqrt", "abs"],
    },
    "as_u8": {
        "purpose": [
            "Realiza uma conversão explícita (cast) de um valor para o tipo inteiro sem sinal de 8 bits (u8).",
            "Se o valor original for maior que 255, ele será truncado (aplica-se a máscara `& 0xFF`).",
            "É a forma recomendada de garantir que um valor caiba em um registrador de byte no perfil systems.",
        ],
        "example": """let valor_grande = 300
let byte = as_u8(valor_grande) // 44 (pois 300 & 0xFF = 44)""",
        "pitfalls": [
            "Diferente do perfil humane, esta função não lança erro em caso de estouro; ela simplesmente trunca os bits excedentes.",
        ],
        "related": ["as_u16", "as_i8", "wrapping_add"],
    },
    "calc_eval": {
        "purpose": [
            "Avalia uma expressão matemática contida em uma string em tempo de execução.",
            "Permite que usuários finais digitem fórmulas simples (como em calculadoras ou campos de entrada de jogos) que o programa pode processar dinamicamente.",
        ],
        "example": """let formula = \"(10 + 5) * 2\"
let resultado = calc_eval(formula) // 30.0""",
        "pitfalls": [
            "API Experimental: a complexidade das expressões suportadas (parênteses, funções) depende da versão do runtime.",
            "Não deve ser usado com entrada de usuário não sanitizada se houver risco de execução de código indesejado (embora no Snask v0.4 ela seja limitada a aritmética).",
        ],
        "related": ["str_to_num", "format"],
    },
    "gui_init": {
        "purpose": [
            "Inicializa o runtime de interface gráfica nativa (GTK/Backend nativo). Deve ser chamada antes de qualquer tentativa de criar janelas ou widgets.",
            "Diferente do `snaskgui_init`, esta função prepara o sistema para uma hierarquia de widgets e gerenciamento de layout automático.",
        ],
        "example": """import \"gui\"
class main {
    fun start() {
        if gui::init() {
            let win = gui::window(\"App Snask\", 400, 300)
            gui::show_all(win)
            gui::run()
        }
    }
}""",
        "pitfalls": [
            "Se o sistema operacional não possuir um servidor gráfico ativo (ex: Linux sem X11/Wayland), a função retornará false.",
        ],
        "related": ["gui_run", "gui_window", "snaskgui_init"],
    },
    "gui_run": {
        "purpose": [
            "Inicia o loop principal de processamento de eventos da GUI. Esta função é **bloqueante**: ela só retorna quando a aplicação é encerrada via `gui_quit` ou quando a última janela é fechada.",
            "Durante a execução desta função, o runtime processa cliques, redesenha widgets e executa handlers de eventos.",
        ],
        "example": """gui::init()
let win = gui::window(\"App\", 200, 100)
gui::show_all(win)
// O programa 'para' aqui e fica ouvindo o usuário
gui::run()
print(\"Aplicação encerrada!\\n\")""",
        "pitfalls": [
            "Qualquer código escrito após `gui::run()` só será executado quando a interface for fechada.",
        ],
        "related": ["gui_init", "gui_quit"],
    },
    "gui_on_click": {
        "purpose": [
            "Conecta uma função (handler) ao evento de clique de um widget (geralmente um botão).",
            "É a base da interatividade no Snask, permitindo que o código responda a ações do usuário.",
        ],
        "example": """import \"gui\"
fun clicar() {
    print(\"Botão clicado!\\n\")
}

class main {
    fun start() {
        gui::init()
        let btn = gui::button(\"Clique aqui\")
        gui::on_click(btn, clicar)
        // ... exibir janela ...
    }
}""",
        "pitfalls": [
            "A função de handler deve estar disponível no escopo global ou ser acessível pelo runtime no momento do evento.",
        ],
        "related": ["gui_button", "gui_set_text"],
    },
    "http_get": {
        "purpose": [
            "Realiza uma requisição HTTP do tipo GET para a URL especificada.",
            "Retorna um dicionário contendo o corpo da resposta, o código de status e os cabeçalhos recebidos.",
            "É a ferramenta padrão para consumir dados de APIs REST externas.",
        ],
        "example": """let res = http_get(\"https://api.github.com/zen\")
print(\"Status: {res[\\\"status\\\"]}\\n\")
print(\"Body: {res[\\\"body\\\"]}\\n\")""",
        "pitfalls": [
            "Operação bloqueante: o programa ficará pausado até que a resposta chegue ou ocorra um timeout.",
            "Requer que o runtime tenha suporte a rede e certificados SSL válidos para URLs HTTPS.",
        ],
        "related": ["http_post", "json_parse"],
    },
    "is_nil": {
        "purpose": [
            "Verifica se uma variável possui o valor `nil` (nulo/ausente).",
            "É essencial para validação de segurança antes de realizar operações em objetos ou ponteiros que podem não ter sido inicializados.",
        ],
        "example": """let dado = read_file(\"inexistente.txt\")
if is_nil(dado) {
    print(\"Erro ao ler arquivo!\\n\")
}""",
        "pitfalls": [
            "Não confunda `nil` com uma string vazia `\"\"` ou o número `0`. `is_nil` retornará false para esses casos.",
        ],
        "related": ["is_str", "is_obj"],
    },
    "print": {
        "purpose": [
            "Exibe um valor na saída padrão (stdout) de forma legível.",
            "Diferente de outras linguagens, o `print` do Snask é polimórfico: ele entende strings, números, booleanos e até estruturas complexas do perfil humane sem necessidade de conversão manual.",
            "Nota importante: Ele NÃO adiciona uma quebra de linha (`\\n`) ao final automaticamente.",
        ],
        "example": """class main {
    fun start() {
        print(\"O resultado é: \")
        print(42)
        print(\"\\n\")
    }
}""",
        "pitfalls": [
            "Em perfis `freestanding` ou `bare-metal`, esta função pode não fazer nada se não houver um driver de console configurado.",
            "Para performance extrema em loops de sistemas, considere usar buffers manuais antes de imprimir.",
        ],
        "related": ["println", "format"],
    },
    "println": {
        "purpose": [
            "Uma conveniência para emitir uma quebra de linha imediata no console.",
            "Em versões recentes do runtime Snask, ela é otimizada para usar a sequência de escape correta do sistema operacional (LF no Linux/macOS, CRLF no Windows).",
        ],
        "example": """class main {
    fun start() {
        print(\"Linha 1\")
        println()
        print(\"Linha 2\")
    }
}""",
        "pitfalls": [
            "Atualmente, no Snask, `println()` não aceita argumentos. Para imprimir um valor com nova linha, use `print(valor)` seguido de `println()`.",
        ],
        "related": ["print"],
    },
}


def paragraph_list(items: list[str]) -> str:
    return "\n".join(f"<p>{html.escape(item)}</p>" for item in items)


def default_purpose(fn: dict) -> list[str]:
    category = fn["category"]
    name = fn["name"]
    summary = fn["summary"]
    if category == "snaskgui":
        return [
            f"`{name}` faz parte da API framebuffer da Snask. Essa familia e pensada para programas que controlam pixels diretamente: emuladores, jogos simples, visualizadores binarios e ferramentas de renderizacao deterministica.",
            f"{summary} Em vez de montar uma interface com botoes e layouts, voce manipula uma janela e envia frames prontos para ela.",
        ]
    if category == "gui":
        return [
            f"`{name}` pertence a camada GUI baseada em widgets nativos. Ela e experimental e serve para construir janelas, labels, botoes, entradas e containers sem sair da sintaxe Snask.",
            f"{summary} Handles retornados por funcoes GUI sao opacos: passe-os para outras funcoes `gui_*`, mas nao tente interpretar o valor manualmente.",
        ]
    if category == "memory":
        return [
            f"`{name}` e uma primitiva de memoria crua do perfil systems. Ela existe para codigo que precisa lidar com bytes, ponteiros e buffers previsiveis.",
            "Esse tipo de funcao pertence a regioes `@unsafe`, porque o compilador nao consegue provar sozinho que offsets, tamanho e tempo de vida estao corretos.",
        ]
    if category == "systems":
        return [
            f"`{name}` e uma primitiva deterministica para sistemas. Ela foi criada para casos como CPU 6502, parsers binarios e logica que precisa controlar bits, overflow ou inteiros de tamanho fixo.",
            f"{summary} A ideia e deixar o comportamento explicito, sem depender de conversoes magicas.",
        ]
    if category in {"json", "sfs"}:
        return [
            f"`{name}` pertence a uma biblioteca nativa importavel. Em codigo Snask normal, prefira chamar pelo modulo, por exemplo `json::parse` ou `sfs::read`, em vez do nome interno com prefixo.",
            f"{summary} A assinatura documenta a superficie que o analisador conhece hoje.",
        ]
    return [
        f"`{name}` e um builtin da superficie Snask atual.",
        f"{summary} Use esta pagina como referencia rapida para assinatura, perfil, retorno e limites conhecidos.",
    ]


def pitfalls_for(fn: dict) -> list[str]:
    if fn["name"] in DEEP_NOTES:
        return DEEP_NOTES[fn["name"]].get("pitfalls", [])
    if fn["status"] == "experimental":
        return ["API experimental: nome, retorno ou comportamento ainda podem mudar conforme o runtime amadurece."]
    if fn["status"] == "parcial":
        return ["API parcial: existe no compilador/runtime atual, mas ainda pode ter limites de tipo, link ou comportamento."]
    if fn["category"] == "memory":
        return ["Use dentro de `@unsafe` e mantenha ownership claro."]
    return ["Sem cuidados especiais alem dos tipos da assinatura e do perfil indicado."]


def related_links(fn: dict) -> str:
    names = DEEP_NOTES.get(fn["name"], {}).get("related", [])
    if not names:
        same = [f["name"] for f in FUNCTIONS if f["category"] == fn["category"] and f["name"] != fn["name"]]
        names = same[:6]
    if not names:
        return "<p>Nenhuma funcao relacionada direta.</p>"
    items = "\n".join(
        f'<li><a href="{slug(name)}.html"><code>{html.escape(name)}</code></a></li>'
        for name in names
    )
    return f"<ul>{items}</ul>"


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
    note = DEEP_NOTES.get(fn["name"], {})
    purpose = paragraph_list(note.get("purpose", default_purpose(fn)))
    pitfalls = "\n".join(f"<li>{html.escape(item)}</li>" for item in pitfalls_for(fn))
    example = note.get("example", fn["example"])
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
        <h2>Para que serve</h2>
        {purpose}
        <h2>Parametros</h2>
        <table><thead><tr><th>Nome</th><th>Descricao</th></tr></thead><tbody>{param_rows}</tbody></table>
        <h2>Retorno</h2>
        <p>{html.escape(fn['returns'])}</p>
        <h2>Exemplo</h2>
        <pre><code>{code(example)}</code></pre>
        <h2>Cuidados</h2>
        <ul>{pitfalls}</ul>
        <h2>Funcoes relacionadas</h2>
        {related_links(fn)}
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
        sections.append(f"<section class=\"category-section\"><h2>{html.escape(category.title())}</h2><div class=\"grid\">{links}</div></section>")
    body = "\n".join(sections)
    total = len(FUNCTIONS)
    return f"""<!doctype html>
<html lang=\"pt-BR\">
  <head>
    <meta charset=\"utf-8\" />
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
    <title>Indice de funcoes - Snask Docs</title>
    <link rel=\"stylesheet\" href=\"../../assets/site.css\" />
    <style>
      .search-container {{
        margin: 24px 0;
        position: sticky;
        top: 70px;
        z-index: 10;
      }}
      #functionSearch {{
        width: 100%;
        padding: 14px 20px;
        border: 2px solid var(--accent);
        border-radius: 12px;
        font-size: 16px;
        box-shadow: var(--shadow);
        outline: none;
      }}
    </style>
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
        <a href=\"index.html\" class=\"active\">Indice de funcoes</a>
        <a href=\"../language.html\">Linguagem</a>
        <a href=\"../types.html\">Tipos</a>
        <a href=\"../runtime.html\">Runtime e builtins</a>
        <a href=\"../diagnostics.html\">Diagnosticos</a>
      </aside>
      <main class=\"content\">
        <p class=\"eyebrow\">Biblioteca padrao</p>
        <h1>Indice de funcoes</h1>
        <p class=\"lead\">Pesquise entre {total} funcoes e builtins da Snask com explicacoes detalhadas.</p>
        
        <div class=\"search-container\">
          <input type="text" id="functionSearch" placeholder="Digite para filtrar (ex: gui, mem, print)..." />
        </div>

        <div id=\"functionsList\">
          {body}
        </div>
      </main>
    </div>
    <script>
      const searchInput = document.getElementById('functionSearch');
      const sections = document.querySelectorAll('.category-section');
      
      searchInput.addEventListener('input', (e) => {{
        const term = e.target.value.toLowerCase();
        
        sections.forEach(section => {{
          const cards = section.querySelectorAll('.function-card');
          let hasVisible = false;
          
          cards.forEach(card => {{
            const name = card.querySelector('h3').textContent.toLowerCase();
            const summary = card.querySelector('p').textContent.toLowerCase();
            if (name.includes(term) || summary.includes(term)) {{
              card.style.display = 'block';
              hasVisible = true;
            }} else {{
              card.style.display = 'none';
            }}
          }});
          
          section.style.display = hasVisible ? 'block' : 'none';
        }});
      }});
    </script>
    <script src=\"../../assets/site.js\"></script>
  </body>
</html>
"""


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    # Criar .nojekyll para o GitHub Pages
    (OUT / ".nojekyll").touch()
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
