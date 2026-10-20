"""Scratch exploration for S35 N-c3: two-process thresholds on needs' linear S.

Integer-free float model of the controller: consult every 15 min. Awake: sleep when S >= H(t) - slack.
On onset, the sleep length is stepped in 15-min steps until S <= L(t), capped at `longest` and an alarm.
"""
import math
import sys

ATUS = [868, 924, 944, 949, 914, 843, 659, 425, 250, 136, 72, 46, 35, 40, 48, 45, 44, 34, 29, 35, 76, 211, 471, 728]
UP, DOWN = 60.0, 125.0
STEP = 15


def C(minute, peak):
    return math.cos(2 * math.pi * (minute - peak) / 1440.0)


def H(m, H0, a, peak):
    return min(1000.0, H0 + a * C(m, peak))


def L(m, L0, a, peak):
    return max(0.0, L0 + a * C(m, peak))


def run(H0, L0, a, peak, days=14, alarm=None, slack=0.0, longest=840, start_s=700.0):
    s = start_s
    t = 0
    asleep_until = None
    hourly = [0] * 24
    counts = [0] * 24
    episodes = []
    onset = None
    while t < days * 1440:
        m = t % 1440
        if asleep_until is not None and t >= asleep_until:
            episodes.append((onset, t))
            asleep_until = None
        if asleep_until is None and s >= H(m, H0, a, peak) - slack:
            # sleep length by L
            ss, tt, n = s, t, 0
            while True:
                n += STEP
                tt += STEP
                ss = max(0.0, ss - DOWN / 4)
                if ss <= L(tt % 1440, L0, a, peak) or n >= longest:
                    break
            if alarm is not None:
                # minutes until next alarm instant
                until = (alarm - m) % 1440
                if until == 0:
                    until = 1440
                n = min(n, until)
            n = max(n, 15)
            asleep_until = t + n
            onset = t
        asleep = asleep_until is not None
        if t >= 2 * 1440 and m % 60 == 0:
            hourly[m // 60] += 1 if asleep else 0
            counts[m // 60] += 1
        s = max(0.0, s - DOWN / 4) if asleep else min(1000.0, s + UP / 4)
        t += STEP
    share = [1000 * hourly[h] // max(1, counts[h]) for h in range(24)]
    late = [e for e in episodes if e[0] >= 2 * 1440]
    mean = sum(e[1] - e[0] for e in late) / 60 / max(1, (days - 2))
    return share, mean, late


def fmt(t):
    d, m = divmod(t, 1440)
    return f"d{d} {m // 60:02d}:{m % 60:02d}"


def score(share):
    return sum(abs(x - y) for x, y in zip(share, ATUS))


def search(a, slack):
    best = []
    for H0 in range(800, 1001, 5):
        for L0 in range(0, 401, 5):
            for peak in range(15 * 60, 22 * 60 + 1, 15):
                share, mean, _ = run(H0, L0, a, peak, slack=slack)
                ok = share[3] >= 850 and share[14] <= 100 and 7.0 <= mean <= 9.5
                if ok:
                    best.append((score(share), H0, L0, peak, mean))
    best.sort()
    return best[:10]


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "one":
        H0, L0, a, peak = (float(x) for x in sys.argv[2:6])
        alarm = int(sys.argv[6]) if len(sys.argv) > 6 and sys.argv[6] != "-" else None
        slack = float(sys.argv[7]) if len(sys.argv) > 7 else 0.0
        share, mean, eps = run(H0, L0, a, peak, alarm=alarm, slack=slack)
        print("share", share, "score", score(share), "mean h/day", round(mean, 2))
        for e in eps[:8]:
            print("  ", fmt(e[0]), "->", fmt(e[1]), (e[1] - e[0]) / 60)
    else:
        a = float(sys.argv[2])
        slack = float(sys.argv[3]) if len(sys.argv) > 3 else 0.0
        for row in search(a, slack):
            print(row)
