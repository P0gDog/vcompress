#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <version> [commit message]" >&2
    exit 1
fi

VERSION="$1"
MSG="${2:-Bump version to $VERSION}"

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "version must be e.g. 0.1.4, got: $VERSION" >&2
    exit 1
fi

REPO_DIR="$HOME/vcompress"
AUR_DIR="$HOME/aur-vcompress"

cd "$REPO_DIR"

if [[ -n "$(git status --porcelain)" ]]; then
    echo "working tree isnt clean." >&2
    exit 1
fi

echo "bumping Cargo.toml to $VERSION"
sed -i "0,/^version = \".*\"/s//version = \"$VERSION\"/" Cargo.toml

echo "regenerating Cargo.lock"
cargo check --quiet

echo "bumping packaging/arch/PKGBUILD"
sed -i "s/^pkgver=.*/pkgver=$VERSION/" packaging/arch/PKGBUILD
sed -i "s/^pkgrel=.*/pkgrel=1/" packaging/arch/PKGBUILD

echo "committing"
git add Cargo.toml Cargo.lock packaging/arch/PKGBUILD
git commit -m "$MSG"
git push

echo "tagging v$VERSION"
git tag "v$VERSION"
git push origin "v$VERSION"

echo "verifying the Arch package"
cp packaging/arch/PKGBUILD "$AUR_DIR/PKGBUILD"
(
    cd "$AUR_DIR"
    updpkgsums
    makepkg -si --noconfirm
)

echo "publishing to crates.io"
cargo publish --dry-run
read -rp "publish v$VERSION to crates.io for real? [y/N] " confirm
if [[ "$confirm" =~ ^[Yy]$ ]]; then
    cargo publish
else
    echo "skipped cargo publish"
fi

echo "done"
