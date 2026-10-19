"""Derive Lakeside's site and far-field profile from USGS data (pr-s33a-setting-far-field.md §5.3).

Three subcommands, standard library only:

    fetch    read USGS NHD (the lake's outline), GNIS (summit coordinates) and 3DEP (EPQS point elevations)
             for every query and point the other two commands need that is not yet in cache/. The only
             command that touches the network. Resumable: the cache is written as it grows.
    site     the candidate table of §5.2 and the rule's choice, from the cache.
    profile  the profile table of §5.3 from the cache and the chosen site. Prints, writes nothing. Run twice
             it prints the same bytes: no clock, no network, sorted output.

The elevation angle is RL-b C8's (pr-rl-b-3d-budget.md:204-205): (h - h_eye - d^2 (1 - k) / (2 R)) / d,
with R 6 371 km and k 0.13 (Gauss, via Hirt et al. 2010, JGR 115 D21102). Bearings and distances are
spherical (haversine; the initial great-circle bearing): at these ranges (< 10 km) the spherical error is far
below 1 degree of bearing.

Usage: python3 tools/far-field/lakeside/derive.py {fetch|site|profile}
"""

from __future__ import annotations

import json
import math
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
CACHE = HERE / "cache"
EPQS = "https://epqs.nationalmap.gov/v1/json"
NHD = "https://hydro.nationalmap.gov/arcgis/rest/services/nhd/MapServer/%d/query"

INPUTS = json.loads((HERE / "inputs.json").read_text(encoding="utf-8"))
C = INPUTS["constants"]
R = C["earth_radius_m"]
K = C["refraction_k"]
EYE = C["eye_height_m"]
RULE = INPUTS["site_rule"]
PLAN = INPUTS["sample_plan"]


# ── cache ────────────────────────────────────────────────────────────────────────────────────────


def load(name: str) -> dict:
    path = CACHE / name
    return json.loads(path.read_text(encoding="utf-8")) if path.is_file() else {}


def save(name: str, data: dict) -> None:
    CACHE.mkdir(exist_ok=True)
    text = json.dumps(data, indent=1, sort_keys=True, ensure_ascii=True) + "\n"
    (CACHE / name).write_text(text, encoding="utf-8")


def key(lat: float, lon: float) -> str:
    return "%.6f,%.6f" % (lat, lon)


# ── geometry ─────────────────────────────────────────────────────────────────────────────────────


def destination(lat: float, lon: float, bearing_deg: float, d: float) -> tuple[float, float]:
    """The point d metres from (lat, lon) along the initial bearing, on the sphere; rounded to 1e-6."""
    p1, l1, b, a = math.radians(lat), math.radians(lon), math.radians(bearing_deg), d / R
    p2 = math.asin(math.sin(p1) * math.cos(a) + math.cos(p1) * math.sin(a) * math.cos(b))
    l2 = l1 + math.atan2(math.sin(b) * math.sin(a) * math.cos(p1), math.cos(a) - math.sin(p1) * math.sin(p2))
    return round(math.degrees(p2), 6), round(math.degrees(l2), 6)


def bearing_distance(lat1: float, lon1: float, lat2: float, lon2: float) -> tuple[float, float]:
    p1, p2 = math.radians(lat1), math.radians(lat2)
    dl = math.radians(lon2 - lon1)
    h = math.sin((p2 - p1) / 2) ** 2 + math.cos(p1) * math.cos(p2) * math.sin(dl / 2) ** 2
    d = 2 * R * math.asin(math.sqrt(h))
    y = math.sin(dl) * math.cos(p2)
    x = math.cos(p1) * math.sin(p2) - math.sin(p1) * math.cos(p2) * math.cos(dl)
    return (math.degrees(math.atan2(y, x)) + 360.0) % 360.0, d


def angle_deg(h: float, h_eye: float, d: float) -> float:
    """RL-b C8's apparent elevation angle of a point h metres high, d metres away, from an eye at h_eye."""
    return math.degrees((h - h_eye - d * d * (1 - K) / (2 * R)) / d)


