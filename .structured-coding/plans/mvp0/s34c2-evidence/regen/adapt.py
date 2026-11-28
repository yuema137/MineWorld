import sys
p = sys.argv[1]
t = open(p).read()
old = "a=xy(p['fallback']['at']);yaw=math.radians(p['fallback']['facing']/1000)"
new = "w=next(s for s in d['sit-points'] if s['key']==p['point']);a=xy(w['at']);yaw=math.radians(w['facing']/1000)"
assert t.count(old) == 1
open(p, "w").write(t.replace(old, new))
