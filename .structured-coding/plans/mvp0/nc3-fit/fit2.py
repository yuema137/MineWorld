"""Mirror of the Rust two_process curves (integer cosine table, hourly points + 1439) for the fit."""
import sys
sys.path.insert(0, "/tmp/nc3fit")
import fit

COS = [1000, 966, 866, 707, 500, 259, 0, -259, -500, -707, -866, -966, -1000]


def wave(h):
    k = h % 24
    return COS[min(k, 24 - k)]


def trunc_div(x, d):
    return x // d if x >= 0 else -((-x) // d)


def points(mean, a, peak_hour):
    def lvl(h):
        return max(0, min(1000, mean + trunc_div(a * wave(h - peak_hour), 1000)))
    return [(h * 60, lvl(h)) for h in range(23)] + [(1439, lvl(24))]


def at(pts, x):
    if x <= pts[0][0]:
        return pts[0][1]
    if x >= pts[-1][0]:
        return pts[-1][1]
    for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
        if x0 <= x <= x1:
            return y0 + ((y1 - y0) * (x - x0)) // (x1 - x0)


def install(H0, L0, a, peak_hour):
    hp, lp = points(H0, a, peak_hour), points(L0, a, peak_hour)
    fit.H = lambda m, *_: at(hp, m)
    fit.L = lambda m, *_: at(lp, m)
