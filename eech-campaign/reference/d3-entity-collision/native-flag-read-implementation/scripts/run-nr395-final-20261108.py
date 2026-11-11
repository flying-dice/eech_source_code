import datetime,json,pathlib,subprocess,sys
E=pathlib.Path('R:/d3-evidence');out=E/'nr395-final-completion-20261108.json'
assert not out.exists()
steps={
 's1-fmt':'rustup component add rustfmt && cargo fmt --version && cargo fmt --check',
 's2-clippy':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-world --all-targets -- -D warnings',
 's3-read-driver':'cargo test --locked -p eech-world --test physical_flags_read_driver -- --nocapture',
}
results={}
for label,command in steps.items():
    code=subprocess.call([sys.executable,'-I','-B','-X','utf8',str(E/'run-physical-flags-nr395-direct-20261108.py'),label,command])
    results[label]=code;print(label,'direct_exit',code,flush=True)
    if code:break
with out.open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_exits':results},f,indent=2)
raise SystemExit(0 if len(results)==len(steps) and not any(results.values()) else 1)
