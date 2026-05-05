# Startup a frio de CLI

Mede quanto tempo um programa leva para produzir a primeira saida no terminal.

## Compara

- Snask com perfis de tamanho;
- C com `gcc` padrao;
- Go;
- Python;
- Node.

## Rodar

```bash
./bench/startup/run.sh
cat bench/startup/out/report.md
```

## Cuidados

Startup abaixo de poucos milissegundos sofre ruido de scheduler, cache, turbo boost e processos do desktop. O runner usa repeticoes e pode intercalar A/B para reduzir viés.
