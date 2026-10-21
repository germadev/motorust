#!/usr/bin/env bash
#
# Motorust installer for Linux and macOS.
#
# It downloads the game, leaves the executable in ~/motorust/motorust and
# creates a symlink in a directory of $PATH, so the game is started from any
# terminal with:
#
#   motorust
#
# Usage:
#   curl -fsSL https://motorust.germade.dev/setup.sh | bash
#   wget -qO- https://motorust.germade.dev/setup.sh | bash
#
# Run without options, it asks whether to install, update or uninstall the
# game, depending on what this computer already has.
#
# Options:
#   --install     install (or update) without asking
#   --update      update only
#   --uninstall   remove the game and its symlink
#   --yes         same as --install, for unattended runs
#   --dir PATH    install somewhere else (default: ~/motorust)
#   --version     print the version this script installs
#   --help        print this help
#
set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
site_url="${MOTORUST_SITE_URL:-https://motorust.germade.dev}"
releases_url="${MOTORUST_RELEASES_URL:-https://github.com/germadev/motorust/releases}"
# web/build.sh replaces this placeholder with the version of the site.
version="__MOTORUST_VERSION__"
home="${HOME:-}"
install_dir="${MOTORUST_HOME:-${home:+$home/motorust}}"
binary_name="motorust"
binary="$install_dir/$binary_name"
version_file="$install_dir/.motorust-version"

action=""
tty_fd=""
tmp_dir=""
link_dir=""
needs_path_fix=0

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ]; then
  bold="$(printf '\033[1m')"
  dim="$(printf '\033[2m')"
  red="$(printf '\033[31m')"
  green="$(printf '\033[32m')"
  yellow="$(printf '\033[33m')"
  rust="$(printf '\033[38;5;208m')"
  off="$(printf '\033[0m')"
else
  bold=""
  dim=""
  red=""
  green=""
  yellow=""
  rust=""
  off=""
fi

say() { printf '%s\n' "$*"; }
info() { printf '%s\n' "${dim}$*${off}"; }
step() { printf '%s\n' "${rust}==>${off} $*"; }
ok() { printf '%s\n' "${green}OK${off} $*"; }
warn() { printf '%s\n' "${yellow}!!${off} $*" >&2; }
err() { printf '%s\n' "${red}XX${off} $*" >&2; }
die() { err "$*"; exit 1; }

cleanup() {
  if [ -n "$tmp_dir" ] && [ -d "$tmp_dir" ]; then
    rm -rf "$tmp_dir"
  fi
}
trap cleanup EXIT

have() { command -v "$1" >/dev/null 2>&1; }

# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------
fetch_to() {
  # fetch_to <url> <destination>
  local url="$1" dest="$2"
  if have curl; then
    if [ -t 2 ]; then
      curl -fL --progress-bar -o "$dest" "$url"
    else
      curl -fL --silent --show-error -o "$dest" "$url"
    fi
  elif have wget; then
    wget -O "$dest" "$url"
  else
    die "Hace falta curl o wget para descargar el juego."
  fi
}

fetch_text() {
  # fetch_text <url>: prints the body, fails when the server says no
  local url="$1"
  if have curl; then
    curl -fsSL "$url"
  elif have wget; then
    wget -qO- "$url"
  else
    return 1
  fi
}

sha256_of() {
  # sha256_of <file>: prints the hex digest, or nothing without tools
  if have sha256sum; then
    sha256sum "$1" | cut -d' ' -f1
  elif have shasum; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    printf ''
  fi
}

version_gt() {
  # version_gt <a> <b>: true when a is newer than b (dot separated numbers)
  local a="$1" b="$2" i x y
  local as bs
  IFS='.' read -r -a as <<< "$a"
  IFS='.' read -r -a bs <<< "$b"
  for i in 0 1 2 3; do
    x="${as[$i]:-0}"
    y="${bs[$i]:-0}"
    x="${x//[!0-9]/}"
    y="${y//[!0-9]/}"
    x="${x:-0}"
    y="${y:-0}"
    if [ "$x" -gt "$y" ]; then return 0; fi
    if [ "$x" -lt "$y" ]; then return 1; fi
  done
  return 1
}

open_tty() {
  # The script usually arrives through a pipe (curl | bash), so the menu has
  # to be read from the terminal itself, never from the standard input.
  if { exec 3<>/dev/tty; } 2>/dev/null; then
    tty_fd=3
  elif [ -t 0 ]; then
    tty_fd=0
  else
    tty_fd=""
  fi
}

