Name:           utpm
Version:        0.2.0
Release:        1%{?dist}
Summary:        Unofficial Typst Package Manager

License:        MIT
URL:            https://github.com/typst-community/utpm
Source0:        https://github.com/typst-community/utpm/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo

%description
UTPM is a powerful command-line package manager for Typst.
Create, manage, and share Typst packages with ease.

%prep
%autosetup

%build
cargo build --release --all-features

%install
install -Dm755 target/release/utpm %{buildroot}%{_bindir}/utpm

%files
%license LICENSE
%doc README.md
%{_bindir}/utpm

%changelog
* Fri Nov 15 2025 Typst Community <https://github.com/typst-community> - 0.2.0-1
- Initial package
