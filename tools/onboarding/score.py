"""Score a World Pack an onboarding agent wrote, against a rubric, from the `mineworld` command's output only.

    python3 tools/onboarding/score.py WORLD_DIR --rubric ON-1 [--binary PATH] [--packs DIR] [--live] [--json]

`tools/onboarding/PROTOCOL.md` §5 is the authority for the checks. Every check reads what the binary prints, or the
pack's own files; none reads the agent's claims. Standard library only.
"""

from __future__ import annotations

import argparse
import datetime
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

HERE = Path(__file__).resolve().parent
RUBRICS = HERE / "rubrics"

PASS, FAIL, SKIPPED = "PASS", "FAIL", "SKIPPED"

# Words that name a model provider, an address or a key: none may appear in a World Pack (MODULE_SPEC.md §4.1).
PROVIDER_PATTERN = re.compile(
    r"https?://|api[_-]?key|key_env|base_url|\bsk-[A-Za-z0-9]|anthropic|openai|ollama|localhost|127\.0\.0\.1",
    re.IGNORECASE,
)


@dataclass
class Check:
    """One check's verdict and the evidence it was decided on."""

    id: str
    verdict: str
    evidence: str


@dataclass
class Score:
    """A whole score: each check, in order."""

    rubric: str
    world: str
    checks: list[Check] = field(default_factory=list[Check])

    def passed(self, gating: tuple[str, ...]) -> bool:
        """Whether every check in `gating` passed."""
        verdicts = {check.id: check.verdict for check in self.checks}
        return all(verdicts.get(name) == PASS for name in gating)

    def render(self) -> str:
        lines = [f"score {self.rubric} {self.world}"]
        lines += [f"  {check.id:<3} {check.verdict:<8} {check.evidence}" for check in self.checks]
        return "\n".join(lines)


@dataclass
class Binary:
    """The `mineworld` binary, run with the world's parent folder as its working directory."""

    path: Path
    packs: list[Path]

    def run(self, *arguments: str, cwd: Path, packs: bool = True) -> tuple[int, str]:
        command = [str(self.path), *arguments]
        if packs:
            for folder in self.packs:
                command += ["--packs", str(folder)]
        done = subprocess.run(
            command, cwd=cwd, capture_output=True, text=True, encoding="utf-8", errors="replace"
        )
        return done.returncode, done.stdout + done.stderr


def load_rubric(name: str) -> dict[str, Any]:
    path = RUBRICS / f"{name}.toml"
    with path.open("rb") as handle:
        return tomllib.load(handle)


def listed(output: str, label: str) -> list[str]:
    """The comma-separated values of a `validate` summary line such as `  people     ana, ben, cai`."""
    for line in output.splitlines():
        match = re.match(rf"^\s+{re.escape(label)}\s+(.*)$", line)
        if match:
            return [value.strip() for value in match.group(1).split(",") if value.strip()]
    return []


def place_words(world: Path, key: str) -> str:
    """The text a place is known by: its key, its `name` and its `tags`, lowercased. The place file is read as
    lines, never parsed as YAML: only those two keys count, at the file's top level."""
    words = [key]
    path = world / "places" / f"{key}.yaml"
    if not path.is_file():
        return key.lower()
    in_tags = False
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        text = line.split("#", 1)[0].rstrip()
        if not text:
            continue
        top = not text.startswith((" ", "\t", "-"))
        if top:
            in_tags = False
        if text.startswith("name:"):
            words.append(text.removeprefix("name:").strip().strip("\"'"))
        elif text.startswith("tags:"):
            inline = text.removeprefix("tags:").strip()
            if inline:
                words.append(inline.strip("[]"))
            else:
                in_tags = True
        elif in_tags and text.lstrip().startswith("-"):
            words.append(text.lstrip()[1:].strip().strip("\"'"))
    return " ".join(words).lower()


def matches(text: str, word: str) -> bool:
    return re.search(rf"\b{re.escape(word.lower())}", text) is not None


def has_pack(binary: Binary, cwd: Path, pack: str) -> bool:
    _, listing = binary.run("packs", "list", cwd=cwd, packs=False)
    return any(len(line.split()) > 1 and line.split()[1] == pack for line in listing.splitlines())


