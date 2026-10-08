#!/usr/bin/env bash
# Runs the demonstration scene against a real server, and regenerates `evidence/`.
#
#   ./run.sh              windowed, scripted, with a screenshot   (what a person watches)
#   ./run.sh evidence     headless: both flavours, each against a fresh world, the other seat, and
#                         two clients at once                      (what AC-13 and AC-15 read)
#   ./run.sh play         windowed, driven by the keyboard        (what a player does)
#   ./run.sh affordances  headless: the module's live check against worlds/market-town, saved to a
#                         temporary directory that is removed afterwards   (checks/affordances_check.gd)
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

# Starts a fresh world, appending what the server prints to `$1`. `$2` is the world (social-cafe
# unless given); `$3`, when given, a directory to save it in.
start_server() {
	local world="${2:-social-cafe}"
	local save=()
	if [ -n "${3:-}" ]; then
		save=(--save "$3")
	fi
	pkill -f "mineworld server worlds/$world" >/dev/null 2>&1
	sleep 1
	"$root/target/debug/mineworld" server "worlds/$world" --listen "$address" --agent alice \
		${save[@]+"${save[@]}"} >> "$1" 2>&1 &
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
affordances)
	# Market Town, because it offers complete affordances (buy, give) at genesis; saved, so that
	# frames carry a revision. The save is scratch and is removed whatever the outcome.
	scratch="$(mktemp -d)"
	: > "$here/evidence/server-affordances.log"
	start_server "$here/evidence/server-affordances.log" market-town "$scratch/save"
	godot --headless --path "$here" --script res://checks/affordances_check.gd -- \
		--address "$address" --seat visitor > "$here/evidence/affordances-market-town.log" 2>&1
	outcome=$?
	stop_server
	rm -rf "$scratch"
	grep '^\[check\]' "$here/evidence/affordances-market-town.log"
	grep -E 'SCRIPT ERROR|Parse Error' "$here/evidence/affordances-market-town.log" && outcome=1
	exit "$outcome"
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
