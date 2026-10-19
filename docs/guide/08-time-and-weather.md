# 08 Time and weather

In this chapter you put Corner Shop on a calendar and on the map, and give it weather. The world then
has a date, a sunrise and a sunset, and rain or sun hour by hour.

<!-- guide: continues 07-rules-of-the-world -->

You keep working in the `corner-shop` folder from [chapter 07](07-rules-of-the-world.md).

Two System Packs do this:

- `calendar` gives the world a date and a place on Earth, which sets when the sun rises and sets.
- `weather` gives it weather. It needs `calendar`, because the weather depends on the date.

### Step 8.1 — Add the calendar and the weather

**Do this.** Replace `corner-shop/world.yaml` with this whole file. The new parts are `calendar` and
`weather`, at the end of `systems` and at the end of `configure`:

```yaml file=corner-shop/world.yaml
# Corner Shop: the world this guide builds.
world:
  id: corner-shop
  name: Corner Shop
  version: 0.1.0
  license: MIT

mineworld: "^0.1"

systems:
  - presence
  - movement
  - conversation
  - relationships
  - naming
  - schedule
  - item
  - inventory
  - economy
  - employment
  - consumption
  - calendar
  - weather

places:
  - home
  - shop

population:
  - ana
  - ben
  - cai

items:
  - bread
  - tea

organizations:
  - corner-shop-co

seats:
  - ana
  - ben
  - cai

configure:
  - classes
  - conversation
  - calendar
  - weather
```

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'calendar', but corner-shop/configure/calendar.yaml does not exist
```

**You should see** the check ask for the calendar's file.

### Step 8.2 — Say when and where the world is

**Do this.** In `corner-shop/configure`, write `calendar.yaml`:

```yaml file=corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2026-10-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

- `epoch` is the date of the world's first day.
- `utc_offset` is the local clock's difference from UTC. It never changes: there is no summer time.
- `latitude` and `longitude` are where the world is on Earth, in degrees. These are San Diego's.