def installed_models(binary: Binary, world: Path) -> list[str]:
    """The decision models the build installs, from the refusal of a model no build installs."""
    first = listed(binary.run("validate", world.name, cwd=world.parent)[1], "seats")
    if not first:
        return []
    _, output = binary.run(
        "decisions", world.name, "--seat-model", f"{first[0]}=no-such-model", cwd=world.parent
    )
    match = re.search(r"it installs: (.*)$", output, re.MULTILINE)
    return [model.strip() for model in match.group(1).split(",")] if match else []


def epoch(world: Path) -> datetime.date | None:
    """The world's first day, from `configure/calendar.yaml`'s `epoch`, read as a line."""
    path = world / "configure" / "calendar.yaml"
    if not path.is_file():
        return None
    match = re.search(
        r"^epoch:\s*[\"']?(\d{4}-\d{2}-\d{2})", path.read_text(encoding="utf-8"), re.MULTILINE
    )
    return datetime.date.fromisoformat(match.group(1)) if match else None


def score(world: Path, rubric: dict[str, Any], binary: Binary, live: bool = False) -> Score:
    world = world.resolve()
    result = Score(rubric=str(rubric["id"]), world=world.name)
    cwd = world.parent
    days = int(rubric["days"])

    code, validated = binary.run("validate", world.name, cwd=cwd)
    valid = code == 0 and "is a valid World Pack" in validated
    last = validated.strip().splitlines()[-1:] or [""]
    result.checks.append(Check("S1", PASS if valid else FAIL, f"validate exited {code}: {last[0]}"))

    code, ran = binary.run(
        "run", world.name, "--headless", "--seed", "1", "--days", str(days), cwd=cwd
    )
    day_lines = len(re.findall(r"^day \d+ ", ran, re.MULTILINE))
    faults = re.search(r"^faults\s+(\d+)$", ran, re.MULTILINE)
    runs = code == 0 and day_lines == days and faults is not None and faults.group(1) == "0"
    result.checks.append(
        Check(
            "S2",
            PASS if runs else FAIL,
            f"run --days {days} exited {code}, {day_lines} day line(s), faults {faults.group(1) if faults else '?'}",
        )
    )
    # S8: a run that exits 0 can still have refused what people tried, such as every walk to a place no door
    # leads to (round 1, PROTOCOL.md §5). Any `rejected` request line fails it.
    rejected = re.findall(r"^requests\s+(\S+) rejected .*?(\d+)$", ran, re.MULTILINE)
    result.checks.append(
        Check(
            "S8",
            PASS if code == 0 and not rejected else FAIL,
            "no request rejected"
            if not rejected
            else "rejected: " + ", ".join(f"{action} x{count}" for action, count in rejected),
        )
    )

    places = listed(validated, "places")
    texts = {key: place_words(world, key) for key in places}
    wanted = rubric.get("places", {})
    missing = [
        word
        for word in wanted.get("required", [])
        if not any(matches(t, word) for t in texts.values())
    ]
    setting = wanted.get("setting", [])
    set_ok = not setting or any(matches(t, word) for t in texts.values() for word in setting)
    s3 = bool(places) and not missing and set_ok
    evidence = f"places {', '.join(places) or 'none'}"
    if missing:
        evidence += f"; no place says {', '.join(missing)}"
    if not set_ok:
        evidence += f"; no place says any of {', '.join(setting)}"
    result.checks.append(Check("S3", PASS if s3 else FAIL, evidence))

    people, seats = listed(validated, "people"), listed(validated, "seats")
    exactly = int(rubric.get("people", {}).get("exactly", 0))
    s4 = len(people) == exactly and sorted(people) == sorted(seats)
    result.checks.append(
        Check(
            "S4",
            PASS if s4 else FAIL,
            f"{len(people)} people ({', '.join(people)}), seats {', '.join(seats)}",
        )
    )

    if "packs" in rubric:
        _, resolved = binary.run("packs", "resolve", world.name, cwd=cwd)
        needed = [
            pack
            for pack in rubric["packs"].get("requires", [])
            if f"requires   {pack} " not in resolved
        ]
        result.checks.append(
            Check(
                "P1",
                FAIL if needed else PASS,
                f"not required: {', '.join(needed)}" if needed else "required",
            )
        )

    if rubric.get("market", {}).get("weekends"):
        result.checks.append(market(world, binary, days, validated))
    if "model" in rubric:
        result.checks.append(model(world, binary, int(rubric["model"]["lm_seats"])))
        result.checks.append(
            Check(
                "S7", SKIPPED, "needs the operator's local model; run with --live on that machine"
            )
            if not live
            else Check(
                "S7",
                FAIL,
                "the live check is defined by P6's run report, which this build does not have",
            )
        )
    return result


