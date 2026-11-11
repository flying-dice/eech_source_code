"""Root-run controller for nr395. Default/preflight is READ ONLY.
Execution requires a materialized plan and exact SAME controller-launch receipt.
This source does not materialize or authorize anything when imported/tested.
"""
import argparse, ctypes, datetime, hashlib, importlib.util, json, os, shutil, subprocess, sys
from pathlib import Path, PurePosixPath

HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('nr395_report',HERE/'native_flag_read.py')
report=importlib.util.module_from_spec(spec);spec.loader.exec_module(report)
RUNTIME=report.RUNTIME
FORBIDDEN=[
 'D:/EECH-Migration-Evidence/height-core-load-native-764-20261107-source',
 'D:/EECH-Migration-Evidence/height-shipping-764-20261107',
 'D:/EECH-Migration-Evidence/height-shipping-fullpack-roots-764-20261107',
 'D:/EECH-Migration-Evidence/supply-boolean-3abf-20261107-source',
 'D:/EECH-Migration-Evidence/supply-shipping-3abf-20261107',
 'D:/EECH-Migration-Evidence/supply-shipping-fullpack-roots-3abf-20261107',
]
def check(ok,why):report.check(ok,why)
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(Path(p).read_bytes())
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def child_path(base,name):
    q=PurePosixPath(name)
    check(name and not q.is_absolute() and '..' not in q.parts and '\\' not in name and ':' not in name,'unsafe manifest path')
    p=(base/Path(*q.parts)).resolve();check(p.is_relative_to(base),'manifest escapes directory');return p
def safe_new_target(text):
    check(Path(text).is_absolute(),'target must be absolute');p=Path(text).resolve()
    for f in FORBIDDEN:
        old=Path(f).resolve();check(not p.is_relative_to(old) and not old.is_relative_to(p),'target overlaps active immutable job')
    check(not p.exists(),'target already exists; do not restart/overwrite');return p
def artifacts(p):
    names={'controller':Path(__file__).resolve(),'validator':HERE/'native_flag_read.py','binding':HERE/'native-flag-read-runtime.json'}
    names.update({k:Path(p[k]).resolve() for k in ('lua','protocol','input_tool','input_reference')})
    for k,path in names.items():check(sha(path)==p['hashes'][k],'artifact identity differs '+k)
    return names
def bundle_check(p,binding):
    base=Path(p['binary_dir']).resolve();manifest=load(p['bundle_manifest'])
    check(sha(p['bundle_manifest'])==p['bundle_manifest_sha256'],'bundle manifest hash changed')
    check(manifest['runtime']==RUNTIME,'bundle runtime differs')
    rows=manifest['files'];check(len({r['path'] for r in rows})==len(rows),'duplicate full-bundle path')
    observed={str(q.relative_to(base)).replace('\\','/') for q in base.rglob('*') if q.is_file()}
    check(observed=={r['path'] for r in rows},'full bundle file set differs')
    byname={r['path']:r for r in rows}
    for row in rows:
        path=child_path(base,row['path']);check(not path.is_symlink() and path.stat().st_size==row['bytes'] and sha(path)==row['sha256'],'bundle bytes differ '+row['path'])
    expected=binding['payloads'];check(len(expected)==91 and len({r['path'] for r in expected})==91,'runtime binding is not complete91')
    for row in expected:check(byname.get(row['path'])==row,'one of original91 payloads differs')
    literal=Path(p['literal_dir']).resolve()
    check({q.resolve() for q in literal.rglob('*') if q.is_file()}=={Path(p[k]).resolve() for k in ('lua','protocol')},'literal directory must contain exactly the two reviewed Lua files')
    for key in ('lua','protocol'):
        path=Path(p[key]).resolve();check(path.is_relative_to(literal) and sha(path)==p['hashes'][key],'staged Lua outside/changed literal copy')
    for f in FORBIDDEN:
        old=Path(f).resolve();check(not base.is_relative_to(old) and not old.is_relative_to(base),'binary copy overlaps running job')
    return base
