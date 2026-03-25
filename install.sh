#!/usr/bin/env bash
set -euo pipefail

# Snask installer/updater (curl | bash)
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
#
# What it does:
# - Clones (or pulls) TheSnask into ~/.snask/src/TheSnask
# - Builds `snask` (cargo --release)
# - Runs `snask setup` to install runtime + binary into ~/.snask/

REPO_URL="https://github.com/rancidavi-dotcom/TheSnask.git"
BRANCH="main"

SNASK_HOME="${HOME}/.snask"
SNASK_SRC="${SNASK_HOME}/src/TheSnask"

color() { printf "\033[%sm%s\033[0m\n" "$1" "$2"; }
info() { color "0;34" "==> $1"; }
ok() { color "0;32" "✓ $1"; }
die() { color "0;31" "✗ $1"; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || die "Dependência faltando: $1"
}

info "Verificando dependências..."
need git
need cargo
need gcc
need clang-18
need llc-18

info "Preparando diretórios em ${SNASK_HOME}..."
mkdir -p "${SNASK_HOME}/src"

if [ -d "${SNASK_SRC}/.git" ]; then
  info "Atualizando código fonte (git pull)..."
  git -C "${SNASK_SRC}" fetch --quiet origin "${BRANCH}"
  git -C "${SNASK_SRC}" checkout --quiet "${BRANCH}"
  git -C "${SNASK_SRC}" pull --quiet --ff-only origin "${BRANCH}"
else
  info "Baixando código fonte (git clone)..."
  rm -rf "${SNASK_SRC}"
  git clone --quiet --branch "${BRANCH}" "${REPO_URL}" "${SNASK_SRC}"
fi

info "Compilando Snask (cargo build --release)..."
(cd "${SNASK_SRC}" && cargo build --release)

info "Instalando/atualizando runtime e binário (snask setup)..."
(cd "${SNASK_SRC}" && ./target/release/snask setup)

ok "Snask instalado/atualizado."
info "Dica: reinicie o terminal ou garanta que ~/.snask/bin está no PATH."
info "Teste: snask --version"
