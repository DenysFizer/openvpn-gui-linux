Name:           openvpn-gui-linux
Version:        0.1.0
Release:        1%{?dist}
Summary:        Simple Linux GUI for OpenVPN

License:        MIT
URL:            https://github.com/DenysFizer/openvpn-gui-linux
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.85
Requires:       openvpn
Requires:       polkit

%description
A lightweight graphical front-end for the OpenVPN client, written in Rust
using the Iced toolkit. Wraps the openvpn CLI via its management interface
and provides a clean UI for connecting to, monitoring, and troubleshooting
VPN sessions.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm644 packaging/%{name}.desktop %{buildroot}%{_datadir}/applications/%{name}.desktop
install -Dm644 assets/logo.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/%{name}.png

%check
cargo test --release --locked

%files
%{_bindir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/256x256/apps/%{name}.png

%changelog
* Tue Apr 14 2026 Denys Fizer <fizerdenys@gmail.com> - 0.1.0-1
- Initial release.
