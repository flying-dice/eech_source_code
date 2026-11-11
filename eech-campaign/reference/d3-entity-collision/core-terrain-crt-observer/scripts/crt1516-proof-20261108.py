"""Compact observer proof; full timed source maps and failed raw compile logs stay
immutable outsideGit with hashes. No retail/native/build-controller execution.
seal | extras <output> | freeze | publish
"""
import pathlib,json,hashlib,subprocess,datetime,ast,re,sys
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029';PACK='eech-campaign/reference/d3-entity-collision/core-terrain-crt-observer';P=W/PACK
READERS=['crt1516-derive-helpers-20261108.py','crt1516-wire-observer-20261108.py','crt1516-bind-controls-20261108.py','run-core-crt-crt1516-direct-20261108.py','check-core-crt-crt1516-linux-20261108.sh','run-crt1516-final-20261108.py','crt1516-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-flag-boundary-disposition-1516-20261108.json','flag-boundary-disposition-1516-Root-binding-20261108.json','codex-d3-flag-supported-boundary-20261108-completion.json','codex-d3-core-crt-characterization-20261108-process.json']
DOCS=['eech-campaign/docs/d3-core-terrain-crt-capture-plan.md','eech-campaign/rustfmt.toml']
EXPECTED=['eech-campaign/crates/'+n for n in ['eech-dc/Cargo.toml','eech-dc/src/module.rs','eech-engine/Cargo.toml','eech-engine/src/lib.rs','eech-engine-sys/Cargo.toml','eech-engine-sys/build/main.rs','eech-engine-sys/build/core_crt_observer.rs','eech-engine-sys/csrc/eech_engine.c','eech-engine-sys/csrc/eech_core_crt_observer.c','eech-engine-sys/csrc/eech_core_crt_observer.h','eech-engine-sys/src/lib.rs','eech-engine-sys/tests/core_terrain_crt_observer.rs','eech-world/lua/core-terrain-crt-observer.lua','eech-world/tests/core_terrain_crt_driver.rs']]
DEV={'d1-observer':101,'d2-observer-libc-order':101,'d3-observer-real-sys-link':0,'d4-staging-driver-census':0,'d5-fmt-and-default-clippy':0,'d6-feature-clippy':0,'d7-final-fmt':0,'d8-format-frozen':0}
FINAL=['s1-fmt','s2-default-clippy','s3-observer-clippy','s4-observer','s5-driver','s6-profile-census']
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,o):new(p,(json.dumps(o,indent=2)+'\n').encode())
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():
    files={n:sha((W/n).read_bytes()) for n in DOCS};files.update({n:sha((E/n).read_bytes()) for n in READERS+ROOTS+[p.name for p in sorted(E.glob('Root-core-crt-feedback*.json'))]});return {'files':files,'aggregate':sha(json.dumps(files,sort_keys=True).encode())}
mode=sys.argv[1]
if mode=='seal':
    for n in READERS:
        if n.endswith('.py'):ast.parse((E/n).read_text(encoding='utf-8'))
    assert not P.exists() and not list(E.glob('core-crt-crt1516-s*.json'))
    dump(E/'crt1516-readers-PRE-20261108.json',readers());print('Sealed',len(READERS),'readers');sys.exit()
