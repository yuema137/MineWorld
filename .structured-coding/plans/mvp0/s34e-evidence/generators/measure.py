import json, hashlib, math, struct
from pathlib import Path
import numpy as np
ROOT=Path('/tmp/mineworld-recover-s34e'); OUT=Path('/tmp/mineworld-s34e-seat-measure')
SLUGS=['dining_chair_02','painted_wooden_chair_01','bar_chair_round_01']
DT={5126:'<f4',5125:'<u4',5123:'<u2',5121:'u1'}; NC={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def load(slug):
 p=ROOT/'clients/3d-spike/assets/models'/slug/(slug+'.gltf');g=json.loads(p.read_text());bs=[(p.parent/b['uri']).read_bytes() for b in g['buffers']]
 def acc(i):
  a=g['accessors'][i];assert not a.get('sparse');v=g['bufferViews'][a['bufferView']];dt=np.dtype(DT[a['componentType']]);n=NC[a['type']];stride=v.get('byteStride',dt.itemsize*n);off=v.get('byteOffset',0)+a.get('byteOffset',0)
  return np.ndarray((a['count'],n),dt,buffer=bs[v['buffer']],offset=off,strides=(stride,dt.itemsize)).copy()
 tris=[]
 def visit(i,parent):
  node=g['nodes'][i];m=np.eye(4)
  if 'matrix' in node:m=np.array(node['matrix']).reshape(4,4).T
  else:
   x,y,z,w=node.get('rotation',[0,0,0,1]);r=np.array([[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w)],[2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w)],[2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y)]])
   m[:3,:3]=r@np.diag(node.get('scale',[1,1,1]));m[:3,3]=node.get('translation',[0,0,0])
  m=parent@m
  if 'mesh' in node:
   for pr in g['meshes'][node['mesh']]['primitives']:
    assert pr.get('mode',4)==4
    v=acc(pr['attributes']['POSITION']).astype(float);v=np.einsum("ij,kj->ik",v,m[:3,:3])+m[:3,3];assert np.isfinite(v).all()
    idx=acc(pr['indices']).reshape(-1) if 'indices' in pr else np.arange(len(v));tris.append(v[idx.reshape(-1,3)])
  for child in node.get('children',[]):visit(child,m)
 for i in g['scenes'][g.get('scene',0)]['nodes']:visit(i,np.eye(4))
 t=np.concatenate(tris);lo=t.min(axis=(0,1));hi=t.max(axis=(0,1));offset=np.array([(lo[0]+hi[0])/2,lo[1],(lo[2]+hi[2])/2]);t-=offset
 hashes={str(f.relative_to(ROOT)):sha(f) for f in [p,p.with_suffix('.bin'),p.with_suffix('.gltf.import')]}
 return t,{'source_hashes':hashes,'nodes':g['nodes'],'raw_bounds_m':[lo.tolist(),hi.tolist()],'centering_and_floor_subtraction_m':offset.tolist(),'triangle_count':len(t)}
def facets(t):
 cross=np.cross(t[:,1]-t[:,0],t[:,2]-t[:,0]);lens=np.linalg.norm(cross,axis=1);up=np.divide(cross[:,1],lens,out=np.zeros(len(lens)),where=lens>1e-12)
 return t.mean(axis=1),lens/2,up
if __name__=='__main__':
 report={}
 for slug in SLUGS:
  t,d=load(slug);c,area,up=facets(t);mask=up>math.cos(math.radians(20));bins=np.floor(c[:,1]/.01).astype(int);hist={int(b):float(area[mask&(bins==b)].sum()) for b in np.unique(bins[mask])};peaks=sorted(hist.items(),key=lambda x:-x[1])[:14]
  d['upward_area_by_10mm_band_top14']=[{'height_band_m':[b*.01,(b+1)*.01],'area_m2':a} for b,a in peaks];report[slug]=d;np.save(OUT/(slug+'.triangles.npy'),t)
  print(slug,'bounds',np.round(np.array(d['raw_bounds_m']),4),'top area bands',[(b,round(a,5)) for b,a in peaks])
 (OUT/'mesh-survey.json').write_text(json.dumps(report,indent=2)+'\n')
