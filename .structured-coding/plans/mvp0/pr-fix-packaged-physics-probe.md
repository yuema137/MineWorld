# PR repair — Packaged 3D physics diagnostic dependency

## Authority and scope

Bounded correction to the existing frozen S23 release contract, not a new design freeze.
Parent: `step-23-release.md` §18.7; `pr-s23-rb-release-workflow.md` §11 as filled on
`mvp0/pr-s23-rb` at `3c2acbe7` (main's copy is stale); ARC-80 and DEP-20.
The operator's 2026-11-22 takeover instruction authorizes PR-based implementation with independent
red-team review. The coordinator delegated this narrow repair. Scope, runtime assertions, physics
selection, and bundle tooling exclusion remain unchanged.

## Audit and bounded decision

Base main `2b4d36fac345b9325a91fd4313d40e3633e728b0`. `slice_probe.gd:1004` refers to the global
`PhysicsEngineProbe` class, declared only in excluded `tools/physics_engine.gd`. This is the same
headless compile/hang hazard documented in §18.7. The later #512 filter correctly excludes all tools;
the dependency was left behind. Only this global tools class has a runtime consumer (source search).

Move the two static diagnostic methods, unchanged, into `scripts/physics_engine.gd` (RefCounted).
Both the slice probe and the existing tools CLI preload that module explicitly. Keep the tools path
as a SceneTree command entry, with the same printed output; it no longer supplies a global class.
This corrects an internal presentation dependency, not authoritative state ownership or a public
interface. No dependency is added and no export exclusion or assertion is relaxed.

## Execution contract

- Branch: `fix/packaged-physics-probe`; isolated worktree `/tmp/mineworld-packaged-physics-probe`.
- Base: `2b4d36fac345b9325a91fd4313d40e3633e728b0`; prerequisites already on main (#512, #519).
- Allowed: implement, targeted validation, semantic commit, push and draft PR, independent review.
  Source: existing R-b §11 autonomous actions and current coordinator delegation.
- Forbidden: merge, tags, releases, paid calls, heavy full suites; source: coordinator delegation.
- Godot runs use the existing two-slot wrapper. Each targeted run bounded to 10 minutes; total
  local runtime budget 30 minutes. Full asset reimport is excluded without a new resource decision.
- Invariants: tools excluded; behavior plus setting engine discrimination preserved; RID freed;
  drive/link assertions intact; no simulation, settings, protocol, or packaging API change.
- Canonical regression owner: exported runtime compile/link probe; editor parse is supporting
  evidence only. Real LLM validation N/A. Full Rust suites N/A for this presentation-only change.
- Stop on material invariant change. Coordinator owns parent/progress synchronization; this
  session owns this ledger and review handoff. No merge authority.

## Commit plan and live ledger

One semantic commit: runtime diagnostic extraction, regression evidence, and this ledger.

- [x] Implementation: runtime helper, slice caller, CLI wrapper.
- [x] Validation: unchanged engine diagnostic output; scripts-only compilation with tools absent;
  actual exported pack outside the checkout when existing cache/resources permit; documentation checks.
- [x] Logic review: inspect discriminator, resource cleanup, all consumers, exclusion preservation.
- [ ] Independent red team: coordinator-owned review of exact diff and evidence.

## Evidence and handoff

Checkpoint: implementation and targeted validation complete; independent final-head review and CI pending.
No background jobs. Next: semantic commit, draft PR, independent final-head review and canonical fast CI.
Full release link/smoke remains deferred to the coordinator's integrated release run.

### Results (2026-11-22)

Environment: macOS arm64, Godot `4.7.2.stable.official.ed1daf0bf`, installed official macOS templates.
Every Godot command ran through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.

- PASS: `python3 scripts/ci_gdparse.py clients/3d-spike`: 147 scripts, zero failed loads;
  import 2 seconds, compile 1 second, six asset folders skipped.
- PASS: original versus relocated static methods are byte-identical from `static func setting()` onward.
- PASS: baseline and new `--script res://tools/physics_engine.gd` diagnostic output is identical:
  `physics/3d/physics_engine setting: 'Jolt Physics'` and
  `running 3D physics engine: Jolt Physics (a new space's solver iterations 8; Godot Physics' setting 16)`.
- PASS: actual scripts-only exported macOS template regression, using the unchanged real export preset.
  Temporary copies carry all script-bearing directories (shared symlinks dereferenced), the real
  project settings, and a small test scene that loads the real `SliceProbe` dependency closure.
  Only main-scene selection and export-required ETC2/ASTC settings differ in the test copy.
  Each variant gets a fresh class cache. After export the entire source project/cache is deleted;
  the `.app` starts headlessly outside the checkout in a path containing a space and `世界`.
  Every subprocess has a 180-second timeout. Baseline uses both original source files from the base.
  Baseline import 2.0 s, export 6.6 s, runtime 1.5 s; fixed import 1.2 s, export 6.1 s, runtime 1.6 s.
  A test-scene `ResourceLoader.exists` check rejects a shipped `tools/physics_engine.gd`.
- PASS: `python3 scripts/package.py --self-test`, 53 cases including tools exclusion and shared-runtime
  settings. Planted budget failures are expected negative cases, not failed self-tests.
- PASS: `python3 scripts/check_doc_headings.py`, 254 numbered sections across 337 documents; no duplicates.
- PASS: `git diff --check`.
- NOT RUN: full release `--slice-link`, rendered gameplay, cross-platform release smoke, full Rust suites.
  The scripts-only export proves the escaped compilation failure only. It deliberately does not claim
  assets load, the server connects, or a release bundle is playable. The coordinator explicitly deferred
  the 45–75 minute full import; release workflow owns final terminal link/smoke proof.

Baseline exported runtime (exit 1):

```text
SCRIPT ERROR: Parse Error: Identifier "PhysicsEngineProbe" not declared in the current scope.
   at: GDScript::reload (res://scripts/slice/slice_probe.gd:1004)
ERROR: Failed to load script "res://scripts/slice/slice_probe.gd" with error "Parse error".
EXPORT PHYSICS FAIL: SliceProbe cannot instantiate
```

Fixed exported runtime (exit 0, no script errors):

```text
Godot Engine v4.7.2.stable.official.ed1daf0bf
EXPORT PHYSICS PASS: real SliceProbe compiles without tools
```

The existing release bundle probe is the permanent regression owner and already rejects the baseline;
no duplicate routine test pipeline is introduced. The exact local bounded reproduction is preserved
below, including the test scene, to keep this evidence reviewable without temporary files.

### Logic review

Only the internal diagnostic dependency moves; no physics engine is changed. The setting/behavior
branch order, solver iteration read, and `free_rid` remain identical. All source consumers found by
`rg PhysicsEngineProbe clients scripts` were migrated; no other tools global class exists. Runtime
paths explicitly preload the shipped helper. All four 3D export presets still exclude `tools/*`.
The existing drive Jolt assertion and link verdict are untouched. The new `.gd.uid` is tracked with
its script, consistent with this repository. Independent red team inspected the uncommitted patch
and reported no logic findings; final-head review remains a distinct pending item.

### Reproduction script

Save this block outside the checkout and run it through the slot wrapper with the repository root
as its first argument. It uses the frozen baseline SHA, not the current HEAD. It requires macOS and
installed templates. Its only writes are temporary projects and the reported evidence directory.


```python
from pathlib import Path
import shutil, subprocess, sys, tempfile, time
root=Path(sys.argv[1]).resolve()
sys.path.insert(0, str(root/'scripts'))
import ci_gdparse, package
out=Path(tempfile.mkdtemp(prefix='mw-physics-evidence-')); print(out)
def run(label, args, cwd=None):
    start=time.monotonic()
    p=subprocess.run(args,cwd=cwd,capture_output=True,text=True,timeout=180)
    log=p.stdout+p.stderr
    (out/(label+'.log')).write_text(log)
    print(label, 'exit',p.returncode,'seconds',round(time.monotonic()-start,1),flush=True)
    return p.returncode,log
for variant in ('baseline','fixed'):
    with tempfile.TemporaryDirectory(prefix='mw-export-physics-') as tmp:
        project=Path(tmp)/'project'; project.mkdir()
        src=root/'clients/3d-spike'
        for folder in src.iterdir():
            if folder.is_dir() and not folder.name.startswith('.') and ci_gdparse.holds_scripts(folder):
                shutil.copytree(folder,project/folder.name,ignore=shutil.ignore_patterns('.godot'))
        for name in ('project.godot','export_presets.cfg'):
            shutil.copy2(src/name,project/name)
        if variant=='baseline':
            for name in ('scripts/slice/slice_probe.gd','tools/physics_engine.gd'):
                (project/name).write_bytes(subprocess.check_output(['git','show','2b4d36fac345b9325a91fd4313d40e3633e728b0:clients/3d-spike/'+name],cwd=root))
            (project/'scripts/physics_engine.gd').unlink()
        settings=project/'project.godot'
        s=settings.read_text().replace('res://scenes/main.tscn','res://check.tscn').replace('res://scenes/slice.tscn','res://check.tscn')
        s=s.replace('[rendering]','[rendering]\ntextures/vram_compression/import_etc2_astc=true')
        settings.write_text(s)
        (project/'check.tscn').write_text('[gd_scene load_steps=2 format=3]\n[ext_resource type="Script" path="res://check.gd" id="1"]\n[node name="Check" type="Node"]\nscript = ExtResource("1")\n')
        # Load the real runtime dependency closure, but do not instantiate scenes or require assets/server.
        (project/'check.gd').write_text('extends Node\nfunc _ready() -> void:\n\tvar probe = load("res://scripts/slice/slice_probe.gd")\n\tif probe == null or not probe.can_instantiate():\n\t\tprinterr("EXPORT PHYSICS FAIL: SliceProbe cannot instantiate")\n\t\tget_tree().quit(1)\n\t\treturn\n\tif ResourceLoader.exists("res://tools/physics_engine.gd"):\n\t\tprinterr("EXPORT PHYSICS FAIL: tools shipped")\n\t\tget_tree().quit(1)\n\t\treturn\n\tprint("EXPORT PHYSICS PASS: real SliceProbe compiles without tools")\n\tget_tree().quit(0)\n')
        code,log=run(variant+'-import',['godot','--headless','--path',str(project),'--import'])
        assert code==0
        archive=Path(tmp)/'export.zip'
        code,log=run(variant+'-export',['godot','--headless','--path',str(project),'--export-release','macOS',str(archive)])
        assert code==0 and archive.exists(), log[-3000:]
        shipped=out/(variant+' 世界'); shipped.mkdir(exist_ok=True)
        app=package.unzip_app(archive,shipped,'Probe')
        exe=next((app/'Contents/MacOS').iterdir())
        # Remove all project source/cache before starting the release template outside the checkout.
        shutil.rmtree(project)
        code,log=run(variant+'-runtime',[str(exe),'--headless','--log-file',str(out/(variant+'-godot.log'))],cwd=shipped)
        passed=code==0 and 'EXPORT PHYSICS PASS:' in log and not any(p.search(log) for p in ci_gdparse.ERROR_PATTERNS)
        print(variant,'PASS' if passed else 'FAIL',flush=True)
        if variant=='baseline':
            assert not passed and 'PhysicsEngineProbe' in log,log
        else:
            assert passed,log
print('REGRESSION PASS: baseline rejected; fixed exported dependency closure accepted')
```

Tested content SHA-256 `clients/3d-spike/scripts/physics_engine.gd`: `cbe35571e7e2b2ffe634bf61498737fc9e19687827228eb0cd6cfe0c4f507a63`.

Tested content SHA-256 `clients/3d-spike/scripts/physics_engine.gd.uid`: `639237919e20ec9e12506400281544367fa715eceb75167070e71b7fd17b6350`.

Tested content SHA-256 `clients/3d-spike/scripts/slice/slice_probe.gd`: `25e6c46c435ae75a083159b8c2d1a5f815fb51e98f10368a1130f4635444c5c7`.

Tested content SHA-256 `clients/3d-spike/tools/physics_engine.gd`: `9dc635adc96ecf472bf25039f1349e6e63cb360c01461b18ea550bb9f6bb3441`.

### Publication checkpoint

Local executable commit: `67cedbeadaa23f40ad0b4472be8f19996da07075`.
Automatic approval review rejected `git push -u origin fix/packaged-physics-probe` twice. After the
first rejection, read-only `git remote -v` and `gh repo view yuema137/MineWorld --json
nameWithOwner,isPrivate,url` verified the configured origin is the public
`https://github.com/yuema137/MineWorld` repository. The second rejection still requires explicit user
authorization for publishing this payload to that public destination. No bypass was attempted.
Branch publication, draft PR creation and canonical exact-head CI are therefore blocked, not complete.
The coordinator has the exact action and review reason for operator resolution. Implementation remains
locally reviewable; this is not READY FOR OPERATOR REVIEW under the full CI contract.
