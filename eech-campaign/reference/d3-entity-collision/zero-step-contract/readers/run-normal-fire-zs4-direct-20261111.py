from pathlib import Path
import subprocess, sys, json, datetime, os, re
# ONE Linux command in the leased volume (check-normal-fire-zs4-linux-20261111.sh) or ONE local Python command: separate stdout/stderr, no
# pipeline; the recorded actual_exit is the direct child's exit. Records zs4-<label>.{json,stdout.log,stderr.log} beside this file.
# Grammar: plain `cargo ...` commands (optionally one leading `env NAME=value`) joined by ` && `, or the literal word `python-tests`.
E = Path(__file__).resolve().parent
W = E.parent / 'collision-characterization-20261029'
label, command = sys.argv[1:]
assert re.fullmatch(r'[A-Za-z0-9_-]+', label)
stem = 'zs4-' + label; record = E / (stem + '.json'); assert not record.exists()
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
if command == 'python-tests':
    args = [sys.executable, '-I', '-B', '-X', 'utf8', str(W / 'eech-campaign/tools/test_normal_fire_capture.py')]
    cwd = W
else:
    part = r'(env [A-Z_]+=[A-Za-z0-9_.-]+ )?(cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?|rustup component add [a-z ]+)'
    assert re.fullmatch(part + r'( && ' + part + r')*', command) and '--workspace' not in command, 'plain cargo commands only'
    args = ['wsl', '-d', 'EECH-D2-Build', '--cd', '/', '--', 'sh', '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/zs4-20261111/check-normal-fire-zs4-linux-20261111.sh', '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/collision-characterization-20261029', command]
    cwd = E
state = dict(controller_pid=os.getpid(), state='running', started_utc=now(), label=label, command=command, args=args, scope='Working tree check; Linux commands run in the sole leased volume eech-linux-d3-physical-flags-9d247-20261107, image sha256:9d247b4e... named, never substituted.')
with (E / (stem + '.stdout.log')).open('xb') as out, (E / (stem + '.stderr.log')).open('xb') as err:
    child = subprocess.Popen(args, cwd=cwd, stdout=out, stderr=err, creationflags=subprocess.CREATE_NO_WINDOW)
    state['pid'] = child.pid; record.write_bytes((json.dumps(state, indent=2) + '\n').encode()); code = child.wait()
state.update(state='terminal', actual_exit=code, completed_utc=now()); record.write_bytes((json.dumps(state, indent=2) + '\n').encode())
print(json.dumps(dict(label=label, actual_exit=code)))
raise SystemExit(code)
