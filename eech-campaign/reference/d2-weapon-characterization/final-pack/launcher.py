"""Verified Windows launch of the repository's unchanged retail regression pack."""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import sys


def require(condition, message):
    if not condition:
        raise ValueError(message)


def stamp():
    return dt.datetime.now(dt.timezone.utc).isoformat()


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return {'bytes': path.stat().st_size, 'sha256': h.hexdigest()}


def save(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')


def command(args, log, *, env=None, cwd=None):
    with log.open('wb') as output:
        return subprocess.run([str(x) for x in args], stdout=output,
                              stderr=subprocess.STDOUT, env=env, cwd=cwd).returncode


def git(repo, *args):
    r = subprocess.run(['git', '-C', str(repo), *args], capture_output=True, text=True)
    require(r.returncode == 0, f'Git failed: {r.stderr.strip()}')
    return r.stdout.strip()


def x64_pe(path):
    with path.open('rb') as f:
        require(f.read(2) == b'MZ', f'Not a PE file: {path}')
        f.seek(0x3c)
        offset = struct.unpack('<I', f.read(4))[0]
        f.seek(offset)
        require(f.read(4) == b'PE\0\0', f'Invalid PE signature: {path}')
        require(struct.unpack('<H', f.read(2))[0] == 0x8664, f'Not Windows x64: {path}')


def manifests(python, repo, roots, out, prefix, env):
    results = {}
    for name, root in roots.items():
        manifest = ('reference/m4-s3-mutation/inputs-georgia.json' if name == 'georgia'
                    else 'reference/lebanon-3h/inputs.json')
        log = out / f'{prefix}-inputs-{name}.log'
        code = command([python, '-X', 'utf8', repo / 'tools/input-manifest.py',
                        root, '--compare', repo / manifest], log, env=env)
        results[name] = {'exit_code': code, 'log': str(log)}
    return results


def active_root_users(shell, env, roots, log):
    query = ("$ErrorActionPreference='Stop'; $rows=@(Get-CimInstance Win32_Process -Filter "
             "\"Name='eech-world.exe' OR Name='python.exe' OR Name='python3.exe' OR Name='powershell.exe' OR Name='pwsh.exe'\" "
             "| Where-Object {$_.ProcessId -ne $PID -and ($_.Name -eq 'eech-world.exe' -or $_.CommandLine -like '*regression-pack.py*')} "
             "| Select-Object ProcessId,ParentProcessId,Name,CreationDate,CommandLine); "
             "ConvertTo-Json -InputObject $rows -Compress")
    result = subprocess.run([shell, '-NoProfile', '-Command', query], env=env, capture_output=True, text=True)
    require(result.returncode == 0, 'Cannot inspect existing campaign processes; inspect before launching')
    rows = json.loads(result.stdout)
    target = {os.path.normcase(os.path.realpath(p)) for p in roots.values()}
    collisions = []
    for row in rows:
        text = row.get('CommandLine') or ''
        if row['Name'].lower() == 'eech-world.exe':
            matches = re.findall(r'"root=([^\"]+)"|\broot="([^\"]+)"|\broot=([^\s\"]+)', text, re.I)
            values = [next(v for v in match if v) for match in matches]
            # An unidentified native may use our data; do not call that ready.
            if not values:
                collisions.append(dict(row, reason='Cannot establish active native root'))
        else:
            matches = re.findall(r'--(?:georgia|lebanon(?:-repeat)?)(?:=|\s+)(?:"([^\"]+)"|([^\s\"]+))', text, re.I)
            values = [next(v for v in match if v) for match in matches]
        if any(os.path.normcase(os.path.realpath(v)) in target for v in values):
            collisions.append(dict(row, reason='Another active campaign/pack uses a requested root'))
    save(log, {'observed_utc':stamp(), 'relevant_processes':rows, 'root_collisions':collisions})
    return collisions


# A separate child writes its own actual exit, even if the waiting tool disconnects.
# runpy executes the repository file without patching or substituting its checks.
BOOTSTRAP = r'''
import datetime,json,os,runpy,sys,traceback
cwd,runner,record,*args=sys.argv[1:]
os.chdir(cwd)
def write(status,code=None):
    data={'status':status,'pid':os.getpid(),'when_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'native_cwd':os.getcwd(),'runner':runner,'arguments':args,'exit_code':code}
    with open(record+'.tmp','w',encoding='utf-8') as f: json.dump(data,f,indent=2)
    os.replace(record+'.tmp',record)
write('running')
print('NATIVE_PACK_CWD='+os.getcwd(),flush=True)
code=1
try:
    sys.argv=[runner,*args]
    runpy.run_path(runner,run_name='__main__')
    code=0
except SystemExit as e:
    code=e.code if isinstance(e.code,int) else (0 if e.code is None else 1)
    if isinstance(e.code,str): print(e.code,file=sys.stderr)
except BaseException:
    traceback.print_exc()
finally:
    write('completed',code)
sys.exit(code)
'''


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('repo', 'python', 'bin', 'georgia', 'lebanon', 'lebanon-repeat',
                 'accepted-report', 'out'):
        p.add_argument('--' + name, required=True)
    p.add_argument('--role', choices=('baseline', 'candidate'), required=True)
    p.add_argument('--expected-commit')
    p.add_argument('--preflight-only', action='store_true')
    a = p.parse_args()
    require(sys.platform == 'win32', 'This launcher validates Windows x64 runs only')
    require(sys.version_info >= (3, 11), 'Run this helper with Python3.11+')
    python = Path(os.path.abspath(a.python))
    require(os.path.samefile(python, sys.executable), 'Invoke this helper with the same explicit --python interpreter')
    require(not any(c in str(python) for c in '%\r\n"'), 'Python path cannot contain batch-expansion characters; choose a verified interpreter path')
    repo, binary, out = [Path(os.path.abspath(x)) for x in (a.repo, a.bin, a.out)]
    roots = {n: Path(os.path.abspath(getattr(a, n))) for n in ('georgia', 'lebanon', 'lebanon_repeat')}
    for path in (repo, binary, out, *roots.values()):
        require(not str(path).startswith('\\\\'), 'Copy runtime inputs and outputs to verified local storage; do not run on the network bundle')
        require(not any(c.isspace() or c in '%&|<>^()' for c in str(path)),
                f'Use short shell-simple alias paths without whitespace or CMD metacharacters: {path}')
    require(not out.exists(), f'Output already exists; preserve it and use a fresh directory: {out}')
    for parent in (repo, binary, *roots.values()):
        require(parent.is_dir(), f'Missing directory: {parent}')
        # Resolve aliases only for containment checks; keep short paths in argv.
        parent_real = os.path.normcase(os.path.realpath(parent))
        try:
            inside = os.path.commonpath((os.path.normcase(os.path.realpath(out)), parent_real)) == parent_real
        except ValueError:  # different physical drives
            inside = False
        require(not inside, f'Put run output outside source/bin/input directories: {out}')
    require(not os.path.samefile(roots['lebanon'], roots['lebanon_repeat']), 'Lebanon repeat must be a separate root')
    runner = repo / 'tools/regression-pack.py'
    require(runner.is_file(), '--repo must name the eech-campaign directory')
    head = git(repo, 'rev-parse', 'HEAD')
    require(not git(repo, 'status', '--porcelain', '--untracked-files=all'), 'Use a clean immutable source snapshot')
    if a.expected_commit:
        require(head == git(repo, 'rev-parse', a.expected_commit + '^{commit}'), 'Unexpected source revision')
    accepted_path = Path(os.path.abspath(a.accepted_report))
    accepted = json.loads(accepted_path.read_text(encoding='utf-8-sig'))
    expected_checks = set(accepted['checks'])
    require(accepted['head']['verdict'] == 'PASS' and bool(expected_checks), 'Accepted report must be a complete PASS')
    require(all(v['status'] == 'PASS' for v in accepted['checks'].values()), 'Accepted report has nonpassing checks')
    info_path = binary / 'BUILD-INFO.txt'
    info = info_path.read_text(encoding='utf-8-sig')
    require('FIXTURE BUILD' not in info.upper(), 'Full shipping regression requires an ordinary build')
    if a.role == 'candidate':
        commit = re.search(r'^commit: (\S+)', info, re.M)
        dirty = re.search(r'^uncommitted changes in the build inputs: (\d+) files', info, re.M)
        require(commit is not None and commit[1] == head, 'Candidate build commit must match frozen source')
        require(dirty is not None and dirty[1] == '0', 'Candidate build inputs must be clean')
    files = {}
    for sha, relative in re.findall(r'^\s*([0-9a-f]{64})  (.+)$', info, re.M):
        relative = relative.rstrip('\r')
        rel = Path(relative)
        require(not rel.is_absolute() and '..' not in rel.parts, 'Unsafe BUILD-INFO path')
        actual = digest(binary / rel)
        require(actual['sha256'] == sha, f'BUILD-INFO mismatch: {relative}')
        files[relative] = actual
    for name in ('eech-world.exe', 'eech_dc.dll', 'lua.dll'):
        require(name in files, f'Missing BUILD-INFO entry: {name}')
        x64_pe(binary / name)
    for name, sha in accepted['head']['build']['files'].items():
        require(not Path(name).is_absolute() and '..' not in Path(name).parts, 'Unsafe accepted build path')
        actual = digest(binary / name)
        if a.role == 'baseline':
            require(actual['sha256'] == sha, f'Supplied baseline differs from accepted file: {name}')
        files[name] = actual
    out.mkdir(parents=True)
    try:
        shim = out / 'python-commands'
        shim.mkdir()
        batch = f'@echo off\r\nsetlocal DisableDelayedExpansion\r\n"{python}" %*\r\nexit /b %errorlevel%\r\n'
        for name in ('python3.cmd', 'python.cmd'):
            (shim / name).write_bytes(batch.encode('utf-8'))
        env = dict(os.environ, PATH=str(shim) + os.pathsep + os.environ.get('PATH', ''),
                   PYTHONIOENCODING='utf-8', PYTHONUTF8='1', PYTHONDONTWRITEBYTECODE='1')
        shell = shutil.which('pwsh', path=env['PATH']) or shutil.which('powershell', path=env['PATH'])
        require(shell is not None, 'PowerShell is required by the repository runners')
        probe_file = out / 'python-probe.py'
        probe_file.write_text('import sys,json\nprint(json.dumps({"executable":sys.executable,"version":list(sys.version_info[:3])}))\n',encoding='utf-8')
        quoted_probe = "'" + str(probe_file).replace("'", "''") + "'"
        probe_code = ("$p=(Get-Command python3,python -ErrorAction Stop|Select-Object -First 1).Source; "
                      "Write-Output $p; & $p " + quoted_probe + "; exit $LASTEXITCODE")
        probe = subprocess.run([shell, '-NoProfile', '-Command', probe_code], env=env, capture_output=True, text=True)
        (out / 'python-resolver.log').write_text(probe.stdout + probe.stderr, encoding='utf-8')
        require(probe.returncode == 0, 'Child PowerShell Python probe failed; see python-resolver.log')
        lines = probe.stdout.strip().splitlines()
        require(len(lines) >= 2 and os.path.samefile(lines[0], shim / 'python3.cmd'), 'Unexpected child Python command')
        selected = json.loads(lines[-1])
        require(os.path.samefile(selected['executable'], python), 'Child uses a different interpreter')
        require(tuple(selected['version']) >= (3, 11), 'Child Python must be3.11+')
        require(not active_root_users(shell, env, roots, out / 'process-preflight.json'),
                'Requested roots are already in use; preserve/reuse the active run, do not duplicate it')
        definition_code = ('import importlib.util,json,sys; '
                           's=importlib.util.spec_from_file_location("verified_pack",sys.argv[1]); '
                           'm=importlib.util.module_from_spec(s); s.loader.exec_module(m); '
                           'print(json.dumps({"checks":sorted(m.CHECKS),"problems":m.validate(json.load(open(m.PACK)))}))')
        definition = subprocess.run([str(python), '-B', '-X', 'utf8', '-c', definition_code, str(runner)],
                                    env=env, cwd=repo, capture_output=True, text=True)
        (out / 'pack-definition.log').write_text(definition.stdout + definition.stderr, encoding='utf-8')
        require(definition.returncode == 0, 'Cannot inspect repo pack definition')
        declared = json.loads(definition.stdout)
        require(not declared['problems'] and set(declared['checks']) == expected_checks,
                'Repo pack differs from accepted check set or has inconsistent definitions')
        inputs = manifests(python, repo, roots, out, 'preflight', env)
        require(all(r['exit_code'] == 0 for r in inputs.values()), 'Retail input mismatch; see preflight-inputs logs')
        require(not git(repo, 'status', '--porcelain', '--untracked-files=all'), 'Preflight must leave the frozen source clean')
        preflight = {'scope': 'Preflight only; no campaign result', 'when_utc': stamp(), 'role': a.role,
                     'source': str(repo), 'head': head, 'runner': digest(runner),
                     'python': selected, 'shell': shell, 'host':{'os':platform.platform(),'machine':platform.machine()},
                     'roots': {k:str(v) for k,v in roots.items()},
                     'inputs': inputs, 'build_info': digest(info_path), 'files': files,
                     'accepted_report': {'path':str(accepted_path), **digest(accepted_path)},
                     'expected_checks': sorted(expected_checks), 'launcher': digest(Path(__file__))}
        save(out / 'preflight.json', preflight)
        print(f'PREFLIGHT READY: {len(expected_checks)} expected checks; child Python {selected["version"]}', flush=True)
        if a.preflight_only:
            print('No campaign started.', flush=True)
            return 0
        require(not active_root_users(shell, env, roots, out / 'process-before-launch.json'),
                'A campaign started using these roots during preflight; do not launch a duplicate')
        args = ['--georgia', str(roots['georgia']), '--lebanon', str(roots['lebanon']),
                '--lebanon-repeat', str(roots['lebanon_repeat']), '--bin', str(binary), '--out', str(out)]
        with (out / 'pack.stdout.log').open('wb') as stdout, (out / 'pack.stderr.log').open('wb') as stderr:
            process = subprocess.Popen([str(python), '-X', 'utf8', '-c', BOOTSTRAP, str(out),
                                        str(runner), str(out / 'run.json'), *args],
                                       env=env, cwd=out, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            print(f'RUNNER_PID={process.pid}; output={out}', flush=True)
            code = process.wait()
        if code != 0:
            print(f'ORIGINAL_PACK_EXIT={code}; preserve failures and inspect actual remaining processes.', flush=True)
            return code if 0 < code < 256 else 1
        report = json.loads((out / 'report.json').read_text())
        report_ok = (report['head']['verdict'] == 'PASS' and set(report['checks']) == expected_checks
                     and all(v['status'] == 'PASS' for v in report['checks'].values()))
        post = manifests(python, repo, roots, out, 'post', env)
        unchanged = (git(repo, 'rev-parse', 'HEAD') == head
                     and not git(repo, 'status', '--porcelain', '--untracked-files=all')
                     and all(digest(binary / name) == value for name,value in files.items())
                     and digest(accepted_path) == {k:preflight['accepted_report'][k] for k in ('bytes','sha256')})
        ok = report_ok and unchanged and all(r['exit_code'] == 0 for r in post.values())
        save(out / 'completion.json', {'when_utc':stamp(), 'original_pack_exit':code,
                                     'expected_check_set_all_pass':report_ok, 'post_inputs':post,
                                     'source_build_accepted_identity_unchanged':unchanged,
                                     'launcher_gate_passed':ok,
                                     'remaining':'Complete artifact comparison, review and delivery are separate gates'})
        print('ORIGINAL_PACK_EXIT=0; LAUNCHER_COMPLETION=' + str(ok), flush=True)
        return 0 if ok else 1
    except BaseException as error:
        save(out / 'launcher-failure.json', {'when_utc':stamp(), 'error':str(error),
                                           'scope':'Launcher failure, not a successful regression result'})
        raise


if __name__ == '__main__':
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, struct.error) as error:
        print('REFUSED: ' + str(error), file=sys.stderr)
        sys.exit(2)
