# populations-us

Cited population profiles for worlds set in the United States. A profile tells the attribute sampler how
likely each built-in `core` attribute value is, so a world's unstated people look like the place they live in.

| Profile | Covers | File |
| --- | --- | --- |
| `san-diego-county` (version 3) | adults | [`populations/san-diego-county.yaml`](populations/san-diego-county.yaml) |

## Data and conventions

- **Data** — sex, age band and ancestry come from the American Community Survey 2024 1-year estimates
  (tables B01001 and B03002); stature and build from NHANES 2015–2018 (Fryar et al. 2021). Each figure
  names its source, the date it was read and the table cell beside it.
- **Conventions** — skin-tone ranges, hair texture, colour and style, facial hair and face shapes are
  authored casting conventions, marked `authored`, never presented as data.
- **Still to verify** — a figure not read from its source says `verify` and why. Build is national
  (NHANES), not the county's own figure.

The ancestry values are the census's own categories, specific to the US. A world elsewhere declares its own.
The skin-tone levels follow the Monk Skin Tone scale as a reference; no swatch or text of it is copied.

## Using it

Require `person-attributes` and `populations-us` in the world, then name the profile in
`configure/attributes.yaml`:

```yaml
format: 1
profile: san-diego-county
seed: <your seed>
```

Lakeside uses this profile (operator ruling F-CAST-2, 2026-10-15). The rules are in
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1.
