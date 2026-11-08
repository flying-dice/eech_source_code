"""Focused report correction: seal/extras/freeze/publish. No build/native replay."""
import pathlib,json,hashlib,subprocess,sys,datetime,ast,re
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/core-terrain-crt-report-correction'
READERS=['run-core-crt-crfix19e-direct-20261108.py','check-core-crt-crfix19e-linux-20261108.sh','run-crfix19e-final-b-20261108.py','crfix19e-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-core-crt-observer-552ee-source-plan-20261108.json','independent-core-crt-observer-552ee-source-plan-20261108-v2.json','core-crt-552ee-19e319-Root-binding-20261108.json','codex-d3-core-crt-characterization-20261108-completion.json','codex-d3-core-crt-report-correction-20261108-process.json']
DOC='eech-campaign/docs/d3-core-terrain-crt-capture-plan.md'
SOURCES=['eech-campaign/crates/eech-world/lua/core-terrain-crt-observer.lua','eech-campaign/crates/eech-world/tests/core_terrain_crt_driver.rs']
BASE='crt1516-inputmap-POST-20261108.json'
RUNS={'d1-old-gate-red':101,'d2-gate-green':0,'d3-inactive-and-diagnostics':101,'d4-final-controls':0,'d5-format-test-style':0,'s1-fmt':0,'s2-clippy':101,'s1b-fmt':0,'s2b-clippy':0,'s3b-driver':0}
def j(p):return json.loads(p.read_bytes())
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,o):new(p,(json.dumps(o,indent=2)+'\n').encode())
def reader_hashes():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():return {**{n:sha((E/n).read_bytes()) for n in READERS+ROOTS+[BASE,'crfix19e-family-20261108.json']},DOC:sha((W/DOC).read_bytes())}
mode=sys.argv[1]
if mode=='seal':
    assert not P.exists() and not list(E.glob('core-crt-crfix19e-s*b-*.json'))
    for n in READERS:
        if n.endswith('.py'):ast.parse((E/n).read_text(encoding='utf-8'))
    dump(E/'crfix19e-b-readers-PRE-20261108.json',reader_hashes());print('Sealed',len(READERS),'readers');sys.exit()
