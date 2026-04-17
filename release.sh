#!/bin/bash
# 🐍 Snask Automatic Release Script (v2.0)
# Desenvolvido para o Davi (TheSnask)
# Uso: ./release.sh <VERSÃO> [--no-push]

set -e

# Cores
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ -z "$1" ]; then
    echo -e "${RED}Erro: Versão não informada!${NC}"
    echo -e "Uso: ./release.sh <VERSÃO> [--no-push]"
    exit 1
fi

VERSION=$1
PUSH=true
if [[ "$*" == *"--no-push"* ]]; then
    PUSH=false
fi

# 1. Build Snask Binary
echo -e "${BLUE}==> 1. Compilando Snask em modo Release...${NC}"
cargo build --release

# 2. Build Runtime (Local) to ensure everything is fresh
echo -e "${BLUE}==> 2. Gerando runtimes locais para o pacote...${NC}"
./target/release/snask setup

# 3. Prepare Debian Package Structure
echo -e "${BLUE}==> 3. Preparando estrutura do pacote Debian...${NC}"
rm -rf pkg-snask
mkdir -p pkg-snask/DEBIAN
mkdir -p pkg-snask/usr/bin
mkdir -p pkg-snask/usr/lib/snask/runtime
mkdir -p pkg-snask/usr/lib/snask/stdlib
mkdir -p pkg-snask/usr/share/doc/snask

# Create control file if it doesn't exist
if [ ! -f pkg-snask/DEBIAN/control ]; then
    cat > pkg-snask/DEBIAN/control <<EOF
Package: snask
Version: ${VERSION}
Section: devel
Priority: optional
Architecture: amd64
Maintainer: Davi <davidev@snask.lang>
Description: Snask Programming Language with Orchestrated Memory (OM)
 A high-performance systems programming language with revolutionary memory safety.
EOF
else
    sed -i "s/^Version: .*/Version: ${VERSION}/" pkg-snask/DEBIAN/control
fi

# Copy files to package
cp target/release/snask pkg-snask/usr/bin/
cp -r src/runtime/* pkg-snask/usr/lib/snask/runtime/
cp -r src/stdlib/* pkg-snask/usr/lib/snask/stdlib/
cp src/runtime.c pkg-snask/usr/lib/snask/runtime/

# Also include the pre-compiled runtimes from ~/.snask/lib for ease of use
cp ~/.snask/lib/runtime.* pkg-snask/usr/lib/snask/runtime/
cp ~/.snask/lib/runtime_tiny.* pkg-snask/usr/lib/snask/runtime/
cp ~/.snask/lib/runtime_nano.* pkg-snask/usr/lib/snask/runtime/
[ -f ~/.snask/lib/rt_extreme.o ] && cp ~/.snask/lib/rt_extreme.o pkg-snask/usr/lib/snask/runtime/

# 4. Build .deb
echo -e "${BLUE}==> 4. Construindo pacote .deb...${NC}"
dpkg-deb --build pkg-snask "snask_${VERSION}_amd64.deb"

# 5. Local APT Repository Management
echo -e "${BLUE}==> 5. Atualizando Repositório APT local...${NC}"
mkdir -p repo/pool/main/s/snask
mkdir -p repo/dists/stable/main/binary-amd64
cp "snask_${VERSION}_amd64.deb" repo/pool/main/s/snask/

(cd repo && apt-ftparchive packages pool/main/s/snask/ > dists/stable/main/binary-amd64/Packages)
gzip -fk repo/dists/stable/main/binary-amd64/Packages
(cd repo && apt-ftparchive -o APT::FTPArchive::Release::Codename=stable release dists/stable/ > dists/stable/Release)

# 6. Commit and Push
if [ "$PUSH" = true ]; then
    echo -e "${BLUE}==> 6. Enviando para o GitHub...${NC}"
    git add repo/ release.sh src/tools.rs .
    git commit -m "chore(release): version ${VERSION} [automated]"
    git push origin main
    echo -e "${GREEN}✅ SUCESSO! Snask v${VERSION} está on-line.${NC}"
else
    echo -e "${YELLOW}⚠️  Aviso: Push ignorado (--no-push). Pacote gerado localmente.${NC}"
fi
