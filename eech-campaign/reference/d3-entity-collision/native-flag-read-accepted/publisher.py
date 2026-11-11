"""Metadata-only finite acceptance publication; no validator/test/native replay."""
import pathlib,json,hashlib,subprocess,datetime
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
PACK='eech-campaign/reference/d3-entity-collision/native-flag-read-accepted';P=W/PACK
HEAD='1ab12d0efc6e088b483ba30e0f12ec74c9ffa2ca'
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def new(p,b):
    p.parent.mkdir(parents=True,exist_ok=True)
    with p.open('xb') as f:f.write(b)
def dump(p,d):new(p,(json.dumps(d,indent=2)+'\n').encode())
assert git('rev-parse','HEAD').decode().strip()==HEAD and not git('status','--porcelain=v1').strip()
assert not P.exists()
review_path=E/'independent-native-flag-read-nr602-actual-20261108-v2.json';review=j(review_path)
assert sha(review_path.read_bytes())=='400394a42c1883c3c50e0a3e3cef552c5ac184e06e5b1f0daa1978f67efea3dd'
binding_path=E/'native-read-nr602-native-Root-binding-20261108.json';binding=j(binding_path)
assert sha(binding_path.read_bytes())=='b0d782efa4bdca51b48dd32fe11fa4a30e838234cf7c5ee880a8dd115442deb0'
identities={
 'eech-campaign/tools/native_flag_read_controller.py':'98f86d6defc96d6fd724c230974985f76aaa6e5a95dda2f5b925ae2d938b5ebd',
 'eech-campaign/tools/native_flag_read.py':'c28432cf6edeb2f4b602a8d977957ec18dd1ebdf669e13a9b6192b46bca7d11d',
 'eech-campaign/tools/native-flag-read-runtime.json':'3c27e9aaed8bfadb6a60c577d054790f74604fbab905515a4b80a24bf6af6084',
 'eech-campaign/crates/eech-world/lua/physical-flags-read-witness.lua':'3bfbeab0e336c2ffe9b8784c3e818c0cded329e9dc30562f939cb6fdfeaf9e7d',
 'eech-campaign/crates/eech-world/lua/physical-flags-read-transaction.lua':'7b1d3cc1b7a083061e224799d50da10605a2ebb737b08624f4dd2184d9386748',
}
for n,digest in identities.items():assert sha((W/n).read_bytes())==digest
P.mkdir();new(P/'.gitattributes',b'* -text\n')
copies=[]
def copy(path,target=None):
    raw=path.read_bytes();target=target or path.name;new(P/target,raw)
    copies.append({'source':str(path),'target':target,'bytes':len(raw),'sha256':sha(raw),'literal':True})
