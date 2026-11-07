from pathlib import Path
import subprocess,sys,json,datetime,os,re
# One real Cargo child on the task-local i686 image (the same route as run-tree-i686-direct.py), records beside the production runner's: separate stdout/stderr files, the real exit, no pipeline, no trailing echo.
b=Path('R:/d3-evidence');label,command=sys.argv[1:];assert re.fullmatch(r'[A-Za-z0-9_-]+',label)
assert re.fullmatch(r'(env (EECH_C_OPT=2|EECH_ORIGINAL_STACK_ATTRIBUTES=1) )?cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?',command) and '--workspace' not in command,'One Cargo child with a plain argument grammar only'
stem='sector-height-production-i686-'+label;record=b/(stem+'.json');assert not record.exists();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
root='/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence'
args=['wsl','-d','EECH-D2-Build','--','sh',root+'/tree-i686-raw-pass-through-20261106-env/check-tree-i686-linux.sh',root+'/collision-characterization-20261029',command]
state=dict(controller_pid=os.getpid(),state='running',started_utc=now(),command=command,args=args,image='eech-d3-tree-i686-20261106:2',scope='One Cargo command on the task-local i686 image. Working-tree check only unless separately frozen/source-bound.')
with (b/(stem+'.stdout.log')).open('xb') as out,(b/(stem+'.stderr.log')).open('xb') as err:
 child=subprocess.Popen(args,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);state['pid']=child.pid;record.write_bytes((json.dumps(state,indent=2)+'\n').encode());code=child.wait()
state.update(state='terminal',actual_exit=code,completed_utc=now());record.write_bytes((json.dumps(state,indent=2)+'\n').encode());raise SystemExit(code)
