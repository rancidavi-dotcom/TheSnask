# cli_full: benchmark de tamanho para CLI realista

Este caso representa uma ferramenta de linha de comando um pouco mais proxima do mundo real:

- tabela pequena de subcomandos;
- string de configuracao em SNIF;
- helpers simples de hash/soma;
- execucao sem argumentos para automatizar `bench/run.sh`.

## Rodar

```bash
./bench/run.sh
cat bench/out/report.md
```

## Objetivo

Medir quanto runtime e codigo ficam no binario quando o app usa caminhos reais, sem depender de GUI ou bibliotecas grandes.
