"""Narrow endpoint residual proof: seal/extras/freeze/publish; no native/C replay."""
import pathlib,json,hashlib,subprocess,sys,datetime,ast,re
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/core-terrain-crt-endpoint-correction'
READERS=['run-core-crt-crend724-direct-20261108.py','check-core-crt-crend724-linux-20261108.sh','run-crend724-final-20261108.py','crend724-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-core-crt-report-correction-b5cb-source-20261108.json','core-crt-report-correction-Root-binding-20261108.json','codex-d3-core-crt-report-correction-20261108-completion.json','codex-d3-core-crt-endpoint-correction-20261108-process.json']
RED=['crend724-RED-core-terrain-crt-observer.lua','crend724-RED-core_terrain_crt_driver.rs']
BASE='crfix19e-b-inputmap-POST-20261108.json';DOC='eech-campaign/docs/d3-core-terrain-crt-capture-plan.md'
SOURCES=['eech-campaign/crates/eech-world/lua/core-terrain-crt-observer.lua','eech-campaign/crates/eech-world/tests/core_terrain_crt_driver.rs']
RUNS={'d1-old-endpoint-red':101,'d2-endpoint-green-format':0,'s1-fmt':0,'s2-clippy':0,'s3-driver':0}
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,o):new(p,(json.dumps(o,indent=2)+'\n').encode())
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():return {**{n:sha((E/n).read_bytes()) for n in READERS+ROOTS+RED+[BASE,'crend724-family-20261108.json']},DOC:sha((W/DOC).read_bytes())}
mode=sys.argv[1]
if mode=='seal':
    assert not P.exists() and not list(E.glob('core-crt-crend724-s*.json'))
    for n in READERS:
        if n.endswith('.py'):ast.parse((E/n).read_text(encoding='utf-8'))
    dump(E/'crend724-readers-PRE-20261108.json',readers());print('readers',len(READERS));sys.exit()
