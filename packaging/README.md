# Packaging

Build distributable artifacts with a single command:

```bash
./scripts/package.sh
```

All output goes to `../dist/` (gitignored):

| File                                       | Target                  | Install command                         |
|--------------------------------------------|-------------------------|-----------------------------------------|
| `openvpn-gui-linux-<ver>-x86_64.tar.gz`    | Any Linux (portable)    | `tar xzf` → `./install.sh`              |
| `openvpn-gui-linux_<ver>_amd64.deb`        | Debian / Ubuntu         | `sudo apt install ./<file>.deb`         |
| `openvpn-gui-linux-<ver>-1.x86_64.rpm`     | Fedora / RHEL / openSUSE | `sudo dnf install ./<file>.rpm`         |

## Optional tooling

The `.deb` and `.rpm` outputs are built via Cargo helpers. Install once:

```bash
cargo install cargo-deb cargo-generate-rpm
```

If either tool is missing, the script skips that artifact and keeps going.

## Shared files

- `openvpn-gui-linux.desktop` — XDG desktop entry (bundled in all three
  artifacts).
- `../assets/logo.png` — icon, bundled as `openvpn-gui-linux.png`.

Metadata for `cargo-deb` and `cargo-generate-rpm` lives under
`[package.metadata.*]` in the top-level `Cargo.toml`.
