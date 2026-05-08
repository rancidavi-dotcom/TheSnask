# Maintainer: Davi <davidev@snask.lang>
pkgname=snask
pkgver=0.4.1
pkgrel=1
pkgdesc="Snask Programming Language with Orchestrated Memory (OM) - Binary Release"
arch=('x86_64')
url="https://github.com/rancidavi-dotcom/TheSnask"
license=('MIT')
depends=('llvm18-libs' 'gtk3' 'zlib' 'sqlite' 'llvm18' 'clang18' 'lld18')
provides=('snask')
conflicts=('snask-git')
source_x86_64=(
  "${pkgname}-${pkgver}-x86_64::${url}/releases/download/v${pkgver}/snask-linux-amd64"
  "${pkgname}-${pkgver}.tar.gz::${url}/archive/refs/tags/v${pkgver}.tar.gz"
)
sha256sums_x86_64=('SKIP' 'SKIP')

package() {
  # Instala o binário
  install -Dm755 "${srcdir}/${pkgname}-${pkgver}-x86_64" "${pkgdir}/usr/bin/snask"

  # Pasta do código fonte extraído
  local src_dir="TheSnask-${pkgver}"

  # Cria diretórios de biblioteca
  install -dm755 "${pkgdir}/usr/lib/snask/src"
  install -dm755 "${pkgdir}/usr/lib/snask/runtime"
  install -dm755 "${pkgdir}/usr/lib/snask/stdlib"

  # Copia stdlib e runtime do código fonte
  cp -r "${src_dir}/src/"* "${pkgdir}/usr/lib/snask/src/"
}
