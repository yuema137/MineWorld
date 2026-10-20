"""The pilot's OpenAI image calls (design §§4.3 R-1, R-7, 4.7; I-p-1, I-p-2; C1).

- **Template v2 only.** Every prompt starts with the template's full-body preamble, read from
  `template.yaml` at run time; a template whose `version` is not the recipe's (2), or whose bytes differ from
  the recipe's SHA-256, is refused before any call (F-S32p-11, M-p-9).
- **The model is always named.** Every call is `python -m mineworld_gen … --model <recipe model>`; a sidecar
  that names another model refuses the candidate (I-p-2).
- **The cap.** `<scratch>/images.json` counts every call; a request that would pass the recipe's call cap
  (70) is refused before it runs.

`mineworld_gen` writes YAML sidecars (`<name>.provenance.yaml`); this stdlib module reads only the one line it
checks, `generation.model`, which `yaml.safe_dump` always writes as `  model: <id>` under `generation:`.
"""

from __future__ import annotations

import json
import re
import subprocess
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

from mwcast.wardrobe.recipe import Garment, Macros, Recipe

GEN_PYTHON_ENV = "MINEWORLD_GEN_PYTHON"
PROMPTS = Path(__file__).resolve().parents[2] / "wardrobe" / "prompts"


class ImageError(Exception):
    """An image call was refused, or its output failed a check."""


@dataclass(frozen=True)
class Template:
    version: int
    full_body_preamble: str


def read_template(path: Path) -> Template:
    """`version` and `full_body.preamble` of the casting template (a block scalar, `preamble: |`)."""
    lines = path.read_text(encoding="utf-8").splitlines()
    version: int | None = None
    for line in lines:
        match = re.fullmatch(r"version:\s*(\d+)\s*(#.*)?", line)
        if match:
            version = int(match.group(1))
            break
    if version is None:
        raise ImageError(f"{path}: no top-level `version:`")
    try:
        start = lines.index("full_body:")
    except ValueError as exc:
        raise ImageError(f"{path}: no top-level `full_body:`") from exc
    preamble: list[str] | None = None
    for line in lines[start + 1 :]:
        if preamble is None:
            if line and not line.startswith(" "):
                break
            if line.strip() == "preamble: |":
                preamble = []
            continue
        if line.strip() and not line.startswith("    "):
            break
        preamble.append(line[4:])
    if not preamble:
        raise ImageError(f"{path}: no `full_body.preamble: |` block")
    while preamble and not preamble[-1].strip():
        preamble.pop()
    return Template(version=version, full_body_preamble="\n".join(preamble))


def checked_template(recipe: Recipe, repo_root: Path) -> Template:
    """The template the recipe names, refused unless it is version 2 and its bytes are the recorded ones."""
    if recipe.template_version != 2:
        raise ImageError(
            f"{recipe.source}: template.version is {recipe.template_version}; only v2 is allowed"
        )
    path = recipe.template.verified(repo_root, None)
    template = read_template(path)
    if template.version != recipe.template_version:
        raise ImageError(
            f"{path}: version {template.version}; the recipe allows only v{recipe.template_version}"
        )
    return template


def _prompt_file(name: str, prompts: Path) -> str:
    return (prompts / name).read_text(encoding="utf-8").strip()


def base_layer_prompt(template: Template, macros: Macros, prompts: Path = PROMPTS) -> str:
    """R-1: the v2 full-body preamble verbatim, then the base-layer paragraph with the person's line (sex,
    age, stature and build from the recipe, so the style anchor's person is never taken for the subject)."""
    text = _prompt_file("base_layer.txt", prompts)
    values = {
        "sex_noun": "woman" if macros.sex == "female" else "man",
        "apparent_age": str(macros.apparent_age),
        "stature_m": f"{macros.stature_m:.2f}",
        "build": macros.build,
    }
    for key, value in values.items():
        text = text.replace("{" + key + "}", value)
    if re.search(r"\{[a-z_]+\}", text):
        raise ImageError(f"{prompts / 'base_layer.txt'}: a placeholder is left unfilled")
    return template.full_body_preamble + "\n\n" + text


def garment_prompt(
    template: Template, garment: Garment, prompts: Path = PROMPTS, mannequin: str = "grey"
) -> str:
    """R-7: the v2 full-body preamble verbatim, then the garment (or hair) paragraph with the description and
    the mannequin's colour words (the recipe's `mannequin`, grey unless L-a's chroma key is set)."""
    name = "hair.txt" if garment.kind == "hair" else "garment.txt"
    text = _prompt_file(name, prompts)
    for key, value in (
        ("description", garment.description),
        ("category", garment.category),
        ("mannequin", mannequin),
    ):
        text = text.replace("{" + key + "}", value)
    if re.search(r"\{[a-z_]+\}", text):
        raise ImageError(f"{prompts / name}: a placeholder is left unfilled")
    return template.full_body_preamble + "\n\n" + text


@dataclass(frozen=True)
class CallLog:
    calls: int
    entries: tuple[dict[str, object], ...]


