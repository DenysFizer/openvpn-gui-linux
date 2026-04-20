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

## Cutting a GitHub Release

`scripts/release.sh` wraps `package.sh`, builds an AppImage on top of the
three artifacts above, generates `SHA256SUMS`, tags the current commit,
and uploads everything to
[GitHub Releases](https://github.com/DenysFizer/openvpn-gui-linux/releases)
via the `gh` CLI:

```bash
./scripts/release.sh              # full release (interactive confirm)
./scripts/release.sh --dry-run    # build everything, skip tag + upload
```

One-time tooling:

```bash
cargo install cargo-deb cargo-generate-rpm
sudo apt install gh               # or equivalent; https://cli.github.com
gh auth login
```

`linuxdeploy` and `appimagetool` are auto-downloaded to `../tools/` on
first run (gitignored).

## Optional tooling

The `.deb` and `.rpm` outputs are built via Cargo helpers. Install once:

```bash
cargo install cargo-deb cargo-generate-rpm
```

If either tool is missing, `package.sh` skips that artifact and keeps
going (`release.sh` requires both).

## Shared files

- `openvpn-gui-linux.desktop` — XDG desktop entry (bundled in all four
  artifacts).
- `../assets/logo.png` — icon, bundled as `openvpn-gui-linux.png`.

Metadata for `cargo-deb` and `cargo-generate-rpm` lives under
`[package.metadata.*]` in the top-level `Cargo.toml`.
