# Asset generation — content-production tooling

Generates **candidate** 2D art from MineWorld's committed style references, with an ARC-9
provenance sidecar per asset.

## Scope — read this before wiring anything to it

This is production tooling, not world logic (`docs/REUSE_POLICY.md` §6, `ARC-9`):

- the server, the clients, `kernel/`, `systems/` and `contracts/` never import it;
- **no contract references it**, and it is never a runtime dependency of anything shipped;
- it runs on an operator's machine, on demand, and writes candidate files.

Everything it produces is a *candidate*. Under `ARC-9` a generated output becomes an asset only
when a person accepts it — the tooling never sets `approval.status` to anything but `candidate`,
and never writes into an approved-asset directory.

## Backend

`OpenAIImageProvider` (OpenAI Images API) is the one implementation of
`ImageGenerationProvider`. There is no registry and no plugin layer; per §6 that comes when a
second backend actually exists.

**Model.** `gpt-image-2.5-flare` by default — verified against the live model catalogue rather
than recalled, because an obsolete model id fails silently. `gpt-image-2.5-sunburst` is the
slower, more capable sibling. Override with `--model` or `MINEWORLD_IMAGE_MODEL`.

**Why an API rather than a local model.** Reference conditioning is the whole requirement, and
the decisive constraint is licensing rather than quality: the only permissively-licensed local
stack that can do reference conditioning is SDXL + IP-Adapter, and FLUX's reference adapter
(Redux) is non-commercial and gated. The API path also matches the references far more closely,
since the references were themselves produced this way. See the branch report for the full
comparison.

**Licence position for outputs.** Under OpenAI's terms the customer owns the output images and
may use and redistribute them commercially, so candidates can be committed to this MIT
repository. No model weights are redistributed. The reference images sent with each request are
MineWorld's own committed style references.

## Credentials

Never stored in the repository. Resolution order:

1. `OPENAI_API_KEY` already in the environment — always wins;
2. otherwise a secret file whose path comes from `MINEWORLD_SECRETS_FILE`;
3. otherwise the documented default `~/.config/mineworld/secrets.env`.

The default is **a convention on one machine, not a dependency** — exporting the variable is
equally valid and nothing here requires the file to exist. No key is ever written to a log, a
sidecar, a manifest or stdout.

## Install

Outside the repository, so nothing large or machine-specific lands in git:

```bash
uv venv --python 3.12 ~/.cache/mineworld-gen/venv
uv pip install --python ~/.cache/mineworld-gen/venv/bin/python -r tools/asset_generation/requirements.txt
```

## Use

One asset:

```bash
python -m mineworld_gen \
  --prompt "…" \
  --reference presentation/mineworld-default/2D/references/01_town_square.png \
  --out presentation/mineworld-default/2D/candidates \
  --name bakery --transparent
```

A whole set — prompts live in `recipes/` as reviewable data, since the prompt is the part that
actually gets iterated on:

```bash
python -m mineworld_gen.batch recipes/2d_default.yaml \
  --out ../../presentation/mineworld-default/2D/candidates
python -m mineworld_gen.contactsheet ../../presentation/mineworld-default/2D/candidates
```

`--dry-run` lists what a recipe would generate and costs nothing.

## Interface

```python
class ImageGenerationProvider(Protocol):
    name: str
    model_id: str
    workflow_version: str

    def generate(
        self,
        prompt: str,
        reference_images: Sequence[Path] = (),
        params: GenerationParams | None = None,
    ) -> GenerationResult: ...
```

`GenerationResult` carries the raw image bytes (never re-encoded, so no generation loss and no
silent alpha flattening), the ARC-9 provenance dict, and per-call wall time, token usage and
cost. A provider that cannot do reference conditioning must raise rather than silently
degrading to a text-only prompt.

## Normalization

Every transparent candidate is normalized on the way in (`normalize.py`), because both defects
below bite at integration time rather than immediately:

- returned alpha peaks at 254, so nothing is fully opaque — rescaled so the peak becomes 255;
- transparent pixels carry arbitrary RGB, which bilinear filtering turns into a dark fringe —
  edge colour is bled 8px outwards.

Both are invisible on the composited image. Nothing subjective happens here: which style
variant is *right* is the operator's call, at the integrated-scene level.

## Cost

Token-based. At 1024×1024, quality `high`, two references: ~1750 output tokens and ~3100 input
tokens, about **$0.077 per image** and **~26 s**. Each extra reference adds roughly $0.012.
Recorded per asset in the sidecar under `generation.runtime`.