class Lake:
    """The NHD polygons of the lake, in a local plane (x east, y north, metres) for point tests."""

    def __init__(self, nhd: dict):
        feats = [f for f in nhd["features"] if f["gnis_name"] == INPUTS["lake"]["gnis_name"]]
        if not feats:
            raise SystemExit("cache/nhd.json holds no %s polygon" % INPUTS["lake"]["gnis_name"])
        self.lat0 = sum(p[1] for f in feats for p in f["rings"][0]) / sum(len(f["rings"][0]) for f in feats)
        self.lon0 = sum(p[0] for f in feats for p in f["rings"][0]) / sum(len(f["rings"][0]) for f in feats)
        self.rings = [[self.xy(p[1], p[0]) for p in ring] for f in feats for ring in f["rings"]]
        self.features = feats

    def xy(self, lat: float, lon: float) -> tuple[float, float]:
        k = math.pi / 180.0 * R
        return (lon - self.lon0) * k * math.cos(math.radians(self.lat0)), (lat - self.lat0) * k

    def latlon(self, x: float, y: float) -> tuple[float, float]:
        k = math.pi / 180.0 * R
        return round(self.lat0 + y / k, 6), round(self.lon0 + x / (k * math.cos(math.radians(self.lat0))), 6)

    def water(self, lat: float, lon: float) -> bool:
        x, y = self.xy(lat, lon)
        inside = False
        for ring in self.rings:
            for (x1, y1), (x2, y2) in zip(ring, ring[1:] + ring[:1]):
                if (y1 > y) != (y2 > y) and x < x1 + (y - y1) * (x2 - x1) / (y2 - y1):
                    inside = not inside
        return inside


# ── the site (§5.2) ──────────────────────────────────────────────────────────────────────────────


def candidates(lake: Lake) -> list[dict]:
    """Points `inland_m` inland of the north-facing shoreline, every `shore_step_m` of shore."""
    out = []
    for ri, ring in enumerate(lake.rings):
        segs = list(zip(ring, ring[1:] + ring[:1]))
        total, walked, next_at = sum(math.dist(a, b) for a, b in segs), 0.0, 0.0
        for (x1, y1), (x2, y2) in segs:
            seg = math.dist((x1, y1), (x2, y2))
            while seg > 0 and next_at <= walked + seg and next_at < total:
                t = (next_at - walked) / seg
                px, py = x1 + t * (x2 - x1), y1 + t * (y2 - y1)
                nx, ny = -(y2 - y1) / seg, (x2 - x1) / seg  # the left normal; flipped to point out of the lake
                if lake.water(*lake.latlon(px + 2 * nx, py + 2 * ny)):
                    nx, ny = -nx, -ny
                normal = (math.degrees(math.atan2(nx, ny)) + 360.0) % 360.0
                off = min(normal, 360.0 - normal)
                if off <= RULE["north_facing_half_width_deg"]:
                    lat, lon = lake.latlon(px + RULE["inland_m"] * nx, py + RULE["inland_m"] * ny)
                    if not lake.water(lat, lon):
                        out.append({"id": "r%d-%05d" % (ri, round(next_at)), "lat": lat, "lon": lon,
                                    "normal_deg": round(normal, 1), "off_ew_deg": round(off, 1)})
                next_at += RULE["shore_step_m"]
            walked += seg
    return out


def frange(a: float, b: float, step: float) -> list[float]:
    n = int(round((b - a) / step))
    return [a + i * step for i in range(n + 1)]


def peak_of(gnis: dict, epqs: dict) -> dict | None:
    return next((s for s in summits(gnis, epqs) if s["name"] == RULE["peak"]), None)


def candidate_points(c: dict, lake: Lake, gnis: dict, epqs: dict) -> dict[str, list]:
    """The points each test of §5.2 reads for one candidate: water tests need no elevation."""
    near = [destination(c["lat"], c["lon"], b, d)
            for b in frange(*RULE["near_sector_deg"], RULE["near_sector_step_deg"])
            for d in frange(RULE["near_from_m"], RULE["near_to_m"], RULE["near_step_m"])]
    near_land = [p for p in near if not lake.water(*p)]
    peak = peak_of(gnis, epqs)
    ray = []
    if peak:
        b, _ = bearing_distance(c["lat"], c["lon"], peak["lat"], peak["lon"])
        ray = [destination(c["lat"], c["lon"], b, d)
               for d in frange(RULE["peak_ray_step_m"], RULE["peak_ray_to_m"], RULE["peak_ray_step_m"])]
    return {"site": [(c["lat"], c["lon"])], "near_land": near_land, "peak_ray": ray}