def load_calls(path: Path) -> CallLog:
    if not path.exists():
        return CallLog(calls=0, entries=())
    raw: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ImageError(f"{path}: not an object")
    data = cast(dict[str, object], raw)
    calls = data.get("calls")
    entries = data.get("entries")
    if isinstance(calls, bool) or not isinstance(calls, int) or not isinstance(entries, list):
        raise ImageError(f"{path}: expected an integer `calls` and an `entries` list")
    return CallLog(calls=calls, entries=tuple(cast(list[dict[str, object]], entries)))


def write_calls(path: Path, log: CallLog) -> None:
    payload = {"calls": log.calls, "entries": list(log.entries)}
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def admit_calls(log: CallLog, count: int, cap: int) -> None:
    if log.calls + count > cap:
        raise ImageError(
            f"refused (image cap): {log.calls} calls made + {count} requested exceeds the cap {cap}; "
            "nothing sent"
        )


def command(
    *,
    python: str,
    recipe: Recipe,
    prompt: str,
    references: Sequence[Path],
    out: Path,
    name: str,
    count: int,
) -> list[str]:
    """The `mineworld_gen` argument list. `--model` is always present (I-p-2)."""
    args = [python, "-m", "mineworld_gen", "--prompt", prompt]
    for reference in references:
        args += ["--reference", str(reference)]
    args += [
        "--out",
        str(out),
        "--name",
        name,
        "--count",
        str(count),
        "--size",
        recipe.image_model.size,
        "--quality",
        recipe.image_model.quality,
        "--model",
        recipe.image_model.model,
    ]
    return args


def sidecar_model(sidecar: Path) -> str:
    """`generation.model` of a `mineworld_gen` sidecar."""
    inside = False
    for line in sidecar.read_text(encoding="utf-8").splitlines():
        if line.startswith("generation:"):
            inside = True
            continue
        if inside and line and not line.startswith(" "):
            break
        match = re.fullmatch(r"  model: ['\"]?([^'\"]+)['\"]?", line) if inside else None
        if match:
            return match.group(1)
    raise ImageError(f"{sidecar}: no generation.model")


def check_outputs(out: Path, name: str, count: int, model: str) -> list[Path]:
    """The candidates written by one command, each with a sidecar naming `model`; refuse any other model."""
    stems = [f"{name}_{i + 1:02d}" for i in range(count)] if count > 1 else [name]
    images: list[Path] = []
    for stem in stems:
        image = out / f"{stem}.png"
        sidecar = out / f"{stem}.provenance.yaml"
        if not image.is_file() or not sidecar.is_file():
            raise ImageError(f"{image}: missing, or its sidecar is missing")
        found = sidecar_model(sidecar)
        if found != model:
            raise ImageError(
                f"{sidecar}: model {found!r}; only {model!r} is allowed (I-p-2): candidate refused"
            )
        images.append(image)
    return images


def check_generator_fix(repo_root: Path, commit: str) -> None:
    """Refuse unless the provenance fix `commit` is an ancestor of HEAD (I-p-2: the fixed `mineworld_gen`)."""
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", commit, "HEAD"],
        cwd=repo_root,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        raise ImageError(f"refused: the mineworld_gen fix {commit} is not an ancestor of HEAD (I-p-2)")


Runner = Callable[[Sequence[str], Path], int]


def run_mineworld_gen(args: Sequence[str], cwd: Path) -> int:
    """The real runner: `mineworld_gen` from `tools/asset_generation` (its key comes from the environment)."""
    return subprocess.run(list(args), cwd=cwd, check=False).returncode


def generate(
    *,
    recipe: Recipe,
    repo_root: Path,
    scratch: Path,
    environ: Mapping[str, str],
    prompt: str,
    references: Sequence[Path],
    out: Path,
    name: str,
    count: int,
    label: str,
    runner: Runner = run_mineworld_gen,
) -> list[Path]:
    """One `mineworld_gen` command of `count` calls, admitted against the cap and logged in `images.json`."""
    log_path = scratch / "images.json"
    log = load_calls(log_path)
    admit_calls(log, count, recipe.image_model.call_cap)
    python = environ.get(GEN_PYTHON_ENV, "")
    if not python:
        raise ImageError(f"{GEN_PYTHON_ENV} is not set: the Python of the mineworld_gen tool venv")
    out.mkdir(parents=True, exist_ok=True)
    args = command(
        python=python, recipe=recipe, prompt=prompt, references=references, out=out, name=name, count=count
    )
    code = runner(args, repo_root / "tools" / "asset_generation")
    # Counted whatever the outcome: a failed command may still have been charged for some calls.
    entry: dict[str, object] = {
        "label": label,
        "name": name,
        "count": count,
        "exit": code,
        "usd_estimate": round(count * recipe.image_model.usd_per_call, 4),
    }
    write_calls(log_path, CallLog(calls=log.calls + count, entries=(*log.entries, entry)))
    if code != 0:
        raise ImageError(f"mineworld_gen exited {code} for {label}")
    return check_outputs(out, name, count, recipe.image_model.model)
