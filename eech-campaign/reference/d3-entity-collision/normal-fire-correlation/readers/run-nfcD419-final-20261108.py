import pathlib,subprocess,sys,json,datetime,os
E=pathlib.Path('R:/d3-evidence');process=E/'nfcD419-final-process-20261108.json';completion=E/'nfcD419-final-completion-20261108.json'
assert not process.exists() and not completion.exists()
steps={
's1-fmt':'rustup component add rustfmt && cargo fmt --version && cargo fmt --check',
's2-clippy':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine -p eech-engine-sys -p eech-dc --all-targets --features eech-engine/normal-fire-check,eech-engine/normal-fire-correlation,eech-engine/model-fixture,eech-dc/normal-fire-correlation -- -D warnings',
's3-controls':'cargo test --locked -p eech-engine -p eech-engine-sys --features eech-engine/normal-fire-check,eech-engine/normal-fire-correlation,eech-engine/model-fixture --test normal_fire -- --nocapture',
's4-shipping-check':'cargo check --locked -p eech-dc'}
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
state={'provider':'actual_python_final_controller','controller_pid':os.getpid(),'started_utc':now(),'state':'running','steps':steps,'actual_exits':{}}
process.write_text(json.dumps(state,indent=2)+'\n')
for label,command in steps.items():
 state['current_label']=label;process.write_text(json.dumps(state,indent=2)+'\n')
 code=subprocess.call([sys.executable,'-I','-B','-X','utf8',str(E/'run-normal-fire-nfcD419-direct-20261108.py'),label,command]);state['actual_exits'][label]=code
 print(label,'direct_exit',code,flush=True)
 if code:break
state.update(state='terminal',completed_utc=now());process.write_text(json.dumps(state,indent=2)+'\n')
with completion.open('x') as f:json.dump(state,f,indent=2)
raise SystemExit(0 if len(state['actual_exits'])==4 and not any(state['actual_exits'].values()) else 1)
