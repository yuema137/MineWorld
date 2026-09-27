#!/usr/bin/env bash
# Fetch the CC0 Poly Haven assets this spike uses. Everything here is CC0 1.0
# (public domain dedication) -- see ../ASSETS.md. Re-running is idempotent.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tex="$here/../assets/textures"
hdri="$here/../assets/hdri"
mkdir -p "$tex" "$hdri"

RES=1k
MAPS=(diff nor_gl arm)
SLUGS=(
  concrete_pavement       # promenade and plaza pavers
  cobblestone_floor_08    # side-street setts
  pavement_02             # concrete sidewalk
  red_brick_03            # brick shopfronts
  beige_wall_001          # stucco / plaster
  weathered_brown_planks  # jetty, decking, bench slats
  asphalt_01              # roadway
  roof_slates_03          # roofs
  leafy_grass             # park lawn
  stone_brick_wall_001    # planter walls, kerbs, plinths
)

for s in "${SLUGS[@]}"; do
  mkdir -p "$tex/$s"
  for m in "${MAPS[@]}"; do
    f="$tex/$s/${s}_${m}_${RES}.jpg"
    [ -s "$f" ] && continue
    url="https://dl.polyhaven.org/file/ph-assets/Textures/jpg/$RES/$s/${s}_${m}_${RES}.jpg"
    echo "fetch $s/$m"
    curl -fsSL --max-time 120 -o "$f" "$url" || { echo "  (no $m map for $s)"; rm -f "$f"; }
  done
done

for h in kloofendal_48d_partly_cloudy_puresky; do
  f="$hdri/${h}_2k.hdr"
  [ -s "$f" ] && continue
  echo "fetch hdri $h"
  curl -fsSL --max-time 180 -o "$f" \
    "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/2k/${h}_2k.hdr"
done

echo "done."

# --- CC0 Poly Haven models -----------------------------------------------------
# 1k tier, not 2k: these are all small props, and 1k over a 1.8 m bench is
# ~570 px/m, already past the ~512 px/m target for a walkable street. It halves
# the repo.
models="$here/../assets/models"
mkdir -p "$models"
MODELS=(
  painted_wooden_bench         # promenade and plaza benches
  outdoor_table_chair_set_01   # cafe pavement seating
  street_lamp_01               # promenade lamps
  street_lamp_02               # side street lamps
  potted_plant_02              # shopfront planting
  celandine_01                 # flowering ground cover
  grass_medium_01              # verge and trail grass
)
for s in "${MODELS[@]}"; do
  [ -s "$models/$s/$s.gltf" ] && continue
  echo "fetch model $s"
  urls=$(curl -fsSL --max-time 60 "https://api.polyhaven.com/files/$s" | python3 -c "
import json,sys
d=json.load(sys.stdin)['gltf']['1k']['gltf']
print('%s.gltf\t%s' % (sys.argv[1], d['url']))
for rel,v in d.get('include',{}).items():
    print('%s\t%s' % (rel, v['url']))
" "$s")
  while IFS=$'\t' read -r rel url; do
    [ -z "$rel" ] && continue
    mkdir -p "$models/$s/$(dirname "$rel")"
    curl -fsSL --max-time 300 -o "$models/$s/$rel" "$url"
  done <<< "$urls"
  echo "    $s ok"
done

echo "assets complete."
