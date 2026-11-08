"""Bounded three diagnostic-gate correction input seal/publication."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/public-session-candidate-contract'
BASE='b50203d48cb46be1dbe52e062b0516d0afa587e4';RUNTIME='2fcf69f9b8972b4353687dc2ca1eaaac780836f3'
READERS=['run-normal-fire-pfb502-direct-20261108.py','check-normal-fire-pfb502-linux-20261108.sh','run-pfb502-final-20261108.py','pfb502-proof-20261108.py','sector-roster-inputmap-batched-20261106.py','pfb502-fixed-query-analysis-20261108.py']
ROOTS=['normal-query-b502-terminal-Root-binding-20261108.json', 'scanned-candidate-lead-decision-live-services-v210-20261108.json', 'independent-normal-query-selector-b502-source-20261108.json', 'independent-normal-query-b502-materialized-launch-20261108.json', 'normal-query-b502-all9-CI-readback-20261108.json', 'normal-fire-observer-2fcf69-build-Root-binding-20261108.json', 'pfb502-fixed-query-analysis-20261108.json', 'codex-d3-public-session-finalization-fix-20261108-process.json', 'D3-REQUIREMENTS.json']
OTHER=['pfb502-family-20261108.json','pfb502-expected-changes-20261108.json','pfb502-readers-sealed-20261108.json','pfb502-source-audit-20261108.json']
ARTIFACTS=['eech-campaign/crates/eech-world/lua/'+n for n in ['normal-fire-session.lua','normal-fire-correlation.lua','physical-flags-read-transaction.lua','normal-fire-session-contract.lua']]+['eech-campaign/tools/'+n for n in ['normal_fire_capture.py','normal_fire_capture_controller.py','test_normal_fire_capture.py','native_flag_read.py','native_flag_read_controller.py']]+['eech-campaign/docs/d3-public-session-finalization.md']
sha=lambda b:hashlib.sha256(b).hexdigest();now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def j(p):return json.loads(p.read_bytes())
def put(p,b):
 with p.open('xb') as f:f.write(b)
def dump(p,o):put(p,(json.dumps(o,indent=2)+'\n').encode())
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():return {**{n:sha((E/n).read_bytes()) for n in READERS+ROOTS+OTHER},**{n:sha((W/n).read_bytes()) for n in ARTIFACTS}}
def dirty():return sorted(set(git('diff','--name-only','HEAD').decode().splitlines()+git('ls-files','--others','--exclude-standard').decode().splitlines()))
def audit():
 assert git('rev-parse','HEAD').decode().strip()==BASE
 old=j(E/'nq56-inputmap-POST-20261108.json');product={};unchanged={};changed=[];test_only='eech-campaign/crates/eech-dc/src/native_flag_read_refusal_check.rs'
 for n,r in old['files'].items():
  h=sha((W/n).read_bytes())
  if h==r['raw_sha256']:unchanged[n]=h
  else:
   assert n==test_only or (n.startswith('eech-campaign/crates/eech-world/') and ('/lua/' in n or '/tests/' in n)),n;changed.append(n)
  if '/tests/' not in n and '/lua/' not in n and n!=test_only:
   assert h==r['raw_sha256'],n;product[n]=h
 # Extra producer configuration outside the source-map roots, including static
 # workspace/build inputs; all must equal the immutable compiled revision.
 extra={}
 for n in git('ls-tree','-r','--name-only',BASE,'--','eech-campaign/.cargo','tools','eech-campaign/tools','eech-campaign/rust-toolchain.toml').decode().splitlines():
  if n.startswith('eech-campaign/tools/'):continue # independent diagnostic tooling is separately bound above
  b=git('cat-file','blob',BASE+':'+n);assert (W/n).read_bytes()==b,n;extra[n]=sha(b)
 module='eech-campaign/crates/eech-dc/src/module.rs';importer=(W/module).read_bytes();assert sha(importer)==old['files'][module]['raw_sha256']
 assert b'#[cfg(test)]\n#[path = \"native_flag_read_refusal_check.rs\"]\nmod native_flag_read_refusal_check;' .replace(b'\"',b'"') in importer
 current=(W/test_only).read_bytes();before=git('cat-file','blob',BASE+':'+test_only);start=current.index(b'// Actual mlua registration');end=current.index(b'#[test]\nfn nr602_',start);assert current[:start]+current[end:]==before
 exception=dict(path=test_only,before_sha256=sha(before),current_sha256=sha(current),cfg_test_importer=module,importer_raw_sha256=sha(importer),production_excluded=True,old_test_body_literal_unchanged=True)
 analysis=j(E/'pfb502-fixed-query-analysis-20261108.json');root=j(E/'normal-query-b502-terminal-Root-binding-20261108.json');assert analysis['complete_EOF'] and analysis['raw']['sha256']==root['events_sha256'] and analysis['fixed_queries']==26
 return dict(compiled_runtime=RUNTIME,prior_raw_map_sha256=sha((E/'nq56-inputmap-POST-20261108.json').read_bytes()),whole_relevant_product_build_Cargo_static_rows=len(product),cfg_test_raw_exception=exception,product_map_sha256=sha(json.dumps(product,sort_keys=True).encode()),extra_producer_files=extra,unchanged_prior_raw_rows=len(unchanged),changed_prior_test_or_Lua_rows=changed,artifacts={n:sha((W/n).read_bytes()) for n in ARTIFACTS},scope='All3815 product/build/Cargo/static rows unchanged; one existing cfg(test)-excluded raw source exception is explicit. Exact2fc runtime retains its source identity and97 unchanged payloads, including unsafe unexecuted old Lua. Four corrected literals/validator/controller are separate. Accepted prior C controls reused at their original scope; retained C bytes are codec replay only.')
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists();dump(E/'pfb502-source-audit-20261108.json',audit());dump(E/'pfb502-expected-changes-20261108.json',dirty());dump(E/'pfb502-readers-sealed-20261108.json',readers());print('sealed',len(READERS));sys.exit()
if mode in ['PRE','POST']:
 assert audit()==j(E/'pfb502-source-audit-20261108.json');assert readers()==j(E/'pfb502-readers-sealed-20261108.json');assert dirty()==j(E/'pfb502-expected-changes-20261108.json')
 x=extras();dump(E/('pfb502-extra-'+mode+'-20261108.json'),x);out=E/('pfb502-inputmap-'+mode+'-20261108.json');assert not out.exists();start=now()
 r=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/'sector-roster-inputmap-batched-20261106.py'),str(W),str(out),BASE],check=False)
 dump(E/('pfb502-'+mode+'-utc-20261108.json'),dict(started_utc=start,completed_utc=now(),actual_exit=r.returncode,map_sha256=sha(out.read_bytes())))
 assert r.returncode==0 and x==extras()
 if mode=='POST':assert j(E/'pfb502-inputmap-PRE-20261108.json')['files']==j(out)['files'] and x==j(E/'pfb502-extra-PRE-20261108.json')
 print(mode,j(out)['file_count'],j(out)['fingerprint_of_raw_sha256_map'],'extras',len(x));sys.exit()
if mode=='freeze':
 pre=j(E/'pfb502-inputmap-PRE-20261108.json');post=j(E/'pfb502-inputmap-POST-20261108.json');assert pre['files']==post['files'];assert extras()==j(E/'pfb502-extra-PRE-20261108.json')==j(E/'pfb502-extra-POST-20261108.json')
 c=j(E/'pfb502-final-completion-20261108.json');assert len(c['actual_exits'])==6 and set(c['actual_exits'].values())=={0} and c['actual_exit']==0
 text=(E/'normal-fire-pfb502-s6-session-lua.stdout.log').read_text();assert 'normal_session_proof_gates=PASS' in text and 'real_flag_store_calls=16' in text and 'ACK_failure_precedes_UNREACHED=3' in text;assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',text)==[('ok','1','0')]
 assert 'public_session_contract=PASS' in text and 'existing_move_causality=PASS' in text
 facade=(E/'normal-fire-pfb502-s5-actual-facade.stdout.log').read_text();assert 'test result: ok. 1 passed; 0 failed;' in facade
 assert 'Ran 12 tests' in (E/'normal-fire-pfb502-s2-python.stderr.log').read_text()
 P.mkdir();(P/'readers').mkdir();(P/'runs').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['pfb502-extra-PRE-20261108.json','pfb502-extra-POST-20261108.json','pfb502-PRE-utc-20261108.json','pfb502-POST-utc-20261108.json','pfb502-final-process-20261108.json','pfb502-final-completion-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in list(c['actual_exits'])+['d1-actual-facade-RED','d2-actual-facade-GREEN','d3-scanned-candidate-RED','d5-batched-contract-controls','d6-batched-contract-controls','d9-facade-tags-and-lint','d10-temporal-contract-controls','d11-temporal-contract-controls']:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-pfb502-'+label+'.'+ext)).read_bytes())

 frozen=dict(utc=now(),source_before=BASE,source_rows=post['file_count'],source_fingerprint=post['fingerprint_of_raw_sha256_map'],readers=readers(),extras_count=len(extras()),direct_exits=c['actual_exits'],Rust_tests=3,Python_tests=12,Lua_semantic_faults=25,before_output_reference_faults=27,before_effect_prefix_faults=4,actual_reference_FlagStore_calls=16,ACK_faults=3,ACK_failure_before_UNREACHED=3,synthetic_healthy_B_pairs=2,underreach_controls=2,old_diagnostic_RED_exit=101,actual_userdata_methods=3,real_serde_mode_tags=6,distinct_candidate_real_FlagStore_healthy_B=2,candidate_registration_flag_pose_faults=4,temporal_healthy_B=4,temporal_wrong_late_identity_B=2,late_only_UNREACHED=1,numeric_outcomes=21,report_schema=2,fixed_raw_query_analysis=j(E/'pfb502-fixed-query-analysis-20261108.json'),reuse=audit(),maps={n:dict(path=str(E/('pfb502-inputmap-'+n+'-20261108.json')),sha256=sha((E/('pfb502-inputmap-'+n+'-20261108.json')).read_bytes())) for n in ['PRE','POST']},scope='ONE public facade/scanned-candidate contract correction underlead210 UNREVIEWED; oldFAIL1500 remainsFAIL; runtime remains2fc. No native/build/root/cutover/all10 closure. Root records actual agent completion separately.')
 dump(E/'D3-PUBLIC-SESSION-CANDIDATE-CONTRACT-FROZEN-20261108.json',frozen);dump(P/'D3-PUBLIC-SESSION-CANDIDATE-CONTRACT-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Public facade/scanned-candidate contract (UNREVIEWED)\n\nLead210 source-aligned finite scanned-candidate and actual public facade correction. Old stronger witness/failed native scopes retained; no oldFAIL reinterpretation. See docs/d3-public-session-finalization.md. Runtime2fc and original97 payloads retain their own source identity; corrected literals/validator/controller are separate. No native run/cutover/all10 closure. Prior source/C controls retain original scope. Root owns actual agent completion.\n');print(json.dumps(frozen));sys.exit()
if mode=='stage':
 for n in j(E/'pfb502-expected-changes-20261108.json'):git('add',n)
 git('add','-A',P.relative_to(W).as_posix());files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',dict(files=files,self_excluded=True));git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'pfb502-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout;offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
 if p.is_file():assert git('cat-file','blob',code+':'+p.relative_to(W).as_posix())==p.read_bytes()
dump(P/'code-publication.json',dict(CODE=code,compiled_runtime=RUNTIME,source_rows=len(paths),raw_equals_Git=len(paths)-3,inherited_raw_filter_qualifications=qualified,source_fingerprint=post['fingerprint_of_raw_sha256_map'],post_POST_metadata=['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],scope='UNREVIEWED facade/scanned-candidate contract; no binary relabel/native/cutover/all10 closure; Root completion separate.'))
files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',dict(files=files,self_excluded=True,original_manifest_retained=True));git('add','-A',P.relative_to(W).as_posix());print(json.dumps(dict(CODE=code,publication_manifest_entries=len(files),packet_files=len(files)+1)))
