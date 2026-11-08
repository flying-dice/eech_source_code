"""Post-POST publication only; no source/check-reader/test mutation or replay."""
import datetime, hashlib, json, pathlib, subprocess
E=pathlib.Path('R:/d3-evidence')
W=E/'collision-characterization-20261029'
def git(*args):
    return subprocess.check_output(['git','-C',str(W),*args])
def sha(data): return hashlib.sha256(data).hexdigest()
def write_new(p,data):
    with p.open('xb') as f: f.write(data)
code=git('rev-parse','HEAD').decode().strip()
assert code=='fda05b65109ea19bc1d10f022555297670495b94'
pack='eech-campaign/reference/d3-entity-collision/physical-flags-motion-responses'
manifest=json.loads((W/pack/'manifest.json').read_bytes())
post=json.loads((E/'mfr332-inputmap-POST-20261107.json').read_bytes())
freeze=json.loads((E/'D3-MOTION-RESPONSE-FLAGS-FROZEN-20261108.json').read_bytes())
tree={}
for line in git('ls-tree','-r','-z',code).split(b'\0'):
    if line:
        meta,p=line.split(b'\t',1);tree[p.decode()]=meta.split()[2].decode()
packet_paths={pack+'/'+r['path']:r['sha256'] for r in manifest['files']}
assert set(p for p in tree if p.startswith(pack+'/'))==set(packet_paths)|{pack+'/manifest.json'}
source_paths={p:r['raw_sha256'] for p,r in post['files'].items()}
source_paths['eech-campaign/docs/d3-physical-flags-contract.md']=sha((W/'eech-campaign/docs/d3-physical-flags-contract.md').read_bytes())
expected={**source_paths,**packet_paths,pack+'/manifest.json':sha((W/pack/'manifest.json').read_bytes())}
ids=sorted({tree[p] for p in expected})
raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=('\n'.join(ids)+'\n').encode(),capture_output=True,check=True).stdout
actual={};offset=0
for oid in ids:
    end=raw.index(b'\n',offset);header=raw[offset:end].split();assert header[0].decode()==oid and header[1]==b'blob'
    n=int(header[2]);data=raw[end+1:end+1+n];actual[oid]=sha(data);offset=end+1+n+1
