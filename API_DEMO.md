# API Demo (Snask + Blaze) — para Insomnia

Arquivo principal: `api_demo.snask`

## Como rodar

1) Compile:
```bash
snask build api_demo.snask
```

2) Execute:
```bash
./api_demo
```

Servidor: `http://127.0.0.1:3000`

## Rotas

### 1) Health
`GET /health`

Resposta:
```json
{ "ok": true, "service": "snask-api" }
```

### 2) Echo
`GET /echo?msg=oi`

### 3) Soma
`POST /sum`

Body (JSON):
```json
{ "a": 10, "b": 25 }
```

### 4) Items (persistência local)
`GET /items`

`POST /items`

Body (JSON):
```json
{ "name": "banana" }
```

Os itens são gravados em `./api_items.json` no diretório atual do binário.

## Observação

O `blaze_run` do runtime recebe **porta numérica** e faz bind em `0.0.0.0:PORT` (acessível via `127.0.0.1` localmente).

## Importar no Insomnia

- Crie uma Collection e adicione requests com as rotas acima.
- Para `POST`, defina o body como JSON e `Content-Type: application/json`.