def judge(c: dict, lake: Lake, gnis: dict, epqs: dict) -> dict:
    pts = candidate_points(c, lake, gnis, epqs)
    ground = epqs.get(key(c["lat"], c["lon"]))
    a = all(lake.water(*destination(c["lat"], c["lon"], b, d))
            for b in frange(*RULE["water_sector_deg"], RULE["water_sector_step_deg"])
            for d in frange(RULE["water_from_m"], RULE["water_to_m"], RULE["water_step_m"]))
    rise = max((epqs[key(*p)] for p in pts["near_land"] if key(*p) in epqs), default=None)
    b_ok = ground is not None and (rise is None or rise <= ground + RULE["rise_tolerance_m"])
    peak, margin = peak_of(gnis, epqs), None
    if peak and ground is not None and all(key(*p) in epqs for p in pts["peak_ray"]):
        h_eye = ground + EYE
        _, dp = bearing_distance(c["lat"], c["lon"], peak["lat"], peak["lon"])
        crest = angle_deg(peak["height_m"], h_eye, dp)
        block = max(angle_deg(epqs[key(*p)], h_eye, bearing_distance(c["lat"], c["lon"], *p)[1])
                    for p in pts["peak_ray"])
        margin = crest - block
    c_ok = margin is not None and margin >= RULE["peak_margin_deg"]
    lo, hi = RULE["land_sector_deg"]
    land_bearings = [(lo + i * RULE["land_sector_step_deg"]) % 360
                     for i in range(int(((hi - lo) % 360) / RULE["land_sector_step_deg"]) + 1)]
    d_ok = not any(lake.water(*destination(c["lat"], c["lon"], float(b), d))
                   for b in land_bearings
                   for d in frange(RULE["land_step_m"], RULE["land_to_m"], RULE["land_step_m"]))
    extent = 0.0
    for d in frange(PLAN["lake_ray_from_m"], PLAN["extent_ray_to_m"], PLAN["lake_ray_step_m"]):
        if not lake.water(*destination(c["lat"], c["lon"], 180.0, d)):
            break
        extent = d
    return dict(c, ground_m=ground, a=a, b=b_ok, rise_m=rise, c=c_ok, d=d_ok,
                peak_margin_deg=None if margin is None else round(margin, 2), extent_180_m=extent,
                kept=a and b_ok and c_ok and d_ok)


def choose(table: list[dict]) -> dict | None:
    kept = [r for r in table if r["kept"]]
    if not kept:
        return None
    best = min(r["off_ew_deg"] for r in kept)
    ties = [r for r in kept if r["off_ew_deg"] - best <= RULE["tie_deg"]]
    return sorted(ties, key=lambda r: (-r["extent_180_m"], r["id"]))[0]


# ── summits ──────────────────────────────────────────────────────────────────────────────────────


def summit_grid(lat0: float, lon0: float) -> list[tuple[float, float]]:
    """3DEP sample points round a GNIS summit point: a square grid, `grid_step_m` apart, `grid_radius_m` out.

    A GNIS point can stand off the true top (Garnet Peak's reads ~19 m low at the point itself), so the summit
    is the grid's highest sample: its position and height are 3DEP's, found from GNIS's name and point."""
    r, step = INPUTS["summit_grid"]["grid_radius_m"], INPUTS["summit_grid"]["grid_step_m"]
    pts = []
    for dy in frange(-r, r, step):
        lat, lon = destination(lat0, lon0, 0.0, dy) if dy else (lat0, lon0)
        for dx in frange(-r, r, step):
            pts.append(destination(lat, lon, 90.0, dx) if dx else (round(lat, 6), round(lon, 6)))
    return pts


def gnis_name(s: dict) -> str:
    return s.get("gnis_name", s["name"])