The fields are in the reference: [`configure/calendar.yaml`](../reference/configure.md#configurecalendaryaml).

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'weather', but corner-shop/configure/weather.yaml does not exist
```

**You should see** the check ask for the weather's file.

### Step 8.3 — Give it San Diego's climate

The weather is drawn every day from a **climate**: for each month, how often it rains and how much,
how warm days and nights are, how often there is fog, and the wind. The weather is part of the world,
so the same world always has the same weather.

**Do this.** In `corner-shop/configure`, write `weather.yaml`. It is long: twelve months, all the
same shape. Copy it whole.

```yaml file=corner-shop/configure/weather.yaml
# San Diego's climate, month by month: the rules MineWorld's weather-fetch tool fitted to NOAA's
# daily record of station USW00023188, 2015 to 2024 (public domain, CC0-1.0). Each day is drawn
# from these rules with `seed`.
source: rules
seed: 19
rules:
  months:
    - # January
      p_wet_after_dry: 143
      p_wet_after_wet: 521
      rain_tenth_mm: [5, 13, 38, 74, 241]
      tmax_dc: 192
      tmin_dc: 100
      t_noise_dc: 34
      t_ar_permille: 718
      wet_tmax_shift_dc: -20
      fog_permille: 452
      thunder_permille: 70
      overcast_morning_permille: 150
      wind_dms: 19
      wind_from_deg: 300
    - # February
      p_wet_after_dry: 115
      p_wet_after_wet: 509
      rain_tenth_mm: [5, 13, 25, 64, 224]
      tmax_dc: 197
      tmin_dc: 104
      t_noise_dc: 36
      t_ar_permille: 766
      wet_tmax_shift_dc: -32
      fog_permille: 314
      thunder_permille: 109
      overcast_morning_permille: 150
      wind_dms: 23
      wind_from_deg: 300
    - # March
      p_wet_after_dry: 169
      p_wet_after_wet: 494
      rain_tenth_mm: [3, 10, 30, 58, 124]
      tmax_dc: 198
      tmin_dc: 123
      t_noise_dc: 33
      t_ar_permille: 708
      wet_tmax_shift_dc: -20
      fog_permille: 345
      thunder_permille: 141
      overcast_morning_permille: 250
      wind_dms: 26
      wind_from_deg: 280
    - # April
      p_wet_after_dry: 67
      p_wet_after_wet: 452
      rain_tenth_mm: [3, 3, 8, 18, 94]
      tmax_dc: 210
      tmin_dc: 141
      t_noise_dc: 33
      t_ar_permille: 673
      wet_tmax_shift_dc: -26
      fog_permille: 263
      thunder_permille: 0
      overcast_morning_permille: 400
      wind_dms: 28
      wind_from_deg: 280
    - # May
      p_wet_after_dry: 81
      p_wet_after_wet: 368
      rain_tenth_mm: [3, 5, 10, 23, 86]
      tmax_dc: 204
      tmin_dc: 155
      t_noise_dc: 22
      t_ar_permille: 663
      wet_tmax_shift_dc: -13
      fog_permille: 184
      thunder_permille: 56
      overcast_morning_permille: 600
      wind_dms: 30
      wind_from_deg: 280
    - # June
      p_wet_after_dry: 24
      p_wet_after_wet: 0
      rain_tenth_mm: [3, 5, 8, 10, 20]
      tmax_dc: 223
      tmin_dc: 172
      t_noise_dc: 23
      t_ar_permille: 680
      wet_tmax_shift_dc: -8
      fog_permille: 310
      thunder_permille: 286
      overcast_morning_permille: 650
      wind_dms: 28
      wind_from_deg: 280
    - # July
      p_wet_after_dry: 7
      p_wet_after_wet: 400
      rain_tenth_mm: [3, 3, 3, 168, 262]
      tmax_dc: 247
      tmin_dc: 194
      t_noise_dc: 20
      t_ar_permille: 724
      wet_tmax_shift_dc: 35
      fog_permille: 201
      thunder_permille: 250
      overcast_morning_permille: 450
      wind_dms: 27
      wind_from_deg: 280
    - # August
      p_wet_after_dry: 13
      p_wet_after_wet: 200
      rain_tenth_mm: [3, 5, 5, 58, 462]
      tmax_dc: 260
      tmin_dc: 204
      t_noise_dc: 19
      t_ar_permille: 784
      wet_tmax_shift_dc: -1
      fog_permille: 219
      thunder_permille: 200
      overcast_morning_permille: 350
      wind_dms: 27
      wind_from_deg: 290
    - # September
      p_wet_after_dry: 39
      p_wet_after_wet: 353
      rain_tenth_mm: [3, 5, 8, 18, 160]
      tmax_dc: 261
      tmin_dc: 197
      t_noise_dc: 28
      t_ar_permille: 792
      wet_tmax_shift_dc: 4
      fog_permille: 253
      thunder_permille: 176
      overcast_morning_permille: 300
      wind_dms: 25
      wind_from_deg: 290
    - # October
      p_wet_after_dry: 48
      p_wet_after_wet: 176
      rain_tenth_mm: [3, 8, 23, 41, 119]
      tmax_dc: 249
      tmin_dc: 166
      t_noise_dc: 35
      t_ar_permille: 758
      wet_tmax_shift_dc: -19
      fog_permille: 329
      thunder_permille: 353
      overcast_morning_permille: 250
      wind_dms: 21
      wind_from_deg: 300
    - # November
      p_wet_after_dry: 72
      p_wet_after_wet: 556
      rain_tenth_mm: [3, 15, 28, 38, 165]
      tmax_dc: 224
      tmin_dc: 122
      t_noise_dc: 40
      t_ar_permille: 682
      wet_tmax_shift_dc: -30
      fog_permille: 370
      thunder_permille: 77
      overcast_morning_permille: 150
      wind_dms: 19
      wind_from_deg: 300
    - # December
      p_wet_after_dry: 113
      p_wet_after_wet: 472
      rain_tenth_mm: [3, 13, 43, 114, 249]
      tmax_dc: 194
      tmin_dc: 98
      t_noise_dc: 37
      t_ar_permille: 604
      wet_tmax_shift_dc: -21
      fog_permille: 461
      thunder_permille: 93
      overcast_morning_permille: 150
      wind_dms: 18
      wind_from_deg: 300
```

- `source: rules` draws every day from the climate. (A world can also replay a weather station's
  real record; see the reference.)
- `seed` picks the weather's dice. It is not the run's `--seed`: the weather stays the same however
  people behave.
- Under `rules`, `months` has one entry for each month, January first. The numbers are whole
  numbers in small units: October's `tmax_dc: 249` says the mean daily high is 24.9 °C.

These numbers come from the same NOAA record as the sample town in MineWorld's source. NOAA asks to be
credited; the comment at the top of the file does that. Every field, with its unit, is in the
reference: [`configure/weather.yaml`](../reference/configure.md#configureweatheryaml).

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule, item, inventory, economy, employment, consumption, calendar, weather
...
corner-shop is a valid World Pack.
```

**You should see** `calendar, weather` at the end of the systems, and a valid world.

### Step 8.4 — Run three days and keep them

**Do this.**

```console
$ mineworld run corner-shop --headless --seed 1 --days 3 --save saves/weather
[mineworld] run Corner Shop (corner-shop): seats ana, ben, cai; seed 1; to day 3; pace 900s; created saves/weather/world.sqlite
...
facts      day-began 4
facts      daylight-changed 18
...
facts      weather-changed [..]
facts      weather-configured 1
facts      weather-day 4
...
faults     0
...
```

**You should see** new facts in the summary:

- `day-began`: a new day started, with its date. Four, because the run ends at the start of day 4.
- `daylight-changed`: dawn, sunrise, sunset and dusk.
- `weather-day` and `weather-changed`: each day's weather, and each change during a day.

### Step 8.5 — Read the date and the weather

Everybody in the world perceives the date and the weather. `perceived` lists what one person
perceived, read from the save. With `--json` it prints every detail, one fact per line.

**Do this.** Read what Ben perceived:

```console
$ mineworld perceived corner-shop --save saves/weather --person ben --json
...
{"id":"[..]","at":0,"event_type":"day-began",[..]"date":{"day":8,"month":10,"year":2026}[..]
...
{"id":"[..]","at":0,"event_type":"weather-changed",[..]"condition":"clear"[..]
...
```

**You should see**, among many lines, the first day beginning on 8 October 2026, and the first
weather of the day: `"condition":"clear"`. `"at":0` means midnight of day 1.

## Why it works this way

Time of day, the date, the sun and the weather are each owned by one System Pack, which records them
as facts. Other packs and clients read those facts; nothing else computes the weather. A world
without `calendar` and `weather` simply has no date and no weather. The rules are in
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#17-time).

Next: [09 Who decides](09-who-decides.md).
