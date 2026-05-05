# Especificacao SNIF v1

SNIF, Snask Interchange Format, e um formato de dados/configuracao humano, seguro e deterministico para o ecossistema Snask.

## Objetivos

- configs legiveis;
- comentarios `//`;
- virgula final;
- objetos e arrays simples;
- literais tipados;
- preservacao de inteiros grandes;
- parse deterministico.

## Nao objetivos

- copiar YAML inteiro;
- schema complexo embutido;
- permitir ambiguidades como bareword string.

## Decisoes canonicas

- separador de chave e sempre `:`;
- comentarios apenas `//`;
- nulo e `null`;
- strings devem usar aspas;
- barewords como valor sao erro.

## Estrutura

Um documento SNIF e um unico valor: objeto, array, string, numero, booleano, null, literal tipado ou referencia.

## Literais tipados

```text
@date"2026-02-18T00:00:00Z"
@dec"19.99"
@bin"..."
@enum"STATUS_OK"
```

Representacao decodificada usa objetos marcados, como `{ "$date": "..." }`.

## Inteiros grandes

Inteiros fora do intervalo seguro IEEE-754 devem ser preservados como string marcada:

```text
{ "$i64": "9007199254740993" }
```

## Referencias

```text
{
  shared: &cfg{ retries: 3, },
  service_a: { config: *cfg, },
}
```

Forward refs e ciclos devem ser rejeitados.
