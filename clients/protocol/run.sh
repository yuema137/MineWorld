#!/usr/bin/env bash
# Runs the demonstration scene against a real server, and regenerates `evidence/`.
#
#   ./run.sh              windowed, scripted, with a screenshot   (what a person watches)
#   ./run.sh evidence     headless: both flavours, each against a fresh world, the other seat, and
#                         two clients at once                      (what AC-13 and AC-15 read)
#   ./run.sh play         windowed, driven by the keyboard        (what a player does)
#
# It starts `mineworld server worlds/social-cafe --agent alice` itself, on 127.0.0.1:7878, and stops
# it afterwards. A clean checkout works: the project's script class cache is built here, not
# committed (`docs/ACCEPTANCE.md` §4.1).
#
# Each flavour's AC-13 run gets a world of its own, because a client now walks from where the world
# seated it, in `move` strides the server accepts one at a time: a second run on the same world would
# start where the first one stopped, and its frames could not be replayed against a fresh world.
set -u
export PATH="$HOME/.cargo/bin:$PATH"

here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
mode="${1:-window}"
address="127.0.0.1:7878"
server=""

cd "$root" || exit 1
cargo build --quiet -p mineworld-cli || exit 1

# Starts a fresh world, appending what the server prints to `$1`.
start_server() {
	pkill -f 'mineworld server worlds/social-cafe' >/dev/null 2>&1
	sleep 1
	"$root/target/debug/mineworld" server worlds/social-cafe --listen "$address" --agent alice \
		>> "$1" 2>&1 &
	server=$!
	sleep 2
}

stop_server() {
	kill "$server" >/dev/null 2>&1
	wait "$server" 2>/dev/null
}

# One headless scripted client: flavour, seat, transcript, and optionally where to write its requests.
client() {
	local requests=()
	if [ -n "${4:-}" ]; then
		requests=(--requests "$4")
	fi
	godot --headless --path "$here" --quit-after 1200 -- \
		--autopilot --flavour "$1" --seat "$2" --address "$address" ${requests[@]+"${requests[@]}"} \
		> "$here/evidence/$3" 2>&1
}

# Godot registers `class_name` in a cache it builds when it imports a project, so a first headless
# run without this cannot find `MineWorldClient`.
godot --headless --path "$here" --import >/dev/null 2>&1

case "$mode" in
evidence)
	: > "$here/evidence/server.log"
	# AC-13: the same seat, the same words, a fresh world each.
	start_server "$here/evidence/server.log"
	client 2d visitor transcript-2d.log evidence/request-2d.json
	stop_server
	start_server "$here/evidence/server.log"
	client 3d visitor transcript-3d.log evidence/request-3d.json
	# AC-15 from inside a client: the OTHER seat, on the world the 3D run just spoke in.
	client 3d wanderer transcript-3d-wanderer.log
	stop_server
	# Two clients at once, with the agent: three participants in one world.
	: > "$here/evidence/server-simultaneous.log"
	start_server "$here/evidence/server-simultaneous.log"
	client 2d visitor simultaneous-2d.log &
	first=$!
	client 3d wanderer simultaneous-3d.log
	wait "$first"
	stop_server
	for transcript in transcript-2d transcript-3d transcript-3d-wanderer simultaneous-2d simultaneous-3d; do
		echo "--- $transcript ---"
		grep '^\[demo\]' "$here/evidence/$transcript.log" | tail -12
	done
	;;
play)
	start_server "$here/evidence/server.log"
	godot --path "$here" -- --seat visitor --address "$address"
	stop_server
	;;
*)
	: > "$here/evidence/server-window.log"
	start_server "$here/evidence/server-window.log"
	godot --path "$here" --quit-after 1500 -- \
		--autopilot --flavour 3d --seat wanderer --address "$address" \
		--screenshot "evidence/demo-scene.png" \
		> "$here/evidence/transcript-window.log" 2>&1
	stop_server
	grep '^\[demo\]' "$here/evidence/transcript-window.log" | tail -12
	;;
esac
