#!/bin/sh
# Bündelt die PixelFX-Brücke samt PixelFX-Kern und den ausgewählten Effekten zu web/pfx.js.
# PixelFX: https://github.com/patch-gg/pixelfx (MIT), Stand a841596 (0.3.0).
# Aufruf: tools/pfx/bundle.sh <pfad-zu-pixelfx> [esbuild]
set -e
PX=${1:?Pfad zum PixelFX-Checkout}
ESB=${2:-esbuild}
DIR=$(cd "$(dirname "$0")" && pwd)
OUT="$DIR/../../web/pfx.js"
TMP=$(mktemp -d)
sed "s|PIXELFX/|$PX/|g" "$DIR/bridge.js" > "$TMP/bridge.js"
{
  echo "/*! Effekte: PixelFX 0.3.0 (https://github.com/patch-gg/pixelfx), MIT License."
  sed 's/^/ * /' "$PX/LICENSE"
  echo " *"
  sed 's/^/ * /' "$PX/NOTICE.md"
  echo " * Gebündelt für Throne of the Drowned mit tools/pfx/bundle.sh. */"
} > "$TMP/banner.txt"
"$ESB" "$TMP/bridge.js" --bundle --format=esm --minify --legal-comments=none --outfile="$TMP/pfx.js" --log-level=warning
cat "$TMP/banner.txt" "$TMP/pfx.js" > "$OUT"
rm -rf "$TMP"
echo "geschrieben: $OUT ($(wc -c < "$OUT") Bytes)"
