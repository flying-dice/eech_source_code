"""Root-only concrete normal-fire capture controller. Default is READ ONLY.
The later materialized plan and SAME launch receipt bind the compiled build,
full payload/input manifests and four separate corrected literal Lua files.
source_revision/runtime always names the compiled source (2fc for the retained
build); hashes name the later diagnostic literals/validator independently. The
full original bundle, including its unexecuted old Lua, must stay byte-exact.
No source/runtime hash
is invented here; no action occurs at import or during offline validation.
"""
import argparse
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec); spec.loader.exec_module(result); return result

common = module('normal_capture_common', HERE / 'native_flag_read_controller.py')
report = module('normal_capture_report', HERE / 'normal_fire_capture.py')
check, sha, load, now = common.check, common.sha, common.load, common.now
LUA = ('driver', 'collector', 'transaction', 'contract')
MODES = ('healthy', 'terrain_lift')

def bundle(p):
    base = Path(p['binary_dir']).resolve(); manifest = load(p['bundle_manifest'])
    check(sha(p['bundle_manifest']) == p['bundle_manifest_sha256'], 'bundle manifest changed')
    check(manifest['source_revision'] == p['source_revision'] and manifest['profile'] == 'normal-fire-correlation', 'build source/profile')
    rows = manifest['files']; check(type(p['payload_count']) is int and len(rows) == p['payload_count'] > 3, 'complete derived observer payload count')
    check(len({r['path'] for r in rows}) == len(rows), 'duplicate payload')
    observed = {str(q.relative_to(base)).replace('\\', '/') for q in base.rglob('*') if q.is_file()}
    check(observed == {r['path'] for r in rows}, 'full payload set differs')
    source_lua = manifest['source_lua']
    check(len({r['path'] for r in source_lua}) == len(source_lua), 'duplicate frozen Lua inventory')
    check({r['path'] for r in source_lua} == {name for name in observed if name.endswith('.lua')}, 'full frozen Lua inventory differs')
    bypath = {r['path']:r for r in rows}
    check(all(bypath.get(r['path']) == r for r in source_lua), 'frozen Lua bytes differ')
    check(observed - {r['path'] for r in source_lua} == {'eech-world.exe', 'eech_dc.dll', 'lua.dll'}, 'runtime payload inventory')
    for row in rows:
        f = common.child_path(base, row['path'])
        check(not f.is_symlink() and f.stat().st_size == row['bytes'] and sha(f) == row['sha256'], 'payload differs ' + row['path'])
    check('eech-world.exe' in observed, 'native executable absent')
    for old in common.FORBIDDEN:
        q = Path(old).resolve(); check(not base.is_relative_to(q) and not q.is_relative_to(base), 'active binary overlap')
    return base

def artifacts(p):
    paths = {k: Path(p[k]).resolve() for k in LUA + ('input_tool', 'input_reference')}
    paths.update(controller=Path(__file__).resolve(), validator=HERE/'normal_fire_capture.py', common=HERE/'native_flag_read_controller.py', common_validator=HERE/'native_flag_read.py')
    check(set(p['hashes']) == set(paths), 'complete artifact identities')
    for k, f in paths.items(): check(sha(f) == p['hashes'][k], 'artifact differs ' + k)
    literal = Path(p['literal_dir']).resolve()
    check({q.resolve() for q in literal.rglob('*') if q.is_file()} == {paths[k] for k in LUA}, 'exact four literal Lua files')
    return paths

def preflight(path):
    p = load(path)
    check(p['schema'] == 1 and p['state'] == 'MATERIALIZED_FOR_REVIEW', 'unmaterialized plan')
    check(re.fullmatch('[0-9a-f]{40}', p['source_revision']) and p['profile'] == 'normal-fire-correlation', 'frozen observer source/profile')
    check(p['modes'] == list(MODES) and p['bounds'] == dict(scenario='lebanon_retail', seed=1, frame_ms=100, warmup=30, settle=120, frames=1500), 'fixed accepted bounds/order')
    check(re.fullmatch(r'[\w.:-]{1,100}', p['capture_prefix']), 'capture identity')
    paths = artifacts(p); base = bundle(p)
    roots, out = (common.safe_new_target(p[k]) for k in ('root_base', 'out_base'))
    pristine, healthy = (Path(p[k]).resolve() for k in ('pristine', 'healthy_root'))
    targets = (roots, out, pristine, healthy, base, Path(p['literal_dir']).resolve())
    for i, x in enumerate(targets):
        for y in targets[i+1:]: check(not x.is_relative_to(y) and not y.is_relative_to(x), 'overlapping plan directories')
    for x in (pristine, healthy):
        check(x.is_dir(), 'Root must materialize pristine/healthy only')
        for old in common.FORBIDDEN:
            q=Path(old).resolve(); check(not x.is_relative_to(q) and not q.is_relative_to(x), 'active root overlap')
    reference = load(paths['input_reference'])
    check(reference['files'] == len(reference['inputs']) == 1179 and len({r['path'] for r in reference['inputs']}) == 1179, 'full unchanged Lebanon inputs')
    for root in (pristine, healthy):
        for row in reference['inputs']:
            f = common.child_path(root, row['path'])
            check(not f.is_symlink() and f.stat().st_size == row['size'] and sha(f) == row['sha256'], 'Root input bytes differ')
    return p, paths, base, roots, out, reference

def expected(p, mode, reference='none'):
    return dict(schema='2', mode=mode, capture=p['capture_prefix']+':'+mode, runtime=p['source_revision'], bundle=p['bundle_manifest_sha256'], reference=reference, **{k:p['hashes'][k] for k in LUA})

