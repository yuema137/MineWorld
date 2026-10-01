#!/usr/bin/env bash
# Fetch the CC0 Poly Haven assets the VIS-3D-GODOT-2 slice adds on top of
# tools/fetch_assets.sh. Everything here is CC0 1.0 -- see ../ASSETS.md.
# Re-running is idempotent; a file already on disk is left alone.
#
# Kept as its own script rather than folded into fetch_assets.sh so that the
# slice's asset set stays identifiable and so this branch does not edit a file
# the concurrent character branch also touches.
#
# NOT fetched, deliberately: Poly Haven's trees. At the 1k tier tree_small_02
# is 101 MB, island_tree_01 is 66 MB and jacaranda_tree is 215 MB, because the
# foliage is real geometry with its own atlases. One street tree would be twice
# the whole rest of this list. The slice's trees are generated instead
# (scripts/slice/foliage.gd), which is also what lets their canopy density,
# leaf size and colour be tuned to the references.
#
# EXCLUDED on licence, after inspecting their texture atlases as pixels -- the
# test is whether we may RELICENSE (DEP-8 as amended), and an asset's CC0 can
# only cover what its author owned. See ../ASSETS.md "Excluded".
#   CashRegister_01           the atlas reproduces a Bank of Canada $1 note
#   hanging_picture_frame_02  the painting is signed "Celine F", who is not among
#                             the asset's credited authors
#
# Poly Haven's API terms (2.4) ask every API call to carry an identifying
# User-Agent; $UA is it.
set -euo pipefail
UA="MineWorld-fetch_slice_assets (https://github.com/yuema137/MineWorld)"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tex="$here/../assets/textures"
models="$here/../assets/models"
mkdir -p "$tex" "$models"

RES=1k
MAPS=(diff nor_gl arm)

# --- textures -----------------------------------------------------------------
# Chosen against the references as images, at magnification, not from the slug.
# Four were fetched, inspected as pixels and rejected: granite_tile_02 is brown
# floor tile and not a grey kerb stone; white_plaster_rough_02 is mossy green,
# not white; yellow_plaster_02 is olive with dark joint bands; grey_cartago_02
# is a herringbone marble. Recorded so the next contributor does not re-fetch
# them on the strength of the slug.
SLUGS=(
  rectangular_paving      # pavement setts, both frontages  (05, 03)
  concrete                # kerb, entrance step, plinths, bollard bases
  worn_asphalt            # carriageway
  sandstone_blocks_05     # the cafe's coursed masonry above the shopfront (03)
  red_bricks_04           # the bookshop's brick                            (05)
  yellow_bricks           # the apartment block's buff brick                (05)
  clay_roof_tiles_02      # terracotta pantiles                             (02)
  wood_floor_worn         # cafe interior floor
  brown_planks_09         # cafe interior ceiling boards                    (03)
  wood_table_worn         # counters, shelving, table tops
  long_white_tiles        # the back-bar tiling behind the counter
  painted_plaster_wall    # interior plaster
)

failed=()
for s in "${SLUGS[@]}"; do
  mkdir -p "$tex/$s"
  for m in "${MAPS[@]}"; do
    f="$tex/$s/${s}_${m}_${RES}.jpg"
    [ -s "$f" ] && continue
    url="https://dl.polyhaven.org/file/ph-assets/Textures/jpg/$RES/$s/${s}_${m}_${RES}.jpg"
    if ! curl -fsSL -A "$UA" --retry 3 --max-time 120 -o "$f" "$url"; then
      rm -f "$f"; failed+=("$s/$m")
    fi
  done
  echo "texture $s"
done

# --- models -------------------------------------------------------------------
MODELS=(
  # the cafe interior: room-scale
  wooden_bookshelf_worn wooden_display_shelves_01 steel_frame_shelves_02 Shelf_01
  # furniture-scale. Chosen by measuring the glTF rather than by the slug:
  # WoodenChair_01 is 2.27 m tall, WoodenTable_02 is a 0.30 m side table and
  # round_wooden_table_01 stands 1.00 m, none of which is cafe furniture.
  # round_wooden_table_02 is 0.80 x 0.75 m and dining_chair_02 is 0.97 m tall.
  round_wooden_table_02 dining_chair_02 painted_wooden_chair_01
  bar_chair_round_01 wooden_crate_01 wicker_basket_01
  # light fittings, as visible objects
  hanging_industrial_lamp modern_ceiling_lamp_01 industrial_pipe_lamp
  # hand-scale: the size class an interior most often lacks
  croissant carrot_cake tea_set_01 wine_bottles_01 jug_01
  wooden_bowl_01 food_apple_01 ceramic_vase_02 antique_ceramic_vase_01
  brass_pot_01 pot_enamel_01
  # decoration
  fancy_picture_frame_01 wall_clock calathea_orbifolia_01
  # the street
  standing_chalkboard_01 planter_box_01 planter_pot_clay shrub_02 shrub_03
  water_manhole_cover
)

for s in "${MODELS[@]}"; do
  [ -s "$models/$s/$s.gltf" ] && continue
  urls=$(curl -fsSL -A "$UA" --retry 3 --max-time 60 "https://api.polyhaven.com/files/$s" | python3 -c "
import json,sys
d=json.load(sys.stdin)['gltf']['1k']['gltf']
# Dependencies first, the .gltf last: its presence is what marks a model
# complete, so an interrupted download is retried rather than skipped.
for rel,v in d.get('include',{}).items():
    print('%s\t%s' % (rel, v['url']))
print('%s.gltf\t%s' % (sys.argv[1], d['url']))
" "$s") || { failed+=("$s"); continue; }
  while IFS=$'\t' read -r rel url; do
    [ -z "$rel" ] && continue
    mkdir -p "$models/$s/$(dirname "$rel")"
    if ! curl -fsSL -A "$UA" --retry 3 --max-time 300 -o "$models/$s/$rel" "$url"; then
      rm -f "$models/$s/$rel"; failed+=("$s/$rel"); break
    fi
  done <<< "$urls"
  echo "model $s"
done

if [ ${#failed[@]} -gt 0 ]; then
  echo "FAILED -- re-run to retry: ${failed[*]}" >&2
  exit 1
fi
echo "slice assets complete."
