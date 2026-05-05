# Snask Store

App experimental para explorar distribuicao, pacotes e interface de loja dentro do ecossistema Snask.

## Build

```bash
./target/debug/snask build apps/snask_store/main.snask --output /tmp/snask_store
/tmp/snask_store
```

## Testes manuais

```bash
./target/debug/snask build apps/snask_store/test_sys.snask --output /tmp/snask_store_sys
./target/debug/snask build apps/snask_store/test_net.snask --output /tmp/snask_store_net
```

## Status

Experimental. A infraestrutura de pacotes ainda esta parcial.
