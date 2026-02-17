# Build Snask Store (Python)

O Snask Store é um app Python/GTK3. A forma mais simples de distribuir é empacotar com **PyInstaller**.

## 1) Dependências (Linux)

```bash
sudo apt update
sudo apt install -y python3-gi gir1.2-gtk-3.0
```

## 2) Rodar em desenvolvimento

```bash
python3 tools/snask_store/snask_store.py
```

## Aba Dev (criar/publicar libs)

O Snask Store tem uma aba **Dev** que chama o CLI do Snask:

- **Criar template**: `snask lib init`
- **Publicar**: `snask lib publish`

Pré-requisitos:
- `snask` no PATH (ou instalado em `~/.snask/bin`)
- `git` instalado

## 3) Gerar executáveis

### Linux (binário)

```bash
python3 -m pip install --user pyinstaller
pyinstaller --noconfirm --clean tools/snask_store/snask_store.spec
```

Saída: `dist/snask-store/snask-store`

### Windows (EXE)

No Windows (recomendado: build nativo):

```powershell
py -m pip install pyinstaller
pyinstaller --noconfirm --clean tools/snask_store\snask_store.spec
```

### macOS (app)

No macOS (build nativo):

```bash
python3 -m pip install pyinstaller
pyinstaller --noconfirm --clean tools/snask_store/snask_store.spec
```

## 4) Linux .deb e AppImage (opcional)

**.deb** (via `fpm`):

```bash
sudo apt install -y ruby ruby-dev build-essential
sudo gem install --no-document fpm
# Gera o binário com PyInstaller antes e depois empacota com .desktop + ícone
cp -f dist/snask-store tools/snask_store/package_root/usr/bin/snask-store
fpm -s dir -t deb -n snask-store -v 0.1.0 -C tools/snask_store/package_root .
```

**AppImage**: recomendado usar `linuxdeploy`/`appimagetool`. Este repo inclui o app; a receita depende do ambiente da sua distro.