if mode=='extras':dump(pathlib.Path(sys.argv[2]),extras());print('extras',len(extras()),sha(json.dumps(extras(),sort_keys=True).encode()));sys.exit()
if mode=='freeze':
    pre=j(E/'crfix19e-b-inputmap-PRE-20261108.json');post=j(E/'crfix19e-b-inputmap-POST-20261108.json');base=j(E/BASE)
    assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
    assert extras()==j(E/'crfix19e-b-extra-PRE-20261108.json')==j(E/'crfix19e-b-extra-POST-20261108.json') and reader_hashes()==j(E/'crfix19e-b-readers-PRE-20261108.json')
    inherited=[n for n in post['changed_from_head'] if post['files'][n]['head_blob']==post['files'][n]['git_blob_of_raw_bytes']]
    assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in n for n in inherited)
    assert sorted(set(post['changed_from_head'])-set(inherited))==sorted(SOURCES) and not post['deleted_from_head']
    unchanged=[n for n in post['files'] if n not in SOURCES];assert set(post['files'])==set(base['files'])
    for n in unchanged:assert post['files'][n]['raw_sha256']==base['files'][n]['raw_sha256'],n
    rows={}
    for name,code in RUNS.items():
        stem='core-crt-crfix19e-'+name;r=j(E/(stem+'.json'));assert r['state']=='terminal' and r['actual_exit']==code
        out=(E/(stem+'.stdout.log')).read_text(encoding='utf-8');results=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',out)
        if name=='s3b-driver':assert results==[('ok','4','0')] and 'single_faults=23 wrongly_accepted=[]' in out
        if name=='d1-old-gate-red':assert 'single_faults=17' in out and 'missing_load_begin' in out and results==[('FAILED','0','1')]
        rows[name]={'actual_exit':code,'command':r['command'],'started_utc':r['started_utc'],'completed_utc':r['completed_utc'],'Rust_result_rows':results}
    assert j(E/'crfix19e-b-final-completion-20261108.json')['actual_exits']=={'s1b-fmt':0,'s2b-clippy':0,'s3b-driver':0}
    P.mkdir();(P/'runs').mkdir();(P/'scripts').mkdir()
    for name in RUNS:
        for ext in ['json','stdout.log','stderr.log']:new(P/'runs'/(name+'.'+ext),(E/('core-crt-crfix19e-'+name+'.'+ext)).read_bytes())
    for n in READERS:new(P/'scripts'/n,(E/n).read_bytes())
    for n in ['run-crfix19e-final-20261108.py','crfix19e-proof-draft1-before-final-b-20261108.py','crfix19e-test-before-clippy-fix-20261108.rs','crfix19e-final-completion-20261108.json','crfix19e-readers-PRE-20261108.json','crfix19e-extra-PRE-20261108.json','crfix19e-extra-POST-20261108.json','crfix19e-PRE-utc-20261108.txt','crfix19e-POST-utc-20261108.txt']+ROOTS+['crfix19e-family-20261108.json','crfix19e-b-readers-PRE-20261108.json','crfix19e-b-extra-PRE-20261108.json','crfix19e-b-extra-POST-20261108.json','crfix19e-b-PRE-utc-20261108.txt','crfix19e-b-POST-utc-20261108.txt','crfix19e-b-final-completion-20261108.json']:new(P/n,(E/n).read_bytes())
    private=[{'path':str(E/n),'bytes':(E/n).stat().st_size,'sha256':sha((E/n).read_bytes())} for n in [BASE,'crfix19e-b-inputmap-PRE-20261108.json','crfix19e-b-inputmap-POST-20261108.json','crfix19e-inputmap-PRE-20261108.json','crfix19e-inputmap-POST-20261108.json']]
    freeze={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_before':git('rev-parse','HEAD').decode().strip(),'source_rows':post['file_count'],'PRE_POST_equal':True,'source_fingerprint':post['fingerprint_of_raw_sha256_map'],'extras':extras(),'readers':reader_hashes(),'changed_sources':[{'path':n,'sha256':post['files'][n]['raw_sha256']} for n in SOURCES],'inherited_raw_filter_qualifications':inherited,'unchanged_raw_rows_against_compiled_observer_base':len(unchanged),'private_maps':private,'failed_final_attempt':'Original s1 format0/s2 Clippy101 for unnecessary_unwrap in new test; original seal/readers/maps/source snapshot preserved. Test style corrected only, then distinct final-b sealed.','runs':rows,'final_commands':3,'final_Rust_tests':4,'single_fault_reports_rejected':23,'positive_variants':['coherent original exact copy','actual early-reseed ordered prefix','coherent inactive skin with zero skin draws'],'genuine_old_gate_RED':'coherent positive first;16 of17 faults accepted;actual101','intermediate_fixture_error':'d3 actual101: validator correctly rejected damaged phase count at last-phase/count before the test expected a later window label; expectation corrected','reused_C_controls':'Exact552ee compiled/staged inputs; prior actual CRT forwarding/staging/gating controls retained in original packet, not replayed','hygiene_problems':[],'scope':'Lua/test/docs correction only, unreviewed. No Windows/native/retail capture, C/Rust production change, loader removal, baseline/config change or shipping acceptance. Compact consistency bounds are not full trace/digest or independent source-value truth. All10/GenericNative1/Task49/D3 and prep2 retained; Root owns completion separately.'}
    dump(E/'D3-CORE-CRT-REPORT-CORRECTION-FROZEN-20261108.json',freeze);dump(P/'D3-CORE-CRT-REPORT-CORRECTION-FROZEN-20261108.json',freeze)
    new(P/'.gitattributes',b'* -text\n');new(P/'README.md',b'# Core CRT report correction (UNREVIEWED)\n\nONE consolidated P2 ENV-CORE-CRT-OBSERVER-REPORT-COMPLETE-1. Exact schema/domain/family, load/seed/phase/ordinal and constructor/skin consistency gate; coherent early-reseed and inactive positives retained. Genuine old-gate RED then GREEN. Three final commands0/fourRust tests/23 single-fault reports rejected. d3 diagnostic expectation error retained separately.\n\nCompiled C/Rust observer/staging remains exact552ee; old forwarding controls reused, no replay. Full timed maps stay private immutable with hashes in the frozen receipt. Literal logs retain original CRLF/EOF blanks. One CURRENT/card and Git publication are post-POST metadata. No Windows/native/capture/removal/wholeD3 acceptance; STOP for SAME review.\n')
    print(json.dumps({'source_rows':post['file_count'],'unchanged_against_base':len(unchanged),'readers':len(READERS),'extras':len(extras()),'final_Rust':4,'single_faults':23,'hygiene':[]}));sys.exit()
assert mode=='publish'
for n in SOURCES+[DOC,'STATUS.md','docs/handover/ACTIVE-EXTRACTION.json']:git('add',n)
git('add','-A',P.relative_to(W).as_posix())
for p in P.rglob('*'):
    if p.is_file():assert git('cat-file','blob',':'+p.relative_to(W).as_posix())==p.read_bytes()
files=[{'path':p.relative_to(P).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()]
dump(P/'manifest.json',{'files':files,'literal_index_bytes_verified':True,'self_excluded':True,'source_docs_status_metadata_after_POST':True});git('add','-A',P.relative_to(W).as_posix());print(json.dumps({'manifest_entries':len(files),'packet_files':len(files)+1}))
