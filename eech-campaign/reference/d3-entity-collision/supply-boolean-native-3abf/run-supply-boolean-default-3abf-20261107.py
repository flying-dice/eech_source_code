from pathlib import Path
import ctypes,datetime,hashlib,json,os,shutil,subprocess,sys
from ctypes import wintypes
B=Path('R:/d3-evidence');P=B/'supply-boolean-default-3abf-plan-20261107.json';p=json.loads(P.read_bytes());case=sys.argv[1];assert case in ('ordinary_short','original_short');role='ordinary' if case=='ordinary_short' else 'original';S=Path(p['snapshot']);binary=Path(p['profiles'][role]['binary']);root=Path(p['root_base'])/case;out=Path(p['out_base'])/case;stem='supply-boolean-default-3abf-'+case+'-20261107';r=B/(stem+'-process.json');c=B/(stem+'-completion.json');assert not any(x.exists() for x in [root,out,r,c]);out.mkdir();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(x):return hashlib.sha256(Path(x).read_bytes()).hexdigest()
for key in ['review','build','profile_audit','source_binding','release']:assert sha(p[key])==p[key+'_sha256']
clearance=json.loads((B/'independent-supply-boolean-3abf-default-plan-20261107.json').read_bytes());assert clearance['decision']['default_native_launch_clearance'] is True and clearance['plan_sha256']==sha(P) and clearance['controller_sha256']==sha(__file__)
for key in ['input_tool','input_reference']:assert sha(p[key])==p[key+'_sha256']
for name,expected in p['accepted_short_files'].items():assert sha(Path(p['accepted_short'])/name)==expected
assert subprocess.check_output(['git','-C',str(S),'rev-parse','HEAD']).decode().strip()==p['runtime_source'] and not subprocess.check_output(['git','-C',str(S),'status','--porcelain=v1','--untracked-files=all'])
payloads=p['profiles'][role]['payloads'];expected={'eech_dc.dll','eech-world.exe','lua.dll'}|{'lua/'+a.name for a in (S/'eech-campaign/crates/eech-world/lua').glob('*.lua')};assert {a['path'] for a in payloads}==expected and len(payloads)==len(expected) and all(sha(binary/a['path'])==a['sha256'] for a in payloads)
state=dict(controller_pid=os.getpid(),state='running',started_utc=now(),source=p['runtime_source'],profile=role,case=case,root=str(root),output=str(out),plan_sha256=sha(P),controller_sha256=sha(__file__),stages=[],scope='Fresh3abf ordinary or Original establisheddefault after EXPLICITdescriptor-onlyboolean defect correction; established35frame/3.5s complete3files compare80fd, hours0.001/frame100/seed1 unchanged. No diagnosticintervention or claimed natural nonzeromask reachability/nativeauthority/full40/all8/M6/height/Task49/D3 acceptance.')
def persist():r.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8')
def run(label,args):
 with (out/(label+'.stdout.log')).open('xb') as stdout,(out/(label+'.stderr.log')).open('xb') as stderr:
  x=subprocess.Popen([str(a) for a in args],cwd=out,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);fs=[wintypes.FILETIME() for _ in range(4)];fn=ctypes.windll.kernel32.GetProcessTimes;fn.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;fn.restype=wintypes.BOOL;assert fn(wintypes.HANDLE(int(x._handle)),*[ctypes.byref(a) for a in fs]);n=(fs[0].dwHighDateTime<<32)|fs[0].dwLowDateTime;row=dict(stage=label,pid=x.pid,birth_utc=datetime.datetime.fromtimestamp((n-116444736000000000)/10000000,datetime.timezone.utc).isoformat(),argv=[str(a) for a in args],state='running',started_utc=now());state['stages'].append(row);persist();code=x.wait()
 row.update(state='terminal',actual_exit=code,completed_utc=now());persist();return code
def inputs(label):return run(label,[sys.executable,'-I','-B','-X','utf8',p['input_tool'],root,'--compare',p['input_reference'],'--output',out/(label+'.json')])
persist()
try:
 shutil.copytree(p['pristine'],root);assert inputs('pre-inputs')==0
 args=[binary/'eech-world.exe',binary/'lua/campaign.lua','root='+root.as_posix(),'scenario=lebanon_retail','out='+(out/'model.jsonl').as_posix(),'metrics='+(out/'metrics.json').as_posix(),'hours=0.001','frame_ms=100','seed=1','record=1','record_every=10','observe_every=1','observe_supply=1','acmi='+(out/'recording.acmi').as_posix(),'observe='+(out/'observations.jsonl').as_posix()];state['actual_native_exit']=run('native',args);post=inputs('post-inputs');assert state['actual_native_exit']==post==0
 files=[]
 for name in ['recording.acmi','observations.jsonl','metrics.json']:
  candidate=out/name;baseline=Path(p['accepted_short'])/name;eq=candidate.read_bytes()==baseline.read_bytes();files.append(dict(file=name,bytes=candidate.stat().st_size,candidate_sha256=sha(candidate),baseline_sha256=sha(baseline),complete_literal_equal=eq));assert eq,name
 state.update(files=files,all_three_complete_literal_equal=True,verified_payloads=len(payloads));assert all(sha(binary/a['path'])==a['sha256'] for a in payloads);assert not subprocess.check_output(['git','-C',str(S),'status','--porcelain=v1','--untracked-files=all']);state.update(state='terminal',actual_exit=0,completed_utc=now());persist()
except BaseException as exc:
 state.update(state='terminal',actual_exit=1,error=repr(exc),completed_utc=now());persist();c.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8');raise
c.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8');print(json.dumps(dict(case=case,actual_exit=0,native_exit=state['actual_native_exit'],all3complete_literal_equal=True)))
