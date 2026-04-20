<div align="center">

# OpenVPN GUI for Linux

A simple, native desktop GUI for managing OpenVPN connections on Linux.

[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/)
[![Iced](https://img.shields.io/badge/UI-Iced%200.14-blueviolet)](https://iced.rs/)
[![License](https://img.shields.io/badge/license-MIT-green)](#license)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey?logo=linux)](#)
[![Download latest](https://img.shields.io/badge/Download-latest-2ea44f?logo=github&logoColor=white)](https://github.com/DenysFizer/openvpn-gui-linux/releases/latest)

</div>

## Screenshots

| Connect | Profiles | Settings |
| :---: | :---: | :---: |
| ![Connect tab](assets/screenshots/connect.png) | ![Profiles tab](assets/screenshots/profiles.png) | ![Settings tab](assets/screenshots/settings.png) |

## Overview

A Rust + [Iced](https://iced.rs/) front-end that drives the `openvpn` CLI through its management interface — a clean, dependency-light alternative to web-wrapped VPN clients.

## Features

|   |   |
|---|---|
| **One-click connect** — connect and disconnect any `.ovpn` profile | **Live status & logs** — real-time connection state and streaming output |
| **Profile management** — browse `.ovpn` files and persist selections across launches | **Flexible auth** — username / password with static-challenge support |
| **Inline certificates** — `<ca>`, `<cert>`, `<key>`, `<tls-auth>` blocks handled | **Native UI** — Iced + Rust, no Electron, no web stack |

## Installation

Requires `openvpn` on your `PATH` (e.g. `sudo apt install openvpn` on Debian/Ubuntu).

<div align="center">

[![Debian / Ubuntu](https://img.shields.io/badge/Debian%20%2F%20Ubuntu-.deb-A81D33?logo=debian&logoColor=white)](https://github.com/DenysFizer/openvpn-gui-linux/releases/latest)
[![Fedora / RHEL](https://img.shields.io/badge/Fedora%20%2F%20RHEL-.rpm-294172?logo=fedora&logoColor=white)](https://github.com/DenysFizer/openvpn-gui-linux/releases/latest)
[![AppImage](https://img.shields.io/badge/Any%20distro-.AppImage-2E6FD0?logo=appimage&logoColor=white)](https://github.com/DenysFizer/openvpn-gui-linux/releases/latest)
[![Portable](https://img.shields.io/badge/Portable-.tar.gz-525252?logo=linux&logoColor=white)](https://github.com/DenysFizer/openvpn-gui-linux/releases/latest)

</div>

| Distro                    | Install                                                 |
|---------------------------|---------------------------------------------------------|
| Debian / Ubuntu           | `sudo apt install ./openvpn-gui-linux_*_amd64.deb`      |
| Fedora / RHEL / openSUSE  | `sudo dnf install ./openvpn-gui-linux-*.rpm`            |
| Arch / NixOS / any distro | `chmod +x *.AppImage && ./openvpn-gui-linux-*.AppImage` |
| Portable tarball          | `tar xzf *.tar.gz && sudo ./*/install.sh`               |

### Verify your download

```bash
sha256sum -c SHA256SUMS
```

### Build from source

Requires Rust **1.85+** (edition 2024) and `pkexec` / `sudo` for elevated privileges when bringing interfaces up.

```bash
git clone git@github.com:DenysFizer/openvpn-gui-linux.git
cd openvpn-gui-linux
cargo run --release
```

The compiled binary will be at `target/release/openvpn-gui-linux`.

### Building the release artifacts yourself

```bash
cargo install cargo-deb cargo-generate-rpm   # one-time
./scripts/package.sh                          # outputs to dist/
```

Maintainers cutting a full release (builds everything above plus an AppImage and uploads to GitHub Releases):

```bash
./scripts/release.sh
```

See [`packaging/README.md`](packaging/README.md) for details.

## Usage

1. Launch the app.
2. Click **Load Config** and pick your `.ovpn` file.
3. Enter credentials if the profile requires them.
4. Press **Connect** — status and logs stream live in the main view.
5. **Disconnect** cleanly tears down the session.

## Project Layout

```
src/
├── app.rs           # Iced application entry point & state
├── config/          # .ovpn parser and configuration model
├── openvpn/         # Process manager + management-interface client
├── ui/              # Views: connect, logs, status bar
├── settings.rs      # Persisted user preferences
├── error.rs         # Unified error types
└── main.rs
tests/
└── fixtures/        # Sample .ovpn files for parser tests
```

## Development

```bash
cargo check          # fast type-check
cargo test           # run unit & integration tests
cargo clippy         # lints
cargo fmt            # format
```

## Contributing

Issues and pull requests are welcome. For substantial changes, please open an issue first to discuss the direction.

## License

MIT © Denys Fizer
