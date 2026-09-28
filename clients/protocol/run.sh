#!/usr/bin/env bash
# Runs the demonstration scene against a real server, and regenerates `evidence/`.
#
#   ./run.sh              windowed, scripted, with a screenshot   (what a person watches)
#   ./run.sh evidence     headless, both flavours, both fixtures  (what AC-13 reads)
#   ./run.sh play         windowed, driven by the keyboard        (what a player does)
#
# It starts `mineworld server worlds/social-cafe --agent alice` itself, on 127.0.0.1:7878, and stops
# it afterwards. A clean checkout works: the project's script class cache is built here, not
# committed (`docs/ACCEPTANCE.md` §4.1).
set -u
export PATH="$HOME/.cargo/bin:$PATH"

here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
mode="${1:-window}"
address="127.0.0.1:7878"

cd "$root" || exit 1
cargo build --quiet -p mineworld-cli || exit 1

pkill -f 'mineworld server worlds/social-cafe' >/dev/null 2>&1
sleep 1
"$root/target/debug/mineworld" server worlds/social-cafe --listen "$address" --agent alice \
	> "$here/evidence/server.log" 2>&1 &
server=$!
sleep 2

# Godot registers `class_name` in a cache it builds when it imports a project, so a first headless
# run without this cannot find `MineWorldClient`.
godot --headless --path "$here" --import >/dev/null 2>&1

case "$mode" in
evidence)
	for flavour in 2d 3d; do
		godot --headless --path "$here" --quit-after 1200 -- \
			--autopilot --flavour "$flavour" --seat visitor --address "$address" \
			--requests "evidence/request-$flavour.json" \
			> "$here/evidence/transcript-$flavour.log" 2>&1
		echo "--- $flavour ---"
		grep '^\[demo\]' "$here/evidence/transcript-$flavour.log" | tail -12
	done
	;;
play)
	godot --path "$here" -- --seat visitor --address "$address"
	;;
*)
	godot --path "$here" --quit-after 1500 -- \
		--autopilot --flavour 3d --seat wanderer --address "$address" \
		--screenshot "evidence/demo-scene.png" \
		> "$here/evidence/transcript-window.log" 2>&1
	grep '^\[demo\]' "$here/evidence/transcript-window.log" | tail -12
	;;
esac

kill "$server" >/dev/null 2>&1
wait "$server" 2>/dev/null
