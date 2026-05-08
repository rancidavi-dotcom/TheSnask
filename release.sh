#!/usr/bin/env bash
# Snask Automatic Release Script
# Uso:
#   ./release.sh <VERSAO> [--no-push]
#
# Publicacao:
#   - empacota o .deb local;
#   - atualiza repo/;
#   - cria commit de release;
#   - envia para GitHub e Codeberg na mesma execucao.
#
# Variaveis opcionais:
#   RELEASE_BRANCH=main
#   GITHUB_REMOTE=github
#   GITHUB_URL=git@github.com:rancidavi-dotcom/TheSnask.git
#   CODEBERG_REMOTE=codeberg
#   CODEBERG_URL=git@codeberg.org:DaviVilasBoas/SnaskLang.git

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info() { echo -e "${BLUE}==> $1${NC}"; }
ok() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
die() { echo -e "${RED}Erro: $1${NC}"; exit 1; }

usage() {
    echo "Uso: ./release.sh <VERSAO> [--no-push]"
}

if [ "${1:-}" = "" ]; then
    usage
    die "Versão não informada."
fi

VERSION="$1"
shift || true

PUSH=true
for arg in "$@"; do
    case "$arg" in
        --no-push) PUSH=false ;;
        *) die "Argumento desconhecido: $arg" ;;
    esac
done

RELEASE_BRANCH="${RELEASE_BRANCH:-main}"
GITHUB_REMOTE_ENV="${GITHUB_REMOTE:-}"
CODEBERG_REMOTE_ENV="${CODEBERG_REMOTE:-}"
GITHUB_URL="${GITHUB_URL:-git@github.com:rancidavi-dotcom/TheSnask.git}"
CODEBERG_URL="${CODEBERG_URL:-git@codeberg.org:DaviVilasBoas/SnaskLang.git}"
AUR_URL="${AUR_URL:-ssh://aur@aur.archlinux.org/snask.git}"

require_tool() {
    command -v "$1" >/dev/null 2>&1 || die "Ferramenta faltando: $1"
}

remote_exists() {
    git remote get-url "$1" >/dev/null 2>&1
}

remote_has_url_fragment() {
    local remote="$1"
    local fragment="$2"
    git remote get-url --all "$remote" 2>/dev/null | grep -qi "$fragment"
}

find_remote_by_url_fragment() {
    local fragment="$1"
    local remote

    while IFS= read -r remote; do
        if remote_has_url_fragment "$remote" "$fragment"; then
            echo "$remote"
            return 0
        fi
    done < <(git remote)

    return 1
}

ensure_named_remote() {
    local preferred="$1"
    local url="$2"

    if remote_exists "$preferred"; then
        echo "$preferred"
        return 0
    fi

    git remote add "$preferred" "$url"
    echo "$preferred"
}

resolve_github_remote() {
    if [ -n "$GITHUB_REMOTE_ENV" ]; then
        remote_exists "$GITHUB_REMOTE_ENV" || git remote add "$GITHUB_REMOTE_ENV" "$GITHUB_URL"
        echo "$GITHUB_REMOTE_ENV"
        return 0
    fi

    if remote="$(find_remote_by_url_fragment "github.com")"; then
        echo "$remote"
        return 0
    fi

    ensure_named_remote "github" "$GITHUB_URL"
}

resolve_codeberg_remote() {
    if [ -n "$CODEBERG_REMOTE_ENV" ]; then
        remote_exists "$CODEBERG_REMOTE_ENV" || git remote add "$CODEBERG_REMOTE_ENV" "$CODEBERG_URL"
        echo "$CODEBERG_REMOTE_ENV"
        return 0
    fi

    if remote="$(find_remote_by_url_fragment "codeberg.org")"; then
        echo "$remote"
        return 0
    fi

    ensure_named_remote "codeberg" "$CODEBERG_URL"
}

push_release() {
    local remote="$1"
    local label="$2"

    info "Forçando envio para ${label} (${remote}/${RELEASE_BRANCH})..."
    git push --force-with-lease "$remote" "HEAD:${RELEASE_BRANCH}"
}

push_release_to_all() {
    local github_remote="$1"
    local codeberg_remote="$2"
    local github_pid codeberg_pid
    local failed=0

    push_release "$github_remote" "GitHub" &
    github_pid=$!

    push_release "$codeberg_remote" "Codeberg" &
    codeberg_pid=$!

    wait "$github_pid" || failed=1
    wait "$codeberg_pid" || failed=1

    if [ "$failed" -ne 0 ]; then
        die "Falha ao enviar para um ou mais remotes."
    fi
}

