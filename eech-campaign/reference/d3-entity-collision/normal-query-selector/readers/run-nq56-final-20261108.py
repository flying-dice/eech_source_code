"""One focused final set after sealed PRE; no C matrix/native action."""
from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def put(p,obj):
 with p.open('x',encoding='utf-8') as f:json.dump(obj,f,indent=2)
sealed=json.loads((E/'nq56-readers-sealed-20261108.json').read_bytes())
def verify():
 for n,h in sealed.items():assert hashlib.sha256((E/n).read_bytes()).hexdigest()==h,n
commands=[
 ('s1-profile-census','cargo test --locked -p eech-world --test host_callbacks every_reference_script_that_prepares_the_installation_selects_the_profile_right_after_it_and_before_booting_and_the_two_diagnostics_do_not -- --nocapture'),
 ('s2-python',None),
 ('s3-fmt','rustup component add rustfmt && cargo fmt --version && cargo fmt --all -- --check'),
 ('s4-world-clippy','rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-world --all-targets -- -D warnings'),
 ('s5-session-lua','cargo test --locked -p eech-world --test normal_fire_session -- --nocapture')]

state=dict(controller_pid=os.getpid(),started_utc=now(),state='running',commands=commands,actual_exits={})
put(E/'nq56-final-process-20261108.json',state)
for label,command in commands:
 verify()
 if command:code=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/'run-normal-fire-nq56-direct-20261108.py'),label,command],check=False).returncode
 else:
  args=[sys.executable,'-I','-B','-X','utf8',str(W/'eech-campaign/tools/test_normal_fire_capture.py')]
  stem=E/('normal-fire-nq56-'+label);r=dict(command=args,controller_pid=os.getpid(),started_utc=now(),state='running')
  with stem.with_suffix('.stdout.log').open('xb') as out,stem.with_suffix('.stderr.log').open('xb') as err:
   child=subprocess.Popen(args,cwd=W,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);r['pid']=child.pid;code=child.wait()
  r.update(state='terminal',actual_exit=code,completed_utc=now());put(stem.with_suffix('.json'),r)
 receipt=json.loads((E/('normal-fire-nq56-'+label+'.json')).read_bytes());assert receipt['state']=='terminal' and receipt['actual_exit']==code
 state['actual_exits'][label]=code;print(json.dumps(dict(label=label,actual_exit=code)),flush=True)
 if code:break
verify();state.update(state='terminal',completed_utc=now(),actual_exit=code);put(E/'nq56-final-completion-20261108.json',state)
raise SystemExit(code)
