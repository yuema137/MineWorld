## ARC-49 — How AC-8 is measured

**Date** 2026-10-09 · **Status** decided; live from S13 PR 13b · **Approved by** the primary session at
13b's design freeze (step-14 §13, 2026-10-08), the operator for QB-1 and for the all-platforms
requirement · **Relates to** `ARC-23`, `ARC-30`, `ARC-48`, `DEP-17`, `DEP-18`, `DEP-19`,
[`MVP.md`](MVP.md) §9 `AC-8`, [`NETWORKING.md`](NETWORKING.md) §8 · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §13

**Problem.** `AC-8`: "The same World Pack runs on a laptop and inside a cloud Docker container with no
semantic differences." The operator added on 2026-10-08: "we must support every platform: Mac, Linux and
Windows." The project needs an instrument that says, for one commit, whether those platforms produce the
same world, that locates a difference when they do not, and that cannot pass by comparing a platform with
itself.

**Options considered.**

```text
(a) a reference file recorded on the laptop and committed; each platform    every PR that changes a
    checks itself against it inside the required `test` check (step-14      world's facts turns red until
    §3.3 as first drawn)                                                     someone re-records on a Mac;
                                                                             slows a required check
(b) a live comparison: on one workflow run, each platform writes a          chosen
    parity record of the same commit, and a separate job compares them
(c) compare the FNV-1a fingerprint `run` prints                             "for reading; not evidence"
                                                                             (step-08 Q9)
(d) a Rust test with `sha2` (the reserved DEP-19 of the step design)        a dev-dependency and a Rust
                                                                             change for what stdlib Python
                                                                             does on every side
```

**Decision.**
1. **The record.** `scripts/ci_parity.py record` runs the `mineworld` binary (natively with `--binary`,
   or in the runtime image with `--image`) and writes one plain-text record, `key value` per line, `\n`
   line endings on every platform. It holds:
   - `[platform]`: OS, architecture, Rosetta translation (macOS), emulation (Windows), the container
     image and its platform when there is one, and `rustc`'s release;
   - `[source]`: the commit and the worlds, enumerated from every `worlds/*/world.yaml`, never listed by
     hand;
   - per world: the digest of `validate`'s output; `summary-300`, seed 7, 300 days in memory, every line
     but `wall`, digested and also kept verbatim; `summary-30s`, seed 7, 30 days with `--save`, every
     line but the header (it names the host's save path) and `wall`; and the save's tables.
   - Tables: `manifest` (the body decoded, `instance` removed, re-encoded with sorted keys, plus the
     `format` column), and `journal`, `facts` and `snapshots`, each as a row count and a SHA-256 over
     every column of every row in primary-key order, with a digest per 1 000-row chunk to locate a
     difference (`ARC-23`). A row encodes each column: NULL as `0x00`; otherwise `0x01`, then an
     integer as 8 bytes little-endian or a blob as its 8-byte length and its bytes.
   - Nothing else is excluded. `instance` is excluded because it is allocated from the wall clock
     (`ARC-27`); any further exclusion is a reviewed change to this record.
2. **The comparison.** `scripts/ci_parity.py compare` exits 0 only if:
   - **G-1** every record is well-formed;
   - **G-2** every record names the same commit and the same worlds, equal to `worlds/` of that commit;
   - **G-3** the records include Darwin arm64 (not translated, no container), Linux x86_64 in a
     container, and Windows x86_64 (native, not emulated). Records from fewer platforms never pass,
     however equal;
   - **G-4** every record's `rustc` release equals `rust-toolchain.toml`'s channel;
   - **G-5** every world's every key is equal across all records, a Linux arm64 record included.

   On a G-5 failure it names the world, groups the platforms that agree (which places the difference in
   an OS or an architecture), and prints the first differing summary line or the first differing table
   chunk with its key range. `--self-test` checks these verdicts against synthetic records and runs in
   `fast`.
3. **Where it runs.** Four jobs record the same commit: `scenario` (the runtime image, `linux/amd64`, on
   `ubuntu-24.04`), `linux-arm` (the runtime image, `linux/arm64`, on `ubuntu-24.04-arm`), `mac`
   (native, `macos-26`) and `windows` (native, `windows-2025`) (`DEP-19`). `ac8` downloads the four
   records and compares them. It runs `if: always()`, so a missing leg is a red `ac8`, never a skip.
4. **When.** On every push to `main`, on `workflow_dispatch`, and on `scratch/*-scenario` branches
   (planted-difference evidence). Not on pull requests. `ac8` is not a required check; like `scenario`
   it blocks main's health (`ARC-48`). A PR touching `systems/`, `kernel/`, `persistence/` or
   `worldpack/` should dispatch the workflow on its branch before review.
5. **The laptop.** At 13b's acceptance the operator's Mac records the final head and is compared with
   that head's Linux and Windows records (step-14 §13.4.4). That is `AC-8` literally; it is repeated by
   one command whenever doubt arises.

**Why not ourselves / why not the others.** Hashing, SQLite reading and process control are the
standard library's; nothing is built that a mature library provides. (a) moves a duty onto every lane and
onto whoever owns a Mac; (c) is not evidence; (d) adds a dependency for no gain and two hashing
implementations.

**Isolating interface.** The record format and `ci_parity.py`'s two subcommands. A runner label is one
line of the workflow (`DEP-19`). A world joins the record by existing under `worlds/`.

**Accepted limitations.**
- The macOS runner is not the operator's laptop: same OS major and target triple, another machine. The
  laptop comparison at acceptance covers the literal claim.
- `main` is checked after a merge, not before: a PR that introduces a platform difference merges green and
  `ac8` turns red on `main`.
- The long horizon (300-day saves, 1 000-day runs) is S13 PR 13c's nightly work.
