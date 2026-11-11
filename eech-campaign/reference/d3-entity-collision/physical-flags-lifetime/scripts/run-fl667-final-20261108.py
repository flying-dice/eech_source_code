import datetime,json,pathlib,subprocess,sys
E=pathlib.Path('R:/d3-evidence');out=E/'fl667-final-completion-20261108.json'
assert not out.exists()
steps={
 's1-fmt':'rustup component add rustfmt && cargo fmt --version && cargo fmt --check',
 's2-clippy-default':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys -p eech-engine --all-targets -- -D warnings',
 's3-clippy-lifetime':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys -p eech-engine --features eech-engine/physical-flags-lifetime-check --all-targets -- -D warnings',
 's4-lifetime':'cargo test --locked -p eech-engine --features physical-flags-lifetime-check --lib flag_lifetime_check -- --nocapture --test-threads 1',
 's5-source-guard':'cargo test --locked -p eech-engine-sys --test physical_flags_lifetime_guard',
 's6-build-default':'cargo build --locked -p eech-dc',
}
results={}
for label,command in steps.items():
    code=subprocess.call([sys.executable,'-I','-B','-X','utf8',str(E/'run-physical-flags-fl667-direct-20261108.py'),label,command])
    results[label]=code
    print(label,'direct_exit',code,flush=True)
    if code:break
with out.open('x',encoding='utf-8') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_exits':results},f,indent=2)
raise SystemExit(0 if len(results)==len(steps) and not any(results.values()) else 1)
