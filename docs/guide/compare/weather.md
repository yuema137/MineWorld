# Weather

On this page Corner Shop gets San Diego's weather, as in chapter 08. Then you change one thing: the
season the world starts in. You run the same week twice and compare the weather, and what people do.

### Step 1.1 — Start from the finished Corner Shop

**Do this.** In your worlds folder, make a folder `compare-weather`. Copy the finished Corner Shop of
chapter 05 into it, from the folder you cloned in chapter 01. On macOS and Linux:

```sh
mkdir compare-weather
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-weather/
```

On Windows, in PowerShell:

```powershell
mkdir compare-weather
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-weather
```

<!-- guide: example docs/guide/examples/corner-shop compare-weather/corner-shop -->

Then check the copy:

```console
$ mineworld validate compare-weather/corner-shop
...
compare-weather/corner-shop is a valid World Pack.
```

**You should see** that the copy is a valid world.

### Step 1.2 — Add the calendar and the weather

**Do this.** Replace `compare-weather/corner-shop/world.yaml` with this whole file. The new parts are
`calendar` and `weather`, at the end of `systems`, and the `configure` list:

```yaml file=compare-weather/corner-shop/world.yaml
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
  - calendar
  - weather
```

```console exit=1
$ mineworld validate compare-weather/corner-shop
[mineworld] configure: names 'calendar', but compare-weather/corner-shop/configure/calendar.yaml does not exist
```

**You should see** the check ask for the calendar's file.

### Step 1.3 — Start the world on 8 October

**Do this.** Make a folder `compare-weather/corner-shop/configure`. In it, write `calendar.yaml`:

```yaml file=compare-weather/corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2026-10-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

`epoch` is the date of the world's first day. The fields are explained in
[chapter 08](../08-time-and-weather.md) and listed in the reference:
[`configure/calendar.yaml`](../../reference/configure.md#configurecalendaryaml).

```console exit=1
$ mineworld validate compare-weather/corner-shop
[mineworld] configure: names 'weather', but compare-weather/corner-shop/configure/weather.yaml does not exist
```

**You should see** the check ask for the weather's file.

### Step 1.4 — Give it San Diego's climate

**Do this.** In `compare-weather/corner-shop/configure`, write `weather.yaml`. It is the same file as in
chapter 08: twelve months of San Diego's climate. Copy it whole.

```yaml file=compare-weather/corner-shop/configure/weather.yaml
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

Every field, with its unit, is in the reference:
[`configure/weather.yaml`](../../reference/configure.md#configureweatheryaml).

```console
$ mineworld validate compare-weather/corner-shop
...
compare-weather/corner-shop is a valid World Pack.
```

**You should see** a valid world.

### Step 1.5 — Run one week in October

**Do this.**

```console
$ mineworld run compare-weather/corner-shop --headless --seed 1 --days 7
...
facts      weather-changed 17
...
activity   days 1-7  ana move 363 talk 196  ben move 380 talk 267  cai move 389 talk 243
...
```

**You should see** these numbers. `weather-changed` counts each time the weather changed: clear to fog,
fog to overcast, overcast to rain, and so on.

### Step 1.6 — Change this: start the world in January

**Do this.** Replace `compare-weather/corner-shop/configure/calendar.yaml` with this whole file. Only
`epoch` changed:

```yaml file=compare-weather/corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2027-01-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

Then run the same week with the same seed:

```console
$ mineworld run compare-weather/corner-shop --headless --seed 1 --days 7
...
facts      weather-changed 24
...
activity   days 1-7  ana move 363 talk 196  ben move 380 talk 267  cai move 389 talk 243
...
```

**You should see** these numbers.

### The world changes like this

| | Step 1.5, October | Step 1.6, January |
| --- | --- | --- |
| `weather-changed` | 17 | 24 |
| `activity` | the same | the same |

Winter is San Diego's rainy season. The climate file says so: January's `p_wet_after_dry` is 143 per
mille, October's 48. So the same place, with the same weather `seed`, has more rain and more
changes of weather in January.

What people do does not change at all. Everyone here is driven by the `rule` decision model, which does
not look at the weather ([chapter 09](../09-who-decides.md)). The weather is still part of the world:
every person perceives it, and a player sees it.

Back to [all comparisons](README.md).