def summits(gnis: dict, epqs: dict) -> list[dict]:
    out = []
    for s in INPUTS["summits"]:
        if not s.get("gnis"):
            out.append(dict(s))
            continue
        g = gnis.get(gnis_name(s))
        if not g or "lat" not in g:
            out.append(dict(s, missing="no GNIS summit named %s (cache/gnis.json: %s hits)"
                            % (gnis_name(s), (g or {}).get("hits", "no query"))))
            continue
        grid = [(p, epqs.get(key(*p))) for p in summit_grid(g["lat"], g["lon"])]
        known = [(h, p) for p, h in grid if h is not None]
        if len(known) < len(grid):
            out.append(dict(s, missing="3DEP grid not fetched (%d of %d)" % (len(known), len(grid))))
            continue
        h, p = max(known, key=lambda t: (t[0], t[1]))
        out.append(dict(s, lat=p[0], lon=p[1], height_m=h, gaz_id=g["gaz_id"],
                        position_source="the highest USGS 3DEP sample within %d m of USGS GNIS gaz_id %s (%s)"
                        % (INPUTS["summit_grid"]["grid_radius_m"], g["gaz_id"], gnis_name(s)),
                        height_source="USGS 3DEP (EPQS) at that sample"))
    return out


# ── fetch ────────────────────────────────────────────────────────────────────────────────────────


def http_json(url: str) -> dict:
    attempts = 8
    for attempt in range(attempts):
        try:
            with urllib.request.urlopen(url, timeout=120) as r:
                return json.loads(r.read().decode("utf-8"))
        except (OSError, ValueError) as e:  # a timeout, a 5xx or a truncated reply: back off, then give up loudly
            if attempt == attempts - 1:
                raise SystemExit("fetch failed: %s: %s" % (url, e)) from e
            time.sleep(min(60.0, 2.0 ** (attempt + 1)))
    raise AssertionError


def fetch_nhd() -> dict:
    lk = INPUTS["lake"]
    q = urllib.parse.urlencode({
        "geometry": ",".join(str(v) for v in lk["nhd_envelope"]), "geometryType": "esriGeometryEnvelope",
        "inSR": 4326, "outSR": 4326, "spatialRel": "esriSpatialRelIntersects",
        "where": "gnis_name = '%s'" % lk["gnis_name"],
        "outFields": "gnis_name,gnis_id,areasqkm,ftype,fcode,permanent_identifier",
        "returnGeometry": "true", "f": "json"})
    url = NHD % lk["nhd_layer"] + "?" + q
    d = http_json(url)
    feats = []
    for f in d.get("features", []):
        a = {k.lower(): v for k, v in f["attributes"].items()}  # the service returns upper-case field names
        feats.append({"gnis_name": a["gnis_name"], "gnis_id": a["gnis_id"], "areasqkm": a["areasqkm"],
                      "fcode": a["fcode"], "permanent_identifier": a["permanent_identifier"],
                      "rings": [[[round(x, 7), round(y, 7)] for x, y in ring] for ring in f["geometry"]["rings"]]})
    return {"query": url, "features": sorted(feats, key=lambda f: f["permanent_identifier"])}


def fetch_gnis(names: list[str]) -> dict:
    g = INPUTS["gnis"]
    out = {}
    for name in names:
        q = urllib.parse.urlencode({
            "geometry": ",".join(str(v) for v in g["envelope"]), "geometryType": "esriGeometryEnvelope",
            "inSR": 4326, "outSR": 4326, "spatialRel": "esriSpatialRelIntersects",
            "where": "gaz_name = '%s' AND gaz_featureclass = 'Summit'" % name,
            "outFields": "gaz_id,gaz_name,gaz_featureclass,county_name", "returnGeometry": "true", "f": "json"})
        d = http_json(g["service"] + "?" + q)
        hits = d.get("features", [])
        if len(hits) == 1:
            p = hits[0]["geometry"]["points"][0] if "points" in hits[0]["geometry"] else \
                [hits[0]["geometry"]["x"], hits[0]["geometry"]["y"]]
            out[name] = {"gaz_id": hits[0]["attributes"]["gaz_id"], "lat": round(p[1], 6), "lon": round(p[0], 6),
                         "query": g["service"] + "?" + q}
        else:
            out[name] = {"hits": len(hits), "query": g["service"] + "?" + q}
    return out


