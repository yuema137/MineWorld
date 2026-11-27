from pathlib import Path
import sys, os, hashlib, json
import ci_layer as layer
import ci_shard as shard
root=Path('/tmp/mineworld-ci-p0-audit/fixture');root.mkdir(exist_ok=True)
(root/'src').mkdir(exist_ok=True);(root/'tests').mkdir(exist_ok=True)
(root/'Cargo.toml').write_text('[package]\nname="p0-proof"\nversion="0.1.0"\nedition="2021"\n[workspace]\n')
(root/'src/lib.rs').write_text('/// ```\n/// assert!(false, "doctest sentinel");\n/// ```\npub fn marker() {}\n')
for name in ('a','b'):
 (root/f'tests/{name}.rs').write_text(f'#[test]\nfn fail_{name}() {{ panic!("intentional {name}"); }}\n')
layer.ROOT=root;layer.FAILED_TESTS=root/'target/ci-failed-tests.md'
layer.ENVIRONMENT_OF['proof']=[]
commands=shard.commands(['a','b',shard.DOC])
commands.append([sys.executable,'-c','from pathlib import Path; Path("after-tests").write_text("reached")'])
layer.LAYERS['proof']=commands
status=layer.run('proof');summary=layer.FAILED_TESTS.read_text()
assert status==101,status
assert all(x in summary for x in ('fail_a','fail_b','`p0_proof`: `src/lib.rs - marker (line 1)`','(exit 101)')),summary
assert (root/'after-tests').read_text()=='reached'
print('PASS: real Cargo two failing binaries + failing doctest; subsequent command reached; first failure 101 retained; summary includes all failures')
# Non-test failure must stop before the following command and preserve its code.
layer.LAYERS['proof']=[[sys.executable,'-c','raise SystemExit(7)'],[sys.executable,'-c','raise RuntimeError("must not run")']]
assert layer.run('proof')==7
assert 'exit 101' not in layer.FAILED_TESTS.read_text()
print('PASS: non-test failure stops with exit 7 and replaces previous failure summary')
layer.LAYERS['proof']=[[sys.executable,'-c','pass']]
assert layer.run('proof')==0 and not layer.FAILED_TESTS.exists()
print('PASS: successful rerun removes stale failure summary')
for cmd in layer.repeat_commands(shard.commands(['a',shard.DOC])):
 if '--no-run' not in cmd: assert cmd.count('--no-fail-fast')==1
print('PASS: core-repeat has exactly one no-fail-fast flag')
print(json.dumps({name:hashlib.sha256(Path('/tmp/mineworld-ci-p0-audit',name).read_bytes()).hexdigest() for name in ('ci_layer.py','ci_shard.py')},indent=2))
