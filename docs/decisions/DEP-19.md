## DEP-19 — AC-8's non-Linux sides: GitHub-hosted macOS arm64 (`macos-26`) and Windows x86_64 (`windows-2025`) runners

**Date** 2026-10-09 · **Status** selected; integrated in S13 PR 13b · **Approved by** the operator
(QB-1, 2026-10-08: "Use the macOS runner (macos-26). If billing later shows a charge, switch to the
designed fallback"; and "we must support every platform: Mac, Linux and Windows") and the primary session
at 13b's freeze · **Relates to** `ARC-49`, `DEP-17`, `ARC-48` · **Design**
`.structured-coding/plans/mvp0/step-14-ci.md` §§13.0.1, 13.3

Ruling 6 reserved `DEP-19` for `sha2`; 13b needs no `sha2` (`ARC-49` option (d)), and QB-3 gave the
number to this decision.

**Problem.** `AC-8`'s Linux side is the runtime container on hosted `ubuntu-24.04` (`DEP-17`). Its Mac
side, and the Windows side the operator requires, need machines that run on every check.

**Options considered** (step-14 §13.3).

```text
(a) the operator's laptop against a CI Linux record, once         chosen for acceptance, not continuous
(b) a committed laptop reference                                  declined (ARC-49 option (a))
(c) GitHub-hosted macos-26 (arm64)                                chosen: continuous
(c') macos-14                                                     deprecated in runner-images
(d) ubuntu-24.04-arm (Linux arm64) as a localizer                 chosen: places a difference in OS or
                                                                  architecture without Rosetta
(e) macos-26-intel (Darwin x86_64)                                not needed while (c) and (d) localize
(f) Rosetta on the laptop                                         emulation; a diagnostic only
(g) Docker Desktop linux/arm64 on the laptop                      a diagnostic only; (d) covers it
(h) a self-hosted runner on the operator's Mac                    runs fork-PR code on the operator's
                                                                  machine; depends on a laptop being awake
(i) larger macOS runners                                          paid, even on a public repository
(j) another CI service for macOS                                  a second platform for one job
(k) GitHub-hosted windows-2025 (Windows x86_64), native           chosen: the operator's third platform;
                                                                  hosted Windows cannot run the Linux image
```

**Choice.** (c) + (a) + (d) + (k). Labels are pinned (`macos-26`, `windows-2025`, `ubuntu-24.04-arm`),
never `-latest`, so a label migration never silently changes a platform under `AC-8`. The native legs
build with rustup taking `rust-toolchain.toml`'s channel, through the shared composite action
`.github/actions/native`, which names a layer of `scripts/ci_layer.py` and never a command.

**Cost basis.** Standard GitHub-hosted runners, macOS and Windows included, carry no per-minute charge on
a public repository (GitHub's published terms; the session cannot read billing, so this is the operator's
to confirm). No larger runner and no paid service is used.

**Isolating interface.** `ci_parity.py` runs anywhere Python 3 and the binary do. A runner is one line;
the native action is one file.

**Revisit triggers.**
- Billing shows a charge: switch to the designed fallback, (a) + (d) + (k), recorded as a deviation.
- A label is deprecated in `runner-images`: move to the next pinned label.
- A need for a machine GitHub does not host.

**Accepted limitations.** Hosted runners are not the operator's laptop (`ARC-49`). About five macOS jobs
run concurrently on the Free plan, so a busy day delays `ac8`; it never holds a merge.
