#!/bin/sh
set -eu

ASSETS="assets"
DOWNLOADS=".cache/downloads"
WORK=".cache/icon"
INTER_VERSION="4.1"

mkdir -p "$ASSETS/fonts" "$DOWNLOADS" "$WORK"

# Background: diagonal blue -> violet gradient, warm pink glow, soft top sheen, squircle mask.
convert -size 512x512 -define gradient:angle=135 gradient:'#2563EB-#7C3AED' "$WORK/base.png"
convert -size 512x512 radial-gradient:'rgba(236,72,153,0.85)-rgba(236,72,153,0)' "$WORK/glow.png"
convert -size 512x300 gradient:'rgba(255,255,255,0.20)-rgba(255,255,255,0)' \
  -background none -extent 512x512 "$WORK/sheen.png"
convert "$WORK/base.png" \( "$WORK/glow.png" -geometry +150+150 \) -compose Over -composite \
  "$WORK/sheen.png" -compose Over -composite "$WORK/background.png"
convert -size 512x512 xc:none -fill white -draw "roundrectangle 0,0,511,511,132,132" "$WORK/mask.png"
convert "$WORK/background.png" "$WORK/mask.png" -compose DstIn -composite "$WORK/squircle.png"

convert -size 512x512 xc:none \
  -fill none -stroke white -strokewidth 30 -draw "roundrectangle 112,128 400,322 40,40" \
  -stroke none -fill white -draw "roundrectangle 196,356 316,382 13,13" \
  -draw "polygon 226,178 226,272 304,225" "$WORK/glyph-large.png"
convert -size 512x512 xc:none \
  -fill none -stroke white -strokewidth 56 -draw "roundrectangle 92,112 420,340 52,52" \
  -stroke none -fill white -draw "polygon 206,162 206,290 316,226" "$WORK/glyph-small.png"

convert "$WORK/squircle.png" "$WORK/glyph-large.png" -compose Over -composite "$ASSETS/icon.png"
convert "$WORK/squircle.png" "$WORK/glyph-small.png" -compose Over -composite "$WORK/icon-small.png"
convert "$WORK/icon-small.png" -resize 64x64 "$ASSETS/icon-64.png"
for size in 16 20 24 32; do convert "$WORK/icon-small.png" -resize "${size}x${size}" "$WORK/icon-$size.png"; done
for size in 48 64 128 256; do convert "$ASSETS/icon.png" -resize "${size}x${size}" "$WORK/icon-$size.png"; done
convert "$WORK/icon-16.png" "$WORK/icon-20.png" "$WORK/icon-24.png" "$WORK/icon-32.png" \
  "$WORK/icon-48.png" "$WORK/icon-64.png" "$WORK/icon-128.png" "$WORK/icon-256.png" "$ASSETS/icon.ico"

if [ ! -f "$ASSETS/fonts/Inter-Regular.ttf" ]; then
  curl -fsSL -o "$DOWNLOADS/inter.zip" \
    "https://github.com/rsms/inter/releases/download/v$INTER_VERSION/Inter-$INTER_VERSION.zip"
  unzip -o -q "$DOWNLOADS/inter.zip" -d "$DOWNLOADS/inter"
  cp "$DOWNLOADS/inter/extras/ttf/Inter-Regular.ttf" "$DOWNLOADS/inter/extras/ttf/Inter-SemiBold.ttf" "$ASSETS/fonts/"
  cp "$DOWNLOADS/inter/LICENSE.txt" "$ASSETS/fonts/OFL.txt"
fi
ls -la "$ASSETS"
