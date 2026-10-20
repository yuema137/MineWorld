# systems/attributes/

**`mineworld-attributes`**: a Person's declared attributes — what a world says its people are like.

```text
data        attributes/, vocabularies/   declarations, overrides, uses; value vocabularies
section     attributes                   a person file's values and restrictions
emits       attributes-composed          the world's catalogue, once at genesis
            traits-stated                each person's plain traits, private to them
            attributes-handed            an owner pack's values, handed to it at genesis
            trait-changed                an accepted proposal
owns        Traits                       component `traits`
discloses   traits                       the holder's own view, or another observer's
```

The model — declarations, values, typed handles, rendering, the owner interface — is
[`mineworld-attributes-model`](../../authoring/attributes/). The rules are
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §§3.1, 4.1.
