from pathlib import Path
import subprocess,sys,json,datetime,os,re
# Same one-real-Cargo-child contract as run-fixed-rules-correction-direct-20261106.py (separate stdout/stderr, no pipeline, no trailing echo), on the task-local i686 image,
# with an optional leading `env NAME=value` for the CI matrix variables (EECH_C_OPT=2, EECH_ORIGINAL_STACK_ATTRIBUTES=1).
b=Path('R:/d3-evidence');label,command=sys.argv[1:];assert re.fullmatch(r'[A-Za-z0-9_-]+',label)
assert re.fullmatch(r'(env (EECH_C_OPT=2|EECH_ORIGINAL_STACK_ATTRIBUTES=1) )?cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?',command) and '--workspace' not in command,'One Cargo child with a plain argument grammar only'
env=b/'tree-i686-raw-pass-through-20261106-env';stem='tree-i686-'+label;record=env/(stem+'.json');assert not record.exists();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
policy=env/'ROOT-UNSTARTED-DUPLICATES-20261106.json'
if policy.exists():
 control=json.loads(policy.read_bytes());prior=control['labels'].get(label)
 if prior:
  previous=env/('tree-i686-'+prior+'.json');old=json.loads(previous.read_bytes())
  assert old['state']=='terminal' and old['command']==command, 'Root duplicate block must match an actual terminal identical command'
  blocked=dict(controller_pid=os.getpid(),state='blocked_before_launch',actual_exit=None,cargo_child_started=False,command=command,recorded_utc=now(),reused_receipt=str(previous),prior_actual_exit=old['actual_exit'],control=str(policy),instruction=control['instruction_to_implementation_lane'],scope='No Cargo/docker/cache job started. Not success, independent clearance or CI acceptance; original results retain their actual source qualifications.')
  record.write_bytes((json.dumps(blocked,indent=2)+'\n').encode())
  (env/(stem+'.stdout.log')).write_bytes(b'')
  (env/(stem+'.stderr.log')).write_text(control['instruction_to_implementation_lane']+'\n')
  raise SystemExit(78)
root='/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence'
args=['wsl','-d','EECH-D2-Build','--','sh',root+'/tree-i686-raw-pass-through-20261106-env/check-tree-i686-linux.sh',root+'/collision-characterization-20261029',command]
state=dict(controller_pid=os.getpid(),state='running',started_utc=now(),command=command,args=args,image='eech-d3-tree-i686-20261106:2',scope='One Cargo command on the task-local i686 image. Separate stdout/stderr files, one Windows writer each. Return is the actual Cargo/docker exit. Working-tree check only unless separately frozen.')
with (env/(stem+'.stdout.log')).open('xb') as out,(env/(stem+'.stderr.log')).open('xb') as err:
 child=subprocess.Popen(args,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);state['pid']=child.pid;record.write_bytes((json.dumps(state,indent=2)+'\n').encode());code=child.wait()
state.update(state='terminal',actual_exit=code,completed_utc=now());record.write_bytes((json.dumps(state,indent=2)+'\n').encode());raise SystemExit(code)
