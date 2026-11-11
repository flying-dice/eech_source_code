"""Bounded three diagnostic-gate correction input seal/publication."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys
PRIVATE=['normal-fire-56d1b-terminal-Root-binding-20261108.json', 'normal-fire-56d1b-offline-event-domain-scan-20261108.json']
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/normal-query-selector'
BASE='56d1b7e31c8e4c4b45ed41f45c0e984ce7bbe6d8';RUNTIME='2fcf69f9b8972b4353687dc2ca1eaaac780836f3'
READERS=['run-normal-fire-nq56-direct-20261108.py','check-normal-fire-nq56-linux-20261108.sh','run-nq56-final-20261108.py','nq56-proof-20261108.py','sector-roster-inputmap-batched-20261106.py','nq56-scan-retained-20261108.py']
ROOTS=['independent-normal-fire-boot-retention-56d1b-source-20261108.json', 'independent-normal-fire-56d1b-materialized-launch-20261108.json', 'normal-fire-56d1b-terminal-Root-binding-20261108.json', 'normal-fire-first-native-426de-Root-binding-20261108.json', 'normal-fire-observer-2fcf69-build-Root-binding-20261108.json', 'normal-fire-56d1b-all9-CI-readback-20261108.json', 'normal-fire-56d1b-offline-event-domain-scan-20261108.json', 'nq56-retained-RED-20261108.json', 'nq56-retained-GREEN-20261108.json', 'nq56-domain-tool-observations-20261108.json', 'codex-d3-normal-query-selector-fix-20261108-process.json', 'D3-REQUIREMENTS.json']
OTHER=['nq56-family-20261108.json','nq56-expected-changes-20261108.json','nq56-readers-sealed-20261108.json','nq56-source-audit-20261108.json']
ARTIFACTS=['eech-campaign/crates/eech-world/lua/'+n for n in ['normal-fire-session.lua','normal-fire-correlation.lua','physical-flags-read-transaction.lua','normal-fire-session-contract.lua']]+['eech-campaign/tools/'+n for n in ['normal_fire_capture.py','normal_fire_capture_controller.py','test_normal_fire_capture.py','native_flag_read.py','native_flag_read_controller.py']]+['eech-campaign/docs/d3-normal-query-selector.md']
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
 old=j(E/'nb426-inputmap-POST-20261108.json');product={};unchanged={};changed=[]
 for n,r in old['files'].items():
  h=sha((W/n).read_bytes())
  if h==r['raw_sha256']:unchanged[n]=h
  else:
   assert n.startswith('eech-campaign/crates/eech-world/') and ('/lua/' in n or '/tests/' in n),n;changed.append(n)
  if '/tests/' not in n and '/lua/' not in n:
   assert h==r['raw_sha256'],n;product[n]=h
 # Extra producer configuration outside the source-map roots, including static
 # workspace/build inputs; all must equal the immutable compiled revision.
 extra={}
 for n in git('ls-tree','-r','--name-only',BASE,'--','eech-campaign/.cargo','tools','eech-campaign/tools','eech-campaign/rust-toolchain.toml').decode().splitlines():
  if n.startswith('eech-campaign/tools/'):continue # independent diagnostic tooling is separately bound above
  b=git('cat-file','blob',BASE+':'+n);assert (W/n).read_bytes()==b,n;extra[n]=sha(b)
 green=j(E/'nq56-retained-GREEN-20261108.json');red=j(E/'nq56-retained-RED-20261108.json');root=j(E/'normal-fire-56d1b-terminal-Root-binding-20261108.json')
 assert green['complete_EOF'] and green['result']=='PASS' and red['result']=='FAIL' and green['raw']==red['raw'] and green['raw']['sha256']==root['events_sha256']
 assert green['validator']['sha256']==sha((W/'eech-campaign/tools/normal_fire_capture.py').read_bytes())
 return dict(compiled_runtime=RUNTIME,prior_raw_map_sha256=sha((E/'nb426-inputmap-POST-20261108.json').read_bytes()),whole_relevant_product_build_Cargo_static_rows=len(product),product_map_sha256=sha(json.dumps(product,sort_keys=True).encode()),extra_producer_files=extra,unchanged_prior_raw_rows=len(unchanged),changed_prior_test_or_Lua_rows=changed,artifacts={n:sha((W/n).read_bytes()) for n in ARTIFACTS},scope='No C/Rust product/build/Cargo/static input change. Exact2fc runtime retains its source identity and97 unchanged payloads, including unsafe unexecuted old Lua. Four corrected literals/validator/controller are separate. Accepted prior C controls reused at their original scope; retained C bytes are codec replay only.')
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists();dump(E/'nq56-source-audit-20261108.json',audit());dump(E/'nq56-expected-changes-20261108.json',dirty());dump(E/'nq56-readers-sealed-20261108.json',readers());print('sealed',len(READERS));sys.exit()
if mode in ['PRE','POST']:
 assert audit()==j(E/'nq56-source-audit-20261108.json');assert readers()==j(E/'nq56-readers-sealed-20261108.json');assert dirty()==j(E/'nq56-expected-changes-20261108.json')
 x=extras();dump(E/('nq56-extra-'+mode+'-20261108.json'),x);out=E/('nq56-inputmap-'+mode+'-20261108.json');assert not out.exists();start=now()
 r=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/'sector-roster-inputmap-batched-20261106.py'),str(W),str(out),BASE],check=False)
 dump(E/('nq56-'+mode+'-utc-20261108.json'),dict(started_utc=start,completed_utc=now(),actual_exit=r.returncode,map_sha256=sha(out.read_bytes())))
 assert r.returncode==0 and x==extras()
 if mode=='POST':assert j(E/'nq56-inputmap-PRE-20261108.json')['files']==j(out)['files'] and x==j(E/'nq56-extra-PRE-20261108.json')
 print(mode,j(out)['file_count'],j(out)['fingerprint_of_raw_sha256_map'],'extras',len(x));sys.exit()
if mode=='freeze':
 pre=j(E/'nq56-inputmap-PRE-20261108.json');post=j(E/'nq56-inputmap-POST-20261108.json');assert pre['files']==post['files'];assert extras()==j(E/'nq56-extra-PRE-20261108.json')==j(E/'nq56-extra-POST-20261108.json')
 c=j(E/'nq56-final-completion-20261108.json');assert len(c['actual_exits'])==5 and set(c['actual_exits'].values())=={0} and c['actual_exit']==0
 text=(E/'normal-fire-nq56-s5-session-lua.stdout.log').read_text();assert 'normal_session_proof_gates=PASS' in text and 'real_flag_store_calls=16' in text and 'ACK_failure_precedes_UNREACHED=3' in text;assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',text)==[('ok','1','0')]
 assert 'signed_query_selector=PASS domains=3 invalid_domains=7 real_provider_contexts=3' in text
 assert 'Ran 10 tests' in (E/'normal-fire-nq56-s2-python.stderr.log').read_text()
 P.mkdir();(P/'readers').mkdir();(P/'runs').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['nq56-extra-PRE-20261108.json','nq56-extra-POST-20261108.json','nq56-PRE-utc-20261108.json','nq56-POST-utc-20261108.json','nq56-final-process-20261108.json','nq56-final-completion-20261108.json']:
  if n not in ['normal-fire-56d1b-terminal-Root-binding-20261108.json', 'normal-fire-56d1b-offline-event-domain-scan-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in list(c['actual_exits'])+['d1-Lua-selector-RED','d2-Lua-selector-GREEN','d3-Lua-selector-GREEN']:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-nq56-'+label+'.'+ext)).read_bytes())

 # Post-POST publication-only safety projection; tested source/readers remain sealed.
 for name in PRIVATE:
  source=E/name;data=j(source);redacted=[]
  if 'first_minus_one' in data:
   rawline=data['first_minus_one'].pop('line');fields=dict(x.split('=',1) for x in rawline.split()[1:])
   data['first_minus_one'].update(seq=int(fields['seq']),tick_ms=int(fields['tick']),selector=int(fields['which']));redacted.append('first_minus_one.line')
  if 'first_faults' in data:
   for i,fault in enumerate(data['first_faults']):
    if 'line' in fault:del fault['line'];redacted.append('first_faults['+str(i)+'].line')
  assert redacted and 'segment=' not in json.dumps(data)
  projection=dict(schema='safe_projection_not_literal_copy',private_original=dict(path=str(source),bytes=source.stat().st_size,sha256=sha(source.read_bytes())),removed_fields=redacted,projection=data)
  dump(P/(source.stem+'-safe-projection.json'),projection)
 put(P/'readers'/'nq56-safe-publish-20261108.py',Path(__file__).read_bytes())
 frozen=dict(utc=now(),post_POST_publication_filter=dict(private_omitted=PRIVATE,safe_projections=2,publisher=dict(path=str(Path(__file__)),sha256=sha(Path(__file__).read_bytes())),test_readers_unchanged=True),source_before=BASE,source_rows=post['file_count'],source_fingerprint=post['fingerprint_of_raw_sha256_map'],readers=readers(),extras_count=len(extras()),direct_exits=c['actual_exits'],Rust_tests=2,Python_tests=10,Lua_semantic_faults=25,before_output_reference_faults=27,before_effect_prefix_faults=4,actual_reference_FlagStore_calls=16,ACK_faults=3,ACK_failure_before_UNREACHED=3,synthetic_healthy_B_pairs=2,underreach_controls=2,old_diagnostic_RED_exit=101,selector_valid_domains=3,selector_invalid_domains=7,selector_real_provider_contexts=3,retained_full_scan=j(E/'nq56-retained-GREEN-20261108.json'),reuse=audit(),maps={n:dict(path=str(E/('nq56-inputmap-'+n+'-20261108.json')),sha256=sha((E/('nq56-inputmap-'+n+'-20261108.json')).read_bytes())) for n in ['PRE','POST']},scope='ONE exact signed selector correction UNREVIEWED; prior bootretention acceptedf0; runtime remains2fc. No native/build/root/cutover/all10 closure. Root records actual agent completion separately.')
 dump(E/'D3-NORMAL-QUERY-SELECTOR-FROZEN-20261108.json',frozen);dump(P/'D3-NORMAL-QUERY-SELECTOR-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Normal-query signed selector (UNREVIEWED)\n\nNamed query selector correction only; prior bootretention acceptedf0. See docs/d3-normal-query-selector.md. Runtime2fc and original97 payloads retain their own source identity; corrected literals/validator/controller are separate. No native run/cutover/all10 closure. Prior source/C controls retain original scope. Root owns actual agent completion.\n');print(json.dumps(frozen));sys.exit()
if mode=='stage':
 for n in j(E/'nq56-expected-changes-20261108.json'):git('add',n)
 git('add','-A',P.relative_to(W).as_posix());files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',dict(files=files,self_excluded=True));git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'nq56-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout;offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
 if p.is_file():assert git('cat-file','blob',code+':'+p.relative_to(W).as_posix())==p.read_bytes()
dump(P/'code-publication.json',dict(CODE=code,compiled_runtime=RUNTIME,source_rows=len(paths),raw_equals_Git=len(paths)-3,inherited_raw_filter_qualifications=qualified,source_fingerprint=post['fingerprint_of_raw_sha256_map'],post_POST_metadata=['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],scope='UNREVIEWED signed selector correction; no binary relabel/native/cutover/all10 closure; Root completion separate.'))
files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',dict(files=files,self_excluded=True,original_manifest_retained=True));git('add','-A',P.relative_to(W).as_posix());print(json.dumps(dict(CODE=code,publication_manifest_entries=len(files),packet_files=len(files)+1)))