if mode=='extras':dump(pathlib.Path(sys.argv[2]),extras());print('extras',len(extras()),sha(json.dumps(extras(),sort_keys=True).encode()));sys.exit()
if mode=='freeze':
    pre=j(E/'crend724-inputmap-PRE-20261108.json');post=j(E/'crend724-inputmap-POST-20261108.json');base=j(E/BASE)
    assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
    assert extras()==j(E/'crend724-extra-PRE-20261108.json')==j(E/'crend724-extra-POST-20261108.json') and readers()==j(E/'crend724-readers-PRE-20261108.json')
    inherited=[n for n in post['changed_from_head'] if post['files'][n]['head_blob']==post['files'][n]['git_blob_of_raw_bytes']]
    assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in n for n in inherited)
    assert sorted(set(post['changed_from_head'])-set(inherited))==sorted(SOURCES) and not post['deleted_from_head'] and set(post['files'])==set(base['files'])
    for n in post['files']:
        if n not in SOURCES:assert post['files'][n]['raw_sha256']==base['files'][n]['raw_sha256'],n
    old=git('show','724183dbfa608aef75532fd8082d25239f56c4fb:'+SOURCES[0]);new_lua=(W/SOURCES[0]).read_bytes()
    assert old.replace(b'(count~=1 or s.last==latest)',b'(latest_count~=1 or s.last==latest)')==new_lua and old==(E/RED[0]).read_bytes()
    rows={}
    for label,code in RUNS.items():
        stem='core-crt-crend724-'+label;r=j(E/(stem+'.json'));assert r['state']=='terminal' and r['actual_exit']==code
        out=(E/(stem+'.stdout.log')).read_text(encoding='utf-8');results=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',out)
        if label=='d1-old-endpoint-red':assert results==[('FAILED','0','1')] and 'one_field_mutation=last8_to9 accepted=true' in out
        if label in ['d2-endpoint-green-format','s3-driver']:assert 'one_field_mutation=last8_to9 accepted=false' in out
        if label=='s3-driver':assert results==[('ok','5','0')] and 'single_faults=23 wrongly_accepted=[]' in out
        rows[label]={'actual_exit':code,'command':r['command'],'started_utc':r['started_utc'],'completed_utc':r['completed_utc'],'Rust_result_rows':results}
    assert j(E/'crend724-final-completion-20261108.json')['actual_exits']=={'s1-fmt':0,'s2-clippy':0,'s3-driver':0}
    P.mkdir();(P/'runs').mkdir();(P/'scripts').mkdir()
    for label in RUNS:
        for ext in ['json','stdout.log','stderr.log']:new(P/'runs'/(label+'.'+ext),(E/('core-crt-crend724-'+label+'.'+ext)).read_bytes())
    for n in READERS:new(P/'scripts'/n,(E/n).read_bytes())
    for n in ROOTS+RED+['crend724-family-20261108.json','crend724-readers-PRE-20261108.json','crend724-extra-PRE-20261108.json','crend724-extra-POST-20261108.json','crend724-PRE-utc-20261108.txt','crend724-POST-utc-20261108.txt','crend724-final-completion-20261108.json']:new(P/n,(E/n).read_bytes())
    private=[{'path':str(E/n),'bytes':(E/n).stat().st_size,'sha256':sha((E/n).read_bytes())} for n in [BASE,'crend724-inputmap-PRE-20261108.json','crend724-inputmap-POST-20261108.json']]
    frozen={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_before':git('rev-parse','HEAD').decode().strip(),'source_rows':post['file_count'],'PRE_POST_equal':True,'source_fingerprint':post['fingerprint_of_raw_sha256_map'],'extras':extras(),'readers':readers(),'changed_sources':[{'path':n,'sha256':post['files'][n]['raw_sha256']} for n in SOURCES],'inherited_raw_filter_qualifications':inherited,'other_raw_rows_unchanged':len(post['files'])-2,'private_maps':private,'runs':rows,'final_commands':3,'final_Rust_tests':5,'new_positive':'Coherent load1/gap1 siteA with B interior draw at9','one_field_mutation':'siteA.last8->9: old gate accepts/actual101, corrected gate rejects/actual0','single_Lua_relation_changed':'latest_count==1 requires last==latest, regardless of total site draws','reused_scope':'Other report contracts/C forwarding/staging remain reviewed at original identities; no new C/Windows/native test or observer algorithm','hygiene_problems':[],'scope':'Same P2 endpoint residual candidate UNREVIEWED. No Windows/native/retail capture/removal/baseline/config/auth change. Compact evidence only; all10/GenericNative1/prep2/Task49/D3 remain open as applicable. Root owns actual agent completion separately.'}
    dump(E/'D3-CORE-CRT-ENDPOINT-CORRECTION-FROZEN-20261108.json',frozen);dump(P/'D3-CORE-CRT-ENDPOINT-CORRECTION-FROZEN-20261108.json',frozen)
    new(P/'.gitattributes',b'* -text\n');new(P/'README.md',b'# Core CRT endpoint correction (UNREVIEWED)\n\nSame P2, one Lua relation: a site with one draw in its final nonempty phase ends at that phase first, even after earlier-phase draws. Coherent multi-phase positive plus last8-to9 one-field contradiction: genuine RED101 then GREEN. Final3 direct0/fiveRust tests; old23 fault controls retained in the affected target. Actual RED source bytes and literal review/binding/completion preserved. Other4008 raw inputs and compiled552ee observer/staging unchanged. Full timed maps remain private immutable/hash-bound. Logs retain raw CRLF/EOF blanks; CURRENT/card/Git publication is post-POST metadata. No Windows/native/capture/removal or whole-D3 acceptance; STOP for SAME changed-line review.\n')
    print(json.dumps({'source_rows':post['file_count'],'other_unchanged':len(post['files'])-2,'readers':len(READERS),'extras':len(extras()),'final_Rust':5,'hygiene':[]}));sys.exit()
assert mode=='publish'
for n in SOURCES+[DOC,'STATUS.md','docs/handover/ACTIVE-EXTRACTION.json']:git('add',n)
git('add','-A',P.relative_to(W).as_posix())
for p in P.rglob('*'):
    if p.is_file():assert git('cat-file','blob',':'+p.relative_to(W).as_posix())==p.read_bytes()
files=[{'path':p.relative_to(P).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()]
dump(P/'manifest.json',{'files':files,'self_excluded':True,'literal_index_bytes_verified':True});git('add','-A',P.relative_to(W).as_posix());print(json.dumps({'manifest_entries':len(files),'packet_files':len(files)+1}))
