# Snask Vault

Demo de cofre de senhas feito para exercitar GUI, SNIF, armazenamento local e empacotamento.

## Aviso de seguranca

Este app nao deve guardar segredos reais. A protecao atual e demonstrativa e nao substitui criptografia.

## Build

```bash
snask build main.snask --output snask_vault
./snask_vault
```

Se estiver na raiz do repositorio, prefira:

```bash
./target/debug/snask build apps/snask_vault/main.snask --output /tmp/snask_vault
/tmp/snask_vault
```

## Dados

```text
~/.snask_vault/vault.snif
```

## Status

Demo experimental.
