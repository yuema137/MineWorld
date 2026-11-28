"""Replay the recorded S34-e implementation thread's file writes into a sandbox.

Usage: replay.py ROLLOUT SB [--from LINE] [--until LINE]
Every exec command is run (bash, under sandbox-exec write restriction) with /tmp/mineworld- rewritten to
SB/tmp/mineworld-; heavy tools are stubbed (see SB/bin). FileChange events are applied strictly.
"""
import json, os, re, subprocess, sys
sys.path.insert(0, '/Users/yuema137/mineworld-worktrees/_recovered-docs-20261127/tools')
from decode import js_unescape

F, SB = sys.argv[1], os.path.abspath(sys.argv[2])
args = sys.argv[3:]
start = int(args[args.index('--from') + 1]) if '--from' in args else 0
until = int(args[args.index('--until') + 1]) if '--until' in args else 10 ** 9
skip = set(args[args.index('--skip') + 1].split(',')) if '--skip' in args else set()
PROFILE = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'sb.profile')
MAP = lambda s: s.replace('/tmp/mineworld-', SB + '/tmp/mineworld-').replace('/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh', SB + '/bin/WITHSLOT__').replace('/opt/homebrew/bin/godot', SB + '/bin/GODOTBIN__')
DEFAULT_CWD = SB + '/nocwd'
os.makedirs(DEFAULT_CWD, exist_ok=True)


def apply_hunks(text, diff):
    lines = text.split('\n')
    out, pos = [], 0
    hunks = re.split(r'(?m)^(?=@@ )', diff)
    for h in hunks:
        if not h.startswith('@@'):
            continue
        head, _, body = h.partition('\n')
        m = re.match(r'@@ -(\d+)(?:,(\d+))? \+(\d+)', head)
        old_start = int(m.group(1))
        old, new = [], []
        bl = body.split('\n')
        if bl and bl[-1] == '':
            bl = bl[:-1]
        for l in bl:
            if l.startswith('\\'):
                continue
            tag, content = (l[0], l[1:]) if l else (' ', '')
            if tag in ' -':
                old.append(content)
            if tag in ' +':
                new.append(content)
        idx = old_start - 1 if old else old_start
        if lines[idx:idx + len(old)] != old:
            cand = [i for i in range(pos, len(lines) - len(old) + 1) if lines[i:i + len(old)] == old]
            if not cand:
                raise ValueError('hunk mismatch at %s' % head)
            idx = min(cand, key=lambda i: abs(i - idx))
        out.extend(lines[pos:idx]); out.extend(new); pos = idx + len(old)
    out.extend(lines[pos:])
    return '\n'.join(out)


def run_cmd(cmd, wd, tag):
    cwd = MAP(wd) if wd else DEFAULT_CWD
    if not os.path.isdir(cwd):
        print(tag, 'NO-CWD', cwd); return
    c = MAP(cmd)
    env = dict(os.environ, PATH=SB + '/bin:' + os.environ['PATH'], HOME=os.environ['HOME'])
    r = subprocess.run(['sandbox-exec', '-f', PROFILE, 'bash', '-c', c], cwd=cwd, env=env, capture_output=True, text=True, timeout=900)
    err = (r.stderr or '').strip().splitlines()
    first = cmd.strip().splitlines()[0][:110]
    print(tag, 'CMD rc=%d' % r.returncode, repr(first), '|', ' / '.join(e[:160] for e in err[-2:]) if r.returncode else '')


for i, l in enumerate(open(F)):
    if i < start or i > until:
        continue
    o = json.loads(l); p = o.get('payload', {})
    tag = 'L%d' % i
    if o.get('type') == 'response_item' and p.get('type') == 'custom_tool_call':
        if p.get('call_id') in skip:
            print(tag, 'SKIP', p.get('call_id')); continue
        inp = p.get('input', '')
        for chunk in inp.split('exec_command(')[1:]:
            m = re.search(r'cmd"?\s*:\s*"((?:[^"\\]|\\.)*)"', chunk, re.S)
            if not m:
                continue
            cmd = js_unescape(m.group(1))
            w = re.search(r'workdir"?\s*:\s*"((?:[^"\\]|\\.)*)"', chunk)
            run_cmd(cmd, js_unescape(w.group(1)) if w else None, tag)
    elif o.get('type') == 'event_msg' and p.get('type') == 'item_completed' and p['item'].get('type') == 'FileChange':
        for path, ch in p['item']['changes'].items():
            dest = MAP(path)
            if ch['type'] == 'add':
                os.makedirs(os.path.dirname(dest), exist_ok=True)
                open(dest, 'w').write(ch['content'])
            elif ch['type'] == 'delete':
                os.remove(dest)
            else:
                new = apply_hunks(open(dest).read(), ch['unified_diff'])
                open(dest, 'w').write(new)
                if ch.get('move_path'):
                    os.rename(dest, MAP(ch['move_path']))
            print(tag, 'FILECHANGE', ch['type'], path)
