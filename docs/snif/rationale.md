# Racional do SNIF

SNIF existe porque JSON e bom para interoperabilidade, mas fraco para configuracao longa e humana.

## Por que mais estrito

- apenas `:` como separador;
- apenas comentarios `//`;
- apenas `null`;
- sem strings bareword;
- virgula final permitida para diffs melhores.

## Por que literais tipados

`@date`, `@dec`, `@bin` e `@enum` expressam intencao sem inventar parser ambiguo.

## Por que inteiros grandes sao preservados

JSON pode perder precisao em ecossistemas que usam IEEE-754. SNIF preserva inteiros grandes com objeto marcado.

## Por que referencias

Configs reais repetem blocos. Referencias reduzem duplicacao e erro de copia.
