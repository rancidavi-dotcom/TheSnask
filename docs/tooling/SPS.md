# SPS: Snask Project System

SPS e o sistema de projeto do Snask: manifesto, dependencias, lockfile e entrada padrao de build.

## Criar projeto

```bash
snask init meu_app
cd meu_app
```

Arquivos esperados:

- `snask.snif`;
- `main.snask` ou caminho configurado como entrada.

## Manifesto atual

A forma historica aceita pelo projeto usa SNIF. Exemplo:

```snif
{
  package: { name: "my_app", version: "0.1.0", entry: "main.snask", },
  dependencies: { json: "*", },
  build: { opt_level: 2, profile: "humane", },
}
```

Campos comuns:

- `package.name`;
- `package.version`;
- `package.entry`;
- `dependencies`;
- `build.profile`;
- `build.strip`;
- `build.lto`.

## Build e run

Dentro de um projeto SPS:

```bash
snask build
snask run
```

Arquivo direto:

```bash
snask build other.snask --output other
snask run other.snask
```

## Dependencias

```bash
snask add json
snask remove json
```

## Lockfile

`snask build` pode gerar `snask.lock` com versoes e hashes para reprodutibilidade.

## Status

`parcial`. Ja e util, mas workspaces, resolucao avancada, registry e lockfile ainda precisam endurecer.
