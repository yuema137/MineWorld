#!/usr/bin/env python3
"""Per-asset provenance check for the VIS-3D-GODOT-2 slice assets.

Run from anywhere:  python3 clients/3d-spike/tools/check_slice_provenance.py

For every slug fetch_slice_assets.sh names, this checks -- individually, not by
assuming the batch -- that:

  1. the slug resolves in Poly Haven's own catalogue (api.polyhaven.com/info)
     and is the asset type the script fetches it as;
  2. its credited authors are printed, so a reader can see who the CC0
     dedication comes from;
  3. for a model, every buffer and image the .gltf references is on disk, none
     is an external URL, nothing unreferenced sits beside it, and the .gltf's own
     `asset.copyright` field is absent or printed;
  4. for a texture, all three maps are present and non-empty.

What it CANNOT check, and why the ASSETS.md record also needs eyes: whether a
texture atlas embeds someone else's work -- a signed painting, a banknote, a
logo. Poly Haven's CC0 can only cover what its credited authors owned. The two
exclusions recorded in ASSETS.md were found by looking at the atlases, not by
this script; every atlas containing imagery or lettering is listed in ASSETS.md
with what was seen in it.

Exit status is non-zero if any check fails. Network: api.polyhaven.com only,
with the identifying User-Agent Poly Haven's API terms (2.4) ask for.
"""
import glob
import json
import os
import re
import sys
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
UA = "MineWorld-check_slice_provenance (https://github.com/yuema137/MineWorld)"
TYPES = {0: "hdri", 1: "texture", 2: "model"}


def slugs():
    sh = open(os.path.join(HERE, "fetch_slice_assets.sh")).read()
    tex_block = sh.split("SLUGS=(")[1].split("\n)")[0]
    tex = re.findall(r"^\s+([A-Za-z0-9_]+)\s", tex_block + "\n", re.M)
    mod_block = sh.split("MODELS=(")[1].split("\n)")[0]
    mod = " ".join(l.split("#")[0] for l in mod_block.splitlines()).split()
    return tex, mod


def info(slug):
    req = urllib.request.Request("https://api.polyhaven.com/info/" + slug,
                                 headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r)


def check_model(slug):
    problems, notes = [], []
    base = os.path.join(ROOT, "assets", "models", slug)
    path = os.path.join(base, slug + ".gltf")
    if not os.path.isfile(path):
        return ["not on disk"], notes
    g = json.load(open(path))
    a = g.get("asset", {})
    if "copyright" in a:
        notes.append("gltf copyright=%r" % a["copyright"])
    uris = [b.get("uri") for b in g.get("buffers", [])]
    uris += [i.get("uri") for i in g.get("images", [])]
    uris = [u for u in uris if u and not u.startswith("data:")]
    for u in uris:
        if re.match(r"^[a-z]+://", u):
            problems.append("external uri " + u)
        elif not os.path.isfile(os.path.join(base, u)):
            problems.append("missing " + u)
    on_disk = {os.path.relpath(p, base)
               for p in glob.glob(os.path.join(base, "**", "*"), recursive=True)
               if os.path.isfile(p) and not p.endswith(".import")}
    extra = on_disk - set(uris) - {slug + ".gltf"}
    if extra:
        problems.append("unreferenced " + ",".join(sorted(extra)))
    notes.append("%d files" % len(on_disk))
    return problems, notes


def check_texture(slug):
    problems = []
    for m in ("diff", "nor_gl", "arm"):
        f = os.path.join(ROOT, "assets", "textures", slug, "%s_%s_1k.jpg" % (slug, m))
        if not os.path.isfile(f) or os.path.getsize(f) == 0:
            problems.append("missing " + m)
    return problems, ["3 maps"]


def main():
    tex, mod = slugs()
    failed = 0
    for kind, items, fn in (("texture", tex, check_texture), ("model", mod, check_model)):
        for s in items:
            try:
                d = info(s)
            except Exception as e:  # noqa: BLE001 -- report and count, never hide
                print("%-8s %-27s FAIL  catalogue lookup failed: %s" % (kind, s, e))
                failed += 1
                continue
            problems, notes = fn(s)
            t = TYPES.get(d.get("type"), "?")
            if t != kind:
                problems.append("catalogue type is %s" % t)
            authors = ", ".join(d.get("authors", {})) or "NONE CREDITED"
            if authors == "NONE CREDITED":
                problems.append("no credited author")
            verdict = "FAIL" if problems else "ok"
            failed += bool(problems)
            print("%-8s %-27s %-4s  %-42s %s" % (
                kind, s, verdict, authors[:42], "; ".join(problems + notes)))
    print("\n%d textures, %d models, %d failed" % (len(tex), len(mod), failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
