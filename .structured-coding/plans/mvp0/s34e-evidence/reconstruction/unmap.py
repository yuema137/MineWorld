import os,sys
SB=os.path.abspath(sys.argv[1]); n=0
pairs=[(SB+'/bin/WITHSLOT__','/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh'),(SB+'/bin/GODOTBIN__','/opt/homebrew/bin/godot'),(SB+'/tmp/mineworld-','/tmp/mineworld-')]
for dp,dn,fs in os.walk(SB+'/tmp'):
    dn[:]=[d for d in dn if d not in ('.git','.godot')]
    for f in fs:
        fp=os.path.join(dp,f)
        if os.path.islink(fp): continue
        try: t=open(fp,encoding='utf-8').read()
        except Exception: continue
        u=t
        for a,b in pairs: u=u.replace(a,b)
        if u!=t: open(fp,'w',encoding='utf-8').write(u); n+=1; print('unmapped',fp[len(SB):])
print(n)