def market(world: Path, binary: Binary, days: int, validated: str) -> Check:
    """S5: a happening begins on every Saturday and Sunday of the run and on no weekday."""
    if not has_pack(binary, world.parent, "mineworld-happenings"):
        return Check("S5", FAIL, "product gap: this build has no happenings System Pack (WE-a)")
    if "happenings" not in listed(validated, "systems"):
        return Check("S5", FAIL, "happenings is not enabled in systems")
    first = epoch(world)
    if first is None:
        return Check("S5", FAIL, "no calendar epoch, so no day is a Saturday or a Sunday")
    seats = listed(validated, "seats")
    with tempfile.TemporaryDirectory() as scratch:
        save = Path(scratch) / "save"
        code, _ = binary.run(
            "run",
            world.name,
            "--headless",
            "--seed",
            "1",
            "--days",
            str(days),
            "--save",
            str(save),
            cwd=world.parent,
        )
        if code != 0:
            return Check("S5", FAIL, f"the saved run exited {code}")
        began: set[int] = set()
        for seat in seats:
            _, perceived = binary.run(
                "perceived",
                world.name,
                "--save",
                str(save),
                "--person",
                seat,
                "--json",
                cwd=world.parent,
            )
            for line in perceived.splitlines():
                if '"event_type":"happening-began"' in line:
                    began.add(int(json.loads(line)["at"]) // 86_400)
    weekend = {d for d in range(days) if (first + datetime.timedelta(days=d)).weekday() >= 5}
    if began == weekend and weekend:
        return Check(
            "S5",
            PASS,
            f"a happening began on day(s) {sorted(d + 1 for d in began)}, every weekend day",
        )
    return Check(
        "S5",
        FAIL,
        f"happenings began on day(s) {sorted(d + 1 for d in began)}; weekend days {sorted(d + 1 for d in weekend)}",
    )


def model(world: Path, binary: Binary, wanted: int) -> Check:
    """S6: the resolved decision list puts exactly `wanted` seats on `lm`, each with a fallback, and the pack names no
    provider, address or key."""
    for path in world.rglob("*"):
        if path.is_file() and path.suffix in {".yaml", ".yml", ".toml", ".json"}:
            hit = PROVIDER_PATTERN.search(path.read_text(encoding="utf-8", errors="replace"))
            if hit:
                return Check(
                    "S6", FAIL, f"{path.relative_to(world).as_posix()} names {hit.group(0)!r}"
                )
    if "lm" not in installed_models(binary, world):
        return Check("S6", FAIL, "product gap: this build installs no decision model lm (P6)")
    code, output = binary.run("decisions", world.name, "--json", cwd=world.parent)
    if code != 0:
        return Check("S6", FAIL, f"decisions exited {code}")
    seats = json.loads(output).get("seats", [])
    on_lm = [seat for seat in seats if seat.get("model", {}).get("id") == "lm"]
    with_fallback = [seat for seat in on_lm if seat.get("model", {}).get("fallback")]
    ok = len(on_lm) == wanted and len(with_fallback) == wanted
    names = ", ".join(str(seat.get("seat")) for seat in on_lm) or "none"
    return Check(
        "S6", PASS if ok else FAIL, f"on lm: {names}; with a fallback: {len(with_fallback)}"
    )


def default_binary() -> Path:
    name = "mineworld.exe" if sys.platform == "win32" else "mineworld"
    return HERE.parents[1] / "target" / "debug" / name


def main(arguments: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Score an onboarding World Pack (PROTOCOL.md §5).")
    parser.add_argument("world", type=Path)
    parser.add_argument("--rubric", required=True)
    parser.add_argument("--binary", type=Path, default=default_binary())
    parser.add_argument("--packs", type=Path, action="append", default=[])
    parser.add_argument("--live", action="store_true")
    parser.add_argument("--json", action="store_true")
    options = parser.parse_args(arguments)
    rubric = load_rubric(options.rubric)
    result = score(
        options.world,
        rubric,
        Binary(options.binary.resolve(), [p.resolve() for p in options.packs]),
        options.live,
    )
    if options.json:
        print(
            json.dumps(
                {
                    "rubric": result.rubric,
                    "world": result.world,
                    "checks": [c.__dict__ for c in result.checks],
                }
            )
        )
    else:
        print(result.render())
    gating = tuple(check.id for check in result.checks if check.id != "S7")
    return 0 if result.passed(gating) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