def healthy_reference(path, digest, identity):
    check(sha(path) == digest, 'healthy report bytes changed')
    r = report.validate(Path(path).read_bytes(), identity)
    check(r['verdict']['status'] == 'PASS' and r['identity']['mode'] == 'healthy', 'healthy positive absent; B forbidden')
    return r

def execute(path, clearance):
    check(os.name == 'nt', 'Root native execution is Windows only')
    p, a, base, roots, out, input_reference = preflight(path)
    review = load(clearance)
    check(review['decision']['native_controller_launch_clearance'] is True and review['plan_sha256'] == sha(path), 'SAME materialized launch clearance')
    check(review['controller_sha256'] == sha(__file__) and review['artifact_hashes'] == p['hashes'], 'review artifact identities')
    out.mkdir(); receipt=out/'process.json'; completion=out/'completion.json'
    state=dict(controller_pid=os.getpid(), state='running', started_utc=now(), plan_sha256=sha(path), controller_sha256=sha(__file__), review_sha256=sha(clearance), source_revision=p['source_revision'], profile=p['profile'], stages=[], cases=[])
    def persist():
        tmp=out/'process.tmp';tmp.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8');tmp.replace(receipt)
    def run(label, argv, cwd):
        row=dict(label=label, argv=list(map(str,argv)), state='running', started_utc=now());state['stages'].append(row)
        with (cwd/(label+'.stdout.log')).open('xb') as stdout, (cwd/(label+'.stderr.log')).open('xb') as stderr:
            child=subprocess.Popen(row['argv'],cwd=cwd,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            row.update(pid=child.pid,created_utc=common.birth(child));persist();code=child.wait()
        row.update(state='terminal', actual_exit=code, completed_utc=now(), stdout_sha256=sha(cwd/(label+'.stdout.log')), stderr_sha256=sha(cwd/(label+'.stderr.log')));persist();return code
    def inputs(root, cwd, label):
        target=cwd/(label+'.json');code=run(label,[sys.executable,'-I','-B','-X','utf8',a['input_tool'],root,'--compare',a['input_reference'],'--output',target],cwd)
        got=load(target);check(code==0 and got['files']==input_reference['files'] and got['inputs']==input_reference['inputs'],'full input comparison')
        return dict(path=str(target),sha256=sha(target),files=got['files'])
    persist();healthy=None;healthy_path=None;healthy_sha='none'
    try:
        for index, mode in enumerate(MODES):
            # Validate COMPLETE positive and literal bytes BEFORE any child
            # directory/root/copy/boot/provider exists, not just a PASS marker.
            if index: healthy=healthy_reference(healthy_path,healthy_sha,expected(p,'healthy'))
            artifacts(p);bundle(p)
            if index:
                roots.mkdir();root=roots/'02-terrain_lift';shutil.copytree(p['pristine'],root)
            else:root=Path(p['healthy_root']).resolve()
            cwd=out/f'{index+1:02d}-{mode}';cwd.mkdir()
            row=dict(mode=mode,root=str(root),out=str(cwd),state='running');state['cases'].append(row)
            row['pre_inputs']=inputs(root,cwd,'pre-inputs')
            argv=[base/'eech-world.exe',a['driver'],f'root={root}',f'out={cwd/"capture.txt"}',f'mode={mode}',f'capture={p["capture_prefix"]}:{mode}',f'runtime={p["source_revision"]}',f'bundle={p["bundle_manifest_sha256"]}',f'driver_sha256={p["hashes"]["driver"]}']
            for k in ('collector','transaction','contract'):argv.extend([f'{k}={a[k]}',f'{k}_sha256={p["hashes"][k]}'])
            if index:argv.extend([f'reference={healthy_path}',f'reference_sha256={healthy_sha}'])
            code=run('native',argv,cwd);row['post_inputs']=inputs(root,cwd,'post-inputs');artifacts(p);bundle(p)
            result_path=cwd/'capture.txt';events=result_path.with_name('capture.txt.events')
            result=report.validate(result_path.read_bytes(),expected(p,mode,healthy_sha),healthy)
            row.update(state='terminal',native_exit=code,report_sha256=sha(result_path),events_sha256=sha(events),status=result['verdict']['status']);persist()
            if not index and row['status']=='UNREACHED':
                check(code!=0,'UNREACHED unexpectedly exited0');state['cases'].append(dict(mode='terrain_lift',status='NOT_LAUNCHED_UNREACHED',native_executed=False));state['verdict']='UNREACHED';break
            check(code==0 and row['status']=='PASS','native/observer failure precedes UNREACHED')
            if not index:healthy,healthy_path,healthy_sha=result,result_path,sha(result_path)
            else:state['verdict']='FINITE_SAME_COHORT_EFFECT'
        state.update(state='terminal',actual_exit=0,completed_utc=now())
    except BaseException as exc:
        state.update(state='terminal',actual_exit=1,completed_utc=now(),error=repr(exc));persist()
        with completion.open('x',encoding='utf-8') as f:json.dump(state,f,indent=2)
        raise
    persist()
    with completion.open('x',encoding='utf-8') as f:json.dump(state,f,indent=2)
    return state

def main():
    q=argparse.ArgumentParser();q.add_argument('plan');q.add_argument('--execute',action='store_true');q.add_argument('--clearance');a=q.parse_args()
    if a.execute:
        check(a.clearance,'SAME exact launch receipt required');print(json.dumps(execute(a.plan,a.clearance)))
    else:preflight(a.plan);print('READ-ONLY PREFLIGHT PASS; no process/root/output created')
if __name__=='__main__':main()
