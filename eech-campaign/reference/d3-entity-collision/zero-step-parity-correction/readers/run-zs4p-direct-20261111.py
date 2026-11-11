"""ONE local Python command for the schema4 parity correction: separate stdout/stderr, no pipeline, direct child exit.
Records zs4p-<label>.{json,stdout.log,stderr.log} beside this file. No Docker/Cargo/native action."""
from pathlib import Path
import datetime, json, os, re, subprocess, sys
E = Path(__file__).resolve().parent
W = E.parent / 'collision-characterization-20261029'
label, *command = sys.argv[1:]
assert re.fullmatch(r'[A-Za-z0-9_-]+', label) and command
allowed = {'python-tests': ['eech-campaign/tools/test_normal_fire_capture.py'],
           'py-compile': ['-m', 'py_compile', 'eech-campaign/tools/normal_fire_capture.py', 'eech-campaign/tools/normal_fire_capture_controller.py', 'eech-campaign/tools/test_normal_fire_capture.py'],
           'parity': [str(E / 'zs4p-parity-20261111.py')]}
assert len(command) == 1 and command[0] in allowed, 'python-tests | py-compile | parity'
args = [sys.executable, '-I', '-B', '-X', 'utf8', *allowed[command[0]]]
stem = 'zs4p-' + label; record = E / (stem + '.json'); assert not record.exists()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
state = dict(controller_pid=os.getpid(), state='running', started_utc=now(), label=label, command=command[0], args=args, cwd=str(W))
with (E / (stem + '.stdout.log')).open('xb') as out, (E / (stem + '.stderr.log')).open('xb') as err:
    child = subprocess.Popen(args, cwd=W, stdout=out, stderr=err, creationflags=subprocess.CREATE_NO_WINDOW)
    state['pid'] = child.pid; record.write_bytes((json.dumps(state, indent=2) + '\n').encode()); code = child.wait()
state.update(state='terminal', actual_exit=code, completed_utc=now()); record.write_bytes((json.dumps(state, indent=2) + '\n').encode())
print(json.dumps(dict(label=label, actual_exit=code)))
raise SystemExit(code)
