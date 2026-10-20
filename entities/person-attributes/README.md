# person-attributes

The built-in attributes a MineWorld person can have — height, hair, voice, temperament, skills,
habits — in the `core` namespace. A world uses them by requiring this Entity Pack; casting, the
clients, the cognition prompt and the decision systems read them by facet.

## What is in it

| Group | Count | Examples |
| --- | --- | --- |
| appearance | 21 | stature, skin tone, hair, glasses, voice |
| background | 15 | age band, pronouns, languages, household |
| psychology | 16 | the five traits, patience, curiosity |
| capability | 13 | twelve skills (`skill.cooking` …), fitness |
| behaviour | 14 | chronotype, walking pace, tipping |
| lifestyle | 14 | diet, caffeine, hobbies, pet |

93 attributes in `attributes/`, five value lists in `vocabularies/`, two masks in `masks/`
(`core`, the default, keeps everything; `appearance` keeps only the appearance group), and the
`mineworld` backend in `backends/`.

## Who reads what

Every mask entry says why the attribute exists, in reader codes:

| Code | Reader | Facet |
| --- | --- | --- |
| C | casting | `casting` |
| OV | what an observer sees | `observed` |
| ME | the cognition prompt | `prompt` |
| CV | how someone speaks | `speech` |
| DM | decisions | `decision` |
| S31 | notices | `notices` |

## Sensitive attributes are private

Natal sex, birth, age in years, ancestry, religion, mass and honesty are `private`: no other
person learns them. Ancestry exists only so casting can draw skin tone and hair from a realistic
population profile; its values follow the US Census ACS table B03002 categories. A world set
elsewhere declares its own.

## More

- The rules: [`docs/MODULE_SPEC.md` §4.1](../../docs/MODULE_SPEC.md)
- The design: [`pr-s30e-builtin-catalogue.md`](../../.structured-coding/plans/mvp0/pr-s30e-builtin-catalogue.md)
- Credits: [`NOTICE`](NOTICE)
