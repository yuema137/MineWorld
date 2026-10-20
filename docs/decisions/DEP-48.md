## DEP-48 — Daily-rhythm calibration: BLS ATUS A-3 public-domain tables, committed as derived integer tables through an offline fit tool

**Date** 2026-10-12 · **Status** accepted (design, step-24); implemented by DL-b · **Approved by** the operator's step-24 rulings of 2026-10-12 (QS24-9 accepted as recommended) · **Relates to** `ARC-68` (the `weather-fetch fit` pattern), `DEP-31`, `ARC-23`, `R-S24-9` · **Design** step-24 §§6.6, 7, 15

**Problem.** `motivated` needs daily rhythms calibrated from real data, so that its sociability and home pull follow a measured hour-of-day curve rather than arbitrary constants. The candidate sources have different licences. The American Time Use Survey's published tables are public domain (BLS), with credit requested. IPUMS ATUS-X extracts require registration, and redistribution needs approval. MTUS requires registration, and its redistribution terms are unstated. HETUS microdata is research-only.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Use IPUMS, MTUS or HETUS microdata | Declined: redistribution is not permitted, and repository licensing would be a risk |
| (b) Commit BLS ATUS A-3 (percent of the population engaging in selected activities by time of day) as derived integer tables with a credit line; an offline tool `routine-fit` produces the curve set and optional routine sections; the run never reads the source | **Chosen** |
| (c) Hand-set the curves | Declined: the arbitrary constants are the problem this record solves, and `ARC-23` requires numbers fixed before measurement from a named source |

**Choice.** Option (b). Only the aggregate public-domain tables are committed, as derived integer tables, and their credit line is in the repository. Microdata never enters the repository. The `routine-fit` tool runs offline, and its output is committed content. HETUS aggregate tables on Eurobase (CC BY 4.0 for content, acknowledgement required) are reserved for a later European world.

**Consequences.** The acceptance of rhythm (AC-DL-4) is measured against the fitted curve, with its tolerances written in the PR design before the run. A future calibration from a different source is a new record.

**Revisit** when a European world needs calibration from HETUS aggregates, or when the BLS terms of use change.

**Note, 2026-10-14 (DL-b, the calibration as built; `.structured-coding/plans/mvp0/pr-dl-b-motivated.md` §7,
§15).**
- **Edition and rows.** The edition is `atus-a3-2025`: BLS ATUS Table A-3, 2025 annual averages (QDLb-3). The
  six rows are Sleeping, Eating and drinking, Working, Socializing and communicating, Leisure and sports, and
  Purchasing goods and services.
- **The committed file.** `cognition/motivated/data/atus-a3-2025.tsv` holds the rows as per-mille integers
  (percent × 10, so the conversion is exact), with a `zero_markers` line and a per-row `checksum` line. The
  credit and public-domain statement are in `cognition/motivated/data/NOTICE`. No microdata is committed.
- **Derivation.** It is in the crate and pure, and there is no offline `routine-fit` tool: the routine
  generator is deferred (QDLb-1). `curves::from_a3` maps each row to a 24-point curve with
  `y(h) = ⌊v(h) × 1 000 / max v⌋`, and `restless` = `1 000 − rest`. The table is compiled in and derived when
  parameters are decoded, so a run never opens a file.
- **Transcription.** The values were transcribed from the HTML table, read through a summarising fetch. The
  reviewer's three-cell check against the PDF is still open (§15, C2).
- **The acceptance oracle.** `A(h)` is the socializing row normalised to sum 1 000, read from the TSV. It is
  not the model's curve output. The measured results are in `DEP-47`'s note of the same date.
