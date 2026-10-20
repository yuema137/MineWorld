"""Strict judge with the Rust-mirrored integer hourly curves, around a candidate (a fixed)."""
import sys
sys.path.insert(0, "/tmp/nc3fit")
import fit, fit2
DAYS, LAST = 60, 28
def judge(alarm):
    _, _, eps = fit.run(0, 0, 0, 0, days=DAYS, alarm=alarm)
    start_t = (DAYS - LAST) * 1440
    tail = [e for e in eps if e[1] > start_t]
    asleep_at = lambda t: any(e[0] <= t < e[1] for e in tail)
    days = range(DAYS - LAST, DAYS)
    share = [sum(asleep_at(d * 1440 + h * 60) for d in days) * 1000 // LAST for h in range(24)]
    mean = sum(min(e[1], DAYS * 1440) - max(e[0], start_t) for e in tail) / 60 / LAST
    per_day = len(tail) / LAST
    ok = share[3] >= 850 and share[14] <= 100 and 7.0 <= mean <= 9.5 and per_day <= 1.1
    on, off = tail[-1][0] % 1440, tail[-1][1] % 1440
    return ok, fit.score(share), (share[3], share[14], round(mean, 2), round(per_day, 2), f"{on//60:02d}:{on%60:02d}-{off//60:02d}:{off%60:02d}")
a = int(sys.argv[1])
rows = []
for H0 in range(880, 1001, 10):
    for L0 in range(0, 301, 10):
        for ph in range(14, 23):
            fit2.install(H0, L0, a, ph)
            r = [judge(x) for x in (None, 390, 270)]
            if all(x[0] for x in r):
                rows.append((r[0][1], H0, L0, ph, [x[2] for x in r]))
rows.sort()
print(len(rows))
for x in rows[:8]: print(x)
