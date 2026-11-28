#!/bin/sh
# usage: shoot2d.sh <worktree> <seat> <shots-dir> <port>
W=$1; SEAT=$2; OUT=$3; PORT=$4
cd "$W" || exit 2
mkdir -p "$OUT"
./target/release/mineworld server ${WORLD:-worlds/lakeside} --packs entities --packs presentation/mineworld-default \
  --listen 127.0.0.1:$PORT --invite s34c2-shots > "$OUT/server.log" 2>&1 &
SP=$!
sleep 4
godot --path clients/2d -- --server=127.0.0.1:$PORT --seat=$SEAT --invite=s34c2-shots --nickname=shots \
  --world-key=${KEY:-lakeside} --drive=steps --steps=${STEPS:-/Users/yuema137/mineworld-worktrees/_scratch/s34c2/steps.json} --capture --shots="$OUT" > "$OUT/client.log" 2>&1
echo "client exit $?"
kill $SP
grep -E "PLACE|EVIDENCE|drive complete" "$OUT/client.log"
