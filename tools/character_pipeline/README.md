# Character pipeline

This folder turns one generated character mesh and its `fit.yaml` into a game-ready, checked character. It is the
tooling behind the protagonist's character (route D+), moved into the repository as versioned, parameterized code.

Check the committed characters:

```sh
python3 tools/character_pipeline/cast.py check-characters
```

Re-run a character's Blender stages (needs Blender 5.2.2 LTS and a scratch folder outside the repository):

```sh
python3 tools/character_pipeline/cast.py run --fit <fit.yaml> --download <download-folder>
```

The full specification, every command, and every gate bound is in [PIPELINE.md](PIPELINE.md).

Licences: the driver (`mwcast/`, `cast.py`) is MIT. The Blender scripts in `blender/` are GPL-2.0-or-later, with their
own `LICENSE`; they run as separate Blender processes and are not imported by the driver.
