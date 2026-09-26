#!/usr/bin/env bash
# Runs the spike server and one Godot client to completion, and collects the evidence.
#
#   ./run.sh 2d      the 2D client
#   ./run.sh 3d      the 3D client
#
# The server is server-authoritative and resets the player to the door on every connection, so
# the two runs start from the same world state and their intents are comparable.
set -u
export PATH="$HOME/.cargo/bin:$PATH"

here="$(cd "$(dirname "$0")" && pwd)"
which="${1:-2d}"
client="$here/client-$which"
logs="$here/evidence"
mkdir -p "$logs"

pkill -f mineworld-spike-server >/dev/null 2>&1
sleep 1

( cd "$here/server" && cargo run --quiet ) > "$logs/server-$which.log" 2>&1 &
server=$!
sleep 3

rm -f "$client"/*.png
( cd "$client" && godot --path . ) > "$logs/client-$which.log" 2>&1 &
godot=$!

for _ in $(seq 1 60); do
	kill -0 "$godot" 2>/dev/null || break
	sleep 1
done
kill -9 "$godot" >/dev/null 2>&1
sleep 1
kill -9 "$server" >/dev/null 2>&1
pkill -f mineworld-spike-server >/dev/null 2>&1

for shot in "$client"/*.png; do
	[ -e "$shot" ] && mv "$shot" "$logs/"
done
echo "--- client $which ---"
grep -v '^\[2d\] DD-15\|^\[3d\] DD-15' "$logs/client-$which.log" | tail -30
echo "--- server ---"
grep '^\[server\]' "$logs/server-$which.log" | tail -20
echo "--- captured ---"
ls "$logs"/*.png 2>/dev/null
