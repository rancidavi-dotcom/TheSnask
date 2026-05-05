# Zenith Framework v0.2.0

Zenith e uma experiencia de framework web para Snask. O estado atual e experimental: serve como direcao de design e biblioteca de laboratorio, nao como framework web estavel.

## Objetivo

- rotas declarativas;
- controllers;
- respostas HTTP;
- uso do OM-Snask-System para ciclos de request;
- API confortavel por cima do runtime nativo.

## Build conceitual

```bash
snask build main.snask --output zenith_app
./zenith_app
```

## Exemplo conceitual

```text
let app = zenith_app()
app.get("/users", "UserController::index")
app.listen(8080)
```

A sintaxe acima documenta a direcao da API. Antes de usar em app real, confira os arquivos `.snask` da biblioteca e `docs/reference/FEATURE_STATUS.md`.

## Status

- roteamento: experimental;
- controllers: experimental;
- ORM: planejado/parcial;
- integracao HTTP: parcial;
- OM por request: direcao de design.
