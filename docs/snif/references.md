# Referencias SNIF

Referencias permitem reutilizar o mesmo valor sem duplicacao.

## Sintaxe

- definir: `&name <value>`
- usar: `*name`

```snif
{
  shared: &cfg{ retries: 3, timeout_ms: 1500, },
  service_a: { config: *cfg, },
  service_b: { config: *cfg, },
}
```

## Regras

- `*name` deve apontar para uma referencia anterior;
- nomes sao identificadores;
- ciclos devem ser rejeitados;
- implementacoes podem limitar quantidade/profundidade para seguranca.
