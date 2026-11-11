"""Execute the reviewed finite plan once; retain every child and unreached gate."""
from pathlib import Path
import ctypes,datetime,hashlib,json,os,re,shutil,subprocess,sys
from ctypes import wintypes
B=Path('R:/d3-evidence');P=B/'physical-flags-witness-5d2c-native-plan-20261107.json'
p=json.loads(P.read_bytes()); now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def load(path):return json.loads(Path(path).read_bytes())
def git(*args):return subprocess.check_output(['git','-C',p['snapshot'],*args]).decode().strip()
clearance=Path(sys.argv[1]);review=load(clearance)
assert review['decision']['native_controller_launch_clearance'] is True
assert review['plan_sha256']==sha(P) and review['controller_sha256']==sha(__file__)
assert review['source_review_sha256']==sha(review['source_review'])
assert review['source_review_sha256']==p['source_review_sha256']
for row in p['bindings']:assert sha(row['path'])==row['sha256'],row['path']
assert sha(p['driver'])==p['native_plan']['driver_literal']['sha256']
assert git('rev-parse','HEAD')==p['runtime_source'] and not git('status','--porcelain=v1','--untracked-files=all')
def payload_check():
 for role,profile in p['profiles'].items():
  assert len(profile['payloads'])==90
  for row in profile['payloads']:
   path=Path(profile['binary'])/row['path'];assert path.stat().st_size==row['bytes'] and sha(path)==row['sha256'],str(path)
payload_check()
R=Path(p['root_base']);O=Path(p['out_base']);receipt=B/'physical-flags-witness-5d2c-native-20261107-process.json';completion=B/'physical-flags-witness-5d2c-native-20261107-completion.json'
assert not any(x.exists() for x in (R,O,receipt,completion));R.mkdir();O.mkdir()
s=dict(controller_pid=os.getpid(),state='running',started_utc=now(),source=p['runtime_source'],driver=p['driver'],driver_sha256=sha(p['driver']),plan_sha256=sha(P),controller_sha256=sha(__file__),launch_review=str(clearance),launch_review_sha256=sha(clearance),cases=[],stages=[],scope=p['scope'])
def persist():
 temp=receipt.with_suffix('.tmp');temp.write_text(json.dumps(s,indent=2)+'\n',encoding='utf-8');temp.replace(receipt)
def run(label,args,cwd):
 with (cwd/(label+'.stdout.log')).open('xb') as out,(cwd/(label+'.stderr.log')).open('xb') as err:
  child=subprocess.Popen([str(a) for a in args],cwd=cwd,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW)
  fs=[wintypes.FILETIME() for _ in range(4)];fn=ctypes.windll.kernel32.GetProcessTimes;fn.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;fn.restype=wintypes.BOOL
  assert fn(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(a) for a in fs]);ticks=(fs[0].dwHighDateTime<<32)|fs[0].dwLowDateTime
  row=dict(stage=label,pid=child.pid,birth_utc=datetime.datetime.fromtimestamp((ticks-116444736000000000)/10000000,datetime.timezone.utc).isoformat(),argv=[str(a) for a in args],cwd=str(cwd),started_utc=now(),state='running');s['stages'].append(row);persist();code=child.wait()
 row.update(state='terminal',actual_exit=code,completed_utc=now(),stdout_sha256=sha(cwd/(label+'.stdout.log')),stderr_sha256=sha(cwd/(label+'.stderr.log')));persist();return code
def inputs(root,out,label):
 return run(label,[sys.executable,'-I','-B','-X','utf8',p['input_tool'],root,'--compare',p['input_reference'],'--output',out/(label+'.json')],out)
def report_facts(path,mode):
 text=path.read_text(encoding='utf-8');lines=text.splitlines();verdict=[line for line in lines if line.startswith('pf-witness-verdict ')];assert len(verdict)==1 and lines[-1]==verdict[0]
 match=re.fullmatch(r'pf-witness-verdict mode='+re.escape(mode)+r' status=(PASS|UNREACHED|FAILURE) detail=(.+)',verdict[0]);assert match
 counts=[line for line in lines if line.startswith('pf-witness-counts ')];assert len(counts)==1
 values=dict(token.split('=',1) for token in counts[0].split()[1:]);assert len(values)==len(counts[0].split())-1
 first=[]
 for line in lines:
  if line.startswith('pf-witness-first '):
   pairs=dict(token.split('=',1) for token in line.split()[1:]);first.append(pairs['event'])
 consume=[line for line in lines if line.startswith('pf-witness-consume ')]
 assert len(consume)<=1
 pattern=r'0x[0-9a-f]{8}'
 complete_consume=bool(consume and re.fullmatch(r'pf-witness-consume fact='+pattern+r' expected='+pattern+r' host='+pattern+r' real_word='+pattern+r' answered_word='+pattern+r' changed=[01]',consume[0]))
 return dict(path=str(path),sha256=sha(path),bytes=path.stat().st_size,status=match[1],detail=match[2],counts=values,first_events=first,complete_consume=complete_consume)
