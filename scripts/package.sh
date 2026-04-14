#!/usr/bin/env bash
# Build release artifacts into dist/. Produces whatever tools are available:
#   - dist/openvpn-gui-linux-<ver>-x86_64.tar.gz   (always)
#   - dist/openvpn-gui-linux_<ver>_amd64.deb       (if `cargo deb` installed)
#   - dist/openvpn-gui-linux-<ver>-1.x86_64.rpm    (if `cargo generate-rpm` installed)
#
# Install the optional helpers once:
#   cargo install cargo-deb cargo-generate-rpm

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
DIST="$ROOT/dist"
mkdir -p "$DIST"

echo "==> cargo build --release"
cargo build --release --locked

# Portable tarball (works on any distro with OpenVPN installed)
STAGE="$(mktemp -d)"
PKG="openvpn-gui-linux-$VERSION-x86_64"
mkdir -p "$STAGE/$PKG"
cp target/release/openvpn-gui-linux "$STAGE/$PKG/"
cp packaging/openvpn-gui-linux.desktop "$STAGE/$PKG/"
cp assets/logo.png "$STAGE/$PKG/openvpn-gui-linux.png"
cp README.md "$STAGE/$PKG/" 2>/dev/null || true
cat > "$STAGE/$PKG/install.sh" <<'EOF'
#!/usr/bin/env bash
set -e
PREFIX="${PREFIX:-/usr/local}"
install -Dm755 openvpn-gui-linux "$PREFIX/bin/openvpn-gui-linux"
install -Dm644 openvpn-gui-linux.desktop "$PREFIX/share/applications/openvpn-gui-linux.desktop"
install -Dm644 openvpn-gui-linux.png "$PREFIX/share/icons/hicolor/256x256/apps/openvpn-gui-linux.png"
echo "Installed to $PREFIX"
EOF
chmod +x "$STAGE/$PKG/install.sh"
tar -C "$STAGE" -czf "$DIST/$PKG.tar.gz" "$PKG"
rm -rf "$STAGE"
echo "==> built $DIST/$PKG.tar.gz"

# .deb (Debian / Ubuntu)
if command -v cargo-deb >/dev/null 2>&1; then
    echo "==> cargo deb"
    cargo deb --no-build --output "$DIST/"
else
    echo "==> skipped .deb (install with: cargo install cargo-deb)"
fi

# .rpm (Fedora / RHEL / openSUSE)
if cargo --list 2>/dev/null | grep -q generate-rpm; then
    echo "==> cargo generate-rpm"
    cargo generate-rpm --output "$DIST/"
else
    echo "==> skipped .rpm (install with: cargo install cargo-generate-rpm)"
fi

echo
echo "Artifacts in $DIST:"
ls -lh "$DIST"
