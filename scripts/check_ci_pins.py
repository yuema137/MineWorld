#!/usr/bin/env python3
"""The container's toolchain must be the toolchain the repository pins, and every base image a digest.

`rust-toolchain.toml` is the one source of the Rust version (it says why: `clippy -D warnings` and the
`trybuild` expected output are compiler-sensitive). The `Dockerfile` restates that version as a `rust:`
image tag, and the root `Cargo.toml` restates it as `rust-version`. If they drift, rustup inside the
container silently downloads the toolchain the file asks for on top of the one the image ships, and CI
then measures something the image does not describe. Nobody notices a drifted tag by reading it, so this
check reads it (`docs/DECISIONS.md` DEP-18, step-14 I-S13-3).

It also requires every `FROM`, and every `COPY --from=` that names an image rather than an earlier stage,
to carry an `@sha256:` digest, because a tag alone can be re-pointed
upstream; and it requires the build and runtime stages to name the same Debian release, because the
binary is linked against the build stage's glibc.

Exits non-zero and names every disagreement with the values on both sides.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

# "FROM image[:tag][@sha256:digest] [AS stage]"; stage-to-stage FROMs ("FROM toolchain AS build") have
# no registry image and are recognized by naming an earlier stage.
FROM = re.compile(r"^FROM\s+(?:--platform=\S+\s+)?(?P<image>\S+)(?:\s+AS\s+(?P<stage>\S+))?\s*$", re.I)
DIGEST = re.compile(r"@sha256:[0-9a-f]{64}$")
COPY_FROM = re.compile(r"^COPY\s+(?:--\S+\s+)*?--from=(?P<source>\S+)", re.I)
RUST_TAG = re.compile(
    r"^(?:docker\.io/)?(?:library/)?rust:(?P<version>\d+\.\d+\.\d+)-slim-(?P<debian>[a-z]+)(?:@|$)"
)
DEBIAN_TAG = re.compile(r"^(?:docker\.io/)?(?:library/)?debian:(?P<debian>[a-z]+)-slim(?:@|$)")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    try:
        channel = tomllib.loads((root / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"][
            "channel"
        ]
        manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
        rust_version = manifest["workspace"]["package"]["rust-version"]
        dockerfile = (root / "Dockerfile").read_text(encoding="utf-8").splitlines()
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"cannot read a pin: {error!r}", file=sys.stderr)
        return 2

    problems: list[str] = []
    if rust_version != channel:
        problems.append(
            f"Cargo.toml rust-version is {rust_version}, rust-toolchain.toml channel is {channel}"
        )

    stages: set[str] = set()
    rust_tags: list[tuple[int, str, str]] = []
    debian_tags: list[tuple[int, str]] = []
    for number, line in enumerate(dockerfile, start=1):
        # `COPY --from=<image>` pulls an image as surely as `FROM` does (the toolchain's uv, DEP-26), so
        # it needs a digest too; `COPY --from=<earlier stage>` names no image.
        if copied := COPY_FROM.match(line.strip()):
            source = copied.group("source")
            if source.lower() not in stages and not DIGEST.search(source):
                problems.append(f"Dockerfile:{number}: COPY --from={source} is not pinned by an @sha256: digest")
            continue
        match = FROM.match(line.strip())
        if not match:
            continue
        image, stage = match.group("image"), match.group("stage")
        if image.lower() not in stages:
            if not DIGEST.search(image):
                problems.append(f"Dockerfile:{number}: {image} is not pinned by an @sha256: digest")
            if rust := RUST_TAG.match(image):
                rust_tags.append((number, rust.group("version"), rust.group("debian")))
            elif debian := DEBIAN_TAG.match(image):
                debian_tags.append((number, debian.group("debian")))
            else:
                problems.append(
                    f"Dockerfile:{number}: {image} is neither rust:<version>-slim-<debian> nor "
                    "debian:<debian>-slim, the two bases DEP-18 selects"
                )
        if stage:
            stages.add(stage.lower())

    if not rust_tags:
        problems.append("Dockerfile: no rust:<version>-slim-<debian> base image found")
    for number, version, _ in rust_tags:
        if version != channel:
            problems.append(
                f"Dockerfile:{number}: rust image tag is {version}, rust-toolchain.toml channel is {channel}"
            )
    releases = {debian for _, _, debian in rust_tags} | {debian for _, debian in debian_tags}
    if len(releases) > 1:
        problems.append(f"Dockerfile: base images name different Debian releases: {sorted(releases)}")

    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    print(
        f"toolchain pins agree: rust {channel} (rust-toolchain.toml, Cargo.toml, Dockerfile), "
        f"Debian {releases.pop()}, every base and copied image pinned by digest"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
