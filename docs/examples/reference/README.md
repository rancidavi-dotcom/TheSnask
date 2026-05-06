# Exemplos da referencia Snask

Estes exemplos sao usados pela documentacao web em `docs/site/reference/functions/`.

Para validar:

```bash
scripts/check_doc_examples.sh
```

Um exemplo so deve ficar nesta pasta quando `snask build` passa no compilador atual.
Quando uma funcao existe no analisador semantico, mas ainda nao tem runtime/link final,
a pagina dela continua documentada com status honesto, porem sem arquivo de teste
versionado.
