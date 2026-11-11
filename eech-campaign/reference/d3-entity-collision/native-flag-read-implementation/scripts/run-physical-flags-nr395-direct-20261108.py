from pathlib import Path
import subprocess, sys, json, datetime, os, re
# ONE Linux command in the leased volume (check-physical-flags-nr395-linux-20261108.sh): separate stdout/stderr, no pipeline, the recorded actual_exit is the container's exit (the shell inside is `sh -c`, whose exit is the last
# command's). Grammar: a plain `cargo ...` command, optionally one leading `env NAME=value`, or `sh -c`-free chains joined by ` && ` of such commands. Records flags-nr395-<label>.{json,stdout.log,stderr.log}.
b = Path('R:/d3-evidence'); label, command = sys.argv[1:]; assert re.fullmatch(r'[A-Za-z0-9_-]+', label)
# Root supervision only: CLI console notes, never test output files.
for root_note in sorted(b.glob('Root-native-read-feedback*.json')):
    print('ROOT SUPERVISION NOTE (read before another expensive retry): ' + root_note.name, flush=True)
    print(root_note.read_text(encoding='utf-8'), flush=True)
part = r'(env [A-Z_]+=[A-Za-z0-9_.-]+ )?(cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?|rustup component add [a-z ]+)'
assert re.fullmatch(part + r'( && ' + part + r')*', command) and '--workspace' not in command, 'plain cargo commands only'
stem = 'flags-nr395-' + label; record = b / (stem + '.json'); assert not record.exists(); now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
args = ['wsl', '-d', 'EECH-D2-Build', '--cd', '/', '--', 'sh', '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/check-physical-flags-nr395-linux-20261108.sh', '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence/collision-characterization-20261029', command]
state = dict(controller_pid=os.getpid(), state='running', started_utc=now(), command=command, args=args, scope='Working-tree check in the sole leased volume eech-linux-d3-physical-flags-9d247-20261107, image sha256:9d247b4e... named, never substituted.')
with (b / (stem + '.stdout.log')).open('xb') as out, (b / (stem + '.stderr.log')).open('xb') as err:
    child = subprocess.Popen(args, stdout=out, stderr=err, creationflags=subprocess.CREATE_NO_WINDOW); state['pid'] = child.pid; record.write_bytes((json.dumps(state, indent=2) + '\n').encode()); code = child.wait()
state.update(state='terminal', actual_exit=code, completed_utc=now()); record.write_bytes((json.dumps(state, indent=2) + '\n').encode()); raise SystemExit(code)