def fetch_points(points: list[tuple[float, float]], epqs: dict, counter: list[int]) -> None:
    todo = sorted({key(*p) for p in points} - set(epqs))
    for i, k in enumerate(todo):
        lat, lon = (float(v) for v in k.split(","))
        q = urllib.parse.urlencode({"x": lon, "y": lat, "wkid": 4326, "units": "Meters", "includeDate": "false"})
        v = http_json(EPQS + "?" + q).get("value")
        epqs[k] = None if v in (None, "", "-1000000") else round(float(v), 2)
        counter[0] += 1
        if i % 50 == 49:
            save("epqs.json", epqs)
    save("epqs.json", epqs)


def profile_points(site: dict, lake: Lake) -> list[tuple[float, float]]:
    return [destination(site["lat"], site["lon"], b, d)
            for b in frange(0, 360 - PLAN["rim_bearing_step_deg"], PLAN["rim_bearing_step_deg"])
            for d in PLAN["rim_distances_m"]]


def cmd_fetch() -> None:
    counter = [0]
    nhd = load("nhd.json")
    if not nhd:
        nhd = fetch_nhd()
        save("nhd.json", nhd)
    gnis = load("gnis.json")
    names = [gnis_name(s) for s in INPUTS["summits"] if s.get("gnis") and gnis_name(s) not in gnis]
    if names:
        gnis.update(fetch_gnis(names))
        save("gnis.json", gnis)
    epqs = load("epqs.json")
    fetch_points([p for g in gnis.values() if "lat" in g for p in summit_grid(g["lat"], g["lon"])], epqs, counter)
    lake = Lake(nhd)
    cands = candidates(lake)
    fetch_points([p for c in cands for p in candidate_points(c, lake, gnis, epqs)["site"]], epqs, counter)
    fetch_points([p for c in cands for p in candidate_points(c, lake, gnis, epqs)["near_land"]], epqs, counter)
    fetch_points([p for c in cands for p in candidate_points(c, lake, gnis, epqs)["peak_ray"]], epqs, counter)
    site = choose([judge(c, lake, gnis, epqs) for c in cands])
    if site:
        fetch_points(profile_points(site, lake), epqs, counter)
    print("fetch: %d EPQS calls this run; cache holds %d points; NHD features %d; GNIS names %d"
          % (counter[0], len(epqs), len(nhd["features"]), len(gnis)))


# ── site and profile ─────────────────────────────────────────────────────────────────────────────


def state() -> tuple[Lake, dict, dict]:
    nhd, gnis, epqs = load("nhd.json"), load("gnis.json"), load("epqs.json")
    if not nhd:
        raise SystemExit("cache/nhd.json is missing: run fetch first")
    return Lake(nhd), gnis, epqs


