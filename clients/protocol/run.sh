#!/usr/bin/env bash
# Runs the demonstration scene against a real server, and regenerates `evidence/`.
#
#   ./run.sh              windowed, scripted, with a screenshot   (what a person watches)
#   ./run.sh evidence     headless: both flavours, each against a fresh world, the other seat, and
#                         two clients at once                      (what AC-13 and AC-15 read)
#   ./run.sh play         windowed, driven by the keyboard        (what a player does)
#   ./run.sh affordances  headless: the module's live check against worlds/market-town, saved to a
#                         temporary directory that is removed afterwards   (checks/affordances_check.gd)
#   ./run.sh reconnect    headless: the module's opt-in reconnect against worlds/market-town with
#                         --town --hold 10: drop, resume, held             (checks/reconnect_check.gd)
#   ./run.sh admin        headless: the clock frame, pause, resume and kick through the admin surface,
#                         with a generated MINEWORLD_ADMIN_TOKEN           (checks/admin_check.gd)
#
# It starts `mineworld server worlds/social-cafe --agent alice` itself, on a port of 127.0.0.1 the
# operating system chooses, and stops it afterwards — only the server it started, by its own PID, so
# runs in other worktrees on the same machine are never touched. The server is given no invite, so it
# generates one and prints its join line (`server/PROTOCOL.md` §4.1); this script reads the address
# and the invite from that line and passes both to every client. The line is kept out of the
# committed server logs, so no invite — not even a dead one — enters the repository. A clean checkout
# works: the project's script class cache is built here, not committed (`docs/ACCEPTANCE.md` §4.1).
#
# Each flavour's AC-13 run gets a world of its own, because a client now walks from where the world
# seated it, in `move` strides the server accepts one at a time: a second run on the same world would
# start where the first one stopped, and its frames could not be replayed against a fresh world.
set -u
export PATH="$HOME/.cargo/bin:$PATH"

here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
mode="${1:-window}"
address=""
invite=""
server=""
server_log=""
scratch=""
# Further server flags for one mode (reconnect's --town --hold 10); none otherwise.
server_extra=()

cd "$root" || exit 1
cargo build --quiet -p mineworld-cli || exit 1

# Starts a fresh world, whose output goes to `$1` once it stops. `$2` is the world (social-cafe unless
# given); `$3`, when given, a directory to save it in. Sets `address` and `invite` from the join line.
start_server() {
	local world="${2:-social-cafe}"
	local save=()
	if [ -n "${3:-}" ]; then
		save=(--save "$3")
	fi
	server_log="$1"
	scratch="$(mktemp)"
	"$root/target/debug/mineworld" server "worlds/$world" --listen 127.0.0.1:0 --agent alice \
		${save[@]+"${save[@]}"} ${server_extra[@]+"${server_extra[@]}"} > "$scratch" 2>&1 &
	server=$!
	local line=""
	for _ in $(seq 1 150); do
		line="$(grep -m1 '^\[mineworld\] invite ' "$scratch")"
		[ -n "$line" ] && break
		kill -0 "$server" 2>/dev/null || break
		sleep 0.2
	done
	if [ -z "$line" ]; then
		echo "the server printed no join line:" >&2
		cat "$scratch" >&2
		stop_server
		exit 1
	fi
	invite="$(printf '%s\n' "$line" | sed -n 's/^\[mineworld\] invite \([^ ]*\) .*/\1/p')"
	address="$(printf '%s\n' "$line" | sed -n 's/.* join with: \([^ ]*\) .*/\1/p')"
}

# Stops the server this script started, and appends what it printed — minus the invite line — to
# its log.
stop_server() {
	kill "$server" >/dev/null 2>&1
	wait "$server" 2>/dev/null
	grep -v '^\[mineworld\] invite ' "$scratch" >> "$server_log"
	rm -f "$scratch"
}

# One headless scripted client: flavour, seat, transcript, and optionally where to write its requests.
client() {
	local requests=()
	if [ -n "${4:-}" ]; then
		requests=(--requests "$4")
	fi
	godot --headless --path "$here" --quit-after 1200 -- \
		--autopilot --flavour "$1" --seat "$2" --address "$address" --invite "$invite" \
		--nickname "demo-$1-$2" ${requests[@]+"${requests[@]}"} > "$here/evidence/$3" 2>&1
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
	save_dir="$(mktemp -d)"
	: > "$here/evidence/server-affordances.log"
	start_server "$here/evidence/server-affordances.log" market-town "$save_dir/save"
	godot --headless --path "$here" --script res://checks/affordances_check.gd -- \
		--address "$address" --seat visitor --invite "$invite" \
		> "$here/evidence/affordances-market-town.log" 2>&1
	outcome=$?
	stop_server
	rm -rf "$save_dir"
	grep '^\[check\]' "$here/evidence/affordances-market-town.log"
	grep -E 'SCRIPT ERROR|Parse Error' "$here/evidence/affordances-market-town.log" && outcome=1
	exit "$outcome"
	;;
reconnect)
	# Market Town hosted as a town (every seat nobody plays is driven in-server), with a 10 s hold.
	server_extra=(--town --hold 10)
	: > "$here/evidence/server-reconnect.log"
	start_server "$here/evidence/server-reconnect.log" market-town
	godot --headless --path "$here" --script res://checks/reconnect_check.gd -- \
		--address "$address" --seat visitor --invite "$invite" \
		> "$here/evidence/reconnect-market-town.log" 2>&1
	outcome=$?
	stop_server
	grep '^\[check\]' "$here/evidence/reconnect-market-town.log"
	grep -E 'SCRIPT ERROR|Parse Error' "$here/evidence/reconnect-market-town.log" && outcome=1
	exit "$outcome"
	;;
admin)
	# The admin surface and the clock frame from the far side (step-12 DA-10). A fresh admin token,
	# made here and handed to the server and the check through the environment — the launcher path —
	# so it is in no argument list, no log and no evidence file.
	MINEWORLD_ADMIN_TOKEN="run-sh-admin-$$-$RANDOM$RANDOM"
	export MINEWORLD_ADMIN_TOKEN
	: > "$here/evidence/server-admin.log"
	start_server "$here/evidence/server-admin.log"
	godot --headless --path "$here" --script res://checks/admin_check.gd -- \
		--address "$address" --seat visitor --invite "$invite" \
		> "$here/evidence/admin-social-cafe.log" 2>&1
	outcome=$?
	stop_server
	grep '^\[check\]' "$here/evidence/admin-social-cafe.log"
	grep -E 'SCRIPT ERROR|Parse Error' "$here/evidence/admin-social-cafe.log" && outcome=1
	grep -l -- "$MINEWORLD_ADMIN_TOKEN" "$here/evidence/server-admin.log" "$here/evidence/admin-social-cafe.log" \
		&& { echo "the admin token reached an evidence file" >&2; outcome=1; }
	exit "$outcome"
	;;
play)
	start_server "$here/evidence/server.log"
	godot --path "$here" -- --seat visitor --address "$address" --invite "$invite"
	stop_server
	;;
*)
	: > "$here/evidence/server-window.log"
	start_server "$here/evidence/server-window.log"
	godot --path "$here" --quit-after 1500 -- \
		--autopilot --flavour 3d --seat wanderer --address "$address" --invite "$invite" \
		--screenshot "evidence/demo-scene.png" \
		> "$here/evidence/transcript-window.log" 2>&1
	stop_server
	grep '^\[demo\]' "$here/evidence/transcript-window.log" | tail -12
	;;
esac