assert offset==len(raw)
mismatches=[p for p,s in expected.items() if actual[tree[p]]!=s]
inherited=[r['path'] for r in freeze['inherited_raw_versus_git_qualifications']]
assert sorted(mismatches)==sorted(inherited),mismatches
assert all(sha((W/p).read_bytes())==s for p,s in expected.items())
publication={
 'kind':'Post-POST actual committed Git blob byte publication join, no experiment replay',
 'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'code_commit':code,
 'packet_manifest_entries':len(packet_paths),'packet_tracked_files':len(packet_paths)+1,
 'packet_all_actual_committed_blob_bytes_literal':True,
 'source_input_rows':post['file_count'],'source_raw_bytes_equal_committed_blob_count':post['file_count']-len(inherited),
 'source_raw_byte_mismatches':inherited,'source_all_current_bytes_equal_frozen_POST':True,
 'contract_actual_committed_blob_bytes_equal':True,
 'source_fingerprint':post['fingerprint_of_raw_sha256_map'],
 'extra_identity_aggregate':json.loads((E/'mfr332-extra-identity-POST-20261107.json').read_bytes())['aggregate_sha256'],
 'final_direct_exits':{k:v['actual_exit'] for k,v in freeze['final_runs'].items()},
 'selected_counts':{k:v['selected_counts'] for k,v in freeze['final_runs'].items() if v['selected_counts']},
 'freeze_sha256':sha((E/'D3-MOTION-RESPONSE-FLAGS-FROZEN-20261108.json').read_bytes()),
 'qualifications':[
  'Three inherited uncompiled Lua working files are CRLF while their Git blobs are LF; their filtered identities match. Four separate inherited Core filter-identity disagreements retain raw bytes equal to Git. No normalization was performed.',
  'Plain staged whitespace check returned nonzero (outer PowerShell exit1; child numeric status not retained) on literal CRLF receipts and blank lines at EOF in raw logs. CR-at-EOL-only check returned nonzero (outer PowerShell exit1; child numeric status not retained) on log EOF blank lines. Source-only whitespace check and receipt-qualified cr-at-eol,-blank-at-eof check each exited0. Literal packet bytes were preserved; no test result changed.',
  'Native UNRUN / nothing ran natively in the packet means no Windows/retail public campaign execution. Compiled Linux C fixtures executed in the Rust tests, and the default Linux eech-dc module build ran (s10 actual0).',
  'Raw mb_hostmodel.c changed only the preprocessor gate of test-only helpers. This source is not the retained 3abf Windows binaries; no relabelling.',
  'Motion-hosted traffic6groups/10sets/4explicit inspection reads/0resets/11fresh bindings. No motion-consumer READ or reset case is claimed.',
  'This helper was created after POST solely for committed byte publication and tracked metadata; it was not a sealed final-test reader, changed no tested source/helper, and requests no replay.',
 ],
 'state':'UNREVIEWED; STOP for SAME independent source/evidence review. Generic NativeAcceptance1, all10, Task49 and D3 OPEN.',
 'publication_helper_sha256':sha(pathlib.Path(__file__).read_bytes()),
}
meta=W/'eech-campaign/reference/d3-entity-collision/physical-flags-motion-responses-publication'
meta.mkdir()
write_new(meta/'.gitattributes',b'* -text\n')
write_new(meta/'committed-bytes-and-checkpoint.json',(json.dumps(publication,indent=2)+'\n').encode())
write_new(meta/'publication-helper.py',pathlib.Path(__file__).read_bytes())
status=W/'STATUS.md';old=status.read_text(encoding='utf-8');at=old.index('\n# HISTORICAL')
new='''# CURRENT — D3 Task49: remaining motion-response flag group characterised and frozen; UNREVIEWED (8 November 2026)

**Candidate:** code/proof `fda05b65109ea19bc1d10f022555297670495b94`; [frozen packet](eech-campaign/reference/d3-entity-collision/physical-flags-motion-responses/README.md), [committed-byte publication](eech-campaign/reference/d3-entity-collision/physical-flags-motion-responses-publication/committed-bytes-and-checkpoint.json). Current source identifies SIX setters: TWO live guided-helicopter, TWO uncalled non-guided, TWO behind the per-entity logging bit; ship/weapon-vector name none of the three flag words. No motion response reads one. Supported-configuration source disposition only, not DCS/custom-host or natural-boot proof.

**Real join:** actual C guided begin/end -> real setter -> real host store, same cases in Hosted and Original. All TEN applicable final commands exit0 (fmt, three strict Clippy configurations, both motion tests, guard10, three existing flag controls in each profile, default Linux module build). Motion Hosted traffic6groups/10sets/4test-inspection reads/0resets/11fresh bindings; positive begin oneSET/zeroREAD. No reset case, broad reset proof, whole-global/FPU/lifetime or public/native READ acceptance. Failed developer compile101 and Claude OAuth failure1 retained; fallback explicitly announced, no authentication changes. Characterisation passes before any move; no manufactured product RED or behaviour correction.

**Identity:** REAL PRE/POST3997 inputs and extras/11readers equal; three inherited Lua CRLF/Git-LF qualifications preserved. Raw product `mb_hostmodel.c` changes the test-helper gate; retained3abf Windows binaries are not these bytes. No Windows/native/war replay. Tacview SAME7a4/873c and supply-descriptor SAME43fe closures retain their exact source scopes.

**STOP / remaining:** SAME independent changed-material review next, no approval/merge. Both immutable fullpacks remain Root-owned/running original jobs with prefix equality PARTIAL only; final current40/all8/fourinputs remain required. Generic NativeAcceptance1/publicREAD/changedanswer/otherneededconsumers/lifetimes/FPU/DCS, wholeheight/sharedterrain/RNG/hierarchy/services/naturalmoving/finalcensus/currentCI/formalreview/normalmerge/all10/Task49/D3 OPEN. Prep2 retained; no D4/D5/TypeScript. D3-CURRENT is Root's live job authority.
'''
status.write_text(new+old[:at].replace('# CURRENT', '# HISTORICAL (motion-response candidate superseded this summary)', 1)+old[at:],encoding='utf-8',newline='\n')
card=W/'docs/handover/ACTIVE-EXTRACTION.json';c=json.loads(card.read_bytes())
c['next_action']='STOP for SAME independent changed-material review of motion-response candidate fda05b651. Root owns appropriate runtime qualification, both fullpacks and remaining required whole-D3 gates; no replay of closed evidence.'
c['phase']='Motion-response flag group characterised and frozen at code/proof fda05b651; UNREVIEWED, STOP for SAME reviewer. Both fullpacks running. GenericNativeAcceptance1/all10/Task49/D3 OPEN.'
c['ready_evidence'].append('UNREVIEWED motion-response flag group fda05b651: six setters TWO live guided heli/TWO uncalled/TWO logging gated; ship/weapon-vector no words, no motion response reads. Real C guided response -> realsetter -> realstore same Hosted/Original cases. Final10direct0, guard10, positive Hosted6groups10sets4inspectionreads0resets11fresh. PRE/POST3997/extras11readers equal; three inheritedLua qualifications. Raw mb_hostmodel.c testgate changed; no 3abf binary relabel/native/Windows/war. Failed devcompile101 and Claude authfailure1 preserved; explicit fallback. Literal packet physical-flags-motion-responses; publication metadata beside it. No acceptance before SAME review.')
for b in c['blockers']:
    if b['id']=='ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1':
        b['evidence']+=' Motion-response finite candidate fda05b651 now frozen UNREVIEWED: real C guided setter/store join Hosted/Original and current six-way roles, final10direct0, stable3997maps. Not generic native acceptance.'
        b['missing_fact']='SAME review of finite motion-response candidate; generic public READ/changed fact, real callback/session/host replacement lifetime and whole-FPU facts, needed DCS disposition and complete current shipping40/all8/fourinputs remain required. Tacview supported-headless source disposition and finite supply correction retain their own closures.'
        b['next_fix']='STOP for SAME independent changed-material review of frozen motion-response candidate fda05b651. Root then appropriate runtime qualification/remaining required gates; do not replay closed characterization for handover.'
card.write_text(json.dumps(c,indent=2,ensure_ascii=False)+'\n',encoding='utf-8',newline='\n')
print(json.dumps({'code':code,'packet_entries':len(packet_paths),'packet_tracked':len(packet_paths)+1,'source_rows':post['file_count'],'raw_git_exact':post['file_count']-len(inherited),'qualifications':len(inherited),'metadata_written':str(meta)}))
