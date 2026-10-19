#!/usr/bin/env bash
# Cross-client identity check: the 2D and the 3D client see the same Person, name and look for every
# seat, and each draws a person from its disclosed look only (docs/DECISIONS.md ARC-91, ARC-92;
# .structured-coding/plans/mvp0/pr-vf-person-identity.md).
#
#   bash clients/shared/checks/identity.sh [world ...]      default: social-cafe market-town
#
# For each world: start a fresh `mineworld server worlds/<world>` (no agent), seat every seat it
# lists with the 2D client's check, stop it; start a fresh one and do the same with the 3D client's
# check; then require the two clients' IDENTITY lines to be identical and non-empty. Each client's
# check also fails by itself when its resolver draws a person by anything but the disclosed look.
# Exit 0 only if every world passes.
set -u
export PATH="$HOME/.cargo/bin:$PATH"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../../.." && pwd)"
worlds=("$@")
[ ${#worlds[@]} -eq 0 ] && worlds=(social-cafe market-town)

cd "$root" || exit 1
cargo build --quiet -p mineworld-cli || exit 1
godot --headless --path clients/2d --import >/dev/null 2>&1
godot --headless --path clients/3d-spike --import >/dev/null 2>&1

scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT
server=""

# Starts a fresh world and sets `address`, `invite` and `seats` from what it prints.
start_server() {
	"$root/target/debug/mineworld" server "worlds/$1" --listen 127.0.0.1:0 > "$scratch/server.log" 2>&1 &
	server=$!
	local line=""
	for _ in $(seq 1 150); do
		line="$(grep -m1 '^\[mineworld\] invite ' "$scratch/server.log")"
		[ -n "$line" ] && break
		kill -0 "$server" 2>/dev/null || break
		sleep 0.2
	done
	[ -n "$line" ] || { echo "no join line from worlds/$1" >&2; cat "$scratch/server.log" >&2; exit 1; }
	invite="$(printf '%s\n' "$line" | sed -n 's/^\[mineworld\] invite \([^ ]*\) .*/\1/p')"
	address="$(printf '%s\n' "$line" | sed -n 's/.* join with: \([^ ]*\) .*/\1/p')"
	seats="$(grep -m1 ' seats: ' "$scratch/server.log" | sed -n 's/.* seats: //p' | tr -d ' ')"
}

stop_server() {
	kill "$server" >/dev/null 2>&1
	wait "$server" 2>/dev/null
}

failed=0
for world in "${worlds[@]}"; do
	for side in 2d 3d; do
		start_server "$world"
		if [ "$side" = 2d ]; then
			project=clients/2d; script=res://scripts/harness/identity_check.gd
		else
			project=clients/3d-spike; script=res://scripts/slice/identity_check.gd
		fi
		godot --headless --path "$project" --script "$script" -- \
			--server="$address" --invite="$invite" --seats="$seats" > "$scratch/$side.out" 2> "$scratch/$side.err"
		status=$?
		stop_server
		grep '^IDENTITY ' "$scratch/$side.out" > "$scratch/$world-$side.lines"
		if [ $status -ne 0 ]; then
			echo "FAIL $world $side: its own check failed" >&2
			grep '^FAIL' "$scratch/$side.err" >&2
			failed=1
		fi
	done
	n="$(wc -l < "$scratch/$world-2d.lines" | tr -d ' ')"
	if [ "$n" -eq 0 ]; then
		echo "FAIL $world: no identity lines" >&2
		failed=1
	elif ! diff "$scratch/$world-2d.lines" "$scratch/$world-3d.lines" >&2; then
		echo "FAIL $world: the 2D and 3D clients disagree about who someone is (diff above)" >&2
		failed=1
	else
		echo "PASS $world: $n identity lines agree across 2D and 3D (seats: $seats)"
	fi
done
exit $failed
