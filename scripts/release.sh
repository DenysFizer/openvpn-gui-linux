#!/usr/bin/env bash
# Cut a GitHub Release from the local machine.
#
# Builds all four artifacts (.tar.gz, .deb, .rpm, .AppImage) plus
# SHA256SUMS via `scripts/package.sh` + linuxdeploy/appimagetool, tags
# the current HEAD, and uploads to GitHub Releases with `gh`.
#
# One-time setup:
#   cargo install cargo-deb cargo-generate-rpm
#   sudo apt install gh          # or equivalent; see https://cli.github.com
#   gh auth login
#
# Usage:
#   ./scripts/release.sh              # full release
#   ./scripts/release.sh --dry-run    # build artifacts only, no tag/upload
#   ./scripts/release.sh --skip-appimage
#   ./scripts/release.sh --yes        # skip interactive confirm before pushing

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DRY_RUN=0
SKIP_APPIMAGE=0
ASSUME_YES=0
for arg in "$@"; do
    case "$arg" in
        --dry-run)       DRY_RUN=1 ;;
        --skip-appimage) SKIP_APPIMAGE=1 ;;
        --yes|-y)        ASSUME_YES=1 ;;
        -h|--help)
            sed -n '2,18p' "$0"
            exit 0
            ;;
        *)
            echo "error: unknown argument '$arg'" >&2
            exit 2
            ;;
    esac
done

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "==> $*"; }

info "preflight checks"

command -v cargo >/dev/null || die "cargo not found on PATH"
command -v git   >/dev/null || die "git not found on PATH"

if [ "$DRY_RUN" -eq 0 ]; then
    command -v gh >/dev/null || die "gh (GitHub CLI) not found; install from https://cli.github.com"
    gh auth status >/dev/null 2>&1 || die "gh is not authenticated; run: gh auth login"

    if ! git diff --quiet || ! git diff --cached --quiet; then
        die "working tree has uncommitted changes; commit or stash first"
    fi
fi

command -v cargo-deb         >/dev/null 2>&1 || die "cargo-deb missing; run: cargo install cargo-deb"
cargo --list 2>/dev/null | grep -q generate-rpm || die "cargo-generate-rpm missing; run: cargo install cargo-generate-rpm"

VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
TAG="v$VERSION"
[ -n "$VERSION" ] || die "could not read version from Cargo.toml"

if [ "$DRY_RUN" -eq 0 ] && git rev-parse "$TAG" >/dev/null 2>&1; then
    die "tag $TAG already exists; bump version in Cargo.toml first"
fi

info "version: $VERSION (tag: $TAG)"

# Ensure AppImage tooling is present.
TOOLS="$ROOT/tools"
LINUXDEPLOY="$TOOLS/linuxdeploy-x86_64.AppImage"
APPIMAGETOOL="$TOOLS/appimagetool-x86_64.AppImage"
if [ "$SKIP_APPIMAGE" -eq 0 ]; then
    mkdir -p "$TOOLS"
    if [ ! -x "$LINUXDEPLOY" ]; then
        info "downloading linuxdeploy"
        curl -fsSL -o "$LINUXDEPLOY" \
            "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
        chmod +x "$LINUXDEPLOY"
    fi
    if [ ! -x "$APPIMAGETOOL" ]; then
        info "downloading appimagetool"
        curl -fsSL -o "$APPIMAGETOOL" \
            "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
        chmod +x "$APPIMAGETOOL"
    fi
fi

# Delegate to package.sh for tarball + .deb + .rpm.
info "building tarball, .deb, .rpm via scripts/package.sh"
bash "$ROOT/scripts/package.sh"

DIST="$ROOT/dist"
TARBALL="$DIST/openvpn-gui-linux-$VERSION-x86_64.tar.gz"
DEB="$DIST/openvpn-gui-linux_${VERSION}_amd64.deb"
RPM="$DIST/openvpn-gui-linux-$VERSION-1.x86_64.rpm"
APPIMAGE="$DIST/openvpn-gui-linux-$VERSION-x86_64.AppImage"

[ -f "$TARBALL" ] || die "missing $TARBALL"
[ -f "$DEB" ]     || die "missing $DEB (is cargo-deb installed?)"
[ -f "$RPM" ]     || die "missing $RPM (is cargo-generate-rpm installed?)"

# Build AppImage.
if [ "$SKIP_APPIMAGE" -eq 0 ]; then
    info "building AppImage"
    APPDIR="$ROOT/build/AppDir"
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR/usr/bin" \
             "$APPDIR/usr/share/applications" \
             "$APPDIR/usr/share/icons/hicolor/256x256/apps"

    cp "$ROOT/target/release/openvpn-gui-linux" "$APPDIR/usr/bin/"
    cp "$ROOT/packaging/openvpn-gui-linux.desktop" "$APPDIR/usr/share/applications/"
    cp "$ROOT/packaging/openvpn-gui-linux.desktop" "$APPDIR/"
    cp "$ROOT/assets/logo.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/openvpn-gui-linux.png"
    cp "$ROOT/assets/logo.png" "$APPDIR/openvpn-gui-linux.png"

    # Run the tooling in extract-and-run mode so we don't require libfuse2
    # on the host (Ubuntu 24.04+ dropped FUSE 2 from the default install).
    export APPIMAGE_EXTRACT_AND_RUN=1
    "$LINUXDEPLOY" --appdir "$APPDIR"
    ARCH=x86_64 "$APPIMAGETOOL" "$APPDIR" "$APPIMAGE"
    chmod +x "$APPIMAGE"
    [ -f "$APPIMAGE" ] || die "AppImage build failed"
fi

info "generating SHA256SUMS"
(
    cd "$DIST"
    # shellcheck disable=SC2046
    sha256sum $(ls openvpn-gui-linux-* openvpn-gui-linux_* 2>/dev/null) > SHA256SUMS
)
cat "$DIST/SHA256SUMS"

if [ "$DRY_RUN" -eq 1 ]; then
    info "dry run complete; artifacts in $DIST/"
    exit 0
fi

# Confirm before pushing the tag.
if [ "$ASSUME_YES" -eq 0 ]; then
    read -r -p "Tag HEAD as $TAG, push to origin, and publish release? [y/N] " ans
    case "$ans" in
        y|Y|yes|YES) ;;
        *) die "aborted by user" ;;
    esac
fi

info "tagging $TAG"
git tag -a "$TAG" -m "Release $TAG"
git push origin "$TAG"

info "creating GitHub Release"
FILES=("$TARBALL" "$DEB" "$RPM")
[ "$SKIP_APPIMAGE" -eq 0 ] && FILES+=("$APPIMAGE")
FILES+=("$DIST/SHA256SUMS")

gh release create "$TAG" \
    --title "v$VERSION" \
    --generate-notes \
    "${FILES[@]}"

info "done"
gh release view "$TAG" --web >/dev/null 2>&1 || true
