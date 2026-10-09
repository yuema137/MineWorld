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

```sh
cargo test -p mineworld-weather
```

The decision is [`ARC-68`](../../docs/DECISIONS.md). Design and evidence:
[`step-19-time-weather.md`](../../.structured-coding/plans/mvp0/step-19-time-weather.md) §6 and §17.
