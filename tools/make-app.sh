#!/bin/sh
# Builds Motorust.app around an executable: the bundle macOS launches from
# Finder or the Dock, and that it knows for a game (its Game Mode).
#
#   tools/make-app.sh <executable> <version> <folder>
#
# Leaves <folder>/Motorust.app, replacing one already there. The game's
# assets are inside the executable; the bundle adds the icon and Info.plist.
set -eu

if [ $# -ne 3 ]; then
	echo "usage: $0 <executable> <version> <folder>" >&2
	exit 2
fi
executable=$1
version=$2
app="$3/Motorust.app"
assets="$(dirname "$0")/../assets"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$executable" "$app/Contents/MacOS/motorust"
chmod +x "$app/Contents/MacOS/motorust"
if [ -f "$assets/icon.icns" ]; then
	cp "$assets/icon.icns" "$app/Contents/Resources/AppIcon.icns"
fi
sed "s/__VERSION__/$version/g" "$assets/Info.plist" > "$app/Contents/Info.plist"
echo "$app"
