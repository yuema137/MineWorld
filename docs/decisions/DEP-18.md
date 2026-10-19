## DEP-18 — Container images: the official `rust` slim image to build, Debian slim to run, both pinned by digest

**Date** 2026-10-08 · **Status** selected; integrated in S13 PR 13a · **Approved by** the primary session
at 13a's design freeze (step-14 §9.0) · **Relates to** `DEP-17`, `ARC-48`,
[`ARCHITECTURE.md`](ARCHITECTURE.md) §§11, 13, [`NETWORKING.md`](NETWORKING.md) §§7–8,
[`MVP.md`](MVP.md) §9 `AC-8` · **Design** `.structured-coding/plans/mvp0/step-14-ci.md` §§3.1–3.2, 5.5

**Problem.** `NETWORKING.md` §7 deploys a world as a Docker container on a VPS, with a persistent volume.
`AC-8` requires the container to run the laptop's World Pack with no semantic difference. CI needs the
same environment for its layers (`DEP-17`). One `Dockerfile` serves all three.

**Options considered.**

```text
build base
(a) rust:1.97.1-slim-trixie (Docker Official Image)        chosen: the exact toolchain as a tag, rustup
                                                           inside, glibc, gcc for libsqlite3-sys
(b) rust:1.97.1-trixie (full)                              the same, with ~0.5 GB of unused packages
(c) musl static (rust:alpine, cargo-zigbuild) to scratch   a third target nobody else runs; musl's
                                                           allocator and libm untested against Rapier
runtime base
(d) debian:trixie-slim                                     chosen: the build stage's glibc; a shell, so
                                                           an operator can `docker exec … mineworld inspect`
(e) gcr.io/distroless/cc-debian13                          smaller, no shell; the planned hardening swap
                                                           once a hosted deployment exists (QS13-10)
(f) cgr.dev/chainguard/glibc-dynamic                       the free tier offers only `latest`, so a
                                                           pinned digest may be garbage-collected
(g) our own base image                                     nothing to gain
```

**Choice.** One `Dockerfile` at the repository root, with three stages.
- **`toolchain`** is `rust:1.97.1-slim-trixie` plus `git`, `python3`, `ca-certificates`, `rustfmt` and
  `clippy`. It copies no source; CI mounts the checkout into it.
- **`build`** copies the Cargo workspace and runs `cargo build --release --locked -p mineworld-cli`.
- **`runtime`** is `debian:trixie-slim` with the `mineworld` binary and `worlds/`.
  - It runs as the non-root user `mineworld`, keeps worlds on a volume at `/var/lib/mineworld`, and
    exposes 7878.
  - Its default command hosts `worlds/social-cafe` with a save on that volume.
  - It stops with `SIGINT`: the server's own stop signal (`tools/cli/src/main.rs`, `serve`), so `docker
    stop` shuts the world down the way an operator's Ctrl-C does.

Every `FROM` names its tag **and** its `@sha256:` digest. `scripts/check_ci_pins.py`, in CI's `fast`
layer, fails unless:
- the `rust:` tag equals `rust-toolchain.toml`'s channel and the root manifest's `rust-version`;
- every `FROM` carries a digest;
- the build and runtime stages name the same Debian release.

**Why not ourselves.** Base images are commodity infrastructure, maintained upstream with security
updates.

**Why not the others.**
- (b): size.
- (c): it would test a different target from the one deployed.
- (e): deferred, not declined. It is a one-line `FROM` swap.
- (f): reproducibility.

**Isolating interface.** The runtime image's interface is `mineworld` plus its ordinary arguments, a
volume at `/var/lib/mineworld`, and port 7878. MineWorld has no container-only flag, environment switch,
`cfg` or code path (I-S13-5). A compose file, a VPS unit or a Kubernetes manifest consumes the image
without changing it. The toolchain version has one source, `rust-toolchain.toml`; the image tag follows it
under `check_ci_pins.py`.

**Accepted limitations.**
- **Digests age.** A digest pins an image that stops receiving Debian security updates. Re-pinning is a
  deliberate, reviewed change. The pinned digests and their dates are in step-14's 13a ledger.
- **The runtime image's default command predates S11's invite token.** S11 adds the argument (QS13-15).
- **No published image.** 13a builds the image in CI and locally. A registry is a later decision.
