#!/usr/bin/env bash
# Compila la versión web en una carpeta lista para servir (por defecto,
# dist/web): la página, el .wasm y el JavaScript que lo carga.
#
# Hace falta el target de wasm (`rustup target add wasm32-unknown-unknown`) y
# wasm-bindgen-cli con la misma versión de wasm-bindgen que Cargo.lock:
#
#   cargo install wasm-bindgen-cli --locked --version <la de Cargo.lock>
#
# Para probarla: `python3 -m http.server -d dist/web` y abrir
# http://localhost:8000.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-$root/dist/web}

cd "$root"
cargo build --release --target wasm32-unknown-unknown
rm -rf "$out"
mkdir -p "$out"
# Sin los nombres de las funciones, que ocupan casi tanto como el código.
wasm-bindgen --target web --no-typescript --remove-name-section --remove-producers-section \
  --out-dir "$out" target/wasm32-unknown-unknown/release/motorust.wasm
bytes=$(wc -c < "$out/motorust_bg.wasm" | tr -d ' ')
version=$(grep '^version = ' Cargo.toml | head -n 1 | cut -d'"' -f 2)
sed "s/const WASM_BYTES = 0;/const WASM_BYTES = $bytes/; s/<div class=\"version\" id=\"game-version\">[^<]*<\/div>/<div class=\"version\" id=\"game-version\">v$version<\/div>/" web/index.html > "$out/index.html"
cp web/favicon.png "$out/favicon.png"
[ -f web/favicon.ico ] && cp web/favicon.ico "$out/favicon.ico"
ls -l "$out"

