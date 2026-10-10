# systems/calendar/

**`mineworld-calendar`**: where and when a world is — its date, its weekday and its sun.

```text
configure   configure/calendar.yaml    epoch (local date at instant 0), utc_offset, latitude, longitude
emits       calendar-configured        at genesis; SystemInternal
            day-began                  at each local midnight: date, weekday, the day's light events,
                                       the sun every 15 minutes; Public
            daylight-changed           at each dawn, sunrise, sunset and dusk: the new phase; Public
owns        one `calendar` Process     its state is the fold of the three facts above
discloses   calendar-day, calendar-light   on the place the observer is in, to whoever is there
```

Instant 0 is local midnight of the epoch date, and every multiple of 86 400 s is a midnight — the same
convention `schedule` keeps. The sun comes from `solar-positioning` built with `libm`, and every value
is an integer (millidegrees, whole seconds) before it reaches a fact. A world that does not enable this
pack has no date and today's fixed light; a world that enables it without `configure/calendar.yaml`
seeds nothing.

```sh
cargo test -p mineworld-calendar
```

The decisions are [`ARC-67`](../../docs/DECISIONS.md) (two time domains) and
[`DEP-30`](../../docs/DECISIONS.md) (solar position and civil dates). Design and evidence:
[`step-19-time-weather.md`](../../.structured-coding/plans/mvp0/step-19-time-weather.md) §5 and §16.
