#!/usr/bin/env python3
"""Generate candidate sprites from the reference plates. Deliberately minimal.

    source ~/.config/mineworld/secrets.env && \
        python3 clients/2d-spike/tools/gen_candidates.py specs/characters.json

This is NOT a generation provider and must not grow into one. `ARC-9`'s
pipeline is owned elsewhere (`vis/generation-backend`); at the time this was
written that branch carried no generation code, so rather than block, this
calls the images endpoint directly with the standard library and nothing else.
If a provider lands, this script is the thing that gets deleted.

What it does:

  * posts each spec to POST /v1/images/edits with the reference crops attached,
    so style comes from committed images rather than from adjectives — a prompt
    cannot carry a style, and the plates are the source of truth for this one
  * writes every result to the candidates directory, which holds candidates and
    not assets: nothing here is approved until a person approves it
  * writes a provenance sidecar per candidate — provider, model, date, the
    reference images used, the prompt, the request parameters and the response
    id — because `ARC-9` prices generated assets in provenance

The key is read from the environment and never logged. Nothing in this file
writes it anywhere.
"""

import json
import mimetypes
import os
import sys
import time
import urllib.error
import urllib.request
import uuid
from datetime import date

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, ".."))
CANDIDATES = os.path.join(ROOT, "art", "candidates")
ENDPOINT = "https://api.openai.com/v1/images/edits"


def _multipart(fields, files):
    """Encode multipart/form-data by hand. `files` is [(field, path)]."""
    boundary = "----mineworld%s" % uuid.uuid4().hex
    out = bytearray()
    for k, v in fields:
        out += b"--%s\r\n" % boundary.encode()
        out += b'Content-Disposition: form-data; name="%s"\r\n\r\n' % k.encode()
        out += b"%s\r\n" % str(v).encode()
    for field, path in files:
        ctype = mimetypes.guess_type(path)[0] or "application/octet-stream"
        name = os.path.basename(path)
        out += b"--%s\r\n" % boundary.encode()
        out += (b'Content-Disposition: form-data; name="%s"; filename="%s"\r\n'
                % (field.encode(), name.encode()))
        out += b"Content-Type: %s\r\n\r\n" % ctype.encode()
        with open(path, "rb") as f:
            out += f.read()
        out += b"\r\n"
    out += b"--%s--\r\n" % boundary.encode()
    return bytes(out), "multipart/form-data; boundary=%s" % boundary


def generate(spec, api_key, refs_root):
    refs = [os.path.join(refs_root, r) for r in spec["refs"]]
    for r in refs:
        if not os.path.exists(r):
            raise SystemExit("missing reference image: %s" % r)

    fields = [
        ("model", spec.get("model", "gpt-image-2.5-sunburst")),
        ("prompt", spec["prompt"]),
        ("size", spec.get("size", "1024x1536")),
        ("n", spec.get("n", 1)),
        ("output_format", "png"),
        ("background", spec.get("background", "transparent")),
    ]
    if "quality" in spec:
        fields.append(("quality", spec["quality"]))
    if "input_fidelity" in spec:
        fields.append(("input_fidelity", spec["input_fidelity"]))

    body, ctype = _multipart(fields, [("image[]", r) for r in refs])
    req = urllib.request.Request(ENDPOINT, data=body, method="POST")
    req.add_header("Authorization", "Bearer %s" % api_key)
    req.add_header("Content-Type", ctype)

    try:
        with urllib.request.urlopen(req, timeout=600) as resp:
            payload = json.load(resp)
    except urllib.error.HTTPError as e:
        detail = e.read().decode("utf-8", "replace")[:900]
        raise SystemExit("HTTP %s for %s\n%s" % (e.code, spec["name"], detail))

    os.makedirs(CANDIDATES, exist_ok=True)
    written = []
    import base64
    for i, item in enumerate(payload.get("data", [])):
        b64 = item.get("b64_json")
        if not b64:
            raise SystemExit("no b64_json in response for %s" % spec["name"])
        suffix = "" if len(payload["data"]) == 1 else "_%d" % (i + 1)
        png = os.path.join(CANDIDATES, "%s%s.png" % (spec["name"], suffix))
        with open(png, "wb") as f:
            f.write(base64.b64decode(b64))
        prov = {
            "source_type": "generated",
            "status": "candidate",          # never "asset" until a person says so
            "generation": {
                "provider": "openai",
                "endpoint": "/v1/images/edits",
                "model": dict(fields)["model"],
                "date": date.today().isoformat(),
                "reference_images": spec["refs"],
                "prompt": spec["prompt"],
                "request": {k: v for k, v in fields if k != "prompt"},
                "response_id": payload.get("created"),
            },
            "postprocess": {"cleaned": False, "normalized": False},
            "human_curated": False,
        }
        with open(png[:-4] + ".json", "w") as f:
            json.dump(prov, f, indent=2)
        written.append(png)
    return written


def main():
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        raise SystemExit(
            "OPENAI_API_KEY is not set. Run with:\n"
            "  source ~/.config/mineworld/secrets.env && python3 %s <spec.json>"
            % os.path.basename(__file__))
    if len(sys.argv) < 2:
        raise SystemExit("usage: gen_candidates.py <spec.json> [name ...]")

    spec_path = sys.argv[1]
    with open(spec_path) as f:
        doc = json.load(f)
    refs_root = os.path.normpath(
        os.path.join(os.path.dirname(os.path.abspath(spec_path)),
                     doc.get("refs_root", ".")))
    wanted = set(sys.argv[2:])
    specs = [s for s in doc["specs"] if not wanted or s["name"] in wanted]
    if not specs:
        raise SystemExit("no matching specs")

    for s in specs:
        merged = dict(doc.get("defaults", {}))
        merged.update(s)
        merged["prompt"] = "\n".join(
            [doc.get("style_preamble", ""), merged["prompt"]]).strip()
        t0 = time.time()
        files = generate(merged, key, refs_root)
        for p in files:
            print("candidate: %s  (%.0fs)" % (os.path.relpath(p, ROOT), time.time() - t0))


if __name__ == "__main__":
    main()
