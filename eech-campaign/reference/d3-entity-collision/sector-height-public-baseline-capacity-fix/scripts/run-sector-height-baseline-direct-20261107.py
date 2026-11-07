from pathlib import Path
import subprocess,sys,json,datetime,os,re
# Same one-real-Cargo-child contract as run-sector-roster-focused-direct-20261106.py (separate stdout/stderr, no pipeline, no trailing echo), with two additions the CI gates need:
# an explicit workspace-wide Cargo command (no -p: the workflow's own `cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt --check`, whole-workspace test) and one leading `env NAME=value` for a CI matrix variable.
b=Path('R:/d3-evidence');label,command=sys.argv[1:];assert re.fullmatch(r'[A-Za-z0-9_-]+',label)
assert re.fullmatch(r'(env EECH_C_OPT=2 )?cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?',command) and '--workspace' not in command,'One Cargo child with a plain argument grammar only'
stem='sector-height-baseline-'+label;record=b/(stem+'.json');assert not record.exists();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
args=['wsl','-d','EECH-D2-Build','--','sh','/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/check-pose-motion-diag-linux-20261105.sh','/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/collision-characterization-20261029',command]
state=dict(controller_pid=os.getpid(),state='running',started_utc=now(),command=command,args=args,scope='One Cargo command. Separate stdout/stderr files: each has one Windows writer. WSL/helper return is the actual Cargo/docker stage exit, no trailing echo/pipeline. Working-tree check only unless separately frozen/source-bound.')
with (b/(stem+'.stdout.log')).open('xb') as out,(b/(stem+'.stderr.log')).open('xb') as err:
 child=subprocess.Popen(args,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);state['pid']=child.pid;record.write_bytes((json.dumps(state,indent=2)+'\n').encode());code=child.wait()
state.update(state='terminal',actual_exit=code,completed_utc=now());record.write_bytes((json.dumps(state,indent=2)+'\n').encode());raise SystemExit(code)
