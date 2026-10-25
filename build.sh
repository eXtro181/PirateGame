#!/bin/sh
# Baut die Web-Version nach dist/ (lokal mit vendorten Crates, in CI mit Netz).
set -e
PROFILE=${PROFILE:-release}
cargo build --target wasm32-unknown-unknown --profile "$PROFILE"
DIR=$PROFILE; [ "$PROFILE" = "dev" ] && DIR=debug
rm -rf dist && mkdir -p dist
${WASM_BINDGEN:-wasm-bindgen} --target web --no-typescript --out-dir dist target/wasm32-unknown-unknown/$DIR/schwarzwasser.wasm
if [ -n "$WASM_OPT" ] && [ "$PROFILE" = "release" ]; then
  $WASM_OPT -Os --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext --enable-mutable-globals -o dist/schwarzwasser_bg.wasm dist/schwarzwasser_bg.wasm
fi
cp web/index.html dist/
cp -r assets dist/ 2>/dev/null || true
cp public/* dist/ 2>/dev/null || true
ls -la dist
