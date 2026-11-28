import os
os.environ['MPLCONFIGDIR']='/tmp/mineworld-s34e-seat-measure/mpl'
import matplotlib;matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.collections import PolyCollection
from measure import *
def hull(points):
 p=sorted(set(map(tuple,points)))
 def cross(o,a,b):return (a[0]-o[0])*(b[1]-o[1])-(a[1]-o[1])*(b[0]-o[0])
 lo=[];hi=[]
 for a in p:
  while len(lo)>1 and cross(lo[-2],lo[-1],a)<=0:lo.pop()
  lo.append(a)
 for a in reversed(p):
  while len(hi)>1 and cross(hi[-2],hi[-1],a)<=0:hi.pop()
  hi.append(a)
 return lo[:-1]+hi[:-1]
def height_at(t,x,z):
 a=t[:,0];b=t[:,1]-a;c=t[:,2]-a;dx=x-a[:,0];dz=z-a[:,2];det=b[:,0]*c[:,2]-b[:,2]*c[:,0]
 valid=np.abs(det)>1e-12
 u=np.divide(dx*c[:,2]-dz*c[:,0],det,out=np.full(len(t),np.nan),where=valid);v=np.divide(b[:,0]*dz-b[:,2]*dx,det,out=np.full(len(t),np.nan),where=valid)
 mask=valid&(u>=-1e-8)&(v>=-1e-8)&(u+v<=1+1e-8)
 ys=(a[:,1]+u*b[:,1]+v*c[:,1])[mask]
 return None if not len(ys) else float(ys.max())
if __name__=='__main__':
 survey=json.loads((OUT/'mesh-survey.json').read_text()); result={}
 windows={'dining_chair_02':(.40,.48),'painted_wooden_chair_01':(.42,.48),'bar_chair_round_01':(.67,.71)}
 fig,axs=plt.subplots(2,3,figsize=(14,8),layout='constrained')
 for j,slug in enumerate(SLUGS):
  t=np.load(OUT/(slug+'.triangles.npy'));assert np.isfinite(t).all();centers,area,up=facets(t);lo,hi=windows[slug];mask=(up>math.cos(math.radians(20)))&(centers[:,1]>=lo)&(centers[:,1]<=hi);seat=t[mask];a=area[mask];c=centers[mask];center=(c*a[:,None]).sum(axis=0)/a.sum();outline=hull(seat[:,:,[0,2]].reshape(-1,2));verts=seat.reshape(-1,3)
  result[slug]={'candidate_filter':{'centroid_height_m':[lo,hi],'max_normal_tilt_deg':20},'surface_triangles':len(seat),'area_m2':float(a.sum()),'bounds_m':[verts.min(axis=0).tolist(),verts.max(axis=0).tolist()],'area_weighted_center_m':center.tolist(),'center_ray_seat_top_m':height_at(seat,center[0],center[2]),'outer_hull_xz_m':outline,'warning':'Hull is an outer envelope, not proof of continuous support; ray against selected original triangles is authoritative for sampled locations. Geometric center is not a validated human pelvis target.'}
  np.save(OUT/(slug+'.seat-triangles.npy'),seat)
  for row,dims in enumerate([[0,2],[2,1]]):
   ax=axs[row,j];ax.add_collection(PolyCollection(t[:,:,dims],facecolors='#dddddd',edgecolors='none',alpha=.2));ax.add_collection(PolyCollection(seat[:,:,dims],facecolors='#d97316',edgecolors='none',alpha=.7));ax.plot(center[dims[0]],center[dims[1]],'kx',markersize=7);ax.autoscale();ax.set_aspect('equal');ax.set_xlabel(['x (m)','z (m)'][row]);ax.set_ylabel(['z (m)','height (m)'][row]);ax.set_title(slug+'\n'+('top view' if row==0 else 'side view'))
 fig.suptitle('Existing mesh geometry, centered and floor-mounted; orange = candidate seat triangles\nBlack cross = geometric area centroid (not a calibrated pelvis target)')
 fig.savefig(OUT/'seat-surfaces.png',dpi=160)
 (OUT/'candidate-surfaces.json').write_text(json.dumps(result,indent=2)+'\n')
 for slug,d in result.items():print(slug,'height',d['center_ray_seat_top_m'],'center',d['area_weighted_center_m'],'bounds',d['bounds_m'])
