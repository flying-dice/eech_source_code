"""Bounded source/offline packet; reuses existing input mapper and direct runner.
No build, native scenario, runtime checkpoint or Root materialization writes.
Usage seal | extras <output> | freeze | publish.
"""
import ast,datetime,hashlib,json,pathlib,re,subprocess,sys
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
PACK='eech-campaign/reference/d3-entity-collision/native-flag-read-correction';P=W/PACK
READERS=['nr602-derive-helpers-20261108.py', 'nr602-add-dc-check-20261108.py', 'nr602-provenance-20261108.py', 'nr602-derive-proof-20261108.py', 'run-physical-flags-nr602-direct-20261108.py', 'check-physical-flags-nr602-linux-20261108.sh', 'run-nr602-final-20261108.py', 'nr602-proof-20261108.py', 'sector-roster-inputmap-batched-20261106.py']
ROOTS=['codex-d3-native-read-correction-20261108-process.json', 'independent-native-flag-read-48570-source-controller-20261108.json', 'independent-native-flag-read-48570-source-controller-20261108-v2.json', 'native-read-48570-Root-binding-20261108.json', 'native-read-602f-Root-publication-20261108.json', 'codex-d3-native-read-implementation-20261108-completion.json', 'native-read-602f-all9-CI-terminal-20261108.json', 'native-read-nr602-materialized-plan-v2-20261108.json', 'native-read-nr602-staging-readonly-preflight-v2-20261108.json', 'nr602-production-prefix-identity-20261108.json', 'native-read-602f-CI-job-128609409742-20261108.log', 'native-read-602f-CI-job-128609409746-20261108.log', 'native-read-602f-CI-job-128609409784-20261108.log', 'native-read-602f-CI-job-128609409851-20261108.log', 'native-read-602f-CI-job-128609409557-20261108.log']
SOURCES=['eech-campaign/crates/eech-dc/src/module.rs', 'eech-campaign/crates/eech-dc/src/native_flag_read_refusal_check.rs', 'eech-campaign/crates/eech-world/lua/physical-flags-read-transaction.lua', 'eech-campaign/crates/eech-world/lua/physical-flags-read-witness.lua', 'eech-campaign/crates/eech-world/tests/host_callbacks.rs', 'eech-campaign/crates/eech-world/tests/physical_flags_read_driver.rs']
EXTRA=['eech-campaign/tools/'+n for n in ['native_flag_read.py','native_flag_read_controller.py','native-flag-read-runtime.json','test_native_flag_read.py']]+['eech-campaign/docs/d3-native-flag-read-correction.md','eech-campaign/docs/d3-native-flag-read-plan.md','eech-campaign/rustfmt.toml']
FINAL=['s1-fmt', 's2-clippy', 's3-read-driver', 's4-converter', 's5-census']
DEV={'d1-refusal-red': 101, 'd2-refusal-green': 0, 'd3-lua-and-census': 0, 'd4-fmt-apply': 0, 'd5-clippy': 0}
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,o):new(p,(json.dumps(o,indent=2)+'\n').encode())
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():
    files={n:sha((W/n).read_bytes()) for n in EXTRA}
    files.update({n:sha((E/n).read_bytes()) for n in READERS+ROOTS+[p.name for p in sorted(E.glob('Root-native-read-feedback*.json'))]})
    return {'files':files,'aggregate':sha(json.dumps(files,sort_keys=True).encode())}
mode=sys.argv[1]
if mode=='seal':
    for n in READERS:
        if n.endswith('.py'):ast.parse((E/n).read_text(encoding='utf-8'))
    for n in EXTRA:
        if n.endswith('.py'):ast.parse((W/n).read_text(encoding='utf-8'))
    assert not P.exists() and not list(E.glob('flags-nr602-s*.json'))
    dump(E/'nr602-readers-PRE-20261108.json',readers());print('Sealed',len(READERS),'readers');sys.exit()
