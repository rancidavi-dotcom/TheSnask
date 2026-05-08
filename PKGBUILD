# Maintainer: Davi <davidev@snask.lang>
pkgname=snask
pkgver=0.4.1
pkgrel=1
pkgdesc="Snask Programming Language with Orchestrated Memory (OM) - Binary Release"
arch=('x86_64')
url="https://github.com/rancidavi-dotcom/TheSnask"
license=('MIT')
depends=('llvm18-libs' 'gtk3' 'zlib' 'sqlite')
provides=('snask')
conflicts=('snask-git')
source_x86_64=(
  "${pkgname}-${pkgver}-x86_64::${url}/releases/download/v${pkgver}/snask-linux-amd64"
  "${pkgname}-${pkgver}.tar.gz::${url}/archive/refs/tags/v${pkgver}.tar.gz"
)
sha256sums_x86_64=('SKIP' 'SKIP') # Updated by release script

package() {
  # Instala o binário
  install -Dm755 "${srcdir}/${pkgname}-${pkgver}-x86_64" "${pkgdir}/usr/bin/snask"
  
  # Pasta do código fonte extraído
  local src_dir="TheSnask-${pkgver}"

  # Cria diretórios de biblioteca
  install -dm755 "${pkgdir}/usr/lib/snask/runtime"
  install -dm755 "${pkgdir}/usr/lib/snask/stdlib"

  # Copia stdlib e runtime do código fonte
  cp -r "${src_dir}/src/runtime/"* "${pkgdir}/usr/lib/snask/runtime/"
  cp -r "${src_dir}/src/stdlib/"* "${pkgdir}/usr/lib/snask/stdlib/"
  cp "${src_dir}/src/runtime.c" "${pkgdir}/usr/lib/snask/runtime/"

  # O usuário ainda precisará rodar 'snask setup' para gerar os .o e .bc
  # ou podemos automatizar isso no post-install se necessário.
}
