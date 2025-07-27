# Maintainer: relrel <relrelbachar@gmail.com>
pkgname=h263-rs-nightly-bin
pkgver=@VERSION@
pkgrel=1
pkgdesc="A pure-rust H.263 decoder"
arch=('x86_64')
url="https://ruffle.rs/"
license=('Apache' 'MIT')
depends=(openssl zlib libxcb alsa-lib)
provides=(h263-rs)
conflicts=(h263-rs-git)
source=("https://github.com/ruffle-rs/h263-rs/releases/download/nightly-${pkgver//./-}/h263-rs-nightly-${pkgver//./_}-linux-x86_64.tar.gz")
sha512sums=('SKIP')

package() {
	cd "$srcdir/"
	install -Dm755 -t "$pkgdir/usr/bin/" h263-rs
	install -Dm644 -t "$pkgdir/usr/share/doc/$pkgname/" README.md
	install -Dm644 -t "$pkgdir/usr/share/licenses/$pkgname/" LICENSE.md
}
