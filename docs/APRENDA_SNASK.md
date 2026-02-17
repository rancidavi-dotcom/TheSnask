# 🎓 Aprenda Snask: Trilha de Aprendizado Passo a Passo

Este guia contém exemplos práticos e testados para você dominar a sintaxe e a estabilidade do Snask.

---

## 1. Olá Mundo
```snask
print("Olá, Snask!");
```
*   O Snask v0.2.1 garante saída limpa e sem falhas de segmentação no encerramento do programa.
*   O Snask v0.2.3 mantém essa estabilidade e adiciona SPS (`snask.toml`) para projetos.

---

## 2. Variáveis e Segurança
```snask
let nome = "Davi";     // Imutável
mut idade = 25;        // Mutável

idade = 26;            // Permitido
print(nome, "tem", idade, "anos.");
```

---

## 3. Lógica e Matemática
O Snask suporta todos os operadores de comparação (`>`, `<`, `==`, `>=`, `<=`).

```snask
let nota = 8.5;

if nota >= 7.0
    print("Aprovado!");
else
    print("Reprovado.");

mut contador = 1;
while contador <= 3
    print("Passo:", contador);
    contador = contador + 1;
```

---

## 4. Funções Reutilizáveis
Funções no Snask são rápidas e suportam recursão.

```snask
fun somar(a, b)
    return a + b;

print("Soma:", somar(10, 20));

fun fatorial(n)
    if n <= 1
        return 1;
    return n * fatorial(n - 1);

print("Fatorial de 5:", fatorial(5));
```

---

## 5. Falando com o Disco (SFS)
Use as funções `sfs_` para manipular arquivos com o desempenho do C.

```snask
let arquivo = "teste.txt";
sfs_write(arquivo, "Escrito via Snask!");

if sfs_exists(arquivo)
    let dados = sfs_read(arquivo);
    print("Conteúdo:", dados);
```

---

💡 **Como rodar:** Salve em um arquivo `.snask` e execute:
`./snask build arquivo.snask && ./arquivo`
