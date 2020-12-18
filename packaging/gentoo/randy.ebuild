EAPI=7

CRATES="
anyhow-1.0.34
atk-0.9.0
atk-sys-0.10.0
bitflags-1.2.1
cairo-rs-0.9.1
cairo-sys-rs-0.10.0
cc-1.0.62
darling-0.10.2
darling_core-0.10.2
darling_macro-0.10.2
either-1.6.1
fnv-1.0.7
futures-0.3.8
futures-channel-0.3.8
futures-core-0.3.8
futures-executor-0.3.8
futures-io-0.3.8
futures-macro-0.3.8
futures-sink-0.3.8
futures-task-0.3.8
futures-util-0.3.8
gdk-0.13.2
gdk-pixbuf-0.9.0
gdk-pixbuf-sys-0.10.0
gdk-sys-0.10.0
gio-0.9.1
gio-sys-0.10.1
glib-0.10.3
glib-macros-0.10.1
glib-sys-0.10.1
gobject-sys-0.10.0
gtk-0.9.2
gtk-sys-0.10.0
heck-0.3.1
ident_case-1.0.1
itertools-0.9.0
lazy_static-1.4.0
libc-0.2.80
libsensors-sys-0.2.0
linked-hash-map-0.5.3
memchr-2.3.4
nvml-wrapper-0.6.0
nvml-wrapper-sys-0.4.2
once_cell-1.5.2
pango-0.9.1
pango-sys-0.10.0
pin-project-1.0.1
pin-project-internal-1.0.1
pin-utils-0.1.0
pkg-config-0.3.19
proc-macro-crate-0.1.5
proc-macro-error-1.0.4
proc-macro-error-attr-1.0.4
proc-macro-hack-0.5.19
proc-macro-nested-0.1.6
proc-macro2-1.0.24
quote-1.0.7
randy-1.6.3
sensors-0.2.1
serde-1.0.117
slab-0.4.2
strsim-0.9.3
strum-0.18.0
strum_macros-0.18.0
syn-1.0.48
system-deps-1.3.2
thiserror-1.0.22
thiserror-impl-1.0.22
toml-0.5.7
unicode-segmentation-1.6.0
unicode-xid-0.2.1
version-compare-0.0.10
version_check-0.9.2
winapi-0.3.9
winapi-i686-pc-windows-gnu-0.4.0
winapi-x86_64-pc-windows-gnu-0.4.0
wrapcenum-derive-0.4.0
yaml-rust-0.4.4
"

inherit cargo desktop

DESCRIPTION="Conky inspired system info viewer written in Rust / GTK3"
HOMEPAGE="https://github.com/iphands/randy"
SRC_URI="https://github.com/iphands/${PN}/archive/${PV}.tar.gz -> ${P}.tar.gz
$(cargo_crate_uris ${CRATES})"

KEYWORDS="amd64 ~arm64 ~ppc64"
LICENSE="GPL-3"
SLOT="0"

IUSE="wayland X lm-sensors nvidia debug"
REQUIRED_USE="|| ( wayland X )"

COMMON_DEPEND="
	dev-lang/rust
	>=x11-libs/gtk+-3.22.0
	lm-sensors? ( sys-apps/lm-sensors )
	nvidia? ( x11-drivers/nvidia-drivers )
"

RDEPEND="${COMMON_DEPEND}"
BDEPEND="${COMMON_DEPEND}"

S="${WORKDIR}/${PN}-${PV}"

src_unpack() {
	cargo_src_unpack
}

src_configure() {
	local myfeatures=(
		packaged
		$(usex debug timings '')
		$(usex lm-sensors sensors '')
		$(usex nvidia nvidia '')
	)
	cargo_src_configure --no-default-features
}

src_compile() {
	cargo_src_compile
}

src_install() {
	cargo_src_install

	insinto /etc/randy
	doins -r config/default.yml

	local DOCS=(
		README.md
	)
	einstalldocs
}

src_test() {
	cargo_src_test
}
