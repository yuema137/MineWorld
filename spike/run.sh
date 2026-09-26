#!/usr/bin/env bash
# Runs the spike server and one Godot client to completion, and collects the evidence.
#
#   ./run.sh 2d      the 2D client
#   ./run.sh 3d      the 3D client
#   ./run.sh both    both clients in turn against ONE server, which is what produces the
#                    AC-13 parity verdict: the server holds both submitted intents and
#                    compares them.
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

run_client() {
	local tag="$1"
	local project="$here/client-$tag"
	rm -f "$project"/*.png
	( cd "$project" && godot --path . ) > "$logs/client-$tag.log" 2>&1 &
	local pid=$!
	for _ in $(seq 1 90); do
		kill -0 "$pid" 2>/dev/null || break
		sleep 1
	done
	kill -9 "$pid" >/dev/null 2>&1
	sleep 1
	for shot in "$project"/*.png; do
		[ -e "$shot" ] && mv "$shot" "$logs/"
	done
	echo "--- client $tag ---"
	grep -v 'DD-15' "$logs/client-$tag.log" | tail -24
}

pkill -f mineworld-spike-server >/dev/null 2>&1
sleep 1

if [ "$which" = "both" ]; then
	rm -f "$logs/intents.jsonl" "$logs/parity.json"
fi

( cd "$here/server" && cargo run --quiet ) > "$logs/server-$which.log" 2>&1 &
server=$!
sleep 3

if [ "$which" = "both" ]; then
	run_client 2d
	sleep 2
	run_client 3d
else
	run_client "$which"
fi

kill -9 "$server" >/dev/null 2>&1
pkill -f mineworld-spike-server >/dev/null 2>&1

echo "--- server ---"
grep '^\[server\]' "$logs/server-$which.log" | grep -v 'move-to' | tail -20
echo "--- captured ---"
ls "$logs"/*.png 2>/dev/null
