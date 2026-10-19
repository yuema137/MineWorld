"""The pilot recipe, `recipes/s32p-pilot.yaml` (design §§4.5–4.7, 4.9; C1).

The recipe is written in the JSON-compatible subset of YAML and parsed with `json`, like `fit.yaml`
(PIPELINE.md §4). Every key is checked and an unknown key is refused with its key path. Cross-references are
checked too: a garment's wearer is a subject, its slot is one of the slots, an outfit names garments of
distinct slots and one hair style, a tight-fit garment has no Meshy run (Q-S32p-5), and every subject's
phases are budget phases.

A subject whose identity reference or base-layer pick is missing cannot run a paid stage: the stages ask the
recipe (`require_base_layer_pick`, `require_reference_pick`) and refuse.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

from mwcast.records import RecordError, Section, load_record

FORMAT_VERSION = 1
SLUG = r"[a-z0-9]+(-[a-z0-9]+)*"
SHA256 = r"[0-9a-f]{64}"
DATE = r"\d{4}-\d{2}-\d{2}"
CAST_OUT_ENV = "MINEWORLD_CAST_OUT"

Location = Literal["repo", "cast-out"]
Kind = Literal["garment", "hair"]


@dataclass(frozen=True)
class FileRef:
    """A file by location and path, with the SHA-256 its bytes must have when it is used."""

    location: Location
    path: str
    sha256: str

    def resolve(self, repo_root: Path, cast_out: Path | None) -> Path:
        if self.location == "repo":
            return repo_root / self.path
        if cast_out is None:
            raise RecordError(
                f"{self.path} lives in the casting output folder: give --cast-out or set {CAST_OUT_ENV}"
            )
        return cast_out / self.path

    def verified(self, repo_root: Path, cast_out: Path | None) -> Path:
        """The resolved path, after checking its bytes against the recorded SHA-256."""
        path = self.resolve(repo_root, cast_out)
        if not path.is_file():
            raise RecordError(f"{path}: not found")
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != self.sha256:
            raise RecordError(f"{path}: SHA-256 {digest} differs from the recipe's {self.sha256}")
        return path


@dataclass(frozen=True)
class Pick:
    """A recorded choice among candidates: the file (scratch-relative), its SHA-256, who, when and where."""

    file: str
    sha256: str
    by: str
    date: str
    source: str


@dataclass(frozen=True)
class Macros:
    """What R-4 builds the fitting template from, with the source of the values."""

    sex: Literal["female", "male"]
    apparent_age: int
    build: str
    stature_m: float
    skin_mst: int
    source: str


@dataclass(frozen=True)
class Subject:
    key: str
    identity: FileRef
    identity_source: str
    base_layer_pick: Pick | None
    macros: Macros
    body_phase: str
    asset_phase: str


@dataclass(frozen=True)
class Garment:
    """A garment or a hair style of the pilot library (design §4.5)."""

    id: str
    kind: Kind
    wearer: str
    category: str
    slot: str | None  # None for hair
    fit: Literal["shell", "tight"]
    weights: str  # "inherit" or "rigid <bone>"
    covers_regions: tuple[str, ...]
    source: Literal["meshy", "tight"]
    description: str
    reference_pick: Pick | None
    under_headwear: Literal["hidden"] | None  # hair only


@dataclass(frozen=True)
class Outfit:
    subject: str
    name: str
    garments: tuple[str, ...]
    hair: str


@dataclass(frozen=True)
class Ceiling:
    name: str
    phases: tuple[str, ...]
    credits: int


@dataclass(frozen=True)
class PriorSpend:
    what: str
    credits: int
    inside_approval: bool


@dataclass(frozen=True)
class Budget:
    """§4.6: the cap, the phases and their ceilings, the retry budgets and pool, the IH gate, the reports."""

    cap: int
    credits_per_run: int
    phases: tuple[str, ...]
    ceilings: tuple[Ceiling, ...]
    retries: dict[str, int]  # per asset kind: body, garment, hair
    retry_pool: int
    gate_phases: tuple[str, ...]
    gate_min_remaining: int
    report_at: tuple[int, ...]
    s26_approval: int
    s26_prior: tuple[PriorSpend, ...]


@dataclass(frozen=True)
class Mannequin:
    """The figure garment references are drawn on (R-6) and the clay R-8a removes again (L-a, ARC-138 ruling
    2026-10-19). `words` names it in the prompts; `srgb` is the render colour; `clay_tolerance_max` bounds the
    colour test (a chroma key sits far from every garment, so its bound is wider than grey's)."""

    words: str
    srgb: tuple[float, float, float]
    clay_tolerance_max: float


GREY_MANNEQUIN = Mannequin(words="grey", srgb=(0.5, 0.5, 0.5), clay_tolerance_max=0.10)


@dataclass(frozen=True)
class ImageModel:
    """§4.7: the one image model, the call cap, the size and quality, and the generator fix required."""

    model: str
    size: str
    quality: str
    call_cap: int
    usd_per_call: float
    generator_fix: str


@dataclass(frozen=True)
class Recipe:
    source: Path
    pilot: str
    template: FileRef
    template_version: int
    style_anchor: FileRef
    image_model: ImageModel
    slots: tuple[str, ...]
    regions: tuple[str, ...]
    subjects: dict[str, Subject]
    garments: dict[str, Garment]
    outfits: dict[tuple[str, str], Outfit]
    budget: Budget
    texel_density: float | None
    texel_density_source: str
    fixtures: Fixtures
    mannequin: Mannequin = GREY_MANNEQUIN

    def subject(self, key: str) -> Subject:
        if key not in self.subjects:
            raise RecordError(f"{self.source}: no subject {key!r} (subjects: {', '.join(self.subjects)})")
        return self.subjects[key]

    def garment(self, garment_id: str) -> Garment:
        if garment_id not in self.garments:
            raise RecordError(f"{self.source}: no garment or hair {garment_id!r}")
        return self.garments[garment_id]

    def slot_layer(self, slot: str) -> int:
        """The push-out order of a slot, 1 for the innermost (design §4.5's slot order)."""
        return self.slots.index(slot) + 1

    def phase_of(self, subject_or_asset: str, *, body: bool) -> str:
        """The budget phase a run is charged to: a subject's body phase, or its wearer's asset phase."""
        if body:
            return self.subject(subject_or_asset).body_phase
        return self.subject(self.garment(subject_or_asset).wearer).asset_phase

    def require_base_layer_pick(self, key: str) -> Pick:
        pick = self.subject(key).base_layer_pick
        if pick is None:
            raise RecordError(
                f"{self.source}: subjects.{key}.base_layer_pick is null: the operator's pick on the "
                "pre-spend page is recorded before any Meshy run for this body (step-26 §6.4 gate 1; R-1)"
            )
        return pick

    def require_reference_pick(self, garment_id: str) -> Pick:
        garment = self.garment(garment_id)
        if garment.source != "meshy":
            raise RecordError(
                f"{self.source}: {garment_id} is a tight-fit garment; it has no Meshy run (R-8b)"
            )
        if garment.reference_pick is None:
            raise RecordError(
                f"{self.source}: garments.{garment_id}.reference_pick is null: the agent's raw-gate "
                "pick of the garment reference is recorded before its Meshy run (Q-S32p-3)"
            )
        return garment.reference_pick


def _file_ref(s: Section) -> FileRef:
    location = s.choice("location", ("repo", "cast-out"))
    ref = FileRef(
        location="repo" if location == "repo" else "cast-out",
        path=s.text("path", pattern=r"[^/\\].*"),
        sha256=s.text("sha256", pattern=SHA256),
    )
    s.done()
    return ref


def _pick(parent: Section, key: str) -> Pick | None:
    if parent.is_null(key):
        return None
    s = parent.section(key)
    pick = Pick(
        file=s.text("file"),
        sha256=s.text("sha256", pattern=SHA256),
        by=s.text("by"),
        date=s.text("date", pattern=DATE),
        source=s.text("source"),
    )
    s.done()
    return pick


def _macros(s: Section) -> Macros:
    sex = s.choice("sex", ("female", "male"))
    macros = Macros(
        sex="female" if sex == "female" else "male",
        apparent_age=s.integer("apparent_age", low=18, high=100),
        build=s.choice("build", ("slim", "average", "heavy")),
        stature_m=s.number("stature_m", low=1.4, high=2.1),
        skin_mst=s.integer("skin_mst", low=1, high=10),
        source=s.text("source"),
    )
    s.done()
    return macros


@dataclass(frozen=True)
class FixtureBody:
    """A hidden W-5 test body (role: fixture): MPFB2 macros, an optional weight-slider value, a stature."""

    name: str
    sex: Literal["female", "male"]
    apparent_age: int
    build: str
    weight_value: float | None
    stature_m: float
    source: str

    def macros(self) -> dict[str, object]:
        out: dict[str, object] = {"sex": self.sex, "apparent_age": self.apparent_age, "build": self.build}
        if self.weight_value is not None:
            out["weight_value"] = self.weight_value
        return out


@dataclass(frozen=True)
class FixtureGarment:
    """A stock CC0 MakeHuman garment, a test fixture only (DEP-72)."""

    id: str
    mhclo: str  # relative to MINEWORLD_MPFB_ASSETS
    slot: str
    covers_regions: tuple[str, ...]
    licence: str


@dataclass(frozen=True)
class Fixtures:
    bodies: dict[str, FixtureBody]
    garments: dict[str, FixtureGarment]


def _fixtures(s: Section, slots: tuple[str, ...], regions: tuple[str, ...], source: Path) -> Fixtures:
    bodies: dict[str, FixtureBody] = {}
    group = s.section("bodies")
    for name in group.keys():
        b = group.section(name)
        sex = b.choice("sex", ("female", "male"))
        weight = None
        if b.has("weight_value"):
            weight = b.number("weight_value", low=0.0, high=1.0)
        bodies[name] = FixtureBody(
            name=name,
            sex="female" if sex == "female" else "male",
            apparent_age=b.integer("apparent_age", low=18, high=100),
            build=b.choice("build", ("slim", "average", "heavy")),
            weight_value=weight,
            stature_m=b.number("stature_m", low=1.4, high=2.1),
            source=b.text("source"),
        )
        b.done()
    garments: dict[str, FixtureGarment] = {}
    group = s.section("garments")
    for gid in group.keys():
        g = group.section(gid)
        garment = FixtureGarment(
            id=gid,
            mhclo=g.text("mhclo", pattern=r"[^/\\].*\.mhclo"),
            slot=g.choice("slot", slots),
            covers_regions=g.strings("covers_regions"),
            licence=g.choice("licence", ("CC0",)),
        )
        g.done()
        if not set(garment.covers_regions) <= set(regions):
            raise RecordError(f"{source}: fixtures.garments.{gid}.covers_regions: unknown region")
        garments[gid] = garment
    s.done()
    return Fixtures(bodies=bodies, garments=garments)


def _subject(key: str, s: Section, phases: tuple[str, ...], source: Path) -> Subject:
    identity = _file_ref(s.section("identity"))
    subject = Subject(
        key=key,
        identity=identity,
        identity_source=s.text("identity_source"),
        base_layer_pick=_pick(s, "base_layer_pick"),
        macros=_macros(s.section("macros")),
        body_phase=s.text("body_phase"),
        asset_phase=s.text("asset_phase"),
    )
    s.done()
    for phase in (subject.body_phase, subject.asset_phase):
        if phase not in phases:
            raise RecordError(f"{source}: subjects.{key}: phase {phase!r} is not a budget phase")
    return subject


def _garment(
    gid: str, kind: Kind, s: Section, slots: tuple[str, ...], regions: tuple[str, ...], source: Path
) -> Garment:
    where = f"{'garments' if kind == 'garment' else 'hair'}.{gid}"
    slot: str | None = None
    if kind == "garment":
        slot = s.choice("slot", slots)
    fit = s.choice("fit", ("shell", "tight"))
    origin = s.choice("source", ("meshy", "tight"))
    weights = s.text("weights", pattern=r"inherit|rigid [A-Za-z0-9]+")
    covers = s.strings("covers_regions")
    under: Literal["hidden"] | None = None
    if kind == "hair":
        s.choice("under_headwear", ("hidden",))
        under = "hidden"
    garment = Garment(
        id=gid,
        kind=kind,
        wearer=s.text("wearer", pattern=SLUG),
        category=s.text("category", pattern=SLUG),
        slot=slot,
        fit="shell" if fit == "shell" else "tight",
        weights=weights,
        covers_regions=covers,
        source="meshy" if origin == "meshy" else "tight",
        description=s.text("description"),
        reference_pick=_pick(s, "reference_pick"),
        under_headwear=under,
    )
    s.done()
    unknown = [r for r in covers if r not in regions]
    if unknown or not covers:
        raise RecordError(f"{source}: {where}.covers_regions: unknown or empty regions {unknown}")
    if (garment.fit == "tight") != (garment.source == "tight"):
        raise RecordError(f"{source}: {where}: a tight-fit garment has source tight, and only it (Q-S32p-5)")
    return garment


def _budget(s: Section, source: Path) -> Budget:
    phases = s.strings("phases")
    ceilings: list[Ceiling] = []
    for index, item in enumerate(s.items("ceilings")):
        c = Section(item, f"budget.ceilings[{index}]", source)
        ceiling = Ceiling(
            name=c.text("name"), phases=c.strings("phases"), credits=c.integer("credits", low=0)
        )
        c.done()
        if not set(ceiling.phases) <= set(phases):
            raise RecordError(
                f"{source}: budget.ceilings[{index}]: names a phase that is not in budget.phases"
            )
        ceilings.append(ceiling)
    r = s.section("retries")
    retries = {kind: r.integer(kind, low=0) for kind in ("body", "garment", "hair")}
    r.done()
    gate = s.section("phase_gate")
    gate_phases = gate.strings("phases")
    gate_min = gate.integer("min_remaining", low=0)
    gate.done()
    s26 = s.section("s26")
    prior: list[PriorSpend] = []
    for index, item in enumerate(s26.items("prior")):
        p = Section(item, f"budget.s26.prior[{index}]", source)
        prior.append(
            PriorSpend(
                what=p.text("what"), credits=p.integer("credits", low=0), inside_approval=p.boolean("inside")
            )
        )
        p.done()
    budget = Budget(
        cap=s.integer("cap", low=0),
        credits_per_run=s.integer("credits_per_run", low=1),
        phases=phases,
        ceilings=tuple(ceilings),
        retries=retries,
        retry_pool=s.integer("retry_pool", low=0),
        gate_phases=gate_phases,
        gate_min_remaining=gate_min,
        report_at=tuple(int(v) for v in s.numbers("report_at", len(s.items("report_at")))),
        s26_approval=s26.integer("approval", low=0),
        s26_prior=tuple(prior),
    )
    s26.done()
    s.done()
    if not set(gate_phases) <= set(phases):
        raise RecordError(f"{source}: budget.phase_gate.phases: names a phase that is not in budget.phases")
    return budget


def _image_model(s: Section) -> ImageModel:
    model = ImageModel(
        model=s.text("model"),
        size=s.text("size", pattern=r"\d+x\d+"),
        quality=s.choice("quality", ("low", "medium", "high")),
        call_cap=s.integer("call_cap", low=0),
        usd_per_call=s.number("usd_per_call", low=0.0),
        generator_fix=s.text("generator_fix", pattern=r"[0-9a-f]{7,40}"),
    )
    s.done()
    return model


def _outfits(
    s: Section, subjects: dict[str, Subject], garments: dict[str, Garment], source: Path
) -> dict[tuple[str, str], Outfit]:
    outfits: dict[tuple[str, str], Outfit] = {}
    for key in s.keys():
        if key not in subjects:
            raise RecordError(f"{source}: outfits.{key}: not a subject")
        per = s.section(key)
        names = per.keys()
        if sorted(names) != ["A", "B"]:
            raise RecordError(f"{source}: outfits.{key}: expected exactly outfits A and B, found {names}")
        for name in names:
            o = per.section(name)
            outfit = Outfit(subject=key, name=name, garments=o.strings("garments"), hair=o.text("hair"))
            o.done()
            check_outfit(outfit, garments, f"{source}: outfits.{key}.{name}")
            outfits[(key, name)] = outfit
        per.done()
    missing = sorted(set(subjects) - {k for k, _ in outfits})
    if missing:
        raise RecordError(f"{source}: outfits: no outfits for {missing}")
    return outfits


def check_outfit(outfit: Outfit, garments: dict[str, Garment], where: str) -> None:
    """An outfit names known garments of distinct slots and exactly one known hair style (exclusivity)."""
    taken: dict[str, str] = {}
    for gid in outfit.garments:
        garment = garments.get(gid)
        if garment is None or garment.kind != "garment" or garment.slot is None:
            raise RecordError(f"{where}: {gid!r} is not a garment of the library")
        if garment.slot in taken:
            raise RecordError(f"{where}: {gid} and {taken[garment.slot]} both take slot {garment.slot}")
        taken[garment.slot] = gid
    hair = garments.get(outfit.hair)
    if hair is None or hair.kind != "hair":
        raise RecordError(f"{where}: {outfit.hair!r} is not a hair style of the library")


def parse_recipe(raw: object, source: Path) -> Recipe:
    s = Section(raw, "", source)
    if s.integer("format") != FORMAT_VERSION:
        raise RecordError(f"{source}: format: expected {FORMAT_VERSION}")
    template_section = s.section("template")
    template_version = template_section.integer("version")
    template = FileRef(
        location="repo",
        path=template_section.text("path"),
        sha256=template_section.text("sha256", pattern=SHA256),
    )
    template_section.done()
    budget = _budget(s.section("budget"), source)
    slots = s.strings("slots")
    regions = s.strings("regions")
    subjects_section = s.section("subjects")
    subjects = {
        key: _subject(key, subjects_section.section(key), budget.phases, source)
        for key in subjects_section.keys()
    }
    garments: dict[str, Garment] = {}
    for kind, key in (("garment", "garments"), ("hair", "hair")):
        group = s.section(key)
        for gid in group.keys():
            if gid in garments:
                raise RecordError(f"{source}: {key}.{gid}: the id is used twice")
            garments[gid] = _garment(
                gid, "garment" if kind == "garment" else "hair", group.section(gid), slots, regions, source
            )
        group.done()
    for garment in garments.values():
        if garment.wearer not in subjects:
            raise RecordError(f"{source}: {garment.id}: wearer {garment.wearer!r} is not a subject")
    w3 = s.section("texel_density")
    density = None if w3.is_null("value") else w3.number("value", positive=True)
    density_source = w3.text("source")
    w3.done()
    mannequin = GREY_MANNEQUIN
    if s.has("mannequin"):
        m = s.section("mannequin")
        srgb = m.numbers("srgb", 3)
        mannequin = Mannequin(
            words=m.text("words"),
            srgb=(srgb[0], srgb[1], srgb[2]),
            clay_tolerance_max=m.number("clay_tolerance_max", positive=True),
        )
        m.done()
    recipe = Recipe(
        source=source,
        pilot=s.text("pilot", pattern=SLUG),
        template=template,
        template_version=template_version,
        style_anchor=_file_ref(s.section("style_anchor")),
        image_model=_image_model(s.section("image_model")),
        slots=slots,
        regions=regions,
        subjects=subjects,
        garments=garments,
        outfits=_outfits(s.section("outfits"), subjects, garments, source),
        budget=budget,
        texel_density=density,
        texel_density_source=density_source,
        fixtures=_fixtures(s.section("fixtures"), slots, regions, source),
        mannequin=mannequin,
    )
    subjects_section.done()
    s.done()
    return recipe


def load_recipe(path: Path) -> Recipe:
    return parse_recipe(load_record(path), path)
