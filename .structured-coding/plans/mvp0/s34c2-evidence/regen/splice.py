"""Write S34-c2's regenerated Lakeside rows into seat-surfaces.json as text: only the `legacy` and
`legacy_surfaces` blocks and the five Lakeside source hashes change; every other byte stays."""
import hashlib, json, sys
from pathlib import Path

def block(key, value, last):
    body = json.dumps(value, indent=2).replace("\n", "\n  ")
    return f'  "{key}": {body}' + ("\n" if last else ",\n")

def splice(text, legacy, surfaces, hashes):
    fx = json.loads(text)
    keys = list(fx)
    assert keys[-2:] == ["legacy", "legacy_surfaces"], keys
    start = text.index('\n  "legacy": {') + 1
    head = text[:start]
    for path, value in hashes.items():
        old = f'"{path}": "{fx["source_sha256"][path]}"'
        assert head.count(old) == 1, path
        head = head.replace(old, f'"{path}": "{value}"')
    return head + block("legacy", legacy, False) + block("legacy_surfaces", surfaces, True) + "}\n"

if __name__ == "__main__":
    root, out, mode = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3]
    f = root / "tools/cli/tests/fixtures/seat-surfaces.json"
    text = f.read_text()
    fx = json.loads(text)
    if mode == "selftest":  # the splice of main's own blocks gives main's bytes back
        assert splice(text, fx["legacy"], fx["legacy_surfaces"], {}) == text
        print("selftest: byte-equal")
    else:
        hashes = {p: hashlib.sha256((root / p).read_bytes()).hexdigest()
                  for p in fx["source_sha256"] if p.startswith("worlds/lakeside/")}
        new = splice(text, json.loads((out / "legacy-baseline-head.json").read_text()),
                     json.loads((out / "pm-head.json").read_text())["lakeside"], hashes)
        f.write_text(new)
        print("spliced; lakeside hashes", len(hashes))
