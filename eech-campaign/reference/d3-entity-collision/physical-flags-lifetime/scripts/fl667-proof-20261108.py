"""Finite lifetime packet. Reuses existing batched mapper/map comparer and direct
runner/cache helper. This small publisher has no build, scenario or baseline logic.
Usage: seal | extras <out> | freeze | publish
"""
import ast,datetime,hashlib,json,pathlib,re,subprocess,sys
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
PACK='eech-campaign/reference/d3-entity-collision/physical-flags-lifetime';P=W/PACK
READERS=['derive-fl667-helpers-20261108.py','run-physical-flags-fl667-direct-20261108.py','check-physical-flags-fl667-linux-20261108.sh','run-fl667-final-20261108.py','fl667-proof-20261108.py','sector-roster-inputmap-batched-20261106.py','sho-compare-maps-20261107.py']
ROOTS=['codex-d3-flag-lifetime-20261108-process.json','motion-flags-reviewed-live-services-20261108.json','ledger-motion-closure-v191-preservation-20261108.json','motion-reviewed-667ff-Root-publication-20261108.json','motion-667ff-all9-CI-readback-20261108.json','motion-58b25-all9-CI-readback-20261108.json']
DOCS=['eech-campaign/docs/d3-flag-provider-lifetime.md','eech-campaign/docs/m5-model-host-authority.md','eech-campaign/tools/migration-progress.py','eech-campaign/rustfmt.toml']
FINAL=['s1-fmt','s2-clippy-default','s3-clippy-lifetime','s4-lifetime','s5-source-guard','s6-build-default']
DEV={'d1-fmt-apply':0,'d2-lifetime-first':101,'d3-lifetime-word-type-fixed':101,'d4-fmt-final-source':0,'d5-lifetime-withheld-refusal':0}
EXPECTED=sorted(['eech-campaign/crates/eech-engine-sys/Cargo.toml','eech-campaign/crates/eech-engine-sys/build/main.rs','eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c','eech-campaign/crates/eech-engine-sys/csrc/eech_flags_lifetime_check.h','eech-campaign/crates/eech-engine-sys/tests/physical_flags_lifetime_guard.rs','eech-campaign/crates/eech-engine/Cargo.toml','eech-campaign/crates/eech-engine/src/lib.rs','eech-campaign/crates/eech-engine/src/flag_lifetime_check.rs'])
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,obj):new(p,(json.dumps(obj,indent=2)+'\n').encode())
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():
    files={n:sha((W/n).read_bytes()) for n in DOCS}
    files.update({n:sha((E/n).read_bytes()) for n in READERS+ROOTS+[p.name for p in sorted(E.glob('Root-flag-lifetime-feedback*.json'))]})
    return {'files':files,'aggregate':sha(json.dumps(files,sort_keys=True).encode())}
mode=sys.argv[1]
if mode=='seal':
    for n in READERS:
        if n.endswith('.py'):ast.parse((E/n).read_text(encoding='utf-8'))
    assert not P.exists()
    assert not list(E.glob('flags-fl667-s*.json'))
    dump(E/'fl667-readers-PRE-20261108.json',readers())
    print('Sealed',len(READERS),'readers');sys.exit()
