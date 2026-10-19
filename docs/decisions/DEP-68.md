## DEP-68 — Population data sources: ACS and NHANES (public domain), the Monk Skin Tone scale (CC BY 4.0)

**Date** 2026-10-15 · **Status** accepted (step plan approved 2026-10-15); used by S30-e's cited profiles · **Approved by** the operator, directly in the primary session, 2026-10-15 (step-30 approval; Q-S30-5 for `verify` marks) · **Relates to** `ARC-95`, `ARC-123`, [`REUSE_POLICY.md`](REUSE_POLICY.md) §§11–12 · **Design** `.structured-coding/plans/mvp0/step-30-person-attributes.md` §§6.2, 8.2

**Problem.** Realistic defaults must follow real-world values with citations, and a profile must keep data apart from authored casting conventions (step-30 F-S30-5).

**Options considered.** Authored figures only — declined: unrealistic, and indistinguishable from data. Commercial demographic datasets — declined: licences incompatible with a CC0 pack. **Public-domain U.S. federal statistics, plus an openly licensed skin-tone scale, each cited per figure** — chosen.

**Choice.**

| Source | Licence | Used for |
| --- | --- | --- |
| American Community Survey (U.S. Census Bureau), e.g. 2024 1-year tables B01001, B03002 for San Diego County | public domain (U.S. federal work) | sex, age-band and ancestry marginals |
| NHANES 2015–2018 anthropometric reference data (Fryar et al. 2021, Vital Health Stat 3(46)) | public domain (U.S. federal work) | stature and body-mass by sex and age; build |
| Monk Skin Tone scale (Ellis Monk; released by Google, 2022) | CC BY 4.0, attribution in the pack | the ten-level `core.skin-tone` domain |

Each figure in a profile names its source and is marked `read` or `verify`; casting conventions are sources of status `authored` and are never presented as data. A release refuses a reference world whose data figures are still `verify` (Q-S30-5).

**Consequences.** The default domain ranges follow real values: stature 0.50–2.50 m and mass 2–300 kg as domains only, the realistic distribution being the profile's. A county figure not yet read (for example build, step-30 §8.2) stays `verify` until it is.

**Revisit** when a world needs a population outside the United States, which brings its own sources and a record of its own.