if mode=='extras':dump(pathlib.Path(sys.argv[2]),extras());print(extras()['aggregate']);sys.exit()
if mode=='freeze':
    assert j(E/'nr602-readers-PRE-20261108.json')==readers()
    pre=j(E/'nr602-inputmap-PRE-20261108.json');post=j(E/'nr602-inputmap-POST-20261108.json')
    assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
    ep=j(E/'nr602-extra-PRE-20261108.json');eo=j(E/'nr602-extra-POST-20261108.json');assert ep==eo==extras()
    inherited=[p for p in post['changed_from_head'] if post['files'][p]['head_blob']==post['files'][p]['git_blob_of_raw_bytes']]
    assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in p for p in inherited)
    changed=sorted(set(post['changed_from_head'])-set(inherited));assert changed==sorted(SOURCES),changed
    assert not post['deleted_from_head']
    rows={}
    for label,expected in list(DEV.items())+[(n,0) for n in FINAL]:
        stem='flags-nr602-'+label;record=j(E/(stem+'.json'))
        assert record['state']=='terminal' and record['actual_exit']==expected,(label,record)
        assert any('check-physical-flags-nr602-linux-20261108.sh' in s for s in record['args'])
        out=(E/(stem+'.stdout.log')).read_text(encoding='utf-8',errors='replace');err=(E/(stem+'.stderr.log')).read_text(encoding='utf-8',errors='replace')
        results=[{'passed':int(m[0]),'failed':int(m[1])} for m in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed;',out)]
        if label in ('s4-converter','s5-census'):
            assert results==[{'passed':1,'failed':0}],results
        if label=='s3-read-driver':
            assert results==[{'passed':9,'failed':0}],results
            assert re.search(r'Ran 5 tests',out) and re.search(r'\nOK\s*',out)
            assert len(re.findall(r'Python report validator read_\w+: actual_exit=Some\(0\)',out))==7,out[-3000:]
        rows[label]={'actual_exit':record['actual_exit'],'command':record['command'],'started_utc':record['started_utc'],'completed_utc':record['completed_utc'],'results':results,'record_sha256':sha((E/(stem+'.json')).read_bytes()),'stdout_sha256':sha((E/(stem+'.stdout.log')).read_bytes()),'stderr_sha256':sha((E/(stem+'.stderr.log')).read_bytes())}
    assert j(E/'nr602-final-completion-20261108.json')['actual_exits']=={n:0 for n in FINAL}
    P.mkdir();(P/'runs').mkdir();(P/'scripts').mkdir()
    for label in rows:
        for suffix in ('json','stdout.log','stderr.log'):new(P/'runs'/(label+'.'+suffix),(E/('flags-nr602-'+label+'.'+suffix)).read_bytes())
    for n in READERS:new(P/'scripts'/n,(E/n).read_bytes())
    evidence=['nr602-family-exclusive-20261108.json','nr602-helper-derivation-20261108.json','nr602-readers-PRE-20261108.json','nr602-PRE-utc-20261108.txt','nr602-POST-utc-20261108.txt','nr602-inputmap-PRE-20261108.json','nr602-inputmap-POST-20261108.json','nr602-extra-PRE-20261108.json','nr602-extra-POST-20261108.json','nr602-final-completion-20261108.json']+ROOTS+[p.name for p in sorted(E.glob('Root-native-read-feedback*.json'))]
    for n in evidence:new(P/n,(E/n).read_bytes())
    limits='Diagnostic-only correction: actual MODEL_BRIDGE/call_model refused channel and exact reason/table, scripted boot/SCENE, real canonical refusal over existing tiny-format model root; no successfulEngineboot/native campaign. Nine existing Unix controls incl seven actual-Lua-report validators/five Python controls; all eight Lua controls remain on Windows/Wine, external Python aggregate Unix-only. Census exact66/2 with guards unchanged. Raw module.rs adds only cfg(test) declaration; production converter prefix/NUL unchanged3abf, whole rawfile differs. Runtime same old3abf91payloads; no rebuild/relabel. No native/retail/fullpack launch; V2materialized plan/preflight0 only. All10/NativeAcceptance1/Task49/D3/currentCI/full40all8/fourinputs/review/merge open as applicable.'
    freeze={'source_before':git('rev-parse','HEAD').decode().strip(),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'kind':'nr602 delegated-refusal/census/Wine harness correction UNREVIEWED','input_rows':post['file_count'],'input_fingerprint':post['fingerprint_of_raw_sha256_map'],'PRE_POST_equal':True,'extras_equal':True,'readers_equal':True,'readers':readers(),'extra_files':eo['files'],'extra_aggregate':eo['aggregate'],'changed_files':[{'path':n,'sha256':post['files'][n]['raw_sha256']} for n in changed],'inherited_Lua_qualifications':inherited,'runs':rows,'hygiene_problems':[],'limits':limits}
    dump(E/'D3-NATIVE-READ-CORRECTION-FROZEN-20261108.json',freeze);dump(P/'D3-NATIVE-READ-CORRECTION-FROZEN-20261108.json',freeze)
    new(P/'.gitattributes',b'* -text\n');new(P/'README.md',('# READ correction (UNREVIEWED)\n\nSee [implementation and limits](../../../docs/d3-native-flag-read-correction.md).\n\n'+limits+'\n\nd1 genuine diagnostic-adapter RED retains actual callback/refused channel mismatch; d2 same control GREEN. Old exact602f Linux/Wine CI failures retained, no old CI replay. Source/readers/extras sealed before final5; counts parsed from direct logs. Root writes separate actual agent completion.\n').encode())
    print(json.dumps({'input_rows':post['file_count'],'changed':changed,'runs':{k:[v['actual_exit'],v['results']] for k,v in rows.items()},'problems':[]}));sys.exit()
assert mode=='publish'
git('add','-A',PACK)
f=j(P/'D3-NATIVE-READ-CORRECTION-FROZEN-20261108.json')
for n in SOURCES+EXTRA:git('add',n)
literal={}
for p in sorted(P.rglob('*')):
    if p.is_file():
        rel=str(p.relative_to(W)).replace('\\','/');assert git('cat-file','blob',':'+rel)==p.read_bytes();literal[str(p.relative_to(P)).replace('\\','/')]=sha(p.read_bytes())
for n in SOURCES+EXTRA:assert git('cat-file','blob',':'+n)==(W/n).read_bytes()
dump(P/'literal-bytes-check.json',{'actual_Git_index_blob_bytes_equal':True,'files':literal,'changed_sources_equal':True})
files=[{'path':str(p.relative_to(P)).replace('\\','/'),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()]
dump(P/'manifest.json',{'files':files,'scope':f['limits']});git('add','-A',PACK)
tracked={p[len(PACK)+1:] for p in git('ls-files','--',PACK).decode().splitlines()}
assert tracked=={r['path'] for r in files}|{'manifest.json'}
print(json.dumps({'manifest_entries':len(files),'tracked':len(tracked),'literal':True}))
