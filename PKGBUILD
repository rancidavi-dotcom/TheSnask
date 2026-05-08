# Maintainer: Davi <davidev@snask.lang>
pkgname=snask
pkgver=04.2
pkgrel=1
pkgdesc="Snask Programming Language with Orchestrated Memory (OM) - Binary Release"
arch=('x86_64')
url="https://github.com/rancidavi-dotcom/TheSnask"
license=('MIT')
depends=('llvm18-libs' 'gtk3' 'zlib' 'sqlite' 'llvm18' 'clang18' 'lld18')
provides=('snask')
conflicts=('snask-git')
source_x86_64=(
  "snask-bin::${url}/releases/download/v${pkgver}/snask-linux-amd64"
  "snask-lsp-bin::${url}/releases/download/v${pkgver}/snask-lsp-linux-amd64"
  "snask-src.tar.gz::${url}/archive/refs/tags/v${pkgver}.tar.gz"
)
sha256sums_x86_64=('SKIP' 'SKIP' 'SKIP')

package() {
  # Instala os binários renomeados
  install -Dm755 "${srcdir}/snask-bin" "${pkgdir}/usr/bin/snask"
  install -Dm755 "${srcdir}/snask-lsp-bin" "${pkgdir}/usr/bin/snask-lsp"

  # Pasta do código fonte extraído (nome do repo no zip)
  local src_dir="TheSnask-${pkgver/-beta/}"

  # Cria diretórios
  install -dm755 "${pkgdir}/usr/lib/snask/src"

  # Copia fonte
  cp -r "${src_dir}/src/"* "${pkgdir}/usr/lib/snask/src/"
}
