#!/bin/sh
set -eu

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
TARGET="x86_64-pc-windows-gnu"
OUT=".cache/release"
NAME="Rustpaper-$VERSION-windows-x64"

RUSTFLAGS='-L .cache/mpv' cargo build --release --target "$TARGET"
mkdir -p dist "$OUT"
cp "target/$TARGET/release/rustpaper-studio.exe" "target/$TARGET/release/rustpaper-daemon.exe" dist/
cp .cache/mpv/libmpv-2.dll dist/

rm -rf "$OUT/Rustpaper" "$OUT/$NAME.zip" "$OUT/$NAME.msi"
mkdir -p "$OUT/Rustpaper"
cp dist/*.exe dist/libmpv-2.dll LICENSE THIRD_PARTY_NOTICES.md CHANGELOG.md "$OUT/Rustpaper/"
(cd "$OUT" && zip -qr "$NAME.zip" Rustpaper)

wixl -a x64 -D Version="$VERSION" -o "$OUT/$NAME.msi" installer/rustpaper.wxs

(cd "$OUT" && sha256sum "$NAME.zip" "$NAME.msi" > "$NAME.sha256" && ls -la "$NAME".*)
