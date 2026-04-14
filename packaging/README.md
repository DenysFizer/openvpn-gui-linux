# Packaging

Distro-specific packaging sources for `openvpn-gui-linux`. Built artifacts
are written to `../dist/<distro>/` (gitignored).

| Directory   | Target                  | Build command                                    |
|-------------|-------------------------|--------------------------------------------------|
| `debian/`   | Debian / Ubuntu `.deb`  | `dpkg-buildpackage -us -uc -b`                   |
| `rpm/`      | Fedora / RHEL `.rpm`    | `rpmbuild -bb packaging/rpm/openvpn-gui-linux.spec` |
| `arch/`     | Arch Linux `.pkg.tar`   | `cd packaging/arch && makepkg -s`                |
| `flatpak/`  | Flatpak bundle          | `flatpak-builder build packaging/flatpak/*.yml`  |
| `appimage/` | Portable `.AppImage`    | `packaging/appimage/build.sh`                    |

## Shared files

- `openvpn-gui-linux.desktop` — XDG desktop entry, installed to
  `/usr/share/applications/`.
- `openvpn-gui-linux.png` symlink → `../assets/logo.png`, installed to
  `/usr/share/icons/hicolor/256x256/apps/`.

## Prerequisites

Every target assumes a release binary has been built:

```bash
cargo build --release
```

and OpenVPN is installed on the user's system (runtime dependency).
