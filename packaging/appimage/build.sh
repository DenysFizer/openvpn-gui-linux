#!/usr/bin/env bash
# Build a portable AppImage for openvpn-gui-linux.
# Requires: appimagetool (https://github.com/AppImage/AppImageKit)

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APPDIR="$ROOT/dist/appimage/openvpn-gui-linux.AppDir"
OUT="$ROOT/dist/appimage"

rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps" "$OUT"

cargo build --release --locked --manifest-path "$ROOT/Cargo.toml"

install -Dm755 "$ROOT/target/release/openvpn-gui-linux" \
    "$APPDIR/usr/bin/openvpn-gui-linux"
install -Dm644 "$HERE/../openvpn-gui-linux.desktop" \
    "$APPDIR/openvpn-gui-linux.desktop"
install -Dm644 "$HERE/../openvpn-gui-linux.desktop" \
    "$APPDIR/usr/share/applications/openvpn-gui-linux.desktop"
install -Dm644 "$ROOT/assets/logo.png" \
    "$APPDIR/openvpn-gui-linux.png"
install -Dm644 "$ROOT/assets/logo.png" \
    "$APPDIR/usr/share/icons/hicolor/256x256/apps/openvpn-gui-linux.png"
install -Dm755 "$HERE/AppRun" "$APPDIR/AppRun"

appimagetool "$APPDIR" "$OUT/openvpn-gui-linux-x86_64.AppImage"
echo "Built: $OUT/openvpn-gui-linux-x86_64.AppImage"
