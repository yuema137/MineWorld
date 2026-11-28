"""bench_backless.svg: bench.svg without its three back slats and two back posts (S33-f D-S33f-9's
backless bench, drawn the same from either side)."""
import sys
src, dst = sys.argv[1], sys.argv[2]
t = open(src).read()
cut = [
    '<polygon points="35,115 265,205 265,194 35,104" fill="#C48C52" stroke="#9C7040" stroke-width="1.7" stroke-linejoin="round"/>',
    '<polygon points="35,100 265,190 265,179 35,89" fill="#DCAE74" stroke="#9C7040" stroke-width="1.7" stroke-linejoin="round"/>',
    '<polygon points="35,85 265,175 265,164 35,74" fill="#C48C52" stroke="#9C7040" stroke-width="1.7" stroke-linejoin="round"/>',
    '<rect x="42.8" y="72.4" width="12" height="58" rx="4" fill="#2E3742"/>',
    '<rect x="245.2" y="151.6" width="12" height="58" rx="4" fill="#2E3742"/>',
]
for c in cut:
    assert t.count(c) == 1, c
    t = t.replace(c, "")
open(dst, "w").write(t)
