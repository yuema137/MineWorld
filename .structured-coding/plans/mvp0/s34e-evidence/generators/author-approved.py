from pathlib import Path
import json,hashlib
import numpy as np
R=Path('/tmp/mineworld-recover-s34e');O=Path('/tmp/mineworld-s34e-seat-measure')
def read(f):return json.loads((O/f).read_text())
t=read('engine-targets.json');worlds=read('engine-world-inputs.json');surfaces=read('candidate-surfaces.json');meshes=read('mesh-survey.json')
for world in worlds:
 points=[]
 for row in t['targets']:
  if row['world']!=world:continue
  p=next(p for p in worlds[world]['sit-points'] if p['key']==row['point'])
  points.append({'tags':worlds[world]['tags'],'point':row['point'],'group':p.get('group',''),'prop':row['prop'],'seat_height_m':row['proposed_seat_height_m'],'seat_displacement_m':row['proposed_seat_displacement_m']})
 data={'format':2,'scope':{'world':world},'about':'S34-B approved central drawing anchors, place-local east/north metres from disclosed sit points; semantic world positions unchanged. Independent engine/mesh evidence: tools/cli/tests/fixtures/seat-surfaces.json. These are drawing anchors, not body-fit/contact acceptance (S34-f).','points':points}
 (R/'presentation/mineworld-default/3D/sit_points'/f'{world}.yaml').write_text(json.dumps(data,indent=2)+'\n')
provenance=read('engine-closure-provenance.json');hashes=provenance['source_sha256']
for m in meshes.values():hashes.update(m['source_hashes'])
legacy=read('legacy-baseline.json');legacy_surface=read('placement-measurements.json')['lakeside']
for slug,s in surfaces.items():s['triangles_m']=np.load(O/(slug+'.seat-triangles.npy')).tolist();s['normalization']=meshes[slug]['centering_and_floor_subtraction_m']
# Remove proposal values from independent target fixture: expected anchors were measured before authoring.
for row in t['targets']:
 del row['proposed_seat_displacement_m'];del row['proposed_seat_height_m'];del row['height_delta_vs_offline_mm']
 for r in read('placement-measurements.json')['cafe']:
  if (r['world'],r['point'])==(row['world'],row['point']):row['model']=r['model'];break
fixture={'about':'Independent pre-authoring S34-B geometry/engine measurements at 4ce72437; no expected target is derived from depiction displacement. Central drawing anchors only; S34-f owns body/foot/arm clearance. Original measurement reports reviewed PASS; operator approved 2026-11-22. Triangle coordinates are source mesh metres normalized to whole-mesh XZ center/lowest Y; engine import differs <0.066 mm. Legacy executable outputs captured before code change.','source_sha256':hashes,'measurement_artifacts_sha256':provenance['artifacts_sha256'],'surfaces':surfaces,'targets':t['targets'],'legacy':legacy,'legacy_surfaces':legacy_surface}
f=R/'tools/cli/tests/fixtures/seat-surfaces.json';f.parent.mkdir(parents=True,exist_ok=True);f.write_text(json.dumps(fixture,indent=2)+'\n');print('wrote 18 approved rows; independent fixture bytes',f.stat().st_size)
