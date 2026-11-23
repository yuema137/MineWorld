#!/usr/bin/env python3
"""Exercise 2D seat drawing in an assembled bundle against pre-authoring mesh targets.

Run after package.py assemble: check_packaged_seating.py BUNDLE --evidence NEW_DIRECTORY.
--source-client runs the same interaction from this checkout without --root, covering
its documented fallback. Expected coordinates never come from authored offsets.
"""
import argparse
import json
import re
import subprocess
from pathlib import Path

import package


def check_pose(text: str) -> list[dict]:
    fixture = json.loads((package.ROOT / "tools/cli/tests/fixtures/seat-surfaces.json").read_text())
    targets = {t["point"]: t for t in fixture["targets"] if t["world"] == "social-cafe"}
    postures = [json.loads(line[8:]) for line in text.splitlines() if line.startswith("POSTURE ")]
    if len(postures) != 3 or postures[1]["seated"] is not None:
        raise package.Failure("expected seated, standing, seated observations")
    results = []
    for row in (postures[0], postures[2]):
        seat = row["seated"]
        if seat is None or seat.get("drawn") is not True or seat.get("point") != row["point"]:
            raise package.Failure("observer was not drawn at the reported seat")
        target = targets[row["point"]]
        actual = [float(x) for x in re.findall(r"-?\d+(?:\.\d+)?", seat["pelvis"])]
        expected = [x * 1000 for x in target["drawing_anchor_place_m"]]
        if len(actual) != 2 or len(expected) != 2:
            raise package.Failure("pelvis must have two horizontal coordinates")
        errors = [abs(a - b) for a, b in zip(actual, expected)]
        errors.append(abs(seat["pose"]["seat_height_m"] - target["seat_height_m"]) * 1000)
        result = {"point": row["point"], "actual_mm": actual, "expected_mm": expected,
                  "errors_mm": errors, "tolerance_mm": 10, "pass": max(errors) <= 10}
        print(json.dumps(result), flush=True)
        results.append(result)
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("--evidence", required=True, type=Path)
    parser.add_argument("--source-client", action="store_true")
    args = parser.parse_args()
    scratch = args.evidence.resolve()
    scratch.mkdir(parents=True)  # Refuse to overwrite an earlier run or user save.
    runtime = args.bundle.resolve() / "runtime"
    info = package.read_bundle_toml(runtime)
    steps = [{"click_sit": "nearest-far"}, {"await": {"component": "posture-state"}},
             {"report_posture": True}, {"open": "self"},
             {"choose": {"action_type": "stand"}},
             {"await": {"component": "posture-state", "present": False}},
             {"report_posture": True}, {"click_sit": "last"},
             {"await": {"component": "posture-state"}}, {"report_posture": True}]
    (scratch / "steps.json").write_text(json.dumps(steps))
    server, handle, address, invite = package.start_server(runtime, info, "social-cafe", scratch)
    try:
        if args.source_client:
            command = ["godot", "--headless", "--path", str(package.ROOT / "clients/2d"), "--"]
        else:
            command = [str(runtime / info["client_2d"]), "--headless", "--", "--client=2d",
                       "--root=" + str(runtime)]
        command += ["--server=" + address, "--invite=" + invite, "--seat=visitor",
                    "--world-key=social-cafe", "--settings=none", "--drive=steps",
                    "--steps=" + str(scratch / "steps.json")]
        with (scratch / "client.log").open("w") as log:
            completed = subprocess.run(command, cwd=scratch, stdout=log, stderr=subprocess.STDOUT,
                                       timeout=150)
    finally:
        package.stop(server)
        handle.close()
    text = (scratch / "client.log").read_text()
    if completed.returncode or "drive complete: PASS" not in text or "SCRIPT ERROR" in text or "Parse Error" in text:
        raise package.Failure("client interaction failed; inspect client.log")
    results = check_pose(text)
    (scratch / "pose-results.json").write_text(json.dumps(results, indent=2))
    passed = all(row["pass"] for row in results) and server.returncode == 0
    print("packaged seating:", "PASS" if passed else "FAIL", "2 measured seated poses; stand observed")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
