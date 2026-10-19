## DEP-22 — `cargo-deny` checks the code graph's licences, sources and bans in CI

**Date** 2026-10-08 · **Status** selected; integrated in S16 PR E-c (Ec-C7) · **Approved by** the
operator's reuse table (step-16 §5 row 11) and the primary session at E-c's freeze (FQ-c6) ·
**Relates to** `ARC-48`, `ARC-55`, `ARC-66`, `DEP-17`, `DEP-18`, `DEP-21` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §5 row 11, §16.2 PD-29 (S16, PR E-c)

**Problem.** `ARC-55` judges the licence a *pack* declares, at resolution. Nothing judged the licences
of every crate compiled into the binary, and nothing stated which sources a crate may come from — the
policy that keeps a crate published under a framework name by a stranger out of the build (`ARC-66`,
"How it fails, closed").

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions):

```text
(a) cargo-deny (EmbarkStudios; MIT OR Apache-2.0): licences, sources, bans, advisories over a Cargo graph
(b) cargo-about / cargo-license: licence listing only, no source or ban policy
(c) a script of our own over `cargo metadata`
```

**Choice: (a)**, run by CI only — it is not a dependency of any crate. (b) lists licences but enforces
neither sources nor bans, so a second tool would still be needed; (c) is the wheel (a) is.

**Policy (`deny.toml`).**
- `[sources]`: crates.io, and git only from `https://github.com/yuema137/mineworld-pack-fishing`
  (`DEP-23`); anything else is refused.
- `[licenses]`: the allow-list the code graph actually needs — `ARC-55`'s eight, plus only what
  `cargo deny list` shows, each addition named in `deny.toml` with the crates that need it.
- `[bans]`: no wildcard version requirements.
- Advisories are **not** checked: the answer depends on a database fetched at run time, and `ARC-48`
  allows nothing to fail non-deterministically.

**Where it runs.** Installed in the CI image's `toolchain` stage at a pinned version with `--locked`,
and run by the `fast` layer (`scripts/ci_layer.py`): `cargo deny check licenses sources bans`.

**Weight, measured** at integration: recorded in step-16 §16.8 (install time, image size).

**Revisit** when an advisory database can be pinned and read offline, or when the graph needs a licence
outside the list (a reviewed addition here first).

**Note, 2026-10-09 (operator ruling on S19 TW-d's TWd-F4) — one crate-scoped exception.** `deny.toml`
gains `[[licenses.exceptions]] crate = "webpki-roots", allow = ["CDLA-Permissive-2.0"]`. `webpki-roots`
is Mozilla's root-certificate list as data; it is reached only through `ureq`'s `rustls` feature in
`mineworld-weather-fetch`'s off-by-default `fetch` feature (`DEP-31`), a developer tool that no world
build compiles. The exception names that crate only: CDLA-Permissive-2.0 is not added to the allow-list,
and any other crate carrying it is still refused. Every rustls route to a root store carries a CDLA root
list (`webpki-roots` or `webpki-root-certs`), and `native-tls` would need OpenSSL headers in the CI
image (step-19 §18.11 TWd-F4); the alternative, dropping `fetch`, was not chosen.
