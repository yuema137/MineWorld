import json, hashlib
from pathlib import Path
for world in ['social-cafe','market-town']:
 p=Path('/Users/yuema137/mineworld-worktrees/_scratch/s34e-redo/records')/f'posture-{world}.json'; raw=p.read_bytes(); data=json.loads(raw)
 frames=[]
 for f in data['frames']:
  frame={k:f[k] for k in ['observer','at','self_location']};frame['entities']=[]
  for e in f['entities']:
   if e['entity_type'] not in ['person','place']:continue
   entity={k:e[k] for k in ['id','entity_type','tags','location'] if k in e}
   entity['components']=[c for c in e.get('components',[]) if c['component_type'] in ['sit-points','posture-state']]
   frame['entities'].append(entity)
  frames.append(frame)
 data['frames']=frames
 data['about']='Projected real server observations from client_2d_sit (re-recorded 2026-11-27 with the original recorder and projection; the 2026-11-22 recording was lost): before sit, seated, after requested stand, second seated. Only identity/location/tags and posture disclosures retained. Raw recording SHA256 '+hashlib.sha256(raw).hexdigest()+'. No fabricated posture or movement in these frames; counterfactual stillness/rename/mirror belong to test-generated variants.'
 Path('tests/fixtures/observations/sitting',f'posture-{world}.json').write_text(json.dumps(data,indent=2)+'\n')
