# 天气

这一页给 Corner Shop 加上圣迭戈的天气，和第 08 章一样。然后只改一处：世界从哪个季节开始。
你把同一个星期跑两次，比一比天气，也比一比大家做了什么。

这一页翻译自英文原文 [weather.md](weather.md)。两边不一致时，以英文为准。

### Step 1.1 — 从做完的 Corner Shop 开始

**照着做。** 在放世界的文件夹里新建文件夹 `compare-weather`。把第 05 章做完的 Corner Shop 复制进去，
它在你第 01 章 clone 下来的文件夹里。macOS 和 Linux 上这样写：

<!-- guide: equals compare-weather/corner-shop docs/guide/examples/corner-shop -->

```sh
mkdir compare-weather
cp -R "$HOME/MineWorld/docs/guide/examples/corner-shop" compare-weather/
```

Windows 上，在 PowerShell 里这样写：

<!-- guide: equals compare-weather/corner-shop docs/guide/examples/corner-shop -->

```powershell
mkdir compare-weather
Copy-Item -Recurse "$HOME\MineWorld\docs\guide\examples\corner-shop" compare-weather
```

然后检查一下副本：

```console
$ mineworld validate compare-weather/corner-shop
...
compare-weather/corner-shop is a valid World Pack.
```

**你应该看到**副本是一个写对了的世界。

### Step 1.2 — 加上日历和天气

**照着做。** 用下面这整份文件替换 `compare-weather/corner-shop/world.yaml`。新加的是 `systems` 最后的
`calendar` 和 `weather`，还有 `configure` 这个列表：

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

# Not a town: a town names its station hall and its landmark (docs/MODULE_SPEC.md §4.3).
settlement:
  kind: none
```

```console exit=1
$ mineworld validate compare-weather/corner-shop
[mineworld] configure: names 'calendar', but compare-weather/corner-shop/configure/calendar.yaml does not exist
```

**你应该看到**检查在要日历的文件。

### Step 1.3 — 让世界从 10 月 8 日开始

**照着做。** 新建文件夹 `compare-weather/corner-shop/configure`，在里面写 `calendar.yaml`：

```yaml file=compare-weather/corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2026-10-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

`epoch` 是世界第一天的日期。这几个字段在[第 08 章](../08-time-and-weather.zh-CN.md)讲过，参考手册里也有：
[`configure/calendar.yaml`](../../reference/configure.md#configurecalendaryaml)。

```console exit=1
$ mineworld validate compare-weather/corner-shop
[mineworld] configure: names 'weather', but compare-weather/corner-shop/configure/weather.yaml does not exist
```

**你应该看到**检查在要天气的文件。

### Step 1.4 — 给它圣迭戈的气候

**照着做。** 在 `compare-weather/corner-shop/configure` 里写 `weather.yaml`。它和第 08 章的文件一模一样：
圣迭戈十二个月的气候。整份复制过去。

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

每个字段和它的单位都在参考手册里：
[`configure/weather.yaml`](../../reference/configure.md#configureweatheryaml)。

```console
$ mineworld validate compare-weather/corner-shop
...
compare-weather/corner-shop is a valid World Pack.
```

**你应该看到**世界写对了。

### Step 1.5 — 跑十月的一个星期

**照着做。**

```console
$ mineworld run compare-weather/corner-shop --headless --seed 1 --days 7
...
facts      weather-changed 17
...
activity   days 1-7  ana move 363 talk 196  ben move 380 talk 267  cai move 389 talk 243
...
```

**你应该看到**这几个数字。`weather-changed` 数的是天气变了几次：晴转雾，雾转阴，阴转雨，等等。

### Step 1.6 — 改这一处：让世界从一月开始

**照着做。** 用下面这整份文件替换 `compare-weather/corner-shop/configure/calendar.yaml`。只改了 `epoch`：

```yaml file=compare-weather/corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2027-01-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

然后用同一个 seed 跑同样的一个星期：

```console
$ mineworld run compare-weather/corner-shop --headless --seed 1 --days 7
...
facts      weather-changed 24
...
activity   days 1-7  ana move 363 talk 196  ben move 380 talk 267  cai move 389 talk 243
...
```

**你应该看到**这几个数字。

### 世界这样变了

| | Step 1.5，十月 | Step 1.6，一月 |
| --- | --- | --- |
| `weather-changed` | 17 | 24 |
| `activity` | 一样 | 一样 |

冬天是圣迭戈的雨季，气候文件里就写着：一月的 `p_wet_after_dry` 是千分之 143，十月是千分之 48。
所以同一个地方，用同一个天气 `seed`，一月下雨更多，天气也变得更勤。

大家做的事一点没变。这里每个人都由 `rule` 这个 decision model 驱动，它不看天气
（[第 09 章](../09-who-decides.zh-CN.md)）。天气照样是世界的一部分：每个人都感知得到，玩家也看得见。

回到[所有对比](README.zh-CN.md)。
