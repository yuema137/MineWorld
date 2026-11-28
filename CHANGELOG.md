# Changelog

All notable changes to MineWorld are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Every entry cites the pull request that made it.
The version of each section is the workspace version (`Cargo.toml`), and `CITATION.cff` carries the same one.

## [0.1.0] — unreleased

First pre-alpha. Packaged clients for macOS, Windows and Linux are not yet published; see Known issues.

### Added

- Runtime foot grounding for 3D people: each foot finds its ground, and a two-bone IK solver places it (#407).
- A route step may name a point inside a place, so a probe walk can end at a named spot (#494).
- Food energy: a world may choose `hunger_model: food-energy`, so hunger is a kcal reserve (2000 kcal per world
  day by default, configurable per world and per actor class) that food refills by its kcal per serving
  (#555).

### Changed

- The packaged 2D launcher plays Market Town as the visitor instead of Carol. An existing 2D save resumes with
  Carol driven by the town; `seat = "carol"` in `runtime/launch.toml` keeps playing her. A packaged 2D or 3D
  run started alone is now a single-player (`--solo`) world. 2D + 3D and smoke runs are not. Smoke checks
  ignore `launch.toml`. 3D and 2D + 3D stay on Social Café until the packaged 3D Market Town timing passes
  (#561).
- Saves of worlds with bodies made by earlier builds no longer load: bodies is now version 8, and a version-7 save is
  refused by name. Keep the previous build to play them (#557).
- Saves of worlds that enable `needs` made by earlier builds no longer load: keep the previous build to play
  them (#555).
- The MCP surface exposes one `act` tool instead of five, which halves the tool list size (#261).
- Cast base colours ship as Basis UASTC with RDO, and the release stays within its asset budget, with
  textures sized by role (#385, #401).

### Fixed

- A person who steps aside for a walker, or is nudged, is never left in a gap too tight for the route planner, from
  which every later walk was refused `no-route` (#557).
- `run --save` and `server --save` create a new save only after the world has assembled and begun, so a world
  refused while loading no longer leaves an empty save that the next launch fails to resume (#555).
- The café's chairs, stools and ledge are solids; 3D people no longer walk through them (#292).
- `HostConfig.solo` is the single source of truth, and a start-time fast-forward is refused without it (#291).
- The 3D package no longer ships the `res://tools` folder (#512).
- A seat that stands in the street shows the street and its doorways (#496).
- A skin that does not read back whole is retried, then reported, and never indexed (#510).
- The S36 props' contact findings are fixed or recorded as by design (#320).
- Lakeside's variant bodies are created once, the 3D walk goes through intents, and the survey catalog has its
  entry (#465).

### Known issues

- Saves may not load across 0.x versions. Keep a copy of any save you care about.
- Packaged apps are unsigned. macOS asks you to open them with "Open Anyway", and Windows asks for "Run anyway"
  (see `packaging/PLAY.txt`).
