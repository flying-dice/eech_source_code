from pathlib import Path
import datetime,hashlib,json,os,subprocess,traceback,sys,re
b=Path('R:/d3-evidence');repo=b/'collision-characterization-20261029'
source,meta,map_arg,review_arg,review_sha=sys.argv[1:]
assert re.fullmatch('[0-9a-f]{40}',source) and re.fullmatch('[0-9a-f]{40}',meta) and re.fullmatch('[0-9a-f]{64}',review_sha)
review_path=Path(review_arg);assert hashlib.sha256(review_path.read_bytes()).hexdigest()==review_sha
review=json.loads(review_path.read_bytes());assert review['decision']['native_launch_clearance'] is True and review['decision']['findings']==[] and source in json.dumps(review['source'])
completion=json.loads((b/'claude-d3-city-remove-identity-20261106-completion.json').read_bytes());assert completion['state']=='terminal' and completion['actual_exit']==0 and completion['result_is_error'] is False and completion['assistant_models']==['claude-sonnet-5-5']
short=source[:9]
snap=Path(f'D:/EECH-Migration-Evidence/city-lifecycle-witness-{short}-20261106-source');receipt=b/f'city-lifecycle-witness-windows-freeze-{short}-20261106.json';assert not receipt.exists() and not snap.exists()
assert snap.parent.resolve()==Path('D:/EECH-Migration-Evidence').resolve()
git=lambda root,*args:subprocess.check_output(['git','-C',str(root),*args])
assert git(repo,'rev-parse','HEAD').decode().strip()==meta and git(repo,'status','--porcelain','--untracked-files=all')==b''
assert Path(map_arg).is_file()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();state=dict(controller_pid=os.getpid(),state='running',source=source,snapshot=str(snap),started_utc=now(),controller_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),source_review=str(review_path),source_review_sha256=review_sha,steps=[],scope='Read-only source freeze preparation. No build/native launch or independent clearance; review gate follows. Existing campaigns/caches/roots untouched.')
save=lambda:receipt.write_text(json.dumps(state,indent=2)+'\n');save()
try:
 commands=[['git','clone','--no-checkout','--no-hardlinks','-c','core.longpaths=true','-c','core.autocrlf=false','-c','core.eol=lf',str(repo),str(snap)],['git','-C',str(snap),'config','--local','core.longpaths','true'],['git','-C',str(snap),'config','--local','core.autocrlf','false'],['git','-C',str(snap),'config','--local','core.eol','lf'],['git','-C',str(snap),'checkout','--detach',source]]
 for i,args in enumerate(commands):
  with (b/f'city-lifecycle-witness-windows-freeze-{short}-step{i}.stdout.log').open('xb') as out,(b/f'city-lifecycle-witness-windows-freeze-{short}-step{i}.stderr.log').open('xb') as err:
   child=subprocess.Popen(args,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);row=dict(args=args,pid=child.pid,started_utc=now());state['steps'].append(row);save();code=child.wait()
  row.update(actual_exit=code,completed_utc=now());save();assert code==0
 assert git(snap,'rev-parse','HEAD').decode().strip()==source and git(snap,'status','--porcelain','--untracked-files=all')==b''
 assert git(snap,'config','--local','--get','core.longpaths').decode().strip()=='true' and not (snap/'.git/objects/info/alternates').exists()
 maps=json.loads(Path(map_arg).read_bytes());paths=sorted(set(maps['files'])|{'.github/workflows/eech-campaign.yml'})
 tree={}
 for row in git(snap,'ls-tree','-r','-z',source).split(b'\0'):
  if row:
   head,p=row.split(b'\t',1);tree[p.decode()]=head.decode().split()[2]
 ids=sorted({tree[p] for p in paths});raw=subprocess.check_output(['git','-C',str(snap),'cat-file','--batch'],input=('\n'.join(ids)+'\n').encode());offset=0;blobs={}
 for bid in ids:
  end=raw.index(b'\n',offset);actual,kind,n=raw[offset:end].decode().split();assert actual==bid and kind=='blob';n=int(n);offset=end+1;blobs[bid]=raw[offset:offset+n];assert raw[offset+n:offset+n+1]==b'\n';offset+=n+1
 assert offset==len(raw);rows={}
 for p in paths:
  actual=(snap/p).read_bytes();assert actual==blobs[tree[p]],p;rows[p]=dict(bytes=len(actual),sha256=hashlib.sha256(actual).hexdigest(),git_blob=tree[p])
 mp=b/f'city-lifecycle-witness-windows-snapshot-{short}-inputs-20261106.json';assert not mp.exists();mp.write_text(json.dumps(dict(source=source,snapshot=str(snap),files=rows),indent=2)+'\n')
 assert git(snap,'status','--porcelain','--untracked-files=all')==b''
 state.update(state='terminal',actual_exit=0,completed_utc=now(),all_snapshot_raw_bytes_equal_Git=True,verified_distinct_source_files=len(paths),input_map=str(mp),input_map_sha256=hashlib.sha256(mp.read_bytes()).hexdigest(),full_status_clean=True,longpaths_set_before_checkout=True,no_hardlinks_or_alternates=True);save()
except Exception:
 state.update(state='terminal',actual_exit=1,error=traceback.format_exc(),completed_utc=now());save();raise
print(json.dumps({k:state[k] for k in ['source','snapshot','actual_exit','all_snapshot_raw_bytes_equal_Git','verified_distinct_source_files','full_status_clean','longpaths_set_before_checkout']}))
