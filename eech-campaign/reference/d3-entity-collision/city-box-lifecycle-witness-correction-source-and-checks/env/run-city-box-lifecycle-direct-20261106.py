"""The CITY box lifecycle retention cut uses scoped Cargo checks only; the full required CI stays in CI."""
from pathlib import Path
import datetime, json, os, re, shlex, subprocess, sys

# One real Cargo child (separate stdout/stderr, one Windows writer each, no pipeline, no trailing echo) on the task-local x86_64 Linux image of
# check-pose-motion-diag-linux-20261105.sh. Scope: the AFFECTED packages and test targets of REQUIRED_CITY_BOX_LIFECYCLE_RETENTION only (the roster, the
# provider wire, the driver script harness, the C dialogue control and the source guards); fmt and scoped clippy of those packages are allowed.

PACKAGES = ('eech-world', 'eech-engine-sys', 'eech-dc', 'eech-engine')
# the feature builds the CITY witness wiring touches: the dialogue control, the hosted fixture profile (weapon-fixture) and the Original-like profile (both fixtures), and the dc binding of the fixture
FEATURES = {('eech-engine-sys', 'entity-dialogue-check'), ('eech-engine-sys', 'weapon-fixture'), ('eech-engine-sys', 'collision-fixture,weapon-fixture'), ('eech-dc', 'weapon-fixture')}
TESTS = {
 'eech-world': {'entity_roster_host', 'entity_authority', 'entity_candidate_host', 'city_box_lifecycle'},
 'eech-engine-sys': {'entity_dialogue', 'entity_roster_guard', 'entity_host_guard'},
 'eech-dc': set(),
 'eech-engine': set(),
}

def validate(command):
 tokens = shlex.split(command)
 if tokens and tokens[0] == 'env':
  assert len(tokens) > 2 and tokens[1] == 'EECH_C_OPT=2', 'only the CI matrix variable'
  tokens = tokens[2:]
 assert len(tokens) > 2 and tokens[0] == 'cargo' and tokens[1] in ('test', 'fmt', 'clippy')
 assert not set(tokens) & {'--workspace', '--all', '--exclude', '--no-fail-fast', '--release'}, 'No local full-matrix replay for the lifecycle cut'
 assert tokens.count('-p') == 1 and tokens[tokens.index('-p') + 1] in PACKAGES, 'One affected package, explicit -p required'
 package = tokens[tokens.index('-p') + 1]
 if tokens[1] == 'test':
  assert '--locked' in tokens and '--target' in tokens and tokens[tokens.index('--target') + 1] == 'x86_64-unknown-linux-gnu'
  named = [tokens[i + 1] for i, token in enumerate(tokens[:-1]) if token == '--test']
  assert named or '--lib' in tokens, 'name the lib or explicit test targets'
  assert set(named) <= TESTS[package], 'only the affected test targets of ' + package
  if '--features' in tokens:
   assert (package, tokens[tokens.index('--features') + 1]) in FEATURES
 elif tokens[1] == 'fmt':
  assert '--check' in tokens or tokens == ['cargo', 'fmt', '-p', package], 'check, or format only the one package'
 else:
  assert '--locked' in tokens and '--all-targets' in tokens
  if '--features' in tokens:
   assert (package, tokens[tokens.index('--features') + 1]) in FEATURES

if __name__ == '__main__':
 label, command, *extra = sys.argv[1:]
 assert extra in ([], ['--validate-only']) and re.fullmatch(r'[A-Za-z0-9_-]+', label)
 assert re.fullmatch(r'(env EECH_C_OPT=2 )?cargo [A-Za-z0-9 =_./:,+@-]+( -- [A-Za-z0-9 =_./:,+@-]+)?', command), 'plain argument grammar only'
 validate(command)
 if extra:
  print(json.dumps({'validated': True, 'cargo_child_started': False, 'command': command}))
  raise SystemExit(0)
 env = Path('R:/d3-evidence/city-box-lifecycle-20261106')
 env.mkdir(exist_ok=True)
 stem = 'city-box-lifecycle-' + label
 record = env / (stem + '.json')
 assert not record.exists(), 'unique label'
 now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
 root = '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence'
 args = ['wsl', '-d', 'EECH-D2-Build', '--', 'sh', root + '/check-pose-motion-diag-linux-20261105.sh', root + '/collision-characterization-20261029', command]
 state = dict(controller_pid=os.getpid(), state='running', started_utc=now(), command=command, args=args,
              scope='One Cargo command, lifecycle-scoped. Separate stdout/stderr files. Return is the actual Cargo/docker exit. Working-tree check only unless separately frozen.')
 with (env / (stem + '.stdout.log')).open('xb') as out, (env / (stem + '.stderr.log')).open('xb') as err:
  child = subprocess.Popen(args, stdout=out, stderr=err, creationflags=subprocess.CREATE_NO_WINDOW)
  state['pid'] = child.pid
  record.write_bytes((json.dumps(state, indent=2) + '\n').encode())
  code = child.wait()
 state.update(state='terminal', actual_exit=code, completed_utc=now())
 record.write_bytes((json.dumps(state, indent=2) + '\n').encode())
 raise SystemExit(code)
