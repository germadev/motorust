#!/usr/bin/env bash
# Prints the folder of the engine's sources as Cargo builds the game with
# them: its checkout of jgermade/motorust-engine at the tag in Cargo.toml,
# or the engine's clone beside this one when a `[patch]` points there (see
# docs/building.md, "The engine"). For what the engine has besides its
# code: its credits, which the packages carry (`assets/CREDITS.md`), and its
# tools (`tools/circuit_layout.py`).
#
#   tools/engine_dir.sh
#   cp "$(tools/engine_dir.sh)/assets/CREDITS.md" dist/CREDITS-ENGINE.md
#
# Needs cargo, and jq or Python 3 to read what it says. On Windows (Git
# Bash), the path is given in its Unix form (cygpath).
set -euo pipefail

cd "$(dirname "$0")/.."
metadata=$(cargo metadata --format-version 1)
if command -v jq > /dev/null 2>&1; then
  manifest=$(jq -r '.packages[] | select(.name == "motorust-engine") | .manifest_path' <<< "$metadata")
else
  python=$(command -v python3 || command -v python)
  manifest=$("$python" -c '
import json, sys
for package in json.load(sys.stdin)["packages"]:
    if package["name"] == "motorust-engine":
        print(package["manifest_path"])
' <<< "$metadata")
fi
if [ -z "$manifest" ]; then
  echo "Cargo has no motorust-engine among the game's dependencies" >&2
  exit 1
fi
if command -v cygpath > /dev/null 2>&1; then
  manifest=$(cygpath -u "$manifest")
fi
dirname "$manifest"
