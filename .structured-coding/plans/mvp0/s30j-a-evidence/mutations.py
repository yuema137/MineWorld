"""Bounded counterfactual runner; invoke through the shared heavy-job slot.

Never runs concurrently with a source writer. Restores exact production bytes in
finally, checks semantic test failures rather than accepting compilation failure,
and reruns all affected test binaries after restoration.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[4]
EVIDENCE = Path(__file__).resolve().parent
SOURCE = ROOT / "authoring/attributes/src/sample/newcomer.rs"
original = SOURCE.read_bytes()
text = original.decode()
cases = [
    ("M-j-6", "distribution.stratify && declaration.owner == ATTRIBUTES",
     "false && distribution.stratify && declaration.owner == ATTRIBUTES", 2,
     "fidelity", "a_thousand_arrivals_preserve_both_profiles_and_reproduce"),
    ("independent-boundary", "cumulative > point", "cumulative >= point", 1,
     "newcomer_independent", "cumulative_boundary_is_exclusive_at_the_integer_point"),
    ("independent-residents", "let word = self.words.draw(id, self.key, 0);",
     "let word = self.words.draw(id, self.key, 0) ^ if self.population.is_empty() { 0 } else { 1 << 63 };", 1,
     "newcomer_independent", "changing_or_reordering_residents_cannot_change_independent_draws"),
    ("independent-weights", "let weights = group_weights(distribution, parents).unwrap_or_default();",
     "let Shape::Prior { rows, .. } = &distribution.shape else { unreachable!() }; let weights: Vec<u128> = rows.row(parents).unwrap().iter().map(|weight| *weight as u128).collect();", 1,
     "newcomer_independent", "independent_rows_masks_adjustments_and_uniforms_match_known_vectors"),
]
if len(sys.argv) > 1:
    cases = [case for case in cases if case[0] in sys.argv[1:]]
    assert len(cases) == len(sys.argv) - 1, "unknown/repeated mutation label"
results = []


def run(label, binary, test=None):
    command = ["cargo", "test", "-p", "mineworld-attributes-model", "--test", binary]
    if test:
        command.append(test)
    command.extend(["--", "--nocapture"])
    start = time.monotonic()
    with (EVIDENCE / (label + ".log")).open("w") as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, timeout=180)
    log = (EVIDENCE / (label + ".log")).read_text()
    return {"label": label, "command": command, "returncode": result.returncode,
            "seconds": round(time.monotonic() - start, 3)}, log


try:
    for label, before, after, expected_count, binary, test in cases:
        assert text.count(before) == expected_count, (label, "source seam moved")
        SOURCE.write_text(text.replace(before, after))
        result, log = run(label, binary, test)
        assert result["returncode"] == 101 and "test result: FAILED" in log, log
        assert test in log and "FAILED" in log, log
        result["outcome"] = "DISCRIMINATING FAIL"
        results.append(result)
        SOURCE.write_bytes(original)
        assert SOURCE.read_bytes() == original
finally:
    SOURCE.write_bytes(original)
    assert SOURCE.read_bytes() == original

for binary in sorted({case[4] for case in cases}):
    result, log = run("restored-" + binary, binary)
    assert result["returncode"] == 0 and "test result: ok" in log, log
    result["outcome"] = "PASS"
    results.append(result)

(EVIDENCE / "mutations.json").write_text(json.dumps({
    "restored_source_sha256": hashlib.sha256(original).hexdigest(), "results": results,
}, indent=2) + "\n")
print(json.dumps(results, indent=2))
