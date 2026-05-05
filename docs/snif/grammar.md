# Gramatica SNIF informal

Espacos, tabs e quebras de linha sao whitespace. Comentarios usam `//` ate o fim da linha.

```text
document  := value ws EOF
value     := object | array | string | number | "true" | "false" | "null" | typed_literal | ref_define | ref_use
object    := "{" ws (member (ws "," ws member)* ws ","? )? ws "}"
member    := key ws ":" ws value
key       := identifier | string
array     := "[" ws (value (ws "," ws value)* ws ","? )? ws "]"
typed_literal := "@" identifier ws string
ref_define := "&" identifier ws value
ref_use    := "*" identifier
identifier := [A-Za-z_$] [A-Za-z0-9_$-]*
```

Regras:

- `:` e o unico separador de membro;
- bareword string nao existe;
- literais tipados precisam de string entre aspas;
- referencias precisam ser definidas antes do uso.
