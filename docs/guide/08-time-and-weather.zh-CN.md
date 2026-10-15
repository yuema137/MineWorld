# 08 时间和天气

这一章你会把街角小店放进日历、放到地图上，再给它天气。这样世界就有了日期、日出日落，
每个小时也有晴有雨。

本页是英文原文 [08-time-and-weather.md](08-time-and-weather.md) 的中文镜像。两者不一致时，以英文为准。

<!-- guide: continues 07-rules-of-the-world -->

接着用[第 07 章](07-rules-of-the-world.zh-CN.md)的 `corner-shop` 文件夹。

这件事由两个系统包来做：

- `calendar`（日历）：给世界一个日期和一个在地球上的位置，由它决定太阳什么时候升起、什么时候落下。
- `weather`（天气）：给世界天气。它要依赖 `calendar`，因为天气跟日期有关。

### Step 8.1 — 加上日历和天气

**照着做。** 用下面这整个文件替换 `corner-shop/world.yaml`。新的部分是 `calendar` 和 `weather`，
分别加在 `systems` 的最后和 `configure` 的最后：

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

**你应该看到**检查在要日历的文件。

### Step 8.2 — 说明世界在什么时候、什么地方

**照着做。** 在 `corner-shop/configure` 里写 `calendar.yaml`：

```yaml file=corner-shop/configure/calendar.yaml
# San Diego, California.
epoch: "2026-10-08"
utc_offset: "-08:00"
latitude: 32.7157
longitude: -117.1611
```

- `epoch`：世界第一天的日期。
- `utc_offset`：当地时间和 UTC 差多少。它永远不变：没有夏令时。
- `latitude` 和 `longitude`：世界在地球上的纬度和经度，单位是度。这里是圣迭戈（San Diego）的。

字段在参考手册里：[`configure/calendar.yaml`](../reference/configure.md#configurecalendaryaml)（英文）。

```console exit=1
$ mineworld validate corner-shop
[mineworld] configure: names 'weather', but corner-shop/configure/weather.yaml does not exist
```

**你应该看到**检查在要天气的文件。

### Step 8.3 — 给它圣迭戈的气候

天气每天都按一份**气候**（climate）来抽：每个月多常下雨、下多少，白天和夜里多暖和，多常起雾，风怎么吹。
天气是世界的一部分，所以同一个世界的天气永远一样。

**照着做。** 在 `corner-shop/configure` 里写 `weather.yaml`。它很长：十二个月，每个月格式都一样。整段复制就好。

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

- `source: rules`：每天都按气候来抽。（世界也可以重放某个气象站的真实记录，见参考手册。）
- `seed`：天气的“骰子”。它不是运行时的 `--seed`：不管人们怎么活动，天气都不变。
- `rules` 下面的 `months`，每个月一项，从一月开始。数字都是小单位的整数：
  十月的 `tmax_dc: 249` 表示平均每日最高气温是 24.9 °C。

这些数字和 MineWorld 源代码里示例小镇用的是同一份 NOAA 记录。NOAA 希望使用者注明出处，
文件开头的注释就是在注明。每个字段和它的单位，都在参考手册里：
[`configure/weather.yaml`](../reference/configure.md#configureweatheryaml)（英文）。

```console
$ mineworld validate corner-shop
Corner Shop (corner-shop)
  systems    presence, movement, conversation, relationships, naming, schedule, item, inventory, economy, employment, consumption, calendar, weather
...
corner-shop is a valid World Pack.
```

**你应该看到** systems 的最后是 `calendar, weather`，世界有效。

### Step 8.4 — 跑三天，并且存下来

**照着做。**

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

**你应该看到**总结里多了几种事实：

- `day-began`：新的一天开始了，带着它的日期。一共四次，因为运行结束在第 4 天刚开始的时候。
- `daylight-changed`：黎明、日出、日落、黄昏。
- `weather-day` 和 `weather-changed`：每天的天气，以及一天里每次天气变化。

### Step 8.5 — 读出日期和天气

世界里每个人都能感知到日期和天气。`perceived` 列出某个人感知到的事，从存档里读出来。
加上 `--json`，它会打印出每个细节，一行一件事。

**照着做。** 读出 Ben 感知到的事：

```console
$ mineworld perceived corner-shop --save saves/weather --person ben --json
...
{"id":"[..]","at":0,"event_type":"day-began",[..]"date":{"day":8,"month":10,"year":2026}[..]
...
{"id":"[..]","at":0,"event_type":"weather-changed",[..]"condition":"clear"[..]
...
```

**你应该看到**，在很多行里，有第一天开始于 2026 年 10 月 8 日的那一行，还有当天第一份天气：
`"condition":"clear"`（晴）。`"at":0` 表示第 1 天的午夜。

## 为什么是这样

一天里的时间、日期、太阳和天气，各由一个系统包管着，它把这些记成事实。别的包和客户端读这些事实；
没有别人去算天气。一个世界不用 `calendar` 和 `weather`，就只是没有日期、没有天气。规则在
[`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md#17-time)（英文）。

下一章：[12 包](12-packs.zh-CN.md)。第 09 到 11 章会在教程以后的更新里补上。
