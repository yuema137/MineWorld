## DEP-23 — The first third-party System Pack: `acme-fishing`, pinned by git revision

**Date** 2026-10-08 · **Status** selected; installed in S16 PR E-c · **Approved by** the operator
(QSE-3, QSE-12, QSE-16; FQ-c1, FQ-c4, FQ-c5) and the primary session at E-c's freeze (step-16 §16.0) ·
**Relates to** `ARC-66`, `ARC-26`, `ARC-34`, `ARC-38`, `ARC-55` · **Design**
`.structured-coding/plans/mvp0/step-16-packages.md` §16.2 PD-27 (S16, PR E-c)

**What.** `acme-fishing` 0.1.0, a System Pack with system id `fishing`: a place's `fishing:` section
(`catch: <item key>`, `minutes: 30..=240`) makes it a fishing spot; a living person at a spot who can
carry one more is offered `fish`, a complete affordance with no target (`ARC-34`); a `catch` process ends
after the spot's minutes and, if the angler is still there and can carry one more, states inventory's
`items-produced` through `mineworld_inventory::produce` (`ARC-26`, `ARC-38`). It depends on `presence` and
`inventory`, discloses nothing, takes no configuration. Its law is step-16 §16.2's table.

**Where.** <https://github.com/yuema137/mineworld-pack-fishing>, branch `main`, a separate public
repository. The build installs it by `ARC-66`'s two lines; the revision is stated once, on its line in
`systems/installed/Cargo.toml` (the tag `v0.1.0` names the same commit, informatively).

**Why this and not a bundled pack.** It is the evidence `ARC-66` exists for: a pack written only
against the published surface, in a repository that is not this one, with its own tests and CI on Linux,
macOS and Windows, compiled into this build without a path into its source.

**Licence.** MIT (its `Cargo.toml` and `LICENSE`), on `ARC-55`'s default allow-list. Its normal
dependencies outside the published surface are `serde` and `serde_json`, both already in this build's
lock: installing it adds no registry crate to the graph.

**Who maintains it.** The MineWorld maintainers (the repository's owner is this project's operator). A
framework PR that breaks it updates it in its repository and the `rev` here, in the same review
(`ARC-66` point 8).

**Upgrade.** Commit to its repository; change the `rev` on its line; `cargo update -p acme-fishing`
regenerates the lock; a world whose `requires:` range no longer admits the new version is refused naming
both.

**Removal.** Delete its two lines; the lock is regenerated. A world enabling `fishing` is then refused
`UnknownSystem`, listing the systems the build provides; every other world is unaffected.
