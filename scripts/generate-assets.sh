#!/bin/sh
set -eu

ASSETS="assets"
DOWNLOADS=".cache/downloads"
INTER_VERSION="4.1"

mkdir -p "$ASSETS/fonts" "$DOWNLOADS"

convert -size 512x512 -define gradient:angle=135 gradient:'#4F8BFF-#A05CFF' \
  \( -size 512x512 xc:none -fill white -draw "roundrectangle 0,0,511,511,120,120" \) \
  -compose DstIn -composite \
  -compose Over -fill 'rgba(255,255,255,0.92)' \
  -draw "polygon 112,372 214,232 282,316 322,268 400,372" \
  -fill white -draw "circle 344,170 344,132" \
  "$ASSETS/icon.png"
convert "$ASSETS/icon.png" -define icon:auto-resize=256,128,64,48,32,24,16 "$ASSETS/icon.ico"
convert "$ASSETS/icon.png" -resize 64x64 "$ASSETS/icon-64.png"

if [ ! -f "$ASSETS/fonts/Inter-Regular.ttf" ]; then
  curl -fsSL -o "$DOWNLOADS/inter.zip" \
    "https://github.com/rsms/inter/releases/download/v$INTER_VERSION/Inter-$INTER_VERSION.zip"
  unzip -o -q "$DOWNLOADS/inter.zip" -d "$DOWNLOADS/inter"
  cp "$DOWNLOADS/inter/extras/ttf/Inter-Regular.ttf" "$DOWNLOADS/inter/extras/ttf/Inter-SemiBold.ttf" "$ASSETS/fonts/"
  cp "$DOWNLOADS/inter/LICENSE.txt" "$ASSETS/fonts/OFL.txt"
fi
ls -la "$ASSETS" "$ASSETS/fonts"