ask() {
  # ask <question>: prints the answer, fails when there is no terminal
  local prompt="$1" answer=""
  if [ -z "$tty_fd" ]; then
    return 1
  fi
  printf '%s' "$prompt" >&"$tty_fd"
  if ! IFS= read -r answer <&"$tty_fd"; then
    printf '\n' >&"$tty_fd"
    return 1
  fi
  printf '%s' "$answer"
}

confirm() {
  # confirm <question>: true when the answer is yes
  local answer
  answer="$(ask "$1 [s/N] ")" || return 1
  case "$answer" in
    s|S|y|Y|si|SI|sí|SÍ|yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

# ---------------------------------------------------------------------------
# System detection
# ---------------------------------------------------------------------------
detect_target() {
  # Prints the Rust target triple of this machine.
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"
  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64)
          printf 'x86_64-unknown-linux-gnu\n'
          ;;
        aarch64|arm64)
          die "Todavía no hay builds para Linux ARM ($arch): en Linux solo hay x86_64."
          ;;
        *)
          die "Arquitectura de Linux no soportada: $arch"
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        arm64|aarch64) printf 'aarch64-apple-darwin\n' ;;
        x86_64) printf 'x86_64-apple-darwin\n' ;;
        *) die "Arquitectura de macOS no soportada: $arch" ;;
      esac
      ;;
    *)
      die "Este instalador solo funciona en Linux y macOS (este sistema es $os)."
      ;;
  esac
}

resolve_version() {
  # The version comes with the script; if the file was not built by
  # web/build.sh, the site is asked which version it is publishing.
  local html found
  case "$version" in
    [0-9]*) return 0 ;;
  esac
  html="$(fetch_text "${site_url}/" 2>/dev/null || true)"
  found="$(printf '%s' "$html" | sed -n 's/.*const GAME_VERSION = "\([0-9][0-9.]*\)".*/\1/p' | head -n 1)"
  if [ -n "$found" ]; then
    version="$found"
    return 0
  fi
  die "No se pudo saber la versión de Motorust. Usa el script publicado en ${site_url}/setup.sh."
}

installed_version() {
  # Prints the version installed in this computer, or nothing.
  if [ -f "$version_file" ]; then
    tr -d ' \t\n\r' < "$version_file"
  fi
}

is_installed() {
  [ -x "$binary" ]
}

# ---------------------------------------------------------------------------
# The symlink
# ---------------------------------------------------------------------------
path_entries() {
  # Prints the directories of $PATH, one per line, without empty entries.
  local entries
  IFS=':' read -r -a entries <<< "$PATH"
  local entry
  for entry in "${entries[@]}"; do
    [ -n "$entry" ] || continue
    case "$entry" in
      "~/"*) entry="$HOME/${entry#~/}" ;;
    esac
    entry="${entry%/}"
    [ -n "$entry" ] || continue
    printf '%s\n' "$entry"
  done
}

choose_link_dir() {
  # Sets $link_dir to the directory where the symlink goes:
  # ~/.local/bin when it is in $PATH, otherwise the first writable directory
  # of $PATH, and ~/.local/bin as a last resort.
  local local_bin="$HOME/.local/bin" entry found=""
  local entries
  entries="$(path_entries)"
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    if [ "$entry" = "$local_bin" ]; then
      found="$local_bin"
      break
    fi
  done <<EOF
$entries
EOF
  if [ -z "$found" ]; then
    while IFS= read -r entry; do
      [ -n "$entry" ] || continue
      if [ -d "$entry" ] && [ -w "$entry" ]; then
        found="$entry"
        break
      fi
    done <<EOF
$entries
EOF
  fi
  if [ -z "$found" ]; then
    found="$local_bin"
    needs_path_fix=1
  fi
  if [ ! -d "$found" ]; then
    mkdir -p "$found" 2>/dev/null || die "No se pudo crear el directorio $found."
  fi
  link_dir="$found"
}

link_path_line() {
  printf 'export PATH="$HOME/.local/bin:$PATH"\n'
}

shell_rc_file() {
  # The file where the PATH of the shell is written.
  case "${SHELL:-}" in
    */zsh) printf '%s\n' "$HOME/.zshrc" ;;
    */bash) printf '%s\n' "$HOME/.bashrc" ;;
    *) printf '%s\n' "$HOME/.profile" ;;
  esac
}

offer_path_fix() {
  # Adds ~/.local/bin to the PATH of the shell, if the user wants it.
  local rc line
  rc="$(shell_rc_file)"
  line="$(link_path_line)"
  if [ -f "$rc" ] && grep -Fq '.local/bin' "$rc"; then
    info "El PATH ya incluye ~/.local/bin en $rc."
    return 0
  fi
  if ! confirm "Añadir ~/.local/bin al PATH en $rc?"; then
    warn "Añade esta línea a $rc para poder ejecutar 'motorust' desde cualquier sitio:"
    say "    $line"
    return 0
  fi
  {
    printf '\n# Motorust\n'
    printf '%s\n' "$line"
  } >> "$rc"
  ok "PATH añadido en $rc. Abre una terminal nueva (o ejecuta 'source $rc')."
}

