%global crate randy

Name:    %{crate}
Version: 1.6.9
Release: 1%{?dist}
Summary: Conky like app in Rust

License: None
URL: https://github.com/iphands/randy
Source0: https://crates.io/api/v1/crates/%{crate}/%{version}/download#/%{crate}-%{version}.crate

ExclusiveArch: %{rust_arches}

BuildRequires: rust-packaging
BuildRequires: (crate(glib/default) >= 0.10.3 with crate(glib/default) < 0.11.0)
BuildRequires: (crate(gio/default) >= 0.9.1 with crate(gio/default) < 0.10.0)
BuildRequires: (crate(gtk/default) >= 0.9.2 with crate(gtk/default) < 0.10.0)
BuildRequires: (crate(yaml-rust/default) >= 0.4.0 with crate(yaml-rust/default) < 0.5.0)
BuildRequires: (crate(libc/default) >= 0.2.80 with crate(libc/default) < 0.3.0)
BuildRequires: (crate(lazy_static/default) >= 1.4.0 with crate(lazy_static/default) < 1.5.0)

%description
Conky like app in Rust. Aiming for parity with my own Conky setup and as little cpu time as possile.. k.i.s.s.

%prep
%autosetup -n %{crate}-%{version} -p1
%cargo_prep

%build
%cargo_build -f packaged

%install
%cargo_install -f packaged
mkdir -p $RPM_BUILD_ROOT%{_sysconfdir}/randy
install -m644 -p config/default.yml $RPM_BUILD_ROOT%{_sysconfdir}/randy

%files -n %{crate}
%{_bindir}/randy
%dir %{_sysconfdir}/randy
%config %{_sysconfdir}/randy/default.yml

%changelog
* Thu Dec 17 2020 Ian Page Hands <iphands@gmail.com> - 1.6.9-1
- Initial RPM release
