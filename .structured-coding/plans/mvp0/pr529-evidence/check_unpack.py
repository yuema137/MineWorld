#!/usr/bin/env python3
"""Offline execution of the workflow's two unpack blocks; no Windows pass implied."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[4]
WORKFLOW = ROOT / '.github/workflows/release.yml'


def block(source, name):
    lines = source.splitlines()
    start = next(i for i, line in enumerate(lines) if line.strip() == '- name: ' + name)
    run = next(i for i in range(start, len(lines)) if lines[i].strip() == 'run: |')
    end = run + 1
    while end < len(lines) and (not lines[end].strip() or lines[end].startswith('          ')):
        end += 1
    return '\n'.join(line[10:] for line in lines[run + 1:end]) + '\n'


def main():
    source = WORKFLOW.read_text()
    build = block(source, 'Unpack the archive and probe the bundle in place')
    smoke = block(source, '"Step 1: unpack as a player would"')
    # bash -n on every literal workflow run block; expressions are GitHub's responsibility.
    for match in re.finditer(r'^        run: \|\n((?:          .*\n|\n)+)', source, re.M):
        body = '\n'.join(line[10:] for line in match[1].splitlines())
        body = re.sub(r'\$\{\{.*?\}\}', 'EXPRESSION', body)
        subprocess.run(['bash', '-n'], input=body, text=True, check=True)
    with tempfile.TemporaryDirectory(prefix='mineworld-unpack-') as tmp:
        folder = Path(tmp)
        tools = folder / 'tools'
        tools.mkdir()
        # This shim owns the argv/cwd seam only. It is not Windows tar or an encoding emulator.
        native = tools / 'native-tar'
        native.write_text('''#!/usr/bin/python3
import os, subprocess, sys
from pathlib import Path
args = sys.argv[1:]
if '-C' in args:
    sys.exit('planted narrow-argv failure: destination passed with -C')
assert Path.cwd().name == 'MineWorld 世界', Path.cwd()
assert args[0] == '-xf' and Path(args[1]).is_absolute(), args
sys.exit(subprocess.run(['/usr/bin/tar', *args]).returncode)
''')
        native.chmod(0o755)
        cygpath = tools / 'cygpath'
        cygpath.write_text('#!/bin/sh\nprintf "%s\\n" "$2"\n')
        cygpath.chmod(0o755)
        probe = tools / 'probe'
        probe.write_text('''#!/usr/bin/python3
import sys
from pathlib import Path
assert sys.argv[1:3] == ['scripts/package.py', 'probe'], sys.argv
assert (Path(sys.argv[3]) / 'payload').read_bytes() == b'fixture bytes'
''')
        probe.chmod(0o755)
        for name, body in [('build', build), ('fresh', smoke)]:
            for mutation in [False, True]:
                case = folder / (name + ('-old' if mutation else '-fixed'))
                case.mkdir()
                runner = case / 'runner temp'
                runner.mkdir()
                download = runner / 'download'
                download.mkdir()
                archive = download / 'MineWorld-0.1.0-windows-x86_64.zip'
                with zipfile.ZipFile(archive, 'w') as output:
                    output.writestr('MineWorld-0.1.0-windows-x86_64/payload', b'fixture bytes')
                envfile = case / 'env'
                envfile.touch()
                run = body.replace('/c/Windows/System32/tar.exe', str(native))
                if mutation:
                    if name == 'build':
                        run = run.replace('(cd "$place" && "$untar" -xf "$archive")',
                                          '"$untar" -xf "$archive" -C "$place"')
                    else:
                        run = run.replace('(cd "$place" && ' + str(native) + ' -xf "$archive")',
                                          str(native) + ' -xf "$archive" -C "$place"')
                env = dict(os.environ, RUNNER_OS='Windows', RUNNER_TEMP=str(runner),
                           TARGET='windows-x86_64', GITHUB_ENV=str(envfile),
                           ARCHIVE=os.path.relpath(archive, case), PY=str(probe),
                           PATH=str(tools) + ':' + os.environ['PATH'])
                result = subprocess.run(['bash', '-e', '-o', 'pipefail'], cwd=case,
                                        input=run, text=True, env=env, capture_output=True)
                if mutation:
                    assert result.returncode != 0 and 'planted narrow-argv failure' in result.stderr, result
                    print(name + ': old -C mutation rejected')
                else:
                    assert result.returncode == 0, result.stderr
                    payload = runner / 'MineWorld 世界/MineWorld-0.1.0-windows-x86_64/payload'
                    assert payload.read_bytes() == b'fixture bytes'
                    if name == 'fresh':
                        assert 'BUNDLE=' + str(payload.parent) in envfile.read_text()
                    print(name + ': Unicode cwd + absolute archive + extracted bytes PASS (POSIX seam only)')
    print('all literal run blocks: bash syntax PASS; Windows native runtime NOT RUN')


if __name__ == '__main__':
    main()
