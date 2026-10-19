## DEP-31 — Weather data: NOAA GHCN-Daily, reshaped offline by `tools/weather-fetch`; `ureq` confined to that tool behind an off-by-default feature

**Date** 2026-10-09 · **Status** selected; one optional dependency (`ureq`) added to `tools/weather-fetch`
only · **Approved by** the primary session at PR TW-d's design freeze (step-19 §18.9.1; QTWd-3 … 8 ruled
as recommended) · **Relates to** `ARC-61` (and its `data:` attachment note), `ARC-68`, `DEP-8`,
`REUSE_POLICY.md` §§11–12, step-19 `INV-TW-7` · **Design**
`.structured-coding/plans/mvp0/step-19-time-weather.md` §3.2, §6.3–§6.5, §18 (S19, PR TW-d)

**Problem.** Market Town's weather should be San Diego's real weather, replayed day by day, with the
rules of `ARC-68` filling only what the record lacks. The data must be redistributable inside the
repository, carry its provenance, be produced by a program anyone can re-run, and reach a world without
any crate that `run`, the server or a client builds ever opening a network connection (`INV-TW-7`).

**Data source** (read 2026-10-08, re-read 2026-10-09; the full table is step-19 §3.2).

```text
(a) NOAA GHCN-Daily, station USW00023188 (San Diego International Airport, Lindbergh Field) —
    CC0-1.0 through NOAA's Open Data Dissemination ("There are no restrictions on the use of the
    data", https://registry.opendata.aws/noaa-ghcn/); a US federal work. NOAA requests attribution,
    forbids stating or implying endorsement, and asks that modified data not be presented as original
    NOAA data. Daily TMAX, TMIN, PRCP, AWND, WDF2 and weather-type flags since 1939.
(b) NOAA GHCNh (hourly) — CC0; ISD's successor, with visibility and present weather (fog)
(c) NOAA ISD / ISD-Lite — CC0; superseded (no updates after about 2025-08-24), no fog field
(d) Meteostat — CC BY 4.0; repackages NOAA for this station and adds an attribution duty
(e) Open-Meteo historical — data CC BY 4.0, but the free API is non-commercial only
(f) ERA5 (Copernicus) — CC BY 4.0 since 2025-07-02; a ~31 km grid that smooths the coastal marine
    layer; needs an account and NetCDF/GRIB extraction
```

**Selected: (a)** as the default daily record, years 2015–2024 (ten whole years, two of them leap
years; QTW-5). **(b)** is the optional hourly layer of a later PR (TW-g). (c) is rejected for new work,
(d) as a redundant intermediary, (e) because its access route fails `DEP-8`'s commercial-use rule, and
(f) is kept only as the fallback for a world placed where no station exists (QTW-10). The generator that
fills gaps stays `ARC-68`'s (WGEN-lite, built from the published algorithm; LARS-WG rejected there for
its non-commercial licence); this decision does not restate it.

**Input format: the station's `.dly` file**, `https://www.ncei.noaa.gov/pub/data/ghcn/daily/all/<station>.dly`
— plain fixed-width text whose columns NOAA's `readme.txt` §III documents (one line per station, month
and element; 31 values of eight columns each; missing is −9999). The by-station CSV is a long format
served gzipped, so it would add a gzip dependency for no gain (QTWd-3).

**The committed form is our own CSV, labelled modified.** `worlds/<world>/data/weather/<name>.csv`, the
`weather` pack's input format (`systems/weather/README.md`, step-19 SD-TW-d-1): one row per day,
`date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,wdf2_deg,fog,thunder`, integers in GHCN-Daily's own units, an
empty cell for a missing or quality-flagged value, `fog = WT01 ∨ WT02 ∨ WT21`, `thunder = WT03`. A `NOTICE`
beside it records the source URL, station, retrieval date, the tool's version and exact command, the
`.dly` input's byte count, the CSV's byte and line counts, the attribution, NOAA's two requested
citations (Menne et al. 2012, *J. Atmos. Oceanic Technol.* 29:897–910, doi:10.1175/JTECH-D-11-00103.1;
Menne et al. 2012, GHCN-Daily Version 3, doi:10.7289/V5D21VHZ), and the statement that the data is
modified (reshaped and gap-reported) from NOAA GHCN-Daily and not endorsed by NOAA. No digest is recorded
(no SHA-256 crate is in the tree, and adding one for this alone is not worth a dependency, QTWd-7):
reproducibility is owned by the tool's determinism test and by `.gitattributes`' `eol=lf` for World Pack
data, which makes every platform check out the same bytes. The pack decodes LF, CRLF and a leading BOM
identically, so a user's own file in either form works too.

**The tool: `tools/weather-fetch`** (crate `mineworld-weather-fetch`), a developer tool that `run`, CI and
the server never execute. `reshape` turns a `.dly` file into the CSV and NOTICE and prints a report
(missing values by year, gap runs, weather-type population by year, quality-flagged values dropped);
`fit` estimates a `configure/weather.yaml` rules block from a CSV in integers only; both are byte-
reproducible for the same input and arguments. It uses the `weather` pack's own CSV and rules types, so
the format has one authority. `fetch` downloads the `.dly` file verbatim and exists only with the cargo
feature `fetch`, **off by default**.

**HTTP client** (`REUSE_POLICY.md`: a commodity, adopted, not built).

```text
(a) ureq 3.4.2 — MIT OR Apache-2.0; blocking, small; rustls with the ring provider and webpki-roots
    (features: default off, `rustls`)
(b) reqwest — async (tokio) for one GET; far larger tree
(c) no HTTP client: the developer downloads the file by hand and runs `reshape`
```

**Selected: (a)**, pinned exactly (`=3.4.2`) in `tools/weather-fetch/Cargo.toml` as an optional dependency
enabled only by the tool's `fetch` feature. No other crate depends on the tool, so no runtime crate can
reach `ureq` (`INV-TW-7`, checked by `cargo tree` and by a test over `Cargo.lock`). The ring provider
needs only a C compiler, which the CI image has; `cargo clippy --all-features` compiles it there. The
feature adds 22 entries to `Cargo.lock` (twelve crates and ten Windows target shims), all under permissive
licences (MIT, Apache-2.0, ISC, BSD-3-Clause; `ring` is
Apache-2.0 AND ISC; `webpki-roots`, Mozilla's root certificates as data, is CDLA-Permissive-2.0); none
is in any default build, and none ships with a world. (b) is
the wrong size for one download. (c) remains the fallback when the network is unavailable: `reshape`
takes any `.dly` file, and the NOTICE records who obtained it.

**Accepted limitations.** WT flags are sparsely reported for some stations and years; the tool reports
their population and the pack then takes fog only from what the record says (TW-g's hourly layer adds
GHCNh). `overcast_morning_permille` cannot be estimated from GHCN-Daily and is carried over from the base
rules by `fit`. The record series adds about 66 KB to every world snapshot (QTWd-2, accepted; retention is
the persistence lane's F-SAVE-1). Windows is argued from `eol=lf`, the CRLF/BOM-tolerant decoder and its
test, not run in CI.

**Note, 2026-10-09 (operator ruling on TWd-F4) — `webpki-roots`' licence.** After this decision was
frozen, `DEP-22`'s `deny.toml` (with `all-features = true`) refused `webpki-roots` (CDLA-Permissive-2.0),
which the `fetch` feature's `ureq` + `rustls` brings. The operator chose a single crate-scoped exception
in `deny.toml` over dropping `fetch` (`DEP-22`'s note of the same date). The tree, the pin and the
confinement above are unchanged.
