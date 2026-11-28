# attributes-matraix

An **optional** person backend: people can be generated from the
[MatrAIx persona schema](https://github.com/MatrAIx-ai/MatrAIx-Persona-8B) (MIT) instead of MineWorld's
built-in attributes. Nothing else in MineWorld depends on it; delete this directory and every reference world
still runs.

**Use it in a world:**

```yaml
# world.yaml
requires: { person-attributes: "^1", attributes-matraix: "^1" }
# configure/attributes.yaml
backend: matraix
```

Then `mineworld attributes check <world> --packs entities`.

**What is here:** `.upstream/` is upstream's schema and label files byte for byte; `attributes/` and
`backends/` are generated from them by `.tools/import.py` (`--check` proves it). Credits: [NOTICE](NOTICE),
[CITATION.cff](CITATION.cff). Specification: [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1.
