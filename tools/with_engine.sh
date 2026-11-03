#!/usr/bin/env bash
# Runs a cargo command with the engine at another version than Cargo.toml's
# tag: a branch, a tag or a commit of jgermade/motorust-engine, or a folder
# with its sources (a clone of it). Used by the Makefile's ENGINE:
#
#   make up ENGINE=my-branch          # the engine's branch, fetched afresh
#   make test ENGINE=v0.0.20          # a tag (or a commit)
#   make up ENGINE=../motorust-engine # a clone of it, as it is
#
#   tools/with_engine.sh my-branch cargo run --features dynamic
#
# A branch, tag or commit is checked out in .tmp/engine/<name> (cloned the
# first time, fetched again each time after); the command is then run with
# a [patch] of the engine pointing there, given to cargo with --config, so
# nothing is written to .cargo/config.toml (see docs/building.md, "The
# engine"). Cargo.lock, which the patch changes, is put back as it was
# afterwards. With ENGINE empty, the command is run as it is.
#
# Needs git and cargo.
set -euo pipefail

URL="https://github.com/jgermade/motorust-engine"
engine="${1:-}"
shift
if [ -z "$engine" ]; then
  exec "$@"
fi

cd "$(dirname "$0")/.."
if [ -d "$engine" ]; then
  folder="$(cd "$engine" && pwd)"
  echo "The engine from $folder ($(git -C "$folder" rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'not a clone'))" >&2
else
  folder="$PWD/.tmp/engine/$(printf '%s' "$engine" | tr -c 'A-Za-z0-9._-' '_')"
  if [ ! -d "$folder/.git" ]; then
    mkdir -p "$(dirname "$folder")"
    git clone --quiet "$URL" "$folder"
  fi
  git -C "$folder" fetch --quiet --tags origin
  # A branch at its latest; else a tag or a commit.
  if git -C "$folder" rev-parse --verify --quiet "origin/$engine" > /dev/null; then
    git -C "$folder" checkout --quiet --detach "origin/$engine"
  else
    git -C "$folder" checkout --quiet --detach "$engine"
  fi
  echo "The engine at $engine ($(git -C "$folder" rev-parse --short HEAD))" >&2
fi
[ -f "$folder/Cargo.toml" ] || { echo "$folder: not the engine's sources" >&2; exit 1; }

# Cargo.lock as it was, put back however the command ends.
lock="$(mktemp)"
cp Cargo.lock "$lock"
restore() {
  if [ -f "$lock" ]; then
    cp "$lock" Cargo.lock
    rm -f "$lock"
  fi
}
trap restore EXIT
trap 'restore; exit 130' INT TERM

cargo_command="$1"
shift
"$cargo_command" --config "patch.\"$URL\".motorust-engine.path=\"$folder\"" "$@"
