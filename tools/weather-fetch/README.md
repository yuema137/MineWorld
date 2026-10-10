# tools/weather-fetch/

**`weather-fetch`** turns a weather station's daily record from NOAA into the file a World Pack's
`weather` pack replays, and fits the pack's rules to it. It is a developer tool: no world, test run or
server ever runs it, and nothing a world builds depends on it.

```sh
# 1. Get the station file (once; or download it by hand from the URL it prints)
cargo run -p mineworld-weather-fetch --features fetch -- fetch --station USW00023188 --out target/USW00023188.dly
# 2. Reshape ten years into the CSV and its NOTICE, with a report of gaps and flags
cargo run -p mineworld-weather-fetch -- reshape --input target/USW00023188.dly --station USW00023188 \
    --from 2015 --to 2024 --retrieved 2026-10-09 --out-dir worlds/market-town/data/weather
# 3. Fit the rules that fill the record's gaps
cargo run -p mineworld-weather-fetch -- fit --input worlds/market-town/data/weather/san-diego-usw00023188-2015-2024.csv \
    --base worlds/market-town/configure/weather.yaml --out worlds/market-town/configure/weather.yaml
```

The same input and arguments always give the same bytes. The data is NOAA GHCN-Daily (CC0); the
NOTICE written beside it carries the attribution, the citations and the provenance.

The CSV format is in [`systems/weather/README.md`](../../systems/weather/README.md). The decision is
`DEP-31` in [`docs/DECISIONS.md`](../../docs/DECISIONS.md); design and evidence are in
[`step-19-time-weather.md`](../../.structured-coding/plans/mvp0/step-19-time-weather.md) §18.
