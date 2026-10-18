# systems/persona/

**`mineworld-persona`**: a person's character. How they are, and how they speak.

```text
section     persona              a person file's `persona:` section
emits       persona-authored     at genesis, from that section; private to the person
owns        PersonaComponent     component `persona`, payload { summary, traits, speech }
discloses   PersonaComponent     to its holder only
depends on  nothing
```

An example section, in a person's file:

```yaml
persona:
  summary: Runs the café counter. Warm with regulars, dry with strangers.
  traits: [warm, dry]        # optional; at most 8, each at most 32 bytes
  speech: Short sentences.   # optional; at most 160 bytes
```

A persona shapes speech and grants nothing. No action, process or reaction reads it, and a rule
controller ignores it. A controller that speaks for the person reads it through the person's own
observation, so no other observer sees it.

```sh
cargo test -p mineworld-persona
```

The decision is [`ARC-74`](../../docs/DECISIONS.md). The section format is
[`MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1. Design:
[`pr-s10-p6-lm-controller.md`](../../.structured-coding/plans/mvp0/pr-s10-p6-lm-controller.md) §5.14.