create_link() {
  # Creates the symlink that makes `motorust` run from any terminal.
  local link
  choose_link_dir
  link="$link_dir/$binary_name"
  if [ -e "$link" ] && [ ! -L "$link" ]; then
    warn "Había un archivo en $link: se guarda como ${link}.old"
    mv -f "$link" "${link}.old" || true
  fi
  ln -sfn "$binary" "$link" || die "No se pudo crear el enlace $link."
  ok "Enlace creado: $link"
  if [ "$needs_path_fix" = "1" ]; then
    offer_path_fix
  fi
}

remove_link() {
  # Removes the symlink of this installer, wherever it is.
  local entry link
  while IFS= read -r entry; do
    [ -n "$entry" ] || continue
    link="$entry/$binary_name"
    if [ -L "$link" ]; then
      if [ "$(readlink "$link")" = "$binary" ]; then
        rm -f "$link"
        ok "Enlace eliminado: $link"
      else
        info "Se deja $link: no apunta a este juego."
      fi
    fi
  done <<EOF
$(path_entries)
EOF
  if [ -L "$HOME/.local/bin/$binary_name" ] &&
    [ "$(readlink "$HOME/.local/bin/$binary_name")" = "$binary" ]; then
    rm -f "$HOME/.local/bin/$binary_name"
    ok "Enlace eliminado: $HOME/.local/bin/$binary_name"
  fi
}

# ---------------------------------------------------------------------------
# Download and install
# ---------------------------------------------------------------------------
verify_checksum() {
  # verify_checksum <archive> <file name>: checks the sum published with the
  # release, when it is available.
  local archive="$1" name="$2" sums expected actual
  sums="$(fetch_text "${releases_url}/download/v${version}/SHA256SUMS.txt" 2>/dev/null || true)"
  if [ -z "$sums" ]; then
    info "No hay sumas SHA-256 publicadas para esta versión: se continúa sin comprobar."
    return 0
  fi
  expected="$(printf '%s\n' "$sums" | awk -v n="$name" '$2 == n || $2 == "*" n { print $1; exit }')"
  if [ -z "$expected" ]; then
    info "El archivo no aparece en SHA256SUMS.txt: se continúa sin comprobar."
    return 0
  fi
  actual="$(sha256_of "$archive")"
  if [ -z "$actual" ]; then
    info "No hay sha256sum ni shasum: se continúa sin comprobar."
    return 0
  fi
  if [ "$actual" != "$expected" ]; then
    die "La suma SHA-256 no coincide con la publicada. El archivo puede estar dañado."
  fi
  ok "Suma SHA-256 correcta."
}

install_game() {
  # Downloads the game and leaves it in $install_dir.
  local target="$1" url archive src name
  name="motorust-${version}-${target}.tar.gz"
  url="${site_url}/downloads/${name}"
  tmp_dir="$(mktemp -d)"
  archive="$tmp_dir/$name"

  step "Descargando Motorust ${version} para ${target}..."
  info "$url"
  if ! fetch_to "$url" "$archive"; then
    err "No se pudo descargar el juego."
    err "Comprueba la conexión o bájalo a mano desde ${site_url} y descomprímelo."
    exit 1
  fi

  verify_checksum "$archive" "$name"

  step "Descomprimiendo..."
  mkdir -p "$tmp_dir/extract"
  tar -xzf "$archive" -C "$tmp_dir/extract" || die "No se pudo descomprimir el archivo descargado."
  src="$(find "$tmp_dir/extract" -type f -name "$binary_name" | head -n 1)"
  [ -n "$src" ] || die "El archivo descargado no contiene el ejecutable del juego."

  step "Instalando en $install_dir..."
  mkdir -p "$install_dir"
  cp "$src" "$install_dir/.$binary_name.new"
  chmod 755 "$install_dir/.$binary_name.new"
  mv -f "$install_dir/.$binary_name.new" "$binary"
  printf '%s\n' "$version" > "$version_file"
  ok "Motorust ${version} instalado en $binary"

  rm -rf "$tmp_dir"
  tmp_dir=""
}

uninstall_game() {
  # Removes the game and its symlink.
  step "Desinstalando Motorust..."
  remove_link
  if is_installed; then
    rm -f "$binary"
    ok "Ejecutable eliminado: $binary"
  fi
  rm -f "$version_file"
  rmdir "$install_dir" 2>/dev/null || true
  ok "Motorust desinstalado."
}

