"""Image calls (I-p-1, I-p-2, §4.7): v2 preamble verbatim, v3 refused, the model always named, the cap."""

from __future__ import annotations

import json
from collections.abc import Sequence
from pathlib import Path

import pytest

from mwcast.wardrobe import images
from mwcast.wardrobe.recipe import load_recipe

PIPELINE = Path(__file__).resolve().parents[1]
REPO = PIPELINE.parents[1]
RECIPE = load_recipe(PIPELINE / "recipes" / "s32p-pilot.yaml")
TEMPLATE = REPO / "presentation/mineworld-default/3D/characters/style/template.yaml"


def test_the_base_layer_prompt_starts_with_the_v2_preamble_verbatim() -> None:
    template = images.checked_template(RECIPE, REPO)
    prompt = images.base_layer_prompt(template, RECIPE.subject("ivan-petrov").macros)
    assert "a man, apparent age about 70, 1.67 m tall," in prompt and "heavy build" in prompt

    assert template.version == 2
    assert prompt.startswith("The attached image is a STYLE REFERENCE ONLY")
    assert "no props,\nno scenery" in template.full_body_preamble or "no props, no scenery" in prompt
    assert "Hair stays clear of the shoulders and arms." in prompt  # the preamble's last line
    assert "BAREFOOT" in prompt and "mid-grey sleeveless athletic tank top" in prompt


def test_m_p_9_a_template_at_version_3_is_refused_before_any_call(tmp_path: Path) -> None:
    """M-p-9: `version: 3` is refused: its bytes differ from the recipe's SHA-256, and the version is read."""
    text = TEMPLATE.read_text(encoding="utf-8").replace("version: 2\n", "version: 3\n", 1)
    planted = tmp_path / "template.yaml"
    planted.write_text(text, encoding="utf-8")

    assert images.read_template(planted).version == 3
    repo = tmp_path / "repo"
    target = repo / RECIPE.template.path
    target.parent.mkdir(parents=True)
    target.write_text(text, encoding="utf-8")
    with pytest.raises(Exception, match="SHA-256"):
        images.checked_template(RECIPE, repo)


def test_the_garment_prompt_fills_the_description_and_the_hair_prompt_is_used_for_hair() -> None:
    template = images.checked_template(RECIPE, REPO)

    tee = images.garment_prompt(template, RECIPE.garment("vera-tee"))
    hair = images.garment_prompt(template, RECIPE.garment("hana-pixie"))

    assert "Good Places Brighter People" in tee and "{description}" not in tee
    assert "HAIR REFERENCE" in hair and "straight black pixie cut" in hair


def test_every_command_names_the_model() -> None:
    args = images.command(
        python="py",
        recipe=RECIPE,
        prompt="p",
        references=[Path("a.png"), Path("b.png")],
        out=Path("o"),
        name="n",
        count=3,
    )

    assert args[args.index("--model") + 1] == "gpt-image-2.5-sunburst"
    assert args.count("--reference") == 2 and args[args.index("--reference") + 1] == "a.png"


def _sidecar(folder: Path, stem: str, model: str) -> None:
    (folder / f"{stem}.png").write_bytes(b"\x89PNG")
    (folder / f"{stem}.provenance.yaml").write_text(
        f"source_type: generated\ngeneration:\n  tool: x\n  provider: openai-images\n  model: {model}\n"
        "  workflow_version: 1.0.0\npostprocess:\n  cleaned: false\n",
        encoding="utf-8",
    )


def test_a_sidecar_naming_another_model_refuses_the_candidate(tmp_path: Path) -> None:
    _sidecar(tmp_path, "c_01", "gpt-image-2.5-sunburst")
    _sidecar(tmp_path, "c_02", "gpt-image-2.5-flare")

    with pytest.raises(images.ImageError, match="flare"):
        images.check_outputs(tmp_path, "c", 2, "gpt-image-2.5-sunburst")


def test_generate_counts_calls_and_refuses_past_the_cap(tmp_path: Path) -> None:
    seen: list[list[str]] = []

    def runner(args: Sequence[str], cwd: Path) -> int:
        seen.append(list(args))
        out = Path(args[args.index("--out") + 1])
        for i in range(int(args[args.index("--count") + 1])):
            _sidecar(out, f"base_layer_{i + 1:02d}", "gpt-image-2.5-sunburst")
        return 0

    (tmp_path / "images.json").write_text(json.dumps({"calls": 67, "entries": []}), encoding="utf-8")
    written = images.generate(
        recipe=RECIPE,
        repo_root=REPO,
        scratch=tmp_path,
        environ={images.GEN_PYTHON_ENV: "py"},
        prompt="p",
        references=[],
        out=tmp_path / "out",
        name="base_layer",
        count=3,
        label="base-layer:x",
        runner=runner,
    )

    assert len(written) == 3 and len(seen) == 1
    assert images.load_calls(tmp_path / "images.json").calls == 70
    with pytest.raises(images.ImageError, match="image cap"):
        images.generate(
            recipe=RECIPE,
            repo_root=REPO,
            scratch=tmp_path,
            environ={images.GEN_PYTHON_ENV: "py"},
            prompt="p",
            references=[],
            out=tmp_path / "out",
            name="base_layer",
            count=1,
            label="x",
            runner=runner,
        )
    assert len(seen) == 1
