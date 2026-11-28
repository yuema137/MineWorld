import json,re,shutil,hashlib
from pathlib import Path
r=Path('/tmp/mineworld-recover-s34e/clients/3d-spike');base=Path('/tmp/mineworld-any-seat-probe/clients/3d-spike');out=Path('/tmp/mineworld-s34e-seat-measure')
b=json.loads((r/'../../presentation/mineworld-default/3D/furniture/furnishings.yaml').read_text())['kinds']; d=json.loads((r/'../../tests/fixtures/observations/furnishings/social-cafe-cafe.json').read_text());slugs=set()
for f in d['place-shape']['furnishings']:
 v=b[f['kind']];slugs.add(v['model'])
 for variant in v.get('dressing',[]):
  for item in variant:slugs.add(item['model'])
records=[]
for slug in sorted(slugs):
 for source in (r/'assets/models'/slug).rglob('*'):
  if not source.is_file():continue
  rel=source.relative_to(r);other=base/rel
  assert other.exists() and hashlib.sha256(source.read_bytes()).digest()==hashlib.sha256(other.read_bytes()).digest(),rel
  if source.suffix!='.import':continue
  for path in re.findall(r'"res://(\.godot/imported/[^"\n]+)"',source.read_text()):
   src=base/path;dest=r/path
   assert src.exists(),src
   dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dest)
   records.append({'path':path,'sha256':hashlib.sha256(dest.read_bytes()).hexdigest(),'source_cache':str(src)})
(out/'engine-cache-provenance.json').write_text(json.dumps({'models':sorted(slugs),'records':records},indent=2)+'\n')
print(len(slugs),'existing cafe model caches staged;',len(records),'entries')
