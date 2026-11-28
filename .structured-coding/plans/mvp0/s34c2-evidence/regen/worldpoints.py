"""World sit points of every Lakeside place, read with Ruby's YAML as placements.py reads place files."""
import json, subprocess, sys
from pathlib import Path
root = Path(sys.argv[1]); out = {}
for f in sorted((root / "worlds/lakeside/places").glob("*.yaml")):
    d = json.loads(subprocess.check_output(["ruby", "-ryaml", "-rjson", "-e", "puts JSON.generate(YAML.load_file(ARGV[0]))", str(f)], text=True))
    if d.get("sit-points"):
        out[f.stem] = {"tags": d.get("tags", []), "points": d["sit-points"]}
Path(sys.argv[2]).write_text(json.dumps(out, indent=1) + "\n")
print(sum(len(v["points"]) for v in out.values()), "world points")
