## ARC-30 — The development profile is optimized at level 1, with its debug checks stated explicitly

**Date** 2026-10-07 · **Relates to** [`MVP.md`](MVP.md) §9 `AC-11`, `AC-12`; `ARC-27`;
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§15–16 · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` §4.2.5 QB-3, §4.2.6 (S8, PR 10b)

**Problem.** MineWorld's strongest evidence is real worlds run for hundreds of simulated days:
`AC-11`/`AC-12` run `social-cafe` for 300 days three times and replay the save. In the default debug
profile (level 0), the default test loop took 272 s on `main @ 0592b3e`, 264 s of it in four
real-lifecycle test binaries, and 10b adds two systems to every run. A slow default loop gets run less,
and the long tests are the first a contributor is tempted to skip.

**Options considered.**

```text
(A) [profile.dev] opt-level = 1        every test stays in the default loop; runs get ~6× faster
(B) #[ignore] the 300-day test behind  keeps level 0, but the slowest and most important evidence
    a named gate command               becomes the easiest to skip
(C) neither                            a default loop of 6–7 minutes after 10b
```

**Choice: (A).** `[profile.dev]` sets `opt-level = 1` for the whole workspace and states
`debug-assertions = true` and `overflow-checks = true` explicitly. Those two are what make a debug build
a debug build. Stating them means a later change to the level cannot silently turn them off.

**Evidence** (this machine, `step-09-social.md` §9 E-B0, E-B8):
- `mineworld run worlds/social-cafe --headless --seed 7 --days 300` in memory: 50.3 s at level 0,
  8.4 s at level 1.
- Every printed line but the header and `wall` is identical (`diff` empty; 327 540 facts, fingerprint
  `fd0fe804108e9bf0` both). The level changes speed, not behaviour, as expected of integer-only
  simulation code (no float anywhere in the workspace).
- A clean `cargo build --workspace --all-targets`: 15.1 s at level 0, 39.4 s at level 1.

**Why it is safe.**
- The determinism claims (`AC-12`) are about the pipeline's semantics, which no optimization level may
  change in safe Rust without floating point.
- Overflow is still checked, and debug assertions still fire.
- Release builds are unaffected.
- No test left the default loop, and none changed.

**Accepted limitations.**
- A slower clean build (above).
- A debugger sees some values optimized out; a contributor who needs level 0 for one session sets
  `CARGO_PROFILE_DEV_OPT_LEVEL=0`.
