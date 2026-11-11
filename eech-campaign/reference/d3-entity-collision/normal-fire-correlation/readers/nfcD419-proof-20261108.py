"""Bounded normal-fire source/check publication. No build/native/recovery action."""
from pathlib import Path
import ast,datetime,hashlib,json,re,subprocess,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/normal-fire-correlation'
BASE='d419afb575adac32b1d6fd1b2fe2706ffb5efcf1'
READERS=['run-normal-fire-nfcD419-direct-20261108.py','check-normal-fire-nfcD419-linux-20261108.sh','run-nfcD419-final-20261108.py','nfcD419-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['independent-natural-moving-d419-plan-20261108.json','natural-moving-plan-d419-Root-binding-20261108.json','codex-d3-natural-moving-public-path-20261108-completion.json','codex-d3-normal-fire-correlation-20261108-process.json','natural-moving-d419-all9-CI-readback-20261108.json']
OTHER=['nfcD419-family-20261108.json','nfcD419-runner-before-seal-draft1.py','nfcD419-original-d419-plan.literal.md','nfcD419-d21-RED-normal-fire-correlation.lua','nfcD419-d21-RED-normal_fire.rs','D3-REQUIREMENTS.json']
DOC='eech-campaign/docs/d3-normal-fire-correlation.md'
SOURCES=['aphavoc/source/entity/mobile/vehicle/vh_wpn.c','aphavoc/source/entity/mobile/weapon/weapon.c','aphavoc/source/entity/mobile/mb_normalfirefx.h','eech-campaign/Cargo.lock','eech-campaign/crates/eech-dc/Cargo.toml','eech-campaign/crates/eech-engine-sys/Cargo.toml','eech-campaign/crates/eech-engine-sys/build/main.rs','eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c','eech-campaign/crates/eech-engine-sys/csrc/eech_normal_fire_check.c','eech-campaign/crates/eech-engine-sys/csrc/eech_normal_fire_d2_check.c','eech-campaign/crates/eech-engine-sys/tests/normal_fire.rs','eech-campaign/crates/eech-engine/Cargo.toml','eech-campaign/crates/eech-engine/src/lib.rs','eech-campaign/crates/eech-engine/tests/normal_fire.rs','eech-campaign/crates/eech-world/lua/normal-fire-correlation.lua']
def sha(b):return hashlib.sha256(b).hexdigest()
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def j(p):return json.loads(p.read_bytes())
def put(p,b):
 with p.open('xb') as f:f.write(b)
def dump(p,o):put(p,(json.dumps(o,indent=2)+'\n').encode())
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():
 names=READERS+ROOTS+OTHER+[p.name for p in E.glob('Root-normal-fire-feedback*.json')]
 return {**{n:sha((E/n).read_bytes()) for n in names},DOC:sha((W/DOC).read_bytes())}
def audit():
 assert git('rev-parse','HEAD').decode().strip()==BASE
 membership=(W/'eech-campaign/crates/eech-engine-sys/sources.txt').read_text().splitlines()
 live='aphavoc/source/entity/mobile/vehicle/vh_wpn.c';dead='aphavoc/source/entity/mobile/vehicle/routed/rv_wpn.c'
 assert membership.count(live)==1 and dead not in membership
 assert '#ifdef 0' in (W/dead).read_text()
 caller=(W/'aphavoc/source/entity/mobile/vehicle/routed/rv_updt.c').read_text();calls=['update_vehicle_weapon_system_ready (en);','update_entity_weapon_systems (en);','update_entity_weapon_system_weapon_and_target_vectors (en);','update_vehicle_weapon_fire (en);'];assert [caller.index(n) for n in calls]==sorted(caller.index(n) for n in calls)
 unchanged={}
 for n in SOURCES[:2]:
  raw=(W/n).read_bytes();stripped=b'\n'.join(line for line in raw.split(b'\n') if b'mb_normalfirefx.h' not in line and not line.lstrip().startswith(b'NF_'))
  old=git('show',BASE+':'+n);assert stripped==old,n;unchanged[n]={'raw_current_sha256':sha(raw),'default_body_equals_original':True,'original_sha256':sha(old)}
 lib='eech-campaign/crates/eech-engine/src/lib.rs';assert (W/lib).read_bytes().startswith(git('show',BASE+':'+lib))
 nf=(W/'eech-campaign/crates/eech-engine-sys/csrc/eech_wpn_fixture.c').read_text().split('#ifdef EECH_NORMAL_FIRE_CORRELATION',1)[1].split('\n#endif',1)[0]
 active=re.sub(r'/\*.*?\*/|//[^\n]*','',nf,flags=re.S)
 assert not re.search(r'\b(ready|trace_reset|campaign_aim_identify|campaign_object_identify|campaign_roster_link_member|rand|srand|malloc|calloc|realloc|fopen|read|write)\s*\(',active)
 assert 'return simulated_ms;' in (W/'eech-campaign/crates/eech-engine-sys/csrc/eech_log.c').read_text()
 assert len(j(E/'D3-REQUIREMENTS.json')['requirements'])==10
 card=j(W/'docs/handover/ACTIVE-EXTRACTION.json');parent=next(r for r in card['blockers'] if r['id']=='ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1');assert parent['status']=='open' and parent['blocks_cutover'] and card['preparation_batches_without_cutover']==2
 return {'source_before':BASE,'compiled_producer':live,'disabled_not_compiled':dead,'caller_order':calls,'default_bodies':unchanged,'engine_production_prefix_literal_unchanged':True,'hooks_no_initializer_reset_allocation_IO_RNG_or_lazy_identity':True,'all10_preserved':True,'generic_parent_open_blocks_cutover':True}
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists() and not (E/'nfcD419-readers-PRE-20261108.json').exists()
 for n in READERS:
  if n.endswith('.py'):ast.parse((E/n).read_text())
 dump(E/'nfcD419-source-audit-20261108.json',audit());dump(E/'nfcD419-readers-PRE-20261108.json',readers());print('sealed_readers',len(READERS));sys.exit()
if mode in ['PRE','POST']:
 assert readers()==j(E/'nfcD419-readers-PRE-20261108.json')
 dump(E/f'nfcD419-{mode}-utc-20261108.json',{'started_utc':now()})
 command=[sys.executable,'-I','-B','-X','utf8',str(E/READERS[-1]),str(W),str(E/f'nfcD419-inputmap-{mode}-20261108.json'),BASE]
 result=subprocess.run(command,capture_output=True);put(E/f'nfcD419-map-{mode}.stdout.log',result.stdout);put(E/f'nfcD419-map-{mode}.stderr.log',result.stderr)
 dump(E/f'nfcD419-map-{mode}-receipt-20261108.json',{'command':command,'actual_exit':result.returncode,'completed_utc':now()});assert result.returncode==0
 dump(E/f'nfcD419-extra-{mode}-20261108.json',extras());print(mode,j(E/f'nfcD419-inputmap-{mode}-20261108.json')['file_count'],'extras',len(extras()));sys.exit()
if mode=='freeze':
 pre=j(E/'nfcD419-inputmap-PRE-20261108.json');post=j(E/'nfcD419-inputmap-POST-20261108.json');assert pre['files']==post['files'] and pre['fingerprint_of_raw_sha256_map']==post['fingerprint_of_raw_sha256_map']
 assert extras()==j(E/'nfcD419-extra-PRE-20261108.json')==j(E/'nfcD419-extra-POST-20261108.json') and readers()==j(E/'nfcD419-readers-PRE-20261108.json')
 inherited=[n for n in post['changed_from_head'] if post['files'][n]['head_blob']==post['files'][n]['git_blob_of_raw_bytes']]
 assert len(inherited)==3 and all('/tests/lua/tree-geometry-' in n for n in inherited)
 assert sorted(set(post['changed_from_head'])-set(inherited))==sorted(SOURCES) and not post['deleted_from_head']
 rows={}
 for label in ['s1-fmt','s2-clippy','s3-controls','s4-shipping-check']:
  stem='normal-fire-nfcD419-'+label;r=j(E/(stem+'.json'));assert r['state']=='terminal' and r['actual_exit']==0;rows[label]=r
 out=(E/'normal-fire-nfcD419-s3-controls.stdout.log').read_text();tests=re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',out);assert tests==[('ok','1','0'),('ok','1','0')]
 assert 'portable_lua_controls=9' in out and 'single_field_faults=5' in out and 'observer_checks=4' in out and 'failures=0' in out
 traffic=json.loads(re.search(r'actual D2 requests=(\{[^\n]+\})',out)[1]);assert all(traffic.get(n,0)>0 for n in ['ready','vectors','forecast','release','birth'])
 assert j(E/'nfcD419-final-completion-20261108.json')['actual_exits']=={n:0 for n in rows}
 P.mkdir();(P/'runs').mkdir();(P/'readers').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['nfcD419-source-audit-20261108.json','nfcD419-readers-PRE-20261108.json','nfcD419-extra-PRE-20261108.json','nfcD419-extra-POST-20261108.json','nfcD419-PRE-utc-20261108.json','nfcD419-POST-utc-20261108.json','nfcD419-final-process-20261108.json','nfcD419-final-completion-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in rows:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-nfcD419-'+label+'.'+ext)).read_bytes())
 dev=[]
 for p in sorted(E.glob('normal-fire-nfcD419-d*.json')):
  r=j(p);dev.append({'path':str(p),'sha256':sha(p.read_bytes()),'actual_exit':r.get('actual_exit'),'scope':'unsealed development; debugger exit is not test success','logs':[{'path':str(q),'bytes':q.stat().st_size,'sha256':sha(q.read_bytes())} for q in [p.with_suffix('.stdout.log'),p.with_suffix('.stderr.log')] if q.exists()]})
 dump(P/'development-index.json',dev)
 for label in ['d8-detach-original-callback','d17-normal-reference-environment','d21-malformed-ineligible-red','d22-malformed-ineligible-green','d23-format-lint','d24-format-lint-correction']:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-nfcD419-'+label+'.'+ext)).read_bytes())
 frozen={'utc':now(),'source_before':BASE,'source_rows':post['file_count'],'source_fingerprint':post['fingerprint_of_raw_sha256_map'],'PRE_POST_equal':True,'readers':readers(),'extras':extras(),'changed_sources':SOURCES,'source_audit':j(E/'nfcD419-source-audit-20261108.json'),'final_direct_exits':{n:0 for n in rows},'Rust_tests':2,'portable_Lua_controls':9,'single_field_faults':5,'observer_rounding_modes':4,'unique_model_fallback_cases':5,'actual_model_case_executions':10,'actual_D2_requests':traffic,'inherited_raw_filter_qualifications':inherited,'private_maps':[{'path':str(E/f'nfcD419-inputmap-{m}-20261108.json'),'sha256':sha((E/f'nfcD419-inputmap-{m}-20261108.json').read_bytes())} for m in ['PRE','POST']],'hygiene_problems':[],'scope':'UNREVIEWED read-only correlation prerequisite. Private Original model/control world + real reference D2 only. No Engine boot/native/natural/Windows/case/controller/cutover/removal/all10 closure. Agent completion is Root-owned and separate.'}
 dump(E/'D3-NORMAL-FIRE-CORRELATION-FROZEN-20261108.json',frozen);dump(P/'D3-NORMAL-FIRE-CORRELATION-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Normal-fire correlation prerequisite (UNREVIEWED)\n\nCompiled producer rv_updt -> vh_wpn; original d419/36b2 direction literals retained with explicit erratum. Opt-in records/portable collector plus real private model/D2 controls; final four direct0, two Rust tests, nine Lua controls, four new observer modes. No native/natural authority or shipping API/decision change. Full maps and failed setup/debug logs stay private/hash-bound; raw copied receipts are literal. CODE publication then separate metadata join; Root records actual agent completion. See d3-normal-fire-correlation.md.\n');print(json.dumps({'source_rows':post['file_count'],'readers':len(READERS),'extras':len(extras()),'final_commands':4,'Rust_tests':2,'hygiene':[]}));sys.exit()
