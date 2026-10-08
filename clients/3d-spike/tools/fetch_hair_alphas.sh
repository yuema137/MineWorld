#!/usr/bin/env bash
# Fetch OwlishMedia's "Hair Alphas For Days" -- 85 hair-strand alpha maps at
# 2048 px -- for the default character's hair cards.
#
#   clients/3d-spike/tools/fetch_hair_alphas.sh <dest dir outside the repo>
#
# Licence, read 2026-10-06 at https://opengameart.org/content/hair-alphas-for-days:
# "CC0", author OwlishMedia; the author adds in the comments "Everything on
# this site is public domain so go wild." CC0 places it under MIT without a
# sublicence, so it passes DEP-8's relicensing test. It is a per-asset source
# (OpenGameArt mixes licences), so the record goes in the pack's LICENSES/
# with the date and the URL above when any map is committed.
#
# The archive is 138.5 MB and is NOT committed: it goes to scratch, and only
# the maps chosen for the atlas, downscaled by the atlas build, enter the
# repository. Until it is fetched, `hair_atlas.py` (our own procedural strands)
# is the atlas, and the card UV layout does not depend on which one is used.
set -euo pipefail
dest="${1:?usage: fetch_hair_alphas.sh <dest dir>}"
mkdir -p "$dest"
zip="$dest/OwlishMedia_HairGalore.zip"
if [ ! -s "$zip" ]; then
  curl -fSL --max-time 900 -o "$zip" \
    "https://opengameart.org/sites/default/files/OwlishMedia_HairGalore.zip"
fi
unzip -o -q "$zip" -d "$dest/HairGalore"
echo "fetched $(find "$dest/HairGalore" -type f | wc -l | tr -d ' ') files into $dest/HairGalore"
