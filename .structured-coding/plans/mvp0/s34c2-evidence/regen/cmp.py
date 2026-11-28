import json, sys
fx = json.load(open(sys.argv[1]))
new = json.load(open(sys.argv[2]))["lakeside"]
old = fx["legacy_surfaces"]
print("rows", len(old), len(new), "equal:", old == new)
for a, b in zip(old, new):
    if a != b:
        print("DIFF", a["point"], {k: (a.get(k), b.get(k)) for k in set(a) | set(b) if a.get(k) != b.get(k)})
        break
