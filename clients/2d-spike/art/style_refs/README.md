# Style conditioning crops

Crops taken from `presentation/mineworld-default/2D/references/`, which are the
source of truth for this style. They exist so generation is conditioned on
committed images rather than on a text description of them: a prompt cannot
carry a style, and adjectives drift between runs where a pinned image does not.

Each crop is chosen to show characters, line weight, palette and the viewing
angle at a readable size, without the whole scene competing for the model's
attention.

| File | From | Shows |
| --- | --- | --- |
| `a_walk_dog.png` | `02_cafe_street.png` | a man mid-stride with a backpack, a dog, paving, a lamppost |
| `b_seated.png` | `02_cafe_street.png` | a seated woman at a café table, parasol, chairs |
| `c_hat_bench.png` | `02_cafe_street.png` | a walking woman in a sun hat, bench, bicycle, planter |
| `d_man_dog.png` | `01_town_square.png` | a man with a dog on the square |

Regenerate with `sips -c HEIGHT WIDTH --cropOffset TOP LEFT` against the plates;
the offsets are in `tools/specs/characters.json` alongside each entry's notes.