if mode=='extras':dump(pathlib.Path(sys.argv[2]),extras());print(extras()['aggregate']);sys.exit()
if mode=='freeze':
    assert readers()==j(E/'crt1516-readers-PRE-20261108.json')
    pre=j(E/'crt1516-inputmap-PRE-20261108.json');post=j(E/'crt1516-inputmap-POST-20261108.json');assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
    ep=j(E/'crt1516-extra-PRE-20261108.json');eo=j(E/'crt1516-extra-POST-20261108.json');assert ep==eo==extras()
    inherited=[p for p in post['changed_from_head'] if post['files'][p]['head_blob']==post['files'][p]['git_blob_of_raw_bytes']];assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in p for p in inherited)
    changed=sorted(set(post['changed_from_head'])-set(inherited));assert changed==sorted(EXPECTED) and not post['deleted_from_head']
    assert not git('diff','--name-only','HEAD','--','aphavoc','modules').strip()
    rows={};private=[]
    for label,expected in list(DEV.items())+[(n,0) for n in FINAL]:
        stem='core-crt-crt1516-'+label;r=j(E/(stem+'.json'));assert r['state']=='terminal' and r['actual_exit']==expected,(label,r)
        out=(E/(stem+'.stdout.log')).read_text(encoding='utf-8',errors='replace');results=[{'passed':int(a),'failed':int(b)} for a,b in re.findall(r'test result: \w+\. (\d+) passed; (\d+) failed;',out)]
        if label in ('s4-observer','s5-driver','s6-profile-census'):assert results==[{'passed':{'s4-observer':4,'s5-driver':3,'s6-profile-census':1}[label],'failed':0}],results
        if label=='s4-observer':assert 'captured_draws=3 forwarded_after_capture=1 seeds=2 load_draws=2 gap_draws=1 inactive_skin_draws=0 active_skin_draws=1 FP_words_equal=1' in out
        rows[label]={'actual_exit':r['actual_exit'],'command':r['command'],'started_utc':r['started_utc'],'completed_utc':r['completed_utc'],'results':results}
    assert j(E/'crt1516-final-completion-20261108.json')['actual_exits']=={n:0 for n in FINAL}
    P.mkdir();(P/'runs').mkdir();(P/'scripts').mkdir()
    for label in rows:
        stem='core-crt-crt1516-'+label
        for suffix in ('json','stdout.log','stderr.log'):
            path=E/(stem+'.'+suffix);raw=path.read_bytes()
            if label in ('d1-observer','d2-observer-libc-order') and suffix!='json':private.append({'path':str(path),'bytes':len(raw),'sha256':sha(raw),'scope':'Original failed compile/link log; setup failure, not product RED'})
            else:new(P/'runs'/(label+'.'+suffix),raw)
    for n in READERS:new(P/'scripts'/n,(E/n).read_bytes())
    for n in ROOTS+['crt1516-family-exclusive-20261108.json','crt1516-helper-derivation-20261108.json','crt1516-readers-PRE-20261108.json','crt1516-PRE-utc-20261108.txt','crt1516-POST-utc-20261108.txt','crt1516-extra-PRE-20261108.json','crt1516-extra-POST-20261108.json','crt1516-final-completion-20261108.json']+[p.name for p in sorted(E.glob('Root-core-crt-feedback*.json'))]:new(P/n,(E/n).read_bytes())
    for which in ('PRE','POST'):
        p=E/('crt1516-inputmap-'+which+'-20261108.json');private.append({'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes()),'scope':'Full actual timed source map; private immutable, compact equality receipt published'})
    dump(P/'private-raw-index.json',{'files':private,'no_raw_normalization':True})
    limits='Opt-in Core CRT observer/forwarding/staged-wiring/delegation only; no real Core terrain retail load, Windows/native/capture/shipping result. Raw aphavoc/modules unchanged; eight hooks in observer-profile staging only. Dedicated profile is not TP/old3abf. Source-compiled Core calls only, not Host/libCRT; ordered seed/milestone prefix plus counts/fingerprint, not full per-draw trace. Freeze at actual first seed after load, no assumed session timing/reload/dormancy. d1 libc force-include ordering101/d2 missing actual sys-link101 retained setup failures, no product RED. Default body guard/duplicate registration correction made before final checks. Wholeloader/Native1/all10/D3 open; Root/SAME review/build/materialization before capture.'
    frozen={'source_before':git('rev-parse','HEAD').decode().strip(),'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':limits,'input_rows':post['file_count'],'source_fingerprint':post['fingerprint_of_raw_sha256_map'],'PRE_POST_source_rows_equal':True,'extras_equal':True,'readers_equal':True,'readers':readers(),'extras':eo,'changed_sources':[{'path':n,'sha256':post['files'][n]['raw_sha256']} for n in changed],'inherited_Lua_qualifications':inherited,'runs':rows,'selected_final_Rust_tests':8,'actual_CRT_control':{'captured_draws':3,'forwarded_after_capture':1,'seeds':2,'load':2,'gap':1,'inactive_skin':0,'active_skin':1,'FP_words_equal':True},'hygiene_problems':[]}
    dump(E/'D3-CORE-CRT-OBSERVER-FROZEN-20261108.json',frozen);dump(P/'D3-CORE-CRT-OBSERVER-FROZEN-20261108.json',frozen)
    new(P/'.gitattributes',b'* -text\n');new(P/'README.md',('# Core terrain CRT observer/plan (UNREVIEWED)\n\nSee [finite capture plan](../../../docs/d3-core-terrain-crt-capture-plan.md).\n\n'+limits+'\n\nFull timed source maps and failed raw compile/link logs remain private immutable with hashes, not normalized or mistaken for product failures. All applicable final source/readers/extras were sealed before six commands; actual exits/counts parsed afterwards. Final4observer+3Lua+1profile guard =8Rust instances. Actual report/skin test inputs are scripted; real CRT forwarding and compiled staging are genuine. No old accepted proof replay or loader removal. Root records actual service completion separately.\n').encode())
    print(json.dumps({'source_rows':post['file_count'],'changed':len(changed),'extras':len(eo['files']),'readers':len(READERS),'final_Rust':8,'problems':[]}));sys.exit()
assert mode=='publish'
git('add','-A',PACK)
for n in EXPECTED+DOCS:git('add',n)
literal={}
for p in sorted(P.rglob('*')):
    if p.is_file():
        rel=p.relative_to(W).as_posix();assert git('cat-file','blob',':'+rel)==p.read_bytes();literal[p.relative_to(P).as_posix()]=sha(p.read_bytes())
for n in EXPECTED+DOCS:assert git('cat-file','blob',':'+n)==(W/n).read_bytes()
dump(P/'literal-bytes-check.json',{'actual_index_blobs_equal':True,'files':literal,'current_sources_equal':True})
files=[{'path':p.relative_to(P).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',{'files':files,'scope':'Unreviewed observer/finite plan only; private maps/logs remain hash-bound, no native/capture/removal acceptance'});git('add','-A',PACK)
print(json.dumps({'manifest_entries':len(files),'tracked_packet_files':len(files)+1,'literal':True}))
