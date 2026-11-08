import pathlib,subprocess,sys,json,datetime
E=pathlib.Path('R:/d3-evidence');out=E/'crt1516-final-completion-20261108.json';assert not out.exists()
steps={
 's1-fmt':'rustup component add rustfmt && cargo fmt --version && cargo fmt --check',
 's2-default-clippy':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world --all-targets -- -D warnings',
 's3-observer-clippy':'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-engine-sys -p eech-engine -p eech-dc -p eech-world --features eech-dc/core-terrain-crt-observer --all-targets -- -D warnings',
 's4-observer':'cargo test --locked -p eech-engine-sys --features core-terrain-crt-observer --test core_terrain_crt_observer -- --nocapture --test-threads 1',
 's5-driver':'cargo test --locked -p eech-world --test core_terrain_crt_driver -- --nocapture',
 's6-profile-census':'cargo test --locked -p eech-world --test host_callbacks every_reference_script_that_prepares_the_installation_selects_the_profile_right_after_it_and_before_booting_and_the_two_diagnostics_do_not',
}
results={}
for label,command in steps.items():
    code=subprocess.call([sys.executable,'-I','-B','-X','utf8',str(E/'run-core-crt-crt1516-direct-20261108.py'),label,command]);results[label]=code;print(label,'direct_exit',code,flush=True)
    if code:break
with out.open('x') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_exits':results},f,indent=2)
raise SystemExit(0 if len(results)==len(steps) and not any(results.values()) else 1)