for name in [
 'audit-native-read-nr602-actual-Root-20261108.py',
 'native-read-04bf-Root-binding-20261108.json','native-read-1ab12-Root-publication-20261108.json',
 'codex-d3-native-read-correction-20261108-completion.json',
 'codex-d3-native-read-accepted-publication-20261108-process.json',
 'independent-native-flag-read-correction-04bf-source-controller-20261108.json',
 'independent-native-flag-read-nr602-actual-20261108.json','independent-native-flag-read-nr602-actual-20261108-v2.json',
 'native-read-nr602-materialized-plan-v2-20261108.json','native-read-nr602-staging-readonly-preflight-v2-20261108.json',
 'native-read-1ab12-all9-CI-readback-20261108.json','native-read-1ab12-Wine-excerpt-20261108.json',
 'native-read-nr602-original-outer-completion-20261108.json','native-read-nr602-controller-7256-identity-20261108.json',
 'npp1ab-family-exclusive-20261108.json',
]:copy(E/name)
base=pathlib.Path('D:/EECH-Migration-Evidence/native-flag-read-nr395-20261108/runs')
copy(base/'process.json','controller/process.json');copy(base/'completion.json','controller/completion.json')
safe_cases=[];control_selection=binding['cases'][0]['selection']
for r in binding['cases']:
    source=next(x['report'] for x in review['cases'] if x['mode']==r['mode'])
    raw=pathlib.Path(source['path']).read_bytes();assert len(raw)==source['bytes'] and sha(raw)==source['sha256']==r['report_sha256']
    selection=r['selection'];reask=r['reask'];fatal=r['fatal'];intervention=r['intervention']
    words=selection['words'].split(',');after=reask['words'].split(',') if reask else None
    safe={
     'schema':'native-flag-read-safe-witness-summary/v1','mode':r['mode'],'native_pid':r['native_pid'],'native_birth_utc':r['native_birth_utc'],'native_actual_exit':r['native_actual_exit'],'status':r['status'],
     'original_private_report':source,'counts':r['counts'],
     'selected_transaction':{'full_tuple_sha256':sha(json.dumps(selection,sort_keys=True).encode()),'same_complete_preintervention_tuple_as_healthy':selection==control_selection,'request':selection['request'],'frame':selection['frame'],'entry':'frame:16','array':selection['array'],'lane':selection['lane'],'original_result':selection['original'],'optional_group_opened':selection['group_opened'],'optional_group_acked':selection['group_acked'],'pre_request_sha256':sha(selection['words'].encode())},
     'store_traffic_oracle':{k:v for k,v in r['store'].items() if k!='key'},
     'reask':None if not reask else {'full_words_sha256':sha(reask['words'].encode()),'changed_fact_indices':[i for i,(a,b) in enumerate(zip(words,after)) if a!=b],'carried_fact':reask['fact']},
     'intervention':intervention,'fatal':None if not fatal else {k:v for k,v in fatal.items() if k!='line'},
     'pre_input_sha256':r['pre_input_sha256'],'post_input_sha256':r['post_input_sha256'],
     'projection':'Derived summary, not a literal witness file. Retail scene/name/id/registration, positions/full fact words and private error line omitted; raw SHA/pointer retained. Store is transcript-derived, not independent snapshot.'}
    dump(P/'reports'/(r['mode']+'.summary.json'),safe);safe_cases.append(safe)
private=[dict(x,retention='Immutable private raw; not copied into Git') for x in binding['retained_raw_files']]
for name in ['native-read-nr602-native-Root-binding-20261108.json','native-read-1ab12-Wine-job-128613769463-20261108.log']:
    path=E/name;raw=path.read_bytes();private.append({'path':str(path),'bytes':len(raw),'sha256':sha(raw),'retention':'Immutable private original; safe projection/excerpt published'})
