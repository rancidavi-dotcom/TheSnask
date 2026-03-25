#!/bin/bash
# 🐍 Snask Automatic Release Script (v1.0)
# Desenvolvido para o Davi (TheSnask)
# Uso: ./release.sh <VERSÃO> (Ex: ./release.sh 0.3.7)

set -e # Para o script se houver erro

# Cores para o terminal
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

if [ -z "$1" ]; then
    echo -e "${RED}Erro: Versão não informada!${NC}"
    echo -e "Uso: ./release.sh <VERSÃO> (Ex: ./release.sh 0.3.7)"
    exit 1
fi

VERSION=$1
DEB_FILE="snask_${VERSION}_amd64.deb"

echo -e "${BLUE}==> 1. Compilando Snask em modo Release...${NC}"
cargo build --release

echo -e "${BLUE}==> 2. Preparando estrutura do pacote Debian...${NC}"
mkdir -p pkg-snask/DEBIAN
mkdir -p pkg-snask/usr/bin
mkdir -p pkg-snask/usr/lib/snask/runtime
mkdir -p pkg-snask/usr/lib/snask/stdlib

# Atualizando a versão no arquivo control de forma automática
sed -i "s/^Version: .*/Version: ${VERSION}/" pkg-snask/DEBIAN/control

# Copiando binários e runtimes atualizados
cp target/release/snask pkg-snask/usr/bin/
cp -r src/runtime/* pkg-snask/usr/lib/snask/runtime/
cp -r src/stdlib/* pkg-snask/usr/lib/snask/stdlib/
# Garantindo que o runtime.o esteja presente
[ -f src/runtime.o ] && cp src/runtime.o pkg-snask/usr/lib/snask/runtime/

echo -e "${BLUE}==> 3. Construindo pacote .deb (${DEB_FILE})...${NC}"
dpkg-deb --build pkg-snask "$DEB_FILE"

echo -e "${BLUE}==> 4. Atualizando Repositório APT local...${NC}"
mkdir -p repo/dists/stable/main/binary-amd64
mkdir -p repo/pool/main/s/snask

# Limpando .debs antigos do repositório para evitar lixo
rm -f repo/pool/main/s/snask/*.deb
cp "$DEB_FILE" repo/pool/main/s/snask/

# Gerando índices do APT (Packages e Release)
(cd repo && apt-ftparchive packages pool/main/s/snask/ > dists/stable/main/binary-amd64/Packages)
gzip -c repo/dists/stable/main/binary-amd64/Packages > repo/dists/stable/main/binary-amd64/Packages.gz

# Gerando o arquivo Release com o Codename correto para remover avisos do apt
(cd repo && apt-ftparchive -o APT::FTPArchive::Release::Codename=stable release dists/stable/ > dists/stable/Release)

echo -e "${BLUE}==> 5. Enviando para o GitHub...${NC}"
git add pkg-snask/ repo/ .gitignore
git commit -m "release: v${VERSION} (automatic deployment)"
git push origin main

echo -e "${GREEN}✅ SUCESSO! Snask v${VERSION} está on-line no seu repositório APT.${NC}"
echo -e "${GREEN}Usuários podem atualizar com: sudo apt update && sudo apt upgrade${NC}"
