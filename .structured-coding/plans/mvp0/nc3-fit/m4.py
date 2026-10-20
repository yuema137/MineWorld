import sys
sys.path.insert(0, "/tmp/nc3fit")
exec(open("/tmp/nc3fit/mirror_check.py").read().split("a = int")[0])
A = int(sys.argv[1]); pts = [(int(a), int(b), int(c)) for a, b, c in (s.split(",") for s in sys.argv[2:])]
for H0, L0, ph in pts:
    fit2.install(H0, L0, A, ph)
    print(H0, L0, ph, [judge(x)[0] for x in (None, 390, 270)], [judge(x)[2] for x in (None, 390, 270)])
