# systems/weather/

**`mineworld-weather`**: the world's weather, hour by hour — the same for everybody, and other packs can
react to it.

```text
depends on  calendar (and presence)    a world that enables weather without calendar is refused
configure   configure/weather.yaml     source: rules, a seed, and twelve months of climate (integers)
emits       weather-configured         at genesis; SystemInternal
            weather-day                each local midnight: the day's summary and its 24 hours
            weather-changed            when the condition changes (clear, fog, rain, …); Public
owns        one `climate` Process      its state is the fold of the three facts above
discloses   weather-today, weather-now on the place the observer is in
```

Each day is drawn by a small weather generator (WGEN-lite: wet or dry by a Markov chain, amounts,
temperature, fog and thunder), seeded by the configuration, and spread over the hours from the day's
sunrise. Everything is an integer (0.1 °C, 0.1 mm, per mille), so the weather is the same on every
machine. A world that enables this pack without `configure/weather.yaml` has no weather.

A world can instead replay a station's real weather (`source: record`): a CSV under the World Pack's
`data/`, named in `configure/weather.yaml`, with the rules filling any gap. The tool that makes such a
file from NOAA's data is [`tools/weather-fetch`](../../tools/weather-fetch/README.md).

```sh
cargo test -p mineworld-weather
```

## The record format

```yaml
# configure/weather.yaml
source: record
seed: 19
record:
  station: USW00023188          # provenance only (a GHCN id, at most 32 ASCII characters)
  data: data/weather/san-diego-usw00023188-2015-2024.csv
  first_year: 2015              # the record year the world's first year replays
fill: rules                     # or `none`: refuse a gap longer than 3 days
rules: { months: [ … ] }        # always required: rule days fill gaps, months fill missing wind
```

The CSV is UTF-8 text, one row per day, ascending, whole calendar years (1 January to 31 December), no
duplicate or skipped day. The header is exactly:

```text
date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,wdf2_deg,fog,thunder
2015-01-01,189,72,0,18,320,0,0
```

| Column | Meaning | Range |
| --- | --- | --- |
| `date` | `YYYY-MM-DD` | |
| `tmax_dc`, `tmin_dc` | daily maximum and minimum, 0.1 °C | −900 … 600, or empty |
| `prcp_tenth_mm` | precipitation, 0.1 mm | 0 … 65535, or empty |
| `awnd_dms` | mean wind speed, 0.1 m/s | 0 … 255, or empty |
| `wdf2_deg` | direction of the fastest 2-minute wind, degrees (360 = north) | 0 … 360, or empty |
| `fog`, `thunder` | the day had fog (GHCN WT01, WT02 or WT21) / thunder (WT03) | `0` or `1` |

- An empty cell is a missing value. A day missing `tmax_dc`, `tmin_dc` or `prcp_tenth_mm` is a missing
  day: a run of at most 3 copies the last complete day; a longer run (or one at the very start) is
  drawn by the rules, or refused with `fill: none`, naming its first and last date. A missing wind
  alone takes the month's wind from the rules.
- LF and CRLF line endings and a leading UTF-8 byte-order mark are read identically; a final newline is
  optional. A tab, a quote, any other `\r` or a non-ASCII byte is refused with its line number.
- The world's year *y* replays record year `file_first + ((first_year − file_first) + (y − epoch year))
  mod years`, keeping month and day. A world 29 February in a non-leap record year takes 28 February; a
  record 29 February is used only by a world 29 February.

The decisions are [`ARC-68`](../../docs/DECISIONS.md) (the pack) and `DEP-31` (the data and the tool). Design and evidence:
[`step-19-time-weather.md`](../../.structured-coding/plans/mvp0/step-19-time-weather.md) §6, §17 and §18.
