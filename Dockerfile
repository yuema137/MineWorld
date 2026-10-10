# MineWorld's one container definition (docs/DECISIONS.md DEP-18, ARC-48).
#
#   toolchain  the CI environment: the pinned Rust toolchain plus git and python3. It holds no source;
#              CI mounts the checkout (with its .git) into it and runs `python3 scripts/ci_layer.py`.
#   build      compiles the `mineworld` binary in release.
#   runtime    hosts a world: the binary and worlds/, nothing else.
#
# Every FROM is pinned by tag and digest. `scripts/check_ci_pins.py` fails CI unless the rust tag
# equals rust-toolchain.toml's channel and the root Cargo.toml's rust-version, every FROM carries a
# digest, and build and runtime name the same Debian release. Write the pins literally (no ARG), so
# that check can read them.

FROM rust:1.97.1-slim-trixie@sha256:8e8cf8f7fd54a2d23d5a743b3a03f56e26b6c774276c33fa0595111704ebb15c AS toolchain
# git: the AC-1 and vocabulary scans read history (ARC-35). python3: scripts/check_*.py, ci_layer.py.
RUN apt-get update \
    && apt-get install --yes --no-install-recommends git python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*
# rust-toolchain.toml's components, installed here so that a non-root CI user never needs to write
# to RUSTUP_HOME.
RUN rustup component add rustfmt clippy
# The code graph's licence, source and ban policy, run by the `fast` layer (DECISIONS.md DEP-22).
# Pinned, built from its own lock; installed into the image's CARGO_HOME bin, which is on PATH.
RUN cargo install --locked cargo-deny@0.20.2 \
    && rm -rf /usr/local/cargo/registry /usr/local/cargo/git
# uv: the Python workspace's one tool (docs/DECISIONS.md DEP-26), copied from its official image and
# pinned by digest like every base image (scripts/check_ci_pins.py). The image's own python3 (Debian's
# 3.13) is the interpreter; uv never downloads another.
COPY --from=ghcr.io/astral-sh/uv:0.12.5@sha256:e85be844203885286c60ffad8a858d48afb6c5a5c237ca0e67f12e74b8f174b1 /uv /uvx /bin/
ENV UV_PYTHON_DOWNLOADS=never
ENV CARGO_TERM_COLOR=always
WORKDIR /work

FROM toolchain AS build
# .dockerignore keeps the context to the Cargo workspace and worlds/.
COPY . /work
RUN cargo build --release --locked -p mineworld-cli

FROM debian:trixie-slim@sha256:a29215f6a35e51e22adffa17f89e9d2ef06214e64a2bad10d765c46aea49f11f AS runtime
RUN useradd --system --create-home --home-dir /home/mineworld mineworld \
    && mkdir -p /var/lib/mineworld \
    && chown mineworld:mineworld /var/lib/mineworld
COPY --from=build /work/target/release/mineworld /usr/local/bin/mineworld
COPY worlds/ /opt/mineworld/worlds/
USER mineworld
WORKDIR /opt/mineworld
VOLUME /var/lib/mineworld
EXPOSE 7878
# The server stops on Ctrl-C (tools/cli/src/main.rs, `serve`); as PID 1 it has no SIGTERM handler,
# so `docker stop` sends the signal it actually handles.
STOPSIGNAL SIGINT
ENTRYPOINT ["mineworld"]
CMD ["server", "worlds/social-cafe", "--listen", "0.0.0.0:7878", "--save", "/var/lib/mineworld/social-cafe"]
