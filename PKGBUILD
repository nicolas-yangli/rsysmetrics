# Maintainer: Your Name <youremail@domain.com>
# This PKGBUILD is for creating a local pre-release package.
# Run makepkg from the project root directory.
pkgname=rsysmetrics-git
_pkgname=rsysmetrics
pkgver=r29.0b05a68
pkgrel=1
pkgdesc="A system metrics collection agent written in Rust (local development build)"
arch=('x86_64')
url="https://github.com/nicolas-yangli/rsysmetrics"
depends=()
makedepends=('rust' 'cargo')
backup=('etc/rsysmetrics/rsysmetrics.toml')
options=(!strip)

pkgver() {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}

build() {
  cargo build --release --locked
}

package() {
  install -D -m755 "../target/release/$_pkgname" "$pkgdir/usr/bin/$_pkgname"
  install -D -m644 "../rsysmetrics.service" "$pkgdir/usr/lib/systemd/system/$_pkgname.service"
  install -D -m644 "../rsysmetrics.toml" "$pkgdir/etc/rsysmetrics/rsysmetrics.toml"
}
