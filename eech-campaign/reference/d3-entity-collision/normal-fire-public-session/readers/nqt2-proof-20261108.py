"""Small source seal/map/publication reader. No build/native/recovery actions."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/normal-fire-public-session'
BASE='94fe4309eedddc634b37fa9294b93263866e6ea7';DOC='eech-campaign/docs/d3-normal-fire-public-session.md'
READERS=['run-normal-fire-nqt2-direct-20261108.py','check-normal-fire-nqt2-linux-20261108.sh','run-nqt2-final-20261108.py','nqt2-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-normal-fire-995ee-source-20261108.json','normal-fire-995ee-94fe-Root-binding-20261108.json','codex-d3-normal-fire-correlation-20261108-completion.json','normal-fire-94fe-all9-CI-readback-20261108.json','codex-d3-native-normal-fire-capture-20261108-process.json']
OTHER=['D3-REQUIREMENTS.json','nqt2-family-20261108.json','nqt2-readers-sealed-20261108.json','nqt2-expected-changes-20261108.json','nq94f-d9-synthetic-fixture-before-name-field.txt']
TOOLS=['eech-campaign/tools/'+n for n in ['normal_fire_capture.py','normal_fire_capture_controller.py','test_normal_fire_capture.py','native_flag_read_controller.py','native_flag_read.py','input-manifest.py']]
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def j(p):return json.loads(p.read_bytes())
def put(p,b):
 with p.open('xb') as f:f.write(b)
def dump(p,o):put(p,(json.dumps(o,indent=2)+'\n').encode())
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():return {**{n:sha((E/n).read_bytes()) for n in READERS+ROOTS+OTHER},**{n:sha((W/n).read_bytes()) for n in [DOC]+TOOLS}}
def dirty():return sorted(set(git('diff','--name-only','HEAD').decode().splitlines()+git('ls-files','--others','--exclude-standard').decode().splitlines()))
def audit():
 assert git('rev-parse','HEAD').decode().strip()==BASE
 result={}
 for n in ['aphavoc/source/entity/mobile/weapon/wn_move.c','aphavoc/source/entity/mobile/mb_hostmodel.c']:
  raw=(W/n).read_bytes();stripped=b'\n'.join(line for line in raw.split(b'\n') if b'mb_normalfirefx.h' not in line and not line.lstrip().startswith(b'NF_'))
  old=git('show',BASE+':'+n);assert stripped==old,n;result[n]=dict(raw_sha256=sha(raw),default_body_exact_94fe=True,old_sha256=sha(old))
 for n in ['aphavoc/source/entity/mobile/vehicle/vh_wpn.c','aphavoc/source/entity/mobile/weapon/weapon.c','eech-campaign/crates/eech-engine/src/lib.rs','eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c','eech-campaign/Cargo.lock','eech-campaign/crates/eech-engine-sys/sources.txt','eech-campaign/crates/eech-engine-sys/build/main.rs']:
  assert (W/n).read_bytes()==git('show',BASE+':'+n),n;result[n]=dict(raw_sha256=sha((W/n).read_bytes()),exact_94fe=True)
 return result
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists();dump(E/'nqt2-source-audit-20261108.json',audit());dump(E/'nqt2-expected-changes-20261108.json',dirty());dump(E/'nqt2-readers-sealed-20261108.json',readers());print('sealed',len(READERS),'readers');sys.exit()
if mode in ['PRE','POST']:
 assert readers()==j(E/'nqt2-readers-sealed-20261108.json');assert audit()==j(E/'nqt2-source-audit-20261108.json')
 assert dirty()==j(E/'nqt2-expected-changes-20261108.json')
 before=extras();dump(E/('nqt2-extra-'+mode+'-20261108.json'),before)
 out=E/('nqt2-inputmap-'+mode+'-20261108.json');assert not out.exists();start=now()
 run=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/READERS[-1]),str(W),str(out),BASE],check=False)
 dump(E/('nqt2-'+mode+'-utc-20261108.json'),dict(started_utc=start,completed_utc=now(),actual_exit=run.returncode,map_sha256=sha(out.read_bytes())))
 assert run.returncode==0 and extras()==before
 if mode=='POST':
  pre=j(E/'nqt2-inputmap-PRE-20261108.json');post=j(out);assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map'];assert before==j(E/'nqt2-extra-PRE-20261108.json')
 print(mode,j(out)['file_count'],j(out)['fingerprint_of_raw_sha256_map'],'extras',len(before));sys.exit()
if mode=='freeze':
 pre=j(E/'nqt2-inputmap-PRE-20261108.json');post=j(E/'nqt2-inputmap-POST-20261108.json');assert pre['files']==post['files'];assert extras()==j(E/'nqt2-extra-PRE-20261108.json')==j(E/'nqt2-extra-POST-20261108.json')
 complete=j(E/'nqt2-final-completion-20261108.json');assert len(complete['actual_exits'])==9 and set(complete['actual_exits'].values())=={0} and complete['actual_exit']==0
 tests=[]
 for label in ['s4-normal-controls','s5-hosted-join','s6-session-lua','s7-profile-census']:
  text=(E/('normal-fire-nqt2-'+label+'.stdout.log')).read_text();tests+=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',text)
 assert tests==[('ok','1','0')]*4,tests
 normal=(E/'normal-fire-nqt2-s4-normal-controls.stdout.log').read_text();assert 'observer_checks=4' in normal and 'delivered_context_negatives=8200 failures=0' in normal and 'portable_lua_controls=10' in normal
 traffic=json.loads(re.search(r'actual D2 requests=(\{[^\n]+\})',normal)[1]);assert all(traffic.get(k,0)>0 for k in ['birth','flight','ready','vectors','release'])
 hosted=(E/'normal-fire-nqt2-s5-hosted-join.stdout.log').read_text();assert 'hosted_query_positive=1 callback_identity_faults=2 observer_error=1 empty_control=1 live_model_calls=3041' in hosted
 assert 'normal_session_portable_controls=13' in (E/'normal-fire-nqt2-s6-session-lua.stdout.log').read_text()
 assert re.search(r'Ran 5 tests', (E/'normal-fire-nqt2-s8-python.stderr.log').read_text())
 old=j(E/'nq94f-inputmap-PRE-20261108.json');joined={}
 for n,r in old['files'].items():
  if n.startswith(('aphavoc/source/','modules/','eech-campaign/crates/eech-engine-sys/','eech-campaign/crates/eech-world/src/','eech-campaign/crates/eech-world/csrc/','eech-campaign/crates/eech-world/build/','eech-campaign/crates/eech-world/compat/')) or n in ['eech-campaign/Cargo.toml','eech-campaign/Cargo.lock','eech-campaign/crates/eech-world/Cargo.toml','eech-campaign/crates/eech-world/build.rs','eech-campaign/rust-toolchain.toml']:
   assert post['files'][n]['raw_sha256']==r['raw_sha256'],n;joined[n]=r['raw_sha256']
 assert j(E/'normal-fire-nq94f-s4-normal-controls.json')['actual_exit']==0
 reuse=dict(scope='ONLY original sys normal_fire C test reused from old two-test group; old engine test superseded by fresh current engine control.',source_rows_exact=len(joined),input_map_sha256=sha(json.dumps(joined,sort_keys=True).encode()),receipt='normal-fire-nq94f-s4-normal-controls.json',receipt_sha256=sha((E/'normal-fire-nq94f-s4-normal-controls.json').read_bytes()),actual_exit=0)
 P.mkdir();(P/'readers').mkdir();(P/'runs').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['nqt2-source-audit-20261108.json','nqt2-extra-PRE-20261108.json','nqt2-extra-POST-20261108.json','nqt2-PRE-utc-20261108.json','nqt2-POST-utc-20261108.json','nqt2-final-process-20261108.json','nqt2-final-completion-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in complete['actual_exits']:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-nqt2-'+label+'.'+ext)).read_bytes())
 development=[]
 for p in sorted(E.glob('normal-fire-nqt2-d*.json')):
  r=j(p);logs=[q for q in [p.with_suffix('.stdout.log'),p.with_suffix('.stderr.log')] if q.exists()];development.append(dict(path=str(p),sha256=sha(p.read_bytes()),actual_exit=r.get('actual_exit'),logs=[dict(path=str(q),sha256=sha(q.read_bytes()),bytes=q.stat().st_size) for q in logs],scope='unsealed development; setup errors not product RED; each passing fact retains its actual source scope'))
 dump(P/'development-index.json',development)
 dump(P/'unchanged-sys-C-control-reuse.json',reuse)
 for n in ['nq94f-final-completion-20261108.json','nq94f-readers-sealed-20261108.json','nq94f-PRE-utc-20261108.json','nq94f-POST-utc-20261108.json','normal-fire-nq94f-s7-profile-census.json','normal-fire-nq94f-s7-profile-census.stdout.log','normal-fire-nq94f-s7-profile-census.stderr.log','normal-fire-nq94f-s4-normal-controls.json','normal-fire-nq94f-s4-normal-controls.stdout.log','normal-fire-nq94f-s4-normal-controls.stderr.log']:put(P/n,(E/n).read_bytes())
 prior=[]
 for p in sorted(E.glob('normal-fire-nq94f-d*.json')):
  r=j(p);prior.append(dict(path=str(p),sha256=sha(p.read_bytes()),actual_exit=r.get('actual_exit'),logs=[dict(path=str(q),sha256=sha(q.read_bytes()),bytes=q.stat().st_size) for q in [p.with_suffix('.stdout.log'),p.with_suffix('.stderr.log')] if q.exists()]))
 dump(P/'prior-development-index.json',prior)
 frozen=dict(utc=now(),source_before=BASE,source_rows=post['file_count'],source_fingerprint=post['fingerprint_of_raw_sha256_map'],maps={n:dict(path=str(E/('nqt2-inputmap-'+n+'-20261108.json')),sha256=sha((E/('nqt2-inputmap-'+n+'-20261108.json')).read_bytes())) for n in ['PRE','POST']},readers=readers(),extras_count=len(extras()),direct_exits=complete['actual_exits'],Rust_tests=4,reused_sys_C_test=1,Python_tests=5,portable_Lua_controls=dict(normal_codec=10,hosted_join=5,synthetic_session_transport=13),own_hook_FPU_modes=4,delivered_invalid_context_control_records=8200,actual_D2_callback_requests=traffic,hosted_callback_interval=[3030,3041],hosted_total_provider_entries=3041,hygiene=[],scope='UNREVIEWED source/private controls + concrete fixed-bound public driver/controller. Original real normal birth/movement distinct from hosted manually registered query (zero normal-birth credit), synthetic session/codec distinct from both. No Windows/native/root creation/cutover/removal/all10 closure. Root owns separate actual agent completion.')
 dump(E/'D3-NORMAL-FIRE-PUBLIC-SESSION-FROZEN-20261108.json',frozen);dump(P/'D3-NORMAL-FIRE-PUBLIC-SESSION-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Normal-fire public-session source/controller candidate\n\nUNREVIEWED. Literal receipts/readers and selected accepted context; full source maps/development logs private and hash-bound. Real Original normal birth/movement, manually registered hosted query (ZERO normal-birth credit), and explicitly synthetic Lua/codec transport controls stay distinct. No native launch or new Windows build occurred. Root must build/materialize this NEW frozen observer and obtain SAME launch clearance. No cutover/removal/all10 closure. See d3-normal-fire-public-session.md.\n');print(json.dumps(frozen));sys.exit()
if mode=='stage':
 for n in j(E/'nqt2-expected-changes-20261108.json'):git('add',n)
 git('add','-A',P.relative_to(W).as_posix())
 files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',dict(files=files,self_excluded=True));git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'nqt2-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout
offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
 if p.is_file():assert git('cat-file','blob',code+':'+p.relative_to(W).as_posix())==p.read_bytes()
dump(P/'code-publication.json',dict(CODE=code,source_rows=len(paths),raw_equals_Git=len(paths)-len(qualified),inherited_raw_filter_qualifications=qualified,source_fingerprint=post['fingerprint_of_raw_sha256_map'],post_POST_metadata=['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],raw_log_qualification='Literal CRLF/EOF retained; not normalized to satisfy source whitespace check.',scope='UNREVIEWED source/controller; Root agent completion separate; no native/cutover/all10 closure'))
files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',dict(files=files,self_excluded=True,original_manifest_retained=True));git('add','-A',P.relative_to(W).as_posix());print(json.dumps(dict(CODE=code,publication_manifest_entries=len(files),packet_files=len(files)+1)))