if mode=='extras':dump(pathlib.Path(sys.argv[2]),extras());print(extras()['aggregate']);sys.exit()
if mode=='freeze':
    assert j(E/'fl667-readers-PRE-20261108.json')==readers()
    pre=j(E/'fl667-inputmap-PRE-20261108.json');post=j(E/'fl667-inputmap-POST-20261108.json')
    assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
    ep=j(E/'fl667-extra-PRE-20261108.json');eo=j(E/'fl667-extra-POST-20261108.json');assert ep==eo==extras()
    inherited=[p for p in post['changed_from_head'] if post['files'][p]['head_blob']==post['files'][p]['git_blob_of_raw_bytes']]
    assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in p for p in inherited)
    changed=sorted(set(post['changed_from_head'])-set(inherited));assert changed==EXPECTED,changed
    assert not post['deleted_from_head']
    rows={}
    for label,expected in list(DEV.items())+[(n,0) for n in FINAL]:
        stem='flags-fl667-'+label;record=j(E/(stem+'.json'))
        assert record['state']=='terminal' and record['actual_exit']==expected,(label,record)
        assert any('check-physical-flags-fl667-linux-20261108.sh' in s for s in record['args'])
        out=(E/(stem+'.stdout.log')).read_text(encoding='utf-8',errors='replace')
        results=[{'passed':int(m[0]),'failed':int(m[1])} for m in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed;',out)]
        if label in ('s4-lifetime','s5-source-guard'):
            assert len(results)==1 and results[0]['passed']>0 and results[0]['failed']==0
        if label=='s4-lifetime':
            traffic=re.search(r'flag lifetime real callback calls: (\d+); groups: (\d+); store word: (\d+); post-poison calls: (\d+)',out)
            assert traffic and int(traffic[1])>0 and int(traffic[2])>0 and int(traffic[3])>0 and int(traffic[4])==0
        rows[label]={'actual_exit':record['actual_exit'],'command':record['command'],'started_utc':record['started_utc'],'completed_utc':record['completed_utc'],'results':results,'record_sha256':sha((E/(stem+'.json')).read_bytes()),'stdout_sha256':sha((E/(stem+'.stdout.log')).read_bytes()),'stderr_sha256':sha((E/(stem+'.stderr.log')).read_bytes())}
    assert j(E/'fl667-final-completion-20261108.json')['actual_exits']=={n:0 for n in FINAL}
    P.mkdir()
    (P/'runs').mkdir();(P/'scripts').mkdir()
    for label in rows:
        for suffix in ('json','stdout.log','stderr.log'):new(P/'runs'/(label+'.'+suffix),(E/('flags-fl667-'+label+'.'+suffix)).read_bytes())
    for n in READERS:new(P/'scripts'/n,(E/n).read_bytes())
    evidence=['fl667-family-exclusive-20261108.json','fl667-derivation-20261108.json','fl667-helper-comment-qualification-20261108.json','fl667-readers-PRE-20261108.json','fl667-PRE-utc-20261108.txt','fl667-POST-utc-20261108.txt','fl667-inputmap-PRE-20261108.json','fl667-inputmap-POST-20261108.json','fl667-extra-PRE-20261108.json','fl667-extra-POST-20261108.json','fl667-final-completion-20261108.json']+ROOTS+[p.name for p in sorted(E.glob('Root-flag-lifetime-feedback*.json'))]
    for n in evidence:new(P/n,(E/n).read_bytes())
    freeze={'source_before':git('rev-parse','HEAD').decode().strip(),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'kind':'Finite supported flag lifetime source/entry collaborator characterization; UNREVIEWED','input_rows':post['file_count'],'input_fingerprint':post['fingerprint_of_raw_sha256_map'],'PRE_POST_equal':True,'extras_equal':True,'readers_equal':True,'readers':readers(),'extra_files':eo['files'],'extra_aggregate':eo['aggregate'],'changed_files':[{'path':n,'sha256':post['files'][n]['raw_sha256']} for n in changed],'inherited_Lua_qualifications':inherited,'runs':rows,'hygiene_problems':[],'limits':'NUL boot fails before C/retail boot; private preboot C entry uses real macros/Rust callback/store. Not successfulEngineboot, Lua, native/publicREAD, reset, replacement, FPU, fullglobal/session/VM/thread lifetime or DCS acceptance. Old proofs not replayed. d2 compile-type failure and d3 wrong expected early-refusal marker retained, not productRED. Code gate/include changed; no oldWindowsbin relabel.'}
    dump(E/'D3-FLAG-LIFETIME-FROZEN-20261108.json',freeze);dump(P/'D3-FLAG-LIFETIME-FROZEN-20261108.json',freeze)
    new(P/'.gitattributes',b'* -text\n')
    new(P/'README.md',('''# Finite flag provider lifetime (UNREVIEWED)

See [contract](../../../docs/d3-flag-provider-lifetime.md). Source before: '''+freeze['source_before']+'''. The NUL-argument safe boot attempt stops before any C/retail boot. The private preboot entry uses the actual ENTER/LEAVE, owning engine unit's MODEL/model_callback/with_host, real C flag binding and reference flag_answer/store. No copied validator or cross-crate engine enum is used. Positive nonzero callback/store traffic, NULL-host normal/argument-error retention, rejected second boot/provider drop, withheld-answer fatal and poisoned refusal are one isolated test process, without global resets. This is characterization before any move, not a product correction or manufactured RED.

The source/readers/extras maps were sealed before the six final commands, and actual selected counts parsed from direct receipts afterwards. d2 compile-type failure and d3 incorrect expected C early-refusal marker are retained development mistakes, not product RED. No old motion/Tacview/default experiment is rerun. The complete-state Task50 model wrapper evidence stays at its historical identity; this is no current FPU measurement. The unsupported fresh-store fN alias possibility remains explicit; no refusal guarantee or reset/epoch framework was added.

All10/NativeAcceptance1/publicnativeREAD/FPU/lifetime/DCS/wholeD3/currentfull40all8/fourinputs/CI/review/merge remain open as applicable. Root owns unchanged original fullpacks. No Windows/native campaign or retail boot was launched; compiled Linux C controls and default Linux module build have their own actual results. Raw eech_engine.c adds one test-header include, original NUL retained; 3abf Windows binaries are not relabelled.
''').encode())
    print(json.dumps({'input_rows':post['file_count'],'changed':changed,'runs':{k:[v['actual_exit'],v['results']] for k,v in rows.items()},'problems':[]}));sys.exit()
assert mode=='publish'
git('add','-A',PACK)
f=j(P/'D3-FLAG-LIFETIME-FROZEN-20261108.json')
for row in f['changed_files']:git('add',row['path'])
git('add','eech-campaign/docs/d3-flag-provider-lifetime.md')
literal={}
for p in sorted(P.rglob('*')):
    if p.is_file():
        rel=str(p.relative_to(W)).replace('\\','/');assert git('cat-file','blob',':'+rel)==p.read_bytes();literal[str(p.relative_to(P)).replace('\\','/')]=sha(p.read_bytes())
for row in f['changed_files']:
    assert git('cat-file','blob',':'+row['path'])==(W/row['path']).read_bytes()
    assert sha((W/row['path']).read_bytes())==row['sha256']
dump(P/'literal-bytes-check.json',{'actual_Git_index_blob_bytes_equal':True,'files':literal,'changed_sources_equal':True})
files=[{'path':str(p.relative_to(P)).replace('\\','/'),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()]
dump(P/'manifest.json',{'files':files,'scope':f['limits']});git('add','-A',PACK)
tracked={p[len(PACK)+1:] for p in git('ls-files','--',PACK).decode().splitlines()}
assert tracked=={r['path'] for r in files}|{'manifest.json'}
print(json.dumps({'manifest_entries':len(files),'tracked':len(tracked),'literal':True}))