# ---------------------------------------------------------------------------
# Menu
# ---------------------------------------------------------------------------
print_state() {
  local current target
  target="$(detect_target)"
  say ""
  say "${bold}${rust}MOTORUST${off} ${bold}v${version}${off}  ${dim}(${target})${off}"
  if is_installed; then
    current="$(installed_version)"
    if [ -n "$current" ]; then
      say "Ya está instalado: ${green}v${current}${off} en ${dim}$binary${off}"
    else
      say "Ya está instalado: ${green}$binary${off}"
    fi
  else
    say "No está instalado todavía: ${dim}$binary${off}"
  fi
}

print_installed_menu() {
  local current
  current="$(installed_version)"
  say ""
  if [ -n "$current" ] && version_gt "$version" "$current"; then
    say "  ${bold}1)${off} Actualizar a la versión ${bold}v${version}${off}"
  else
    say "  ${bold}1)${off} Reinstalar la versión ${bold}v${version}${off} ${dim}(ya estás al día)${off}"
  fi
  say "  ${bold}2)${off} Desinstalar"
  say "  ${bold}3)${off} Salir"
}

print_fresh_menu() {
  say ""
  say "  ${bold}1)${off} Instalar la versión ${bold}v${version}${off}"
  say "  ${bold}2)${off} Salir"
}

print_help() {
  cat <<EOF
Motorust ${version} - instalador para Linux y macOS.

Uso:
  curl -fsSL ${site_url}/setup.sh | bash
  wget -qO- ${site_url}/setup.sh | bash

Opciones:
  --install     instala (o actualiza) sin preguntar
  --update      solo actualiza
  --uninstall   borra el juego y su enlace
  --yes         igual que --install, para installations automáticas
  --dir RUTA    instala en otro sitio (por defecto, ~/motorust)
  --version     muestra la versión que instala este script
  --help        muestra esta ayuda
EOF
}

run_menu() {
  local choice
  while true; do
    print_state
    if is_installed; then
      print_installed_menu
    else
      print_fresh_menu
    fi
    printf '\n'
    choice="$(ask "Elige una opción: ")" || {
      say ""
      die "No hay terminal para preguntar. Usa --install, --update o --uninstall."
    }
    case "$choice" in
      1)
        install_game "$(detect_target)"
        create_link
        print_ready
        return 0
        ;;
      2)
        if is_installed; then
          uninstall_game
          return 0
        fi
        say "Hasta luego."
        return 0
        ;;
      3)
        say "Hasta luego."
        return 0
        ;;
      *)
        warn "Opción no válida."
        ;;
    esac
  done
}

print_ready() {
  say ""
  ok "${bold}Motorust v${version} está instalado.${off}"
  say ""
  say "  Arranca el juego desde el terminal con:"
  say ""
  say "    ${bold}${rust}motorust${off}"
  say ""
  info "El ejecutable está en $binary"
  info "Para desinstalarlo, ejecuta este script otra vez y elige «Desinstalar»."
}

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --install|-i|--yes|-y)
        action="install"
        ;;
      --update|-u)
        action="update"
        ;;
      --uninstall|--remove)
        action="uninstall"
        ;;
      --dir)
        shift
        [ $# -gt 0 ] || die "--dir necesita una ruta."
        install_dir="$1"
        binary="$install_dir/$binary_name"
        version_file="$install_dir/.motorust-version"
        ;;
      --version|-v)
        action="print-version"
        ;;
      --help|-h)
        action="help"
        ;;
      *)
        die "Opción desconocida: $1 (usa --help)"
        ;;
    esac
    shift
  done
}

main() {
  local current
  parse_args "$@"

  case "$action" in
    help)
      print_help
      exit 0
      ;;
    print-version)
      resolve_version
      say "$version"
      exit 0
      ;;
  esac

  [ -n "$home" ] || die "No sé cuál es tu carpeta de usuario (HOME no está definida)."

  resolve_version
  open_tty

  case "$action" in
    install)
      install_game "$(detect_target)"
      create_link
      print_ready
      ;;
    update)
      if is_installed; then
        current="$(installed_version)"
        if [ -n "$current" ] && ! version_gt "$version" "$current"; then
          say "Ya tienes la versión ${current}, que es la que publica el sitio."
          return 0
        fi
      else
        info "No está instalado: se instala."
      fi
      install_game "$(detect_target)"
      create_link
      print_ready
      ;;
    uninstall)
      uninstall_game
      ;;
    *)
      run_menu
      ;;
  esac
}

main "$@"
