#!/usr/bin/env bash
# Compila la versión web en una carpeta lista para servir (por defecto,
# dist/web): la página, el .wasm y el JavaScript que lo carga, y el contenido
# del juego aparte (no dentro del .wasm): content.pack, que la página descarga
# y le pasa al juego, y las canciones sueltas en content/, que el juego pide
# cuando suenan (ver tools/web_pack.py en el motor).
#
# Hace falta Python 3, el target de wasm (`rustup target add wasm32-unknown-unknown`) y
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
# The content packs, the game's over the engine's, apart from the .wasm.
engine=$(tools/engine_dir.sh)
python=$(command -v python3 || command -v python)
"$python" "$engine/tools/web_pack.py" "$out" "$root/assets" "$engine/assets"
sha256() {
  if command -v sha256sum > /dev/null; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}
bytes=$(wc -c < "$out/motorust_bg.wasm" | tr -d ' ')
content_bytes=$(wc -c < "$out/content.pack" | tr -d ' ')
# The SHA-256 names this build: the page asks for the .wasm (and the
# content.pack) with it, so a browser never reuses another build's copy,
# and checks the copy it keeps.
hash=$(sha256 "$out/motorust_bg.wasm")
content_hash=$(sha256 "$out/content.pack")
version=$(grep '^version = ' Cargo.toml | head -n 1 | cut -d'"' -f 2)
sed "s/const WASM_BYTES = 0;/const WASM_BYTES = $bytes;/; s/const WASM_SHA256 = \"\";/const WASM_SHA256 = \"$hash\";/; s/const CONTENT_BYTES = 0;/const CONTENT_BYTES = $content_bytes;/; s/const CONTENT_SHA256 = \"\";/const CONTENT_SHA256 = \"$content_hash\";/; s/const GAME_VERSION = \"[^\"]*\";/const GAME_VERSION = \"$version\";/; s/<div class=\"version\" id=\"game-version\">[^<]*<\/div>/<div class=\"version\" id=\"game-version\">v$version<\/div>/" web/index.html > "$out/index.html"
# The installer for Linux and macOS, with the version of this build, so it
# always downloads the same release the page is publishing.
sed "s/__MOTORUST_VERSION__/$version/g" web/setup.sh > "$out/setup.sh"
chmod +x "$out/setup.sh"
# The version this site publishes, which the game reads to know whether it
# can update itself (see `updates` in assets/game.ron).
printf '%s\n' "$version" > "$out/version.txt"
cp web/favicon.png "$out/favicon.png"
[ -f web/favicon.ico ] && cp web/favicon.ico "$out/favicon.ico"
[ -d web/images ] && cp -r web/images "$out/images"
ls -l "$out"

