#!/usr/bin/env bash
set -euo pipefail

# Snask universal Linux installer/updater.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/rancidavi-dotcom/TheSnask/main/install.sh | bash
#   ./install.sh
#
# Environment knobs:
#   SNASK_INSTALL_DEPS=0     skip package-manager dependency install
#   SNASK_BRANCH=main        git branch/tag to install
#   SNASK_HOME=~/.snask      install root
#   SNASK_SRC=...            source checkout path
#   SNASK_PROFILE=release    cargo profile: release or debug

REPO_URL="${SNASK_REPO_URL:-https://github.com/rancidavi-dotcom/TheSnask.git}"
BRANCH="${SNASK_BRANCH:-main}"
SNASK_HOME="${SNASK_HOME:-${HOME}/.snask}"
SNASK_SRC="${SNASK_SRC:-${SNASK_HOME}/src/TheSnask}"
SNASK_INSTALL_DEPS="${SNASK_INSTALL_DEPS:-1}"
SNASK_PROFILE="${SNASK_PROFILE:-release}"

color() { printf "\033[%sm%s\033[0m\n" "$1" "$2"; }
info() { color "0;34" "==> $1"; }
ok() { color "0;32" "✓ $1"; }
warn() { color "1;33" "⚠ $1"; }
die() { color "0;31" "✗ $1"; exit 1; }

have() { command -v "$1" >/dev/null 2>&1; }

run_sudo() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  else
    sudo "$@"
  fi
}

detect_pm() {
  if have pacman; then echo pacman; return; fi
  if have apt-get; then echo apt; return; fi
  if have dnf; then echo dnf; return; fi
  if have zypper; then echo zypper; return; fi
  if have apk; then echo apk; return; fi
  echo unknown
}

install_deps() {
  [ "${SNASK_INSTALL_DEPS}" = "1" ] || {
    warn "Instalação de dependências pulada por SNASK_INSTALL_DEPS=0."
    return
  }

  local pm
  pm="$(detect_pm)"
  info "Gerenciador detectado: ${pm}"

  case "${pm}" in
    pacman)
      run_sudo pacman -Syu --needed --noconfirm \
        base-devel git rust cargo llvm18 llvm18-libs clang18 lld18 pkgconf gtk3 zlib sqlite
      ;;
    apt)
      run_sudo apt-get update
      run_sudo apt-get install -y \
        build-essential git curl ca-certificates pkg-config rustc cargo \
        clang-18 llvm-18 llvm-18-dev lld-18 libclang-18-dev \
        libgtk-3-dev zlib1g-dev libsqlite3-dev
      ;;
    dnf)
      run_sudo dnf install -y \
        git rust cargo clang clang-devel llvm llvm-devel lld \
        pkgconf-pkg-config gtk3-devel zlib-devel sqlite-devel
      ;;
    zypper)
      run_sudo zypper --non-interactive install \
        git rust cargo clang llvm llvm-devel lld pkg-config gtk3-devel zlib-devel sqlite3-devel
      ;;
    apk)
      run_sudo apk add \
        build-base git rust cargo clang llvm18 llvm18-dev lld pkgconf gtk+3.0-dev zlib-dev sqlite-dev
      ;;
    *)
      warn "Não sei instalar dependências automaticamente nessa distro."
      warn "Instale manualmente: git, rust/cargo, clang/LLVM 18, lld, pkg-config, gtk3-dev, zlib-dev, sqlite-dev."
      ;;
  esac
}

major_version() {
  "$1" --version 2>/dev/null | sed -n 's/^[^0-9]*\([0-9][0-9]*\)\..*/\1/p' | head -n 1
}

first_working() {
  local candidate
  for candidate in "$@"; do
    if [ -x "${candidate}" ]; then
      echo "${candidate}"
      return 0
    elif command -v "${candidate}" >/dev/null 2>&1; then
      command -v "${candidate}"
      return 0
    fi
  done
  return 1
}

first_llvm18_tool() {
  local generic="$1"
  shift

  local tool
  if tool="$(first_working "$@")"; then
    echo "${tool}"
    return 0
  fi

  if command -v "${generic}" >/dev/null 2>&1 && [ "$(major_version "${generic}")" = "18" ]; then
    command -v "${generic}"
    return 0
  fi

  return 1
}