def preflight(plan_path):
    p=load(plan_path);check(p.get('schema')==1 and p.get('state')=='MATERIALIZED_FOR_REVIEW','UNMATERIALIZED: no launch-ready plan')
    check(p['runtime']==RUNTIME and p['source_plan_sha256']==report.PLAN,'wrong runtime/source plan')
    check(p['modes']==report.MODES,'not exactly accepted seven-case order')
    a=artifacts(p);binding=load(a['binding']);check(binding['runtime']==RUNTIME,'wrong binding runtime')
    base=bundle_check(p,binding)
    roots=safe_new_target(p['root_base']);out=safe_new_target(p['out_base'])
    check(not roots.is_relative_to(out) and not out.is_relative_to(roots),'root/output trees overlap')
    pristine=Path(p['pristine']).resolve();check(pristine.is_dir(),'pristine unavailable')
    check(not roots.is_relative_to(pristine) and not pristine.is_relative_to(roots),'roots overlap pristine')
    healthy=Path(p['healthy_root']).resolve();check(healthy.is_dir(),'Root must materialize the healthy root')
    check(not healthy.is_relative_to(pristine) and not pristine.is_relative_to(healthy),'healthy root overlaps pristine')
    for f in FORBIDDEN:
        old=Path(f).resolve();check(not healthy.is_relative_to(old) and not old.is_relative_to(healthy),'healthy root overlaps running job')
    ref=load(a['input_reference']);check(ref['files']==len(ref['inputs'])>0,'incomplete input reference')
    check(len({r['path'] for r in ref['inputs']})==ref['files'],'duplicate input reference')
    return p,a,binding,base,roots,out,ref