persist()
try:
 reports={};steps=p['native_plan']['steps'];assert len(steps)==23 and [v['step'] for v in steps]==list(range(1,24))
 for step in steps:
  n=step['step'];mode=step['mode'];case=dict(step=n,mode=mode,profile=step['binaries'],state='pending');s['cases'].append(case)
  reason=None
  for need in step['needs']:
   if need not in reports or reports[need]['status']!='PASS':reason='required positive step '+str(need)+' did not PASS'
  if n>=3 and reason is None:
   event='read_consumable' if mode=='read_changed' else 'group_second' if mode=='group_duplicate_key' else mode.split('_',1)[0]
   if event not in reports[2]['first_events']:reason='paired positive control did not reach '+event
   if mode=='read_changed' and reports[2]['counts'].get('reads_consumable')=='0':reason='paired positive control did not consume a read'
   if mode=='read_changed' and not reports[2]['complete_consume']:reason='paired positive control has no complete carried-consume tuple'
  if reason:
   case.update(state='not_launched',status='UNREACHED_PREREQUISITE',detail=reason);persist();continue
  case_name=f'{n:02d}-{mode}';root=R/case_name;out=O/case_name;assert not root.exists() and not out.exists();out.mkdir();case.update(root=str(root),output=str(out),state='preparing');persist()
  shutil.copytree(p['pristine'],root);assert inputs(root,out,'pre-inputs')==0
  references={1:reports[1]['path']} if n==2 else {2:reports[2]['path']} if n>=3 else {}
  hashes={str(key):sha(value) for key,value in references.items()};case['reference_reports']=dict(paths=references,sha256=hashes);persist()
  args=[]
  for arg in step['argv_after_script']:
   arg=arg.replace('<fresh verified retail root>',root.as_posix()).replace('<new report path, created exclusively>',(out/'witness.txt').as_posix())
   for number,path in references.items():arg=arg.replace(f'<step {number} report>',Path(path).as_posix())
   assert '<' not in arg and '>' not in arg;args.append(arg)
  role=step['binaries'];payload_check();assert sha(p['driver'])==p['native_plan']['driver_literal']['sha256'];case['state']='running';persist()
  native=run('native',[Path(p['profiles'][role]['binary'])/'eech-world.exe',p['driver'],*args],out)
  case['native_exit']=native;post=inputs(root,out,'post-inputs');case['post_inputs_exit']=post;assert post==0
  assert all(sha(value)==hashes[str(key)] for key,value in references.items());payload_check();assert sha(p['driver'])==p['native_plan']['driver_literal']['sha256']
  report=report_facts(out/'witness.txt',mode);case.update(state='terminal',**report);reports[n]=report;persist()
  assert (native==0 and report['status']=='PASS') or (native!=0 and report['status']=='UNREACHED'),case
 assert not git('status','--porcelain=v1','--untracked-files=all');payload_check()
 s.update(state='terminal',actual_exit=0,completed_utc=now(),native_pass=sum(c.get('status')=='PASS' for c in s['cases']),unreached=sum(c.get('status','').startswith('UNREACHED') for c in s['cases']),interpretation='Controller0 means the predeclared finite plan was executed and limitations retained. UNREACHED or unlaunched prerequisite is never a passing native case; no whole flag-store/native/consumer/D3 acceptance.');persist()
except BaseException as exc:
 s.update(state='terminal',actual_exit=1,error=repr(exc),completed_utc=now());persist();completion.write_text(json.dumps(s,indent=2)+'\n',encoding='utf-8');raise
completion.write_text(json.dumps(s,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:s[k] for k in ('actual_exit','native_pass','unreached','completed_utc')}))
