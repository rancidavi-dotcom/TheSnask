# Aprender Snask v0.4.1-alpha

Este guia mostra apenas exemplos que combinam com o compilador atual. Quando uma ideia ainda for roadmap, ela aparece como texto conceitual, nao como bloco `snask` compilavel.

## 1. Hello World

Todo programa executavel precisa de uma `class main`. O metodo `start` e o ponto de entrada mais claro e recomendado.

```snask
class main {
    fun start() {
        print("Ola, Snask!\n")
    }
}
```

Build:

```bash
./target/debug/snask build hello.snask --output hello
./hello
```

## 2. Variaveis

- `let`: imutavel.
- `mut`: mutavel.
- `const`: constante de modulo.

```snask
class main {
    fun start() {
        let name = "Davi"
        mut age = 25
        age = age + 1

        print("Nome: {name}\n")
        print("Idade: {age}\n")
    }
}
```

## 3. Condicionais e loops

```snask
class main {
    fun start() {
        let score = 8

        if score >= 7 {
            print("Aprovado\n")
        } else {
            print("Reprovado\n")
        }

        mut i = 1
        while i <= 3 {
            print("Passo: {i}\n")
            i = i + 1
        }
    }
}
```

## 4. Funcoes

O compilador atual ainda usa bastante `Any` em funcoes sem anotacao. Para exemplos didaticos simples, prefira manter operacoes aritmeticas dentro de `main` ou usar funcoes ja validadas pelo projeto real.

```snask
class main {
    fun start() {
        let a = 10
        let b = 20
        let sum = a + b
        print("Soma: {sum}\n")
    }
}
```

## 5. Listas e dicionarios

Colecoes existem, mas ainda sao `parcial` no type system. Use para casos simples e confira `docs/reference/FEATURE_STATUS.md` antes de depender delas em API publica.

```snask
class main {
    fun start() {
        let fruits: list<str> = ["maca", "banana", "uva"]
        print(fruits[0])
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
        print(user["id"])
        print("\n")
    }
}
```

## 6. Classes

`class main` e a forma estavel de declarar o ponto de entrada. O modelo de classes de usuario, instancia nominal e heranca ainda esta `parcial`; por isso, exemplos de OOP rica devem ser tratados como experimentais por enquanto.

## 7. OM-Snask-System

`zone`, `new stack`, `new arena`, `promote`, recursos nativos e `import_c_om` pertencem ao mesmo sistema: o OM-Snask-System.

```snask
class main {
    fun start() {
        zone "request" {
            let message = "valor dentro da zona"
            print("{message}\n")
        }
    }
}
```

## 8. Perfil systems

Para memoria crua e emuladores, use `--profile systems`.

```bash
./target/debug/snask build apps/nes_emulator/nes_master.snask --profile systems --output /tmp/snask_nes
/tmp/snask_nes
```

## 9. Comandos uteis

```bash
snask init meu_app
snask build caminho/main.snask --output meu_app
snask run caminho/main.snask
snask explain S1005
snask doctor
snask om scan SDL2/SDL.h --lib sdl2 --output /tmp/sdl2.generated.om.snif
```

## 10. Regra honesta

Se uma doc antiga mostrar sintaxe bonita demais, confira se ela esta marcada como conceito. A fonte de verdade da linguagem atual e este guia junto com `docs/reference/FEATURE_STATUS.md`.
