"""Bounded three diagnostic-gate correction input seal/publication."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/normal-fire-proof-gates-literal'
BASE='e8cb7944e75af0ea3fbcbb4d4414015af3eb140c';RUNTIME='2fcf69f9b8972b4353687dc2ca1eaaac780836f3'
READERS=['run-normal-fire-nf3lf-direct-20261108.py','check-normal-fire-nf3lf-linux-20261108.sh','run-nf3lf-final-20261108.py','nf3lf-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-normal-fire-f88d6-combined-source-20261108.json','normal-fire-f88d6-2fcf69-Root-binding-20261108.json','codex-d3-native-normal-fire-capture-20261108-completion.json','codex-d3-normal-fire-proof-gate-fixes-20261108-process.json','normal-fire-observer-2fcf69-build-Root-binding-20261108.json','normal-fire-2fcf69-all9-CI-readback-20261108.json','D3-REQUIREMENTS.json']
OTHER=['nf3lf-family-20261108.json','nf3lf-expected-changes-20261108.json','nf3lf-readers-sealed-20261108.json','nf3lf-source-audit-20261108.json']
ARTIFACTS=['eech-campaign/crates/eech-world/lua/'+n for n in ['normal-fire-session.lua','normal-fire-correlation.lua','physical-flags-read-transaction.lua','normal-fire-session-contract.lua']]+['eech-campaign/tools/'+n for n in ['normal_fire_capture.py','normal_fire_capture_controller.py','test_normal_fire_capture.py','native_flag_read.py','native_flag_read_controller.py']]+['eech-campaign/docs/d3-normal-fire-proof-gates.md']
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
 old=j(E/'nf3p2-inputmap-POST-20261108.json');product={};unchanged={};changed=[]
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
 return dict(compiled_runtime=RUNTIME,prior_raw_map_sha256=sha((E/'nf3p2-inputmap-POST-20261108.json').read_bytes()),whole_relevant_product_build_Cargo_static_rows=len(product),product_map_sha256=sha(json.dumps(product,sort_keys=True).encode()),extra_producer_files=extra,unchanged_prior_raw_rows=len(unchanged),changed_prior_test_or_Lua_rows=changed,artifacts={n:sha((W/n).read_bytes()) for n in ARTIFACTS},scope='No C/Rust product/build/Cargo/static input change. Exact2fc runtime retains its source identity and97 unchanged payloads, including unsafe unexecuted old Lua. Four corrected literals/validator/controller are separate. Accepted prior C controls reused at their original scope; retained C bytes are codec replay only.')
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists();dump(E/'nf3lf-source-audit-20261108.json',audit());dump(E/'nf3lf-expected-changes-20261108.json',dirty());dump(E/'nf3lf-readers-sealed-20261108.json',readers());print('sealed',len(READERS));sys.exit()
if mode in ['PRE','POST']:
 assert audit()==j(E/'nf3lf-source-audit-20261108.json');assert readers()==j(E/'nf3lf-readers-sealed-20261108.json');assert dirty()==j(E/'nf3lf-expected-changes-20261108.json')
 x=extras();dump(E/('nf3lf-extra-'+mode+'-20261108.json'),x);out=E/('nf3lf-inputmap-'+mode+'-20261108.json');assert not out.exists();start=now()
 r=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/READERS[-1]),str(W),str(out),BASE],check=False)
 dump(E/('nf3lf-'+mode+'-utc-20261108.json'),dict(started_utc=start,completed_utc=now(),actual_exit=r.returncode,map_sha256=sha(out.read_bytes())))
 assert r.returncode==0 and x==extras()
 if mode=='POST':assert j(E/'nf3lf-inputmap-PRE-20261108.json')['files']==j(out)['files'] and x==j(E/'nf3lf-extra-PRE-20261108.json')
 print(mode,j(out)['file_count'],j(out)['fingerprint_of_raw_sha256_map'],'extras',len(x));sys.exit()
if mode=='freeze':
 pre=j(E/'nf3lf-inputmap-PRE-20261108.json');post=j(E/'nf3lf-inputmap-POST-20261108.json');assert pre['files']==post['files'];assert extras()==j(E/'nf3lf-extra-PRE-20261108.json')==j(E/'nf3lf-extra-POST-20261108.json')
 c=j(E/'nf3lf-final-completion-20261108.json');assert len(c['actual_exits'])==5 and set(c['actual_exits'].values())=={0} and c['actual_exit']==0
 text=(E/'normal-fire-nf3lf-s3-session-lua.stdout.log').read_text();assert 'normal_session_proof_gates=PASS' in text and 'real_flag_store_calls=16' in text and 'ACK_failure_precedes_UNREACHED=3' in text;assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',text)==[('ok','1','0')]
 assert 'Ran 8 tests' in (E/'normal-fire-nf3lf-s5-python.stderr.log').read_text()
 P.mkdir();(P/'readers').mkdir();(P/'runs').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['nf3lf-extra-PRE-20261108.json','nf3lf-extra-POST-20261108.json','nf3lf-PRE-utc-20261108.json','nf3lf-POST-utc-20261108.json','nf3lf-final-process-20261108.json','nf3lf-final-completion-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in list(c['actual_exits']):
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-nf3lf-'+label+'.'+ext)).read_bytes())
 for n in ['nf3p2-publication-join-failure-20261108.json','D3-NORMAL-FIRE-PROOF-GATES-FROZEN-20261108.json'] :put(P/n,(E/n).read_bytes())
 frozen=dict(utc=now(),source_before=BASE,source_rows=post['file_count'],source_fingerprint=post['fingerprint_of_raw_sha256_map'],readers=readers(),extras_count=len(extras()),direct_exits=c['actual_exits'],Rust_tests=2,Python_tests=8,Lua_semantic_faults=25,before_output_reference_faults=27,before_effect_prefix_faults=4,actual_reference_FlagStore_calls=16,ACK_faults=3,ACK_failure_before_UNREACHED=3,synthetic_healthy_B_pairs=2,underreach_controls=2,old_diagnostic_RED_exit=1,prior_raw_pass_freeze='D3-NORMAL-FIRE-PROOF-GATES-FROZEN-20261108.json',reuse=audit(),maps={n:dict(path=str(E/('nf3lf-inputmap-'+n+'-20261108.json')),sha256=sha((E/('nf3lf-inputmap-'+n+'-20261108.json')).read_bytes())) for n in ['PRE','POST']},scope='THREE P2 source corrections UNREVIEWED; runtime remains2fc. No native/build/root/cutover/all10 closure. Root records actual agent completion separately.')
 dump(E/'D3-NORMAL-FIRE-PROOF-GATES-LITERAL-FROZEN-20261108.json',frozen);dump(P/'D3-NORMAL-FIRE-PROOF-GATES-LITERAL-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Normal-fire diagnostic proof gates (UNREVIEWED)\n\nTHREE P2 corrections only. See docs/d3-normal-fire-proof-gates.md. Runtime2fc and original97 payloads retain their own source identity; corrected literals/validator/controller are separate. No native run/cutover/all10 closure. Prior source/C controls retain original scope. Root owns actual agent completion.\n');print(json.dumps(frozen));sys.exit()
if mode=='stage':
 for n in j(E/'nf3lf-expected-changes-20261108.json'):git('add',n)
 git('add','-A',P.relative_to(W).as_posix());files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',dict(files=files,self_excluded=True));git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'nf3lf-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout;offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
 if p.is_file():assert git('cat-file','blob',code+':'+p.relative_to(W).as_posix())==p.read_bytes()
dump(P/'code-publication.json',dict(CODE=code,compiled_runtime=RUNTIME,source_rows=len(paths),raw_equals_Git=len(paths)-3,inherited_raw_filter_qualifications=qualified,source_fingerprint=post['fingerprint_of_raw_sha256_map'],post_POST_metadata=['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],scope='UNREVIEWED three P2 corrections; no binary relabel/native/cutover/all10 closure; Root completion separate.'))
files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',dict(files=files,self_excluded=True,original_manifest_retained=True));git('add','-A',P.relative_to(W).as_posix());print(json.dumps(dict(CODE=code,publication_manifest_entries=len(files),packet_files=len(files)+1)))