configure_llvm() {
  local llvm_config clang llc llvm_strip ld_lld

  llvm_config="$(first_llvm18_tool llvm-config \
    llvm-config-18 \
    /usr/bin/llvm-config-18 \
    /usr/lib/llvm18/bin/llvm-config \
    /usr/lib/llvm-18/bin/llvm-config)" || true

  if [ -z "${llvm_config:-}" ]; then
    cat >&2 <<'EOF'
✗ LLVM 18 não foi encontrado.

Snask ainda compila com inkwell/llvm-sys ligado ao LLVM 18. Instale LLVM 18 ou aponte:

  LLVM_CONFIG_PATH=/caminho/para/llvm-config-18
  LLVM_SYS_180_PREFIX=/prefixo/do/llvm18

Exemplos:
  Arch:   sudo pacman -S llvm18 llvm18-libs clang18 lld18
  Ubuntu: sudo apt install llvm-18 llvm-18-dev clang-18 lld-18 libclang-18-dev
EOF
    exit 1
  fi

  export LLVM_CONFIG_PATH="${llvm_config}"
  export LLVM_SYS_180_PREFIX
  LLVM_SYS_180_PREFIX="$(cd "$(dirname "${llvm_config}")/.." && pwd)"

  clang="$(first_llvm18_tool clang \
    clang-18 \
    /usr/bin/clang-18 \
    /usr/lib/llvm18/bin/clang \
    /usr/lib/llvm-18/bin/clang)" || true
  llc="$(first_llvm18_tool llc \
    llc-18 \
    /usr/bin/llc-18 \
    /usr/lib/llvm18/bin/llc \
    /usr/lib/llvm-18/bin/llc)" || true
  llvm_strip="$(first_llvm18_tool llvm-strip \
    llvm-strip-18 \
    /usr/bin/llvm-strip-18 \
    /usr/lib/llvm18/bin/llvm-strip \
    /usr/lib/llvm-18/bin/llvm-strip)" || true
  ld_lld="$(first_working ld.lld ld.lld-18 /usr/bin/ld.lld /usr/bin/ld.lld-18 /usr/lib/llvm18/bin/ld.lld /usr/lib/llvm-18/bin/ld.lld)" || true

  [ -n "${clang:-}" ] || die "clang 18 não encontrado."
  [ -n "${llc:-}" ] || die "llc 18 não encontrado."

  export SNASK_CLANG="${clang}"
  export SNASK_LLC="${llc}"
  [ -n "${llvm_strip:-}" ] && export SNASK_LLVM_STRIP="${llvm_strip}"
  [ -n "${ld_lld:-}" ] && export SNASK_LD_LLD="${ld_lld}"

  ok "LLVM: ${LLVM_CONFIG_PATH}"
  ok "clang: ${SNASK_CLANG}"
  ok "llc: ${SNASK_LLC}"
}

ensure_core_tools() {
  have git || die "git não encontrado."
  have cargo || die "cargo não encontrado."
}

checkout_source() {
  info "Preparando diretórios em ${SNASK_HOME}..."
  mkdir -p "${SNASK_HOME}/src"

  if [ -d "${SNASK_SRC}/.git" ]; then
    info "Atualizando código fonte em ${SNASK_SRC}..."
    git -C "${SNASK_SRC}" fetch --quiet origin "${BRANCH}"
    git -C "${SNASK_SRC}" checkout --quiet "${BRANCH}"
    git -C "${SNASK_SRC}" pull --quiet --ff-only origin "${BRANCH}"
  else
    info "Baixando código fonte em ${SNASK_SRC}..."
    rm -rf "${SNASK_SRC}"
    git clone --quiet --branch "${BRANCH}" "${REPO_URL}" "${SNASK_SRC}"
  fi
}

build_snask() {
  local cargo_args=(build)
  local bin_path="${SNASK_SRC}/target/debug/snask"

  if [ "${SNASK_PROFILE}" = "release" ]; then
    cargo_args+=(--release)
    bin_path="${SNASK_SRC}/target/release/snask"
  fi

  info "Compilando Snask com cargo ${cargo_args[*]}..."
  (cd "${SNASK_SRC}" && cargo "${cargo_args[@]}")

  info "Gerando runtime com snask setup..."
  (cd "${SNASK_SRC}" && "${bin_path}" setup)

  mkdir -p "${SNASK_HOME}/bin"
  cp "${bin_path}" "${SNASK_HOME}/bin/snask"
  chmod +x "${SNASK_HOME}/bin/snask"
}

print_path_help() {
  case ":${PATH}:" in
    *":${SNASK_HOME}/bin:"*) ;;
    *)
      warn "${SNASK_HOME}/bin ainda não está no PATH desta sessão."
      printf "\nAdicione no shell:\n\n"
      printf '  export PATH="%s/bin:$PATH"\n\n' "${SNASK_HOME}"
      ;;
  esac
}

install_deps
ensure_core_tools
configure_llvm
checkout_source
build_snask

ok "Snask instalado/atualizado em ${SNASK_HOME}/bin/snask"
print_path_help
info "Teste agora: ${SNASK_HOME}/bin/snask doctor"