def cmd_site() -> None:
    lake, gnis, epqs = state()
    table = [judge(c, lake, gnis, epqs) for c in candidates(lake)]
    print("| id | lat | lon | shore normal deg | off E-W deg | ground m | (a) water 150-210 | (b) no rise | "
          "near max m | (c) peak margin deg | (d) land north | extent 180 m | kept |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for r in table:
        print("| %s | %.6f | %.6f | %.1f | %.1f | %s | %s | %s | %s | %s | %s | %.0f | %s |" % (
            r["id"], r["lat"], r["lon"], r["normal_deg"], r["off_ew_deg"], r["ground_m"], r["a"], r["b"],
            r["rise_m"], r["peak_margin_deg"], r["d"], r["extent_180_m"], "**yes**" if r["kept"] else "no"))
    best = choose(table)
    print()
    print("chosen: %s" % (best["id"] if best else "NONE (material stop, section 5.2 step 4)"))


def cmd_profile() -> None:
    lake, gnis, epqs = state()
    site = choose([judge(c, lake, gnis, epqs) for c in candidates(lake)])
    if not site:
        raise SystemExit("no candidate satisfies section 5.2 (a)-(c): material stop")
    h0 = site["ground_m"]
    h_eye = h0 + EYE
    print("site %s  lat %.6f  lon %.6f  ground %.2f m (3DEP)  eye %.2f m" % (site["id"], site["lat"], site["lon"],
                                                                            h0, h_eye))
    print()
    print("## Summits")
    print("| summit | layer | lat | lon | height m | bearing deg | distance m | crest angle deg | source |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for s in summits(gnis, epqs):
        if s.get("missing") or s.get("height_m") is None:
            print("| %s | %s | — | — | — | — | — | — | %s |" % (s["name"], s["layer"], s.get("missing", "no height")))
            continue
        b, d = bearing_distance(site["lat"], site["lon"], s["lat"], s["lon"])
        print("| %s | %s | %.6f | %.6f | %.1f | %.1f | %.0f | %.3f | %s; %s |" % (
            s["name"], s["layer"], s["lat"], s["lon"], s["height_m"], b, d, angle_deg(s["height_m"], h_eye, d),
            s["position_source"], s["height_source"]))
    print()
    print("## Near rim, per %d-degree sector (land samples only; the highest angle)" % PLAN["rim_bearing_step_deg"])
    print("| bearing deg | crest height m | distance m | angle deg | lat | lon |")
    print("| --- | --- | --- | --- | --- | --- |")
    step = PLAN["rim_bearing_step_deg"]
    for b in frange(0, 360 - step, step):
        best = None
        for d in PLAN["rim_distances_m"]:
            p = destination(site["lat"], site["lon"], b, d)
            h = epqs.get(key(*p))
            if h is None or lake.water(*p):
                continue
            a = angle_deg(h, h_eye, d)
            if best is None or a > best[0]:
                best = (a, h, d, p)
        if best is None:
            print("| %.0f | water | — | — | — | — |" % b)
        else:
            print("| %.0f | %.1f | %.0f | %.3f | %.6f | %.6f |" % (b, best[1], best[2], best[0], best[3][0], best[3][1]))
    print()
    lake_bearings = []
    for b in range(0, 360, PLAN["lake_sector_step_deg"]):
        p = destination(site["lat"], site["lon"], float(b), INPUTS["reach_m"])
        if lake.water(*p):
            lake_bearings.append(b)
    run = contiguous(lake_bearings, 180)
    print("## Lake sector (rays that meet NHD water at reach_m = %d m, where the far field's water begins)"
          % INPUTS["reach_m"])
    print("bearings: %s" % ("none contains 180 deg" if not run else
                             "%d -> %d deg (rounded outward: %d -> %d)"
                             % (run[0], run[1], (run[0] - 1) % 360, (run[1] + 1) % 360)))
    print("NHD features: %s" % ", ".join("%s %.3f km2" % (f["permanent_identifier"], f["areasqkm"])
                                         for f in lake.features))
    print()
    print("## Slots (scene frame -> bearing from the origin; true north is -z)")
    print("| slot | place | centre x, z m | bearing deg | distance m | footprint m | inside reach_m | status |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- |")
    for slot, place in (("station", "station"), ("landmark", "pier")):
        f = INPUTS["places"][place]
        cx, cz = sum(f["x"]) / 2, sum(f["z"]) / 2
        b = (math.degrees(math.atan2(cx, -cz)) + 360.0) % 360.0
        foot = max(f["x"][1] - f["x"][0], f["z"][1] - f["z"][0])
        print("| %s | %s | %.1f, %.1f | %.1f | %.1f | %.1f | %s | %s |" % (
            slot, place, cx, cz, b, math.hypot(cx, cz), foot, math.hypot(cx, cz) <= INPUTS["reach_m"],
            f.get("provisional", "frozen source")))
    print()
    print("## reach_m check (every halt floor corner and the pier head inside %d m)" % INPUTS["reach_m"])
    worst = 0.0
    for place in ("pier", "station", "station-platform"):
        f = INPUTS["places"][place]
        for x in f["x"]:
            for z in f["z"]:
                worst = max(worst, math.hypot(x, z))
    print("farthest corner %.1f m -> %s" % (worst, "inside" if worst <= INPUTS["reach_m"] else "OUTSIDE"))


def contiguous(bearings: list[int], through: int) -> tuple[int, int] | None:
    s = set(bearings)
    if through not in s:
        return None
    lo = hi = through
    while (lo - 1) % 360 in s and (lo - 1) % 360 != through:
        lo -= 1
    while (hi + 1) % 360 in s and (hi + 1) % 360 != through:
        hi += 1
    return lo % 360, hi % 360


def main() -> None:
    cmds = {"fetch": cmd_fetch, "site": cmd_site, "profile": cmd_profile}
    if len(sys.argv) != 2 or sys.argv[1] not in cmds:
        raise SystemExit(__doc__)
    cmds[sys.argv[1]]()


if __name__ == "__main__":
    main()
