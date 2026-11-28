from surfaces import *
import subprocess

def world(path):
 return json.loads(subprocess.check_output(['ruby','-ryaml','-rjson','-e','puts JSON.generate(YAML.load_file(ARGV[0]))',str(ROOT/path)],text=True))
def rotation(front,facing):
 a=math.radians((front-facing)/1000);return np.array([[math.cos(a),math.sin(a)],[-math.sin(a),math.cos(a)]])
def xy(at):return np.array([at['x'],at['y']],float)/1000
surfaces=json.loads((OUT/'candidate-surfaces.json').read_text());bindings=json.loads((ROOT/'presentation/mineworld-default/3D/furniture/furnishings.yaml').read_text())['kinds']
results={'cafe':[],'lakeside':[]}
for key in ['social-cafe','market-town']:
 d=world(Path('worlds')/key/'places/cafe.yaml');fs={f['key']:f for f in d['body']['furnishings']}
 mapping=['stool.1','stool.2','stool.3','table.east.chair-w','table.east.chair-e','table.north.chair-w','table.north.chair-e','table.back.chair-w','table.back.chair-e']
 for p,prop in zip(d['sit-points'],mapping):
  f=fs[prop];binding=bindings[f['kind']];slug=binding['model'];r=rotation(binding['front_md'],f['facing']);surf=surfaces[slug];c=np.array(surf['area_weighted_center_m']);delta=r@c[[0,2]]*np.array([1,-1]);target=xy(f['at'])+delta;origin=xy(p['at']);yaw=math.radians(p['facing']/1000);u=np.array([math.sin(yaw),math.cos(yaw)]);v=np.array([math.cos(yaw),-math.sin(yaw)]);projected=origin+u*np.dot(target-origin,u)
  t=np.load(OUT/(slug+'.seat-triangles.npy'));worldtri=t[:,:,[0,2]]@r.T*np.array([1,-1])+xy(f['at']);lateral=(worldtri-origin)@v;intersects=bool(np.any((lateral.min(axis=1)<=0)&(lateral.max(axis=1)>=0)));local=r.T@((projected-xy(f['at']))*np.array([1,-1]));h=height_at(t,*local)
  results['cafe'].append({'world':key,'point':p['key'],'prop':prop,'model':slug,'furnishing_facing_md':f['facing'],'point_facing_md':p['facing'],'geometric_surface_centroid_target_mm':(target*1000).tolist(),'candidate_displacement_m':(target-origin).tolist(),'candidate_height_m':surf['center_ray_seat_top_m'],'centroid_projection_lateral_mm':float(np.dot(target-origin,v)*1000),'scalar_line_intersects_candidate_triangles':intersects,'scalar_line_min_lateral_gap_mm':0 if intersects else float(np.min(np.abs(lateral))*1000),'projected_centroid_ray_surface_m':h})
places={k:world(Path('worlds/lakeside/places')/(k+'.yaml')) for k in ['bakery','pier','square','station','station-platform']}
deps=json.loads((ROOT/'presentation/mineworld-default/3D/sit_points/lakeside.yaml').read_text())['points']
for p in deps:
 name=next(k for k,d in places.items() if set(p['tags'])<=set(d.get('tags',[])) and not set(p.get('unless',[]))&set(d.get('tags',[])))
 d=places[name];f=next(f for f in d['body']['furnishings'] if f['key']==p['prop']);b=bindings[f['kind']];w=next(s for s in d['sit-points'] if s['key']==p['point']);a=xy(w['at']);yaw=math.radians(w['facing']/1000);pose=a-np.array([math.sin(yaw),math.cos(yaw)])*p['seat_offset_m'];surface=None;support=False;detail={}
 if 'model' in b:
  r=rotation(b['front_md'],f['facing']);local=r.T@((pose-xy(f['at']))*np.array([1,-1]));t=np.load(OUT/(b['model']+'.seat-triangles.npy'));surface=height_at(t,*local);support=surface is not None;detail={'model':b['model'],'local_xz_m':local.tolist()}
 elif b.get('form')=='bench':
  s=next(s for s in d['body']['solids'] if s.get('furnishing')==p['prop']);lo=xy(s['min']);hi=xy(s['max']);support=bool(np.all(pose>=lo)&np.all(pose<=hi));surface=s['height']/1000 if support else None;detail={'primitive':'solid-sized continuous top','support_rectangle_mm':[s['min'],s['max']]}
 elif b.get('piece')=='bench_row':
  s=next(s for s in d['body']['solids'] if s.get('furnishing')==p['prop']);size=xy(s['max'])-xy(s['min']);r=rotation(b['front_md'],f['facing']);angle=math.radians((b['front_md']-f['facing'])/1000)
  if abs(math.sin(angle))>.7:size=size[::-1]
  length,depth=size;local=r.T@((pose-xy(f['at']))*np.array([1,-1]));seat_z=-depth/2+.42/2+.02;slats=[(seat_z-.42/2+.42*(k+.5)/3-(.42/3-.012)/2,seat_z-.42/2+.42*(k+.5)/3+(.42/3-.012)/2) for k in range(3)];support=abs(local[0])<=(length-.06)/2 and any(lo<=local[1]<=hi for lo,hi in slats);surface=s['height']/1000-.0025 if support else None
  count=max(2,math.ceil(length/1.5)+1);standards=[-length/2+.04+(length-.08)*i/(count-1) for i in range(count)];arm=any(abs(local[0]-x)<=.03 and abs(local[1]-seat_z)<=.42/2 for x in standards)
  detail={'primitive':'three slats','local_xz_m':local.tolist(),'slat_z_intervals_m':slats,'armrest_above_sample':arm,'armrest_top_m':s['height']/1000+.24 if arm else None}
 results['lakeside'].append({'place':name,'point':p['point'],'pose_mm':(pose*1000).tolist(),'authored_height_m':p['seat_height_m'],'sample_over_candidate_seat':bool(support),'surface_height_m':surface,'height_gap_mm':None if surface is None else (p['seat_height_m']-surface)*1000,**detail})
(OUT/'placement-measurements.json').write_text(json.dumps(results,indent=2)+'\n')
for row in results['cafe'][:9]:print(row['point'], 'vector',np.round(row['candidate_displacement_m'],4),'height',round(row['candidate_height_m'],4),'scalar_intersection',row['scalar_line_intersects_candidate_triangles'],'gap_mm',round(row['scalar_line_min_lateral_gap_mm'],1))
for place in places:
 rows=[r for r in results['lakeside'] if r['place']==place];print(place,len(rows),'supported',sum(r['sample_over_candidate_seat'] for r in rows),'heightgaps_mm',[round(r['height_gap_mm'],2) if r['height_gap_mm'] is not None else None for r in rows],'arms',[r['point'] for r in rows if r.get('armrest_above_sample')])
