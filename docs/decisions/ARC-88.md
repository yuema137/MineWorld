## ARC-88 — World evolution: a deterministic `happenings` System Pack, and LLM directors that only propose

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by WE-a (`happenings`); the director is implemented by WE-b (MVP-1) · **Approved by** the operator's step-24 rulings of 2026-10-12 (§1.2 ruling 3; QS24-2 ruled yes, narrowly, for the director) · **Relates to** `ARC-68` (weather), `ARC-34`, `ARC-63`, `ARC-62`, `DEP-46`, `PA-10`, `INV-6`, `INV-10`, `CLAUDE.md` §4 rule 2 · **Design** step-24 §§3.1, 9

**Problem.** A world needs changes beyond individuals: market days, festivals, prices, shop stock, news and weather-driven happenings. Such changes could be written into controllers, or an LLM could be given authority to change the world directly, as a game master would. Neither fits: the first hides world change in controller code, and the second makes the world invalid without a model.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) An LLM controller mutates world state, or resolves actions as a referee | Declined: an LLM never mutates world state (`INV-6`, ruling 3); the world must stay valid with the model removed |
| (b) A deterministic System Pack, `happenings`, generates scheduled and seeded happenings; an LLM director is a Controller Pack that submits `propose-happening` intents, which `happenings` validates and may refuse | **Chosen** |
| (c) A director drives a designated Person as a puppet | Kept as the alternative to the Organization seat; not chosen, because a puppet Person would appear to act in biography |

**Choice.** `happenings` (WE-a, MVP-0) owns `Happenings` on places and emits `happening-announced`, `happening-began` and `happening-ended`. Its configuration is a plain `ARC-61` configure file, `configure/happenings.yaml`, not an Interaction List section, because `ARC-63` forbids a second file beside a section. Happening kinds are content in that file. Generators are the `ARC-62` code catalog `HappeningGenerator`. Calendar entries are pure functions of the date; random entries are seeded integer draws from the content `seed:` in that file, since a world has no world seed; and its own `limits` (per day, per place, per actor per day) are caps it enforces. Other packs react to happenings only through their own opt-in configuration (for example, economy price modifiers). The director (WE-b, MVP-1) is a Controller Pack seated on an Organization, which is valid only for the director (QS24-2). It has no referee authority: it proposes, `happenings` validates the kind, place, window, limits and text, and it may refuse with a typed reason, and a refused proposal changes nothing. With the director removed, its seat runs its `fallback` (`none` by default), and `happenings` keeps its own schedule, so the world stays valid and still evolves.

**Consequences.** `CORE_CONCEPTS.md` §14 ("a Controller decides what a Person attempts") is amended by WE-b, not by this record, and WE-b's PR design states the exact wording. No `worldpack` change: `happenings` is an ordinary enabled system and its file is an ordinary configure entry.

**Revisit** when WE-b's Organization-seat rule is recorded (the amendment of `CORE_CONCEPTS.md` §14), or when a director must act on world state beyond proposals, which would require a new decision.

**Note, 2026-10-15 (WE-a, PR #189, as built; `.structured-coding/plans/mvp0/pr-we-a-happenings.md` §§5–7, 11).**
- **State.** `happenings` owns no component on places. Its state lives in Processes, as calendar's does:
  one `almanac` Process per world and one `happening` Process per announced happening. It discloses two
  records that no entity carries, `happenings-here` and `happenings-announced`, to an observer in a place
  (F-WE-a-6). "Owns `Happenings` on places" above is superseded by this.
- **Facts.** It also states `happenings-configured` at genesis, and `happening-withheld` for each refused
  draft. Both are `SystemInternal` with no subjects, so an author can see why a happening never happened.
- **Limits.** WE-a's `limits` are `per_day` and `per_place_running`. `per actor per day` is a proposals limit
  and arrives with WE-b's `propose-happening`.
- **Dependencies.** `happenings` depends on `presence` and `calendar` (a world without calendar is refused
  naming it). A selected generator's pack is a configuration `requires` (`ARC-61`), checked by the loader.
- **Reuse considered.** `rrule` (RFC 5545 recurrence) was declined: three recurrence forms over calendar's
  `CalendarDate` are a few dozen lines, and `rrule` would bring `chrono` and wall-clock types into a System
  Pack. `rand` was declined: counter-based draws (SplitMix64, pinned to its reference vector, as weather's)
  are required for resume equality. No third-party crate was added.
- **The first reaction.** `economy` gained an optional `configure/economy.yaml` with `price_modifiers`. It
  states its own `prices-adjusted` / `prices-restored`, caused by `happening-began` / `happening-ended`, and
  reduces them into `Shop.adjustments`. Its `SystemVersion` is 2.
- **Market Town opts in by configuration** (QWE-1 (a)), with the store at 900 ‰ on market days. AC-1
  admits that file under `ARC-35`'s note of 2026-10-15.