def birth(child):
    # Exact existing accepted Windows GetProcessTimes technique: handle remains
    # valid even if the child has already exited; never infer birth from our clock.
    from ctypes import wintypes
    fs=[wintypes.FILETIME() for _ in range(4)]
    fn=ctypes.windll.kernel32.GetProcessTimes
    fn.argtypes=[wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4;fn.restype=wintypes.BOOL
    check(fn(wintypes.HANDLE(int(child._handle)),*[ctypes.byref(x) for x in fs]),'actual process birth unavailable')
    ticks=(fs[0].dwHighDateTime<<32)|fs[0].dwLowDateTime
    return datetime.datetime.fromtimestamp((ticks-116444736000000000)/10000000,datetime.timezone.utc).isoformat()

def child_reference(path,digest,expected):
    raw=Path(path).read_bytes();check(hashlib.sha256(raw).hexdigest()==digest,'healthy file changed')
    healthy=report.validate(raw,expected)
    check(healthy['verdict']['status']=='PASS','healthy transaction absent')
    return healthy

def execute(plan_path,clearance):
    check(os.name=='nt','native execution requires reviewed Windows environment')
    p,a,binding,base,roots,out,input_reference=preflight(plan_path)
    review=load(clearance)
    check(review['decision']['native_controller_launch_clearance'] is True,'SAME native controller clearance absent')
    check(review['plan_sha256']==sha(plan_path) and review['controller_sha256']==sha(__file__),'review does not bind this concrete plan/controller')
    for key in ('lua','protocol','validator','binding'):check(review[key+'_sha256']==p['hashes'][key],'review artifact differs '+key)
    # No hash cycle: the materialized plan hashes controller/Lua; the separate
    # later review hashes that plan. Neither source contains its own digest.
    out.mkdir()
    receipt=out/'process.json';completion=out/'completion.json'
    state={'controller_pid':os.getpid(),'state':'running','started_utc':now(),'plan_sha256':sha(plan_path),'controller_sha256':sha(__file__),'review_sha256':sha(clearance),'runtime':RUNTIME,'bundle_manifest_sha256':p['bundle_manifest_sha256'],'stages':[],'cases':[]}
    def persist():
        tmp=out/'process.tmp';tmp.write_text(json.dumps(state,indent=2)+'\n',encoding='utf-8');tmp.replace(receipt)
    def run(label,argv,cwd):
        with (cwd/(label+'.stdout.log')).open('xb') as stdout,(cwd/(label+'.stderr.log')).open('xb') as stderr:
            child=subprocess.Popen([str(v) for v in argv],cwd=cwd,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
            row={'stage':label,'pid':child.pid,'birth_utc':birth(child),'argv':[str(v) for v in argv],'cwd':str(cwd),'state':'running','started_utc':now()}
            state['stages'].append(row);persist();code=child.wait()
        row.update(state='terminal',actual_exit=code,completed_utc=now(),stdout_sha256=sha(cwd/(label+'.stdout.log')),stderr_sha256=sha(cwd/(label+'.stderr.log')));persist();return code
    def inputs(root,case_dir,label):
        target=case_dir/(label+'.json')
        code=run(label,[sys.executable,'-I','-B','-X','utf8',a['input_tool'],root,'--compare',a['input_reference'],'--output',target],case_dir)
        check(code==0,'input manifest tool failed')
        got=load(target);check(got['files']==input_reference['files'] and got['inputs']==input_reference['inputs'],'FULL input rows differ')
        return {'path':str(target),'sha256':sha(target),'files':got['files']}
    expected={'lua':p['hashes']['lua'],'protocol':p['hashes']['protocol'],'bundle':p['bundle_manifest_sha256'],'reference':'none'}
    healthy=None;healthy_path=None;healthy_sha=None
    persist()
    try:
        for index,mode in enumerate(report.MODES):
            if index:
                # Full reference and actual file hash validated BEFORE this
                # child output directory, retail copy, boot or provider exists.
                healthy=child_reference(healthy_path,healthy_sha,expected)
            bundle_check(p,binding)
            case_dir=out/f'{index+1:02d}-{mode}'
            if index:
                if not roots.exists():roots.mkdir()
                root=roots/f'{index+1:02d}-{mode}'
            else:root=Path(p['healthy_root']).resolve()
            case_dir.mkdir()
            if index:shutil.copytree(p['pristine'],root)
            row={'mode':mode,'root':str(root),'out':str(case_dir),'state':'running'};state['cases'].append(row)
            row['pre_inputs']=inputs(root,case_dir,'pre-inputs')
            argv=[base/'eech-world.exe',a['lua'],f'root={root}',f'out={case_dir / "witness.txt"}',f'protocol={a["protocol"]}',f'mode={mode}','request_updated=first_eligible',f'runtime_code={RUNTIME}',f'lua_sha256={expected["lua"]}',f'protocol_sha256={expected["protocol"]}',f'plan_sha256={report.PLAN}',f'bundle_sha256={expected["bundle"]}','scenario=lebanon_retail','hours=0.001','frame_ms=100','seed=1']
            if index:argv += [f'control={healthy_path}',f'control_sha256={healthy_sha}']
            code=run('native',argv,case_dir)
            row['post_inputs']=inputs(root,case_dir,'post-inputs');bundle_check(p,binding)
            path=case_dir/'witness.txt';raw=path.read_bytes()
            facts=report.validate(raw,{**expected,'reference':healthy_sha if index else 'none'},mode,healthy)
            row.update(state='terminal',native_exit=code,report_sha256=sha(path),status=facts['verdict']['status']);persist()
            if not index and row['status']=='UNREACHED':
                check(code!=0,'UNREACHED diagnostic unexpectedly exited0')
                for child_mode in report.MODES[1:]:state['cases'].append({'mode':child_mode,'status':'NOT_LAUNCHED_UNREACHED','native_executed':False})
                break
            check(code==0 and row['status']=='PASS','native diagnostic failed')
            if not index:healthy,healthy_path,healthy_sha=facts,path,sha(path)
        state.update(state='terminal',actual_exit=0,completed_utc=now(),scope='Finite diagnostics only; UNREACHED never a pass; caught expected refusal0 is not healthy campaign continuation')
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
        check(a.clearance is not None,'exact launch clearance required');s=execute(a.plan,a.clearance);print(json.dumps({'actual_exit':s['actual_exit'],'cases':len(s['cases'])}))
    else:
        preflight(a.plan);print('READ-ONLY PREFLIGHT PASS; no process/root/output created')
if __name__=='__main__':main()