if mode=='stage':
 for n in SOURCES+[DOC,'STATUS.md','docs/handover/ACTIVE-EXTRACTION.json']:git('add',n)
 git('add','-A',P.relative_to(W).as_posix())
 for p in P.rglob('*'):
  if p.is_file():assert git('cat-file','blob',':'+p.relative_to(W).as_posix())==p.read_bytes()
 files=[{'path':p.relative_to(P).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',{'files':files,'self_excluded':True,'literal_index_bytes_verified':True});git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'nfcD419-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout
offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
dump(P/'code-publication.json',{'CODE':code,'source_rows':len(paths),'raw_equals_Git':len(paths)-3,'inherited_raw_filter_qualifications':qualified,'source_fingerprint':post['fingerprint_of_raw_sha256_map'],'final_direct_exits':[0,0,0,0],'post_POST_metadata':['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],'raw_log_qualification':'Literal CRLF/EOF retained; packet excluded from source whitespace checker.','scope':'UNREVIEWED private prerequisite only; no native/cutover/all10 closure. Root agent completion separate.'})
files=[{'path':p.relative_to(P).as_posix(),'sha256':sha(p.read_bytes())} for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',{'files':files,'self_excluded':True,'original_manifest_retained':True});git('add','-A',P.relative_to(W).as_posix());print(json.dumps({'CODE':code,'publication_manifest_entries':len(files),'packet_files':len(files)+1}))