dump(P/'private-evidence-index.json',{'raw_run_files':63,'full_input_observations':14,'files_per_observation':1179,'distinct_roots':7,'items':private,'note':'This publication does not re-run the Root audit, report validator or input tool. Hashes/pointers preserve original evidence; no retail asset, binary, full input manifest or decoded CI assembly log copied.'})
safe_binding={k:v for k,v in binding.items() if k not in ('cases','retained_raw_files')}
safe_binding.update(projection_schema='native-flag-read-safe-Root-binding/v1',original_private_binding={'path':str(binding_path),'sha256':sha(binding_path.read_bytes())},cases=safe_cases,retained_raw_index='private-evidence-index.json',projection='SAFE DERIVED PROJECTION: not the original b0d7 bytes; raw retail tuple/facts/private fatal lines omitted, original retained immutable.')
dump(P/'native-read-nr602-native-Root-binding-safe.json',safe_binding)
dump(P/'literal-copies.json',{'copies':copies,'projection_files':'reports/*.summary.json, native-read-nr602-native-Root-binding-safe.json and private-evidence-index.json are explicitly derived; no false literal-byte claim.'})
dump(P/'unchanged-source-artifacts.json',{'source_HEAD':HEAD,'artifacts':identities,'all_current_raw_bytes_equal_recorded_hashes':True,'metadata_only':True})
new(P/'README.md',b'''# Accepted finite public UPDATED READ

SAME actual review V2 `400394a42c1883c3c50e0a3e3cef552c5ac184e06e5b1f0daa1978f67efea3dd` has no findings. It accepts ONE observed public Lua -> native -> host UPDATED-array READ, its changed carried service fact, and five immediate matched refusals. Runtime remains true ordinary3abf; diagnostic source04bf/meta1ab12 and V2plan4ee21 retain their own identities. Source review4cda closes the two source P2s. Exact1ab12 all9 CI passed; retained Wine excerpt proves eight Lua controls plus actual converter unit, with external Python aggregate Unix-only. Old failed602f CI and all earlier partial17PASS/sixUNREACHED remain unchanged.

Original session27818/outer0/controller7256 actual0; all seven native exits and all21 stages0. Healthy completes FIRST; every case reaches the same full selected transaction at frame16 inside the unchanged35-frame Lebanon/100ms/seed1 window. Fourteen full1179-file input observations cover seven distinct fresh roots. The91 payloads and two Lua literals remain unchanged. Original controller/process/outer/CIM receipts, reviews, source Root bindings, CI summary, plan/staging proof and Root checker are literal copies. No checker, test, build, native process or experiment was rerun for publication.

Seven witness summaries and the Root native binding SAFE PROJECTION are derived from immutable raw originals. They omit retail scene/name/id/registration, positions/full fact words and private fatal lines. Original hashes/private paths remain bound; raw63 files, full input manifests, original b0d7 binding and decoded Wine log remain private. These summaries are not labelled literal witness reports. The Root checker is retained as provenance, not replayed.

Healthy real preword/read0 carries fact1; changing ONLY the returned answer to0x00000400 carries fact2 on the same candidate, other43 facts unchanged. The transcript-derived store word remains real0. Five single faults stop at entity_scan.callback, entity_scan.answer, entity_scan.state, flag.answer and flag.word in the same entry with no following callback or normal return credit. Caught-refusal native0 is diagnostic success, not healthy campaign continuation. No optional GROUP was opened. Zero initial word supplies NO new native nonzero/allbit/allarray/populated-neighbour destruction proof; the oracle is not an independent store snapshot.

This extra diagnostic READ is separate from unchanged defaults, collision outcome and natural moving-weapon integration. Generic NativeAcceptance1, supported needed provider/lifetime/FPU/DCS qualification, wholeheight/sharedterrain/RNG/hierarchy/services/finalcensus/CURRENT40all8/fourinputs/formalreview/normalmerge/all10/Task49/D3 remain OPEN as applicable. Both original fullpacks remain Root-owned and separate; prefix equality is partial only. Prep2 remains; no D4/D5/TypeScript.

Next bounded gate is source-first: determine the first still-required supported flag-boundary FPU/provider-lifetime/DCS fact from ACCEPTED wrappers/raw controls and now successful public boot/READ evidence. No repeat experiment, new epoch/replacement framework or broad unsupported guarantee is implied. That audit is not performed by this publication.
''')
copy(pathlib.Path(__file__),'publisher.py')
for n,digest in identities.items():assert sha((W/n).read_bytes())==digest
git('add','-A',PACK)
literal={}
for path in sorted(P.rglob('*')):
    if path.is_file():
        rel=path.relative_to(W).as_posix();raw=path.read_bytes();assert git('cat-file','blob',':'+rel)==raw;literal[path.relative_to(P).as_posix()]=sha(raw)
dump(P/'index-bytes-check.json',{'all_pre_manifest_index_blobs_literal':True,'files':literal,'five_source_artifacts_unchanged':True,'no_tests_build_native_cache_or_source_audit':True})
files=[{'path':p.relative_to(P).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()]
dump(P/'manifest.json',{'files':files,'scope':'Accepted finite public UPDATED READ/carry/five refusals; derived safe summaries labelled; no wholeflags/D3 acceptance'})
git('add','-A',PACK)
print(json.dumps({'manifest_entries':len(files),'tracked_packet_files':len(files)+1,'literal_external_copies':len(copies),'derived_witness_summaries':len(safe_cases),'private_raw_run_index':len(binding['retained_raw_files']),'source_artifacts_unchanged':len(identities)}))