update_aur() {
    local version="$1"
    info "7. Atualizando AUR (Arch User Repository)..."

    if ! command -v docker >/dev/null 2>&1; then
        warn "Docker não encontrado. Pulando AUR."
        return
    fi

    # Remove tarballs locais para forçar o download do arquivo real pelo updpkgsums
    rm -f snask-*.tar.gz

    # Dá um tempo para o GitHub processar a tag recém-enviada
    info "Aguardando 5 segundos para o GitHub processar a tag..."
    sleep 5

    # Atualiza versão no PKGBUILD local
    sed -i "s/^pkgver=.*/pkgver=${version}/" PKGBUILD

    info "Gerando checksums reais e .SRCINFO via Docker (Arch Linux)..."
    docker run --rm -v "$(pwd):/work" -w /work archlinux:latest bash -c "
        pacman -Syu --noconfirm pacman-contrib sudo binutils --needed
        useradd -m builder
        echo 'builder ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers
        chown -R builder:builder /work
        
        # O updpkgsums precisa encontrar os arquivos baixados. 
        # Como o PKGBUILD define 'source_x86_64', o updpkgsums baixará os arquivos se eles não existirem.
        sudo -u builder updpkgsums
        sudo -u builder makepkg --printsrcinfo > .SRCINFO
    " || { warn "Falha ao processar AUR via Docker."; return; }

    local aur_dir="target/aur-snask"
    rm -rf "$aur_dir"
    
    info "Clonando repositório AUR..."
    if git clone "$AUR_URL" "$aur_dir"; then
        cp PKGBUILD .SRCINFO "$aur_dir/"
        (
            cd "$aur_dir"
            # Configura identidade local para este commit
            git config user.email "davidev@snask.lang"
            git config user.name "Davi (Snask Release Bot)"
            
            git add PKGBUILD .SRCINFO
            if git commit -m "chore: update to version ${version}"; then
                if [ "$PUSH" = true ]; then
                    # No AUR o branch principal é sempre 'master'
                    git push origin master
                    ok "AUR atualizado com sucesso."
                else
                    warn "Push para AUR ignorado (--no-push)."
                fi
            else
                warn "Nada para commitar no AUR (versão já atualizada?)."
            fi
        )
    else
        warn "Não foi possível clonar o AUR em $AUR_URL. Verifique permissões/SSH."
    fi
}

require_tool cargo
require_tool git
require_tool dpkg-deb
require_tool apt-ftparchive
require_tool gzip

info "1. Compilando Snask em modo release..."
cargo build --release

info "2. Gerando runtimes locais para o pacote..."
./target/release/snask setup

info "3. Preparando estrutura do pacote Debian..."
rm -rf pkg-snask
mkdir -p pkg-snask/DEBIAN
mkdir -p pkg-snask/usr/bin
mkdir -p pkg-snask/usr/lib/snask/runtime
mkdir -p pkg-snask/usr/lib/snask/stdlib
mkdir -p pkg-snask/usr/share/doc/snask

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

cp target/release/snask pkg-snask/usr/bin/
cp -r src/runtime/* pkg-snask/usr/lib/snask/runtime/
cp -r src/stdlib/* pkg-snask/usr/lib/snask/stdlib/
cp src/runtime.c pkg-snask/usr/lib/snask/runtime/

cp ~/.snask/lib/runtime.* pkg-snask/usr/lib/snask/runtime/
cp ~/.snask/lib/runtime_tiny.* pkg-snask/usr/lib/snask/runtime/
cp ~/.snask/lib/runtime_nano.* pkg-snask/usr/lib/snask/runtime/
[ -f ~/.snask/lib/rt_extreme.o ] && cp ~/.snask/lib/rt_extreme.o pkg-snask/usr/lib/snask/runtime/

info "4. Construindo pacote .deb..."
dpkg-deb --build pkg-snask "snask_${VERSION}_amd64.deb"

info "5. Atualizando repositório APT local..."
mkdir -p repo/pool/main/s/snask
mkdir -p repo/dists/stable/main/binary-amd64
cp "snask_${VERSION}_amd64.deb" repo/pool/main/s/snask/

(cd repo && apt-ftparchive packages pool/main/s/snask/ > dists/stable/main/binary-amd64/Packages)
gzip -fk repo/dists/stable/main/binary-amd64/Packages
(cd repo && apt-ftparchive -o APT::FTPArchive::Release::Codename=stable release dists/stable/ > dists/stable/Release)

if [ "$PUSH" = true ]; then
    info "6. Criando commit de release..."
    git add .

    if git diff --cached --quiet; then
        warn "Nada novo para commitar. Vou apenas enviar o HEAD atual."
    else
        git commit -m "chore(release): version ${VERSION} [automated]"
    fi

    info "Criando tag v${VERSION}..."
    git tag -a "v${VERSION}" -m "Release v${VERSION}" --force

    GITHUB_REMOTE_RESOLVED="$(resolve_github_remote)"
    CODEBERG_REMOTE_RESOLVED="$(resolve_codeberg_remote)"

    push_release_to_all "$GITHUB_REMOTE_RESOLVED" "$CODEBERG_REMOTE_RESOLVED"

    info "Enviando tags..."
    git push "$GITHUB_REMOTE_RESOLVED" --tags --force
    git push "$CODEBERG_REMOTE_RESOLVED" --tags --force

    info "Criando Release no GitHub e enviando binário..."
    if command -v gh >/dev/null 2>&1; then
        # Garante que o binário atual seja renomeado para o padrão da release
        cp target/release/snask snask-linux-amd64
        
        # Tenta criar a release. Se já existir, o comando falha e seguimos para o upload.
        gh release create "v${VERSION}" --title "Release v${VERSION}" --notes "Automated binary release for v${VERSION}" || true
        
        # Faz o upload do binário. O --clobber garante a substituição se o arquivo já existir.
        gh release upload "v${VERSION}" snask-linux-amd64 --clobber
        
        ok "Binário enviado para GitHub Releases."
    else
        warn "GitHub CLI (gh) não encontrada. Binário não enviado para as Releases."
        warn "O AUR VAI FALHAR pois o link de download não existirá."
    fi

    update_aur "$VERSION"

    ok "Snask v${VERSION} enviado para GitHub e Codeberg."
else
    warn "Push ignorado (--no-push). Pacote gerado localmente."
fi
