import os,sys,concurrent.futures as cf
MAIN='<repo>'
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
for p in (MAIN+'/work/emitpred/pipeline',MAIN+'/work/w-roots',MAIN+'/work/w-refs',MAIN+'/work/w-skip',MAIN+'/work/w-db'):
    sys.path.insert(0,p)
os.environ['C2RS_LANEROOT']=MAIN
import alias as al
def one(r):
    src,e=r[0],r[1]
    b=[x[:-2] for x in os.listdir(e) if x.startswith('_CL_') and x.endswith('gl')][0]
    glb=open(os.path.join(e,b+'gl'),'rb').read()
    A,_,st=al.scan(glb)
    E=set(x for x in open(MAIN+'/work/w-emit/truth/'+src.replace('/','__')+'.txt').read().split() if x)
    return (len(set(A)&E), len(set(A.values())&E), sum(1 for n in E if n.startswith('??_E')), st['tag10'], st['bound'])
if __name__=='__main__':
    rows=[l.rstrip('\n').split('\t') for l in open('cacheidx.tsv')]
    tot=[0]*5
    with cf.ProcessPoolExecutor(max_workers=6) as ex:
        for v in ex.map(one, rows, chunksize=8):
            for i,x in enumerate(v): tot[i]+=x
    print("alias NAMES that are emitted (dom(alias) in E): %d" % tot[0])
    print("alias TARGETS that are emitted:                 %d" % tot[1])
    print("emitted names starting ??_E (all TUs):          %d" % tot[2])
    print("tag-0x10 records %d ; bound %d" % (tot[3],tot[4]))
