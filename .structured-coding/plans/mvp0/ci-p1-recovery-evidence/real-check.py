import subprocess,time,json,pathlib,sys,os
root=pathlib.Path('/tmp/mineworld-recover-ci-p1').resolve()
out=root/'.structured-coding/plans/mvp0/ci-p1-recovery-evidence'
archive=root/'artifacts/real-core.tar.zst';archive.parent.mkdir(exist_ok=True)
scratch=root/'artifacts/real-temp';scratch.mkdir(exist_ok=True)
os.environ.update(TMPDIR=str(scratch),TMP=str(scratch),TEMP=str(scratch))
args=['-p','mineworld-packages','-p','mineworld-test-support','--test','manifest','--test','structure','--test','scratch']
def run(args):
 print('$ '+' '.join(args),flush=True); start=time.monotonic()
 p=subprocess.run(args,cwd=root,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=600)
 print(p.stdout,flush=True);print(f'exit={p.returncode} wall={time.monotonic()-start:.2f}s',flush=True)
 assert p.returncode==0
 return p.stdout
run(['cargo','nextest','archive','--locked','--profile','ci',*args,'--archive-file',str(archive)])
baseline=json.loads(subprocess.check_output(['cargo','nextest','list',*args,'--message-format','json'],cwd=root,text=True))
run(['cargo','nextest','run','--profile','ci','--no-fail-fast','--archive-file',str(archive),'--extract-to',str(root),'--extract-overwrite'])
sys.path.insert(0,str(root/'scripts'))
import ci_results
assert not ci_results.failures(root/'target/nextest/ci/junit.xml')
import shutil
shutil.copyfile(root/'target/nextest/ci/junit.xml',out/'mineworld-junit.xml')
run([sys.executable,'scripts/check_scratch.py','left','--target-dir','target'])
wanted={(binary,name) for binary,suite in baseline['rust-suites'].items() for name,t in suite['testcases'].items() if not t['ignored']}
actual={(suite.attrib['name'],case.attrib['name']) for suite in ci_results.ElementTree.parse(out/'mineworld-junit.xml').getroot().findall('testsuite') for case in suite.findall('testcase')}
assert actual==wanted,(actual,wanted)
print(f'MineWorld archive PASS: {len(actual)} baseline identities, source fixtures and scratch paths, cleanup clean')
