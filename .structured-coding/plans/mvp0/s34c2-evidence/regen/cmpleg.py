import json, sys
old = json.load(open(sys.argv[1]))["legacy"]; new = json.load(open(sys.argv[2]))
print("legacy equal:", old == new, len(old["rows"]), len(new["rows"]))
if old != new:
    for a, b in zip(old["rows"], new["rows"]):
        if a != b: print("first diff", a["point"], json.dumps(a)[:300], "\n", json.dumps(b)[:300]); break
