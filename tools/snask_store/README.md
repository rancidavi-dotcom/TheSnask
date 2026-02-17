# Snask Store (Python)

GUI simples para instalar/remover/atualizar pacotes do registry do Snask, no mesmo diretório usado pelo CLI (`~/.snask/packages`).

## Dependências (Pop!_OS/Ubuntu)

```bash
sudo apt update
sudo apt install -y python3-gi gir1.2-gtk-3.0
```

## Rodar (dev)

No repositório do `TheSnask`:

```bash
python3 tools/snask_store/snask_store.py
```

## Rodar via CLI

```bash
snask store
```

## Onde instala

Arquivos `.snask` são baixados para:

`~/.snask/packages/`

