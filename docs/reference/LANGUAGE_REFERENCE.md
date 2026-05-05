# Referencia da Linguagem Snask v0.4.1-alpha

Esta referencia descreve a superficie atual da linguagem. Ela evita usar blocos `snask` para features que ainda sao apenas plano.

Status dos termos usados aqui:

- `estavel`: funciona de ponta a ponta no compilador atual.
- `parcial`: existe, mas tem limites relevantes.
- `experimental`: existe e pode mudar.
- `planejada`: ainda nao e contrato da linguagem.

Consulte `docs/reference/FEATURE_STATUS.md` para a tabela completa.

## 1. Arquivo e entrada

Um programa executavel deve ter `class main`. Use `fun start()` como ponto de entrada recomendado.

```snask
class main {
    fun start() {
        print("Ola\n")
    }
}
```

## 2. Blocos

Snask aceita blocos por indentacao e tambem por chaves. Para exemplos de documentacao, preferimos chaves porque elas sao mais faceis de copiar para testes pequenos.

```snask
class main {
    fun start() {
        if true {
            print("ok\n")
        }
    }
}
```

## 3. Comentarios

```snask
class main {
    fun start() {
        // comentario de linha
        print("comentarios usam //\n")
    }
}
```

Comentario de bloco ainda nao e parte da linguagem.

## 4. Variaveis

```snask
class main {
    fun start() {
        let fixed = 10
        mut counter = 0
        counter = counter + 1
        print("{fixed} {counter}\n")
    }
}
```

`let` nao pode ser reatribuido. Exemplo de erro esperado:

```text
let x = 10
x = 20
# erro: nao e possivel atribuir de novo a uma variavel imutavel
```

## 5. Constantes

```snask
const API_VERSION = "0.4.1-alpha"

class main {
    fun start() {
        print(API_VERSION)
        print("\n")
    }
}
```

## 6. Numeros, strings e interpolacao

```snask
class main {
    fun start() {
        let name = "Snask"
        let value = 40 + 2
        print("{name}: {value}\n")
    }
}
```

## 7. Operadores

Operadores aritmeticos comuns existem: `+`, `-`, `*`, `/`. Comparacoes como `==`, `!=`, `<`, `<=`, `>` e `>=` existem, mas a semantica ainda e `parcial` para alguns tipos mistos.

```snask
class main {
    fun start() {
        let a = 8
        let b = 4
        print(a + b)
        print("\n")
        print(a - b)
        print("\n")
        print(a * b)
        print("\n")
        print(a / b)
        print("\n")
    }
}
```

## 8. Condicionais

```snask
class main {
    fun start() {
        let score = 9
        if score >= 7 {
            print("aprovado\n")
        } else {
            print("reprovado\n")
        }
    }
}
```

## 9. Loop `while`

```snask
class main {
    fun start() {
        mut i = 0
        while i < 3 {
            print("i={i}\n")
            i = i + 1
        }
    }
}
```

## 10. Colecoes

Listas e dicionarios funcionam para casos simples e continuam `parcial` no type system.

```snask
class main {
    fun start() {
        let fruits: list<str> = ["maca", "banana", "uva"]
        print(fruits[1])
        print("\n")
    }
}
```

```snask
class main {
    fun start() {
        let user: dict<str, int> = {
            "id": 1,
            "level": 10,
        }
        print(user["level"])
        print("\n")
    }
}
```

## 11. Funcoes

Funcoes existem, mas chamadas com parametros sem tipo ainda podem cair em `Any`. Use com cuidado ate o type system fechar melhor essa area.

```snask
fun say_ok() {
    print("ok\n")
}

class main {
    fun start() {
        say_ok()
    }
}
```

## 12. Classes

`class main` esta consolidada como ponto de entrada. Classes de usuario e heranca ainda estao `parcial`: parser e partes do analisador/codegen existem, mas ainda nao devem ser documentadas como OOP completa.

## 13. Modulos

Imports simples existem:

```text
import "modulo"
from "modulo" import nome
```

A semantica de pacotes ainda depende do SPS e deve ser consultada em `docs/tooling/SPS.md` e `docs/tooling/PROJECT_SNIF.md`.

## 14. OM e zonas

```snask
class main {
    fun start() {
        zone "frame" {
            let label = "dentro da zona"
            print("{label}\n")
        }
    }
}
```

`new stack`, `new arena`, `new heap`, `promote`, `scope`, `entangle` e recursos nativos existem em niveis diferentes de maturidade. Veja `docs/systems/OM_SNASK_SYSTEM.md`.

## 15. `@unsafe`

`@unsafe` marca uma regiao onde chamadas restritas e memoria manual podem ser aceitas. Use apenas quando o codigo realmente precisar de comportamento de baixo nivel.

```text
@unsafe:
    let mem: ptr = mem_alloc_zero(65536)
    mem_free(mem)
```

## 16. Perfis

```bash
snask build app.snask --profile humane --output app
snask build app.snask --profile systems --output app
```

Sem `--profile`, o perfil efetivo e `humane`.

## 17. Diagnosticos

Use:

```bash
snask explain S1005
```

para entender um codigo de erro. A direcao do projeto e fazer os diagnosticos ficarem mais humanos que os de linguagens de sistemas tradicionais.
