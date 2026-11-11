"""The ONE final set after the sealed PRE census; every command through the direct runner (separate logs, direct exits).
No C matrix, native, Windows or default build action. Stops at the first nonzero exit and records it."""
from pathlib import Path
import datetime, hashlib, json, os, subprocess, sys
E = Path(__file__).resolve().parent
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
def put(p, obj):
    with p.open('x', encoding='utf-8') as f: json.dump(obj, f, indent=2)
sealed = json.loads((E / 'zs4-readers-sealed-20261111.json').read_bytes())
def verify():
    for n, h in sealed.items(): assert hashlib.sha256((E / n).read_bytes()).hexdigest() == h, n
commands = [
    ('s1-python', 'python-tests'),
    ('s2-fmt', 'rustup component add rustfmt && cargo fmt --version && cargo fmt --all -- --check'),
    ('s3-clippy', 'rustup component add clippy && cargo clippy --version && cargo clippy --locked -p eech-world --all-targets -- -D warnings'),
    ('s4-session-lua', 'cargo test --locked -p eech-world --test normal_fire_session -- --nocapture'),
    ('s5-facade-default', 'cargo test --locked -p eech-dc --lib public_normal_session_driver_consumes_only_registered_userdata_methods -- --nocapture'),
    ('s6-facade-observer', 'cargo test --locked -p eech-dc --lib --features normal-fire-correlation public_normal_session_driver_consumes_only_registered_userdata_methods -- --nocapture')]
state = dict(controller_pid=os.getpid(), started_utc=now(), state='running', commands=commands, actual_exits={})
put(E / 'zs4-final-process-20261111.json', state)
code = 0
for label, command in commands:
    verify()
    code = subprocess.run([sys.executable, '-I', '-B', '-X', 'utf8', str(E / 'run-normal-fire-zs4-direct-20261111.py'), label, command], check=False).returncode
    receipt = json.loads((E / ('zs4-' + label + '.json')).read_bytes()); assert receipt['state'] == 'terminal' and receipt['actual_exit'] == code
    state['actual_exits'][label] = code; print(json.dumps(dict(label=label, actual_exit=code)), flush=True)
    if code: break
verify(); state.update(state='terminal', completed_utc=now(), actual_exit=code); put(E / 'zs4-final-completion-20261111.json', state)
raise SystemExit(code)
