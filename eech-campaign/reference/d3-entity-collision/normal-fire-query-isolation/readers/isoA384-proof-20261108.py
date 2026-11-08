"""Bounded test-executable isolation seal/publication; no native action."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys,tomllib
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029';P=W/'eech-campaign/reference/d3-entity-collision/normal-fire-query-isolation';BASE='a3845271de0ce04e39c3c3535592d9a3fd4efa33'
DOC='eech-campaign/docs/d3-normal-fire-test-isolation.md'
READERS=['run-normal-fire-isoA384-direct-20261108.py','check-normal-fire-isoA384-linux-20261108.sh','run-isoA384-final-20261108.py','isoA384-proof-20261108.py','sector-roster-inputmap-batched-20261106.py']
ROOTS=['nqt2-publication-checkpoint-20261108.json','D3-NORMAL-FIRE-PUBLIC-SESSION-FROZEN-20261108.json','nqt2-final-completion-20261108.json','codex-d3-native-normal-fire-capture-20261108-process.json','D3-REQUIREMENTS.json']
OTHER=['isoA384-family-20261108.json','isoA384-expected-changes-20261108.json','isoA384-readers-sealed-20261108.json','isoA384-source-audit-20261108.json']
SYS='eech-campaign/crates/eech-engine-sys/';CARGO=SYS+'Cargo.toml';LEGACY=SYS+'tests/physical_flags_collision.rs';SUPPORT=SYS+'tests/support/lua_provider.rs';NEW=SYS+'tests/normal_fire_query.rs'
sha=lambda b:hashlib.sha256(b).hexdigest()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def j(p):return json.loads(p.read_bytes())
def put(p,b):
 with p.open('xb') as f:f.write(b)
def dump(p,obj):put(p,(json.dumps(obj,indent=2)+'\n').encode())
def readers():return {n:sha((E/n).read_bytes()) for n in READERS}
def extras():
 artifacts=j(E/'nqt2-publication-checkpoint-20261108.json')['artifacts']
 return {**{n:sha((E/n).read_bytes()) for n in READERS+ROOTS+OTHER},**{n:sha((W/n).read_bytes()) for n in list(artifacts)+[DOC]}}
def dirty():return sorted(set(git('diff','--name-only','HEAD').decode().splitlines()+git('ls-files','--others','--exclude-standard').decode().splitlines()))
def audit():
 assert git('rev-parse','HEAD').decode().strip()==BASE
 old=j(E/'nqt2-inputmap-POST-20261108.json');unchanged={}
 for n,r in old['files'].items():
  if n not in [CARGO,LEGACY,SUPPORT]:
   assert sha((W/n).read_bytes())==r['raw_sha256'],n;unchanged[n]=r['raw_sha256']
 old_cargo=tomllib.loads(git('show',BASE+':'+CARGO).decode());current=tomllib.loads((W/CARGO).read_text());old_cargo['test'].append(dict(name='normal_fire_query',path='tests/normal_fire_query.rs',**{'required-features':['physical-flags-collision-check','normal-fire-check','normal-fire-correlation']}));assert current==old_cargo
 old_legacy=git('show',BASE+':'+LEGACY).decode().replace('\r\n','\n');marker='#[cfg(all(feature = "normal-fire-check"';start=old_legacy.index(marker);end=old_legacy.index('#[test]\nfn the_moving',start);assert (W/LEGACY).read_text()==old_legacy[:start]+old_legacy[end:]
 body=old_legacy[start:end];body=body[body.index('fn normal_query_'):].rstrip();inner='    extern "C" {\n        fn eech_nf_query_join_check(text: *mut c_char, capacity: c_int, checks: *mut c_int) -> c_int;\n    }\n';body=body.replace(inner,'',1);new=(W/NEW).read_text();assert new[new.index('fn normal_query_'):].rstrip()==body
 assert (W/SUPPORT).read_text().replace('#[allow(dead_code)]\n','')==git('show',BASE+':'+SUPPORT).decode().replace('\r\n','\n')
 artifacts=j(E/'nqt2-publication-checkpoint-20261108.json')['artifacts'];assert all(sha((W/n).read_bytes())==h for n,h in artifacts.items())
 return dict(source_before=BASE,unchanged_prior_source_rows=len(unchanged),unchanged_raw_map_sha256=sha(json.dumps(unchanged,sort_keys=True).encode()),prior_map_sha256=sha((E/'nqt2-inputmap-POST-20261108.json').read_bytes()),changed_prior_rows=[CARGO,LEGACY,SUPPORT],new_source=NEW,old_control_body_moved=True,production_C_Rust_Lua_controller_exact=True,cargo_only_one_test_target_added=True,artifacts=artifacts,scope='No compiled C/library/feature/physical contract change. Exact unchanged source closure reuses prior nqt2 actuals; new test executables run unfiltered separately.')
mode=sys.argv[1]
if mode=='seal':
 assert not P.exists();dump(E/'isoA384-source-audit-20261108.json',audit());dump(E/'isoA384-expected-changes-20261108.json',dirty());dump(E/'isoA384-readers-sealed-20261108.json',readers());print('sealed',len(READERS));sys.exit()
if mode in ['PRE','POST']:
 assert audit()==j(E/'isoA384-source-audit-20261108.json');assert readers()==j(E/'isoA384-readers-sealed-20261108.json');assert dirty()==j(E/'isoA384-expected-changes-20261108.json')
 x=extras();dump(E/('isoA384-extra-'+mode+'-20261108.json'),x);out=E/('isoA384-inputmap-'+mode+'-20261108.json');assert not out.exists();start=now();r=subprocess.run([sys.executable,'-I','-B','-X','utf8',str(E/READERS[-1]),str(W),str(out),BASE],check=False);dump(E/('isoA384-'+mode+'-utc-20261108.json'),dict(started_utc=start,completed_utc=now(),actual_exit=r.returncode,map_sha256=sha(out.read_bytes())));assert r.returncode==0 and x==extras()
 if mode=='POST':assert j(E/'isoA384-inputmap-PRE-20261108.json')['files']==j(out)['files'] and x==j(E/'isoA384-extra-PRE-20261108.json')
 print(mode,j(out)['file_count'],j(out)['fingerprint_of_raw_sha256_map'],'extras',len(x));sys.exit()
if mode=='freeze':
 pre=j(E/'isoA384-inputmap-PRE-20261108.json');post=j(E/'isoA384-inputmap-POST-20261108.json');assert pre['files']==post['files'];assert extras()==j(E/'isoA384-extra-PRE-20261108.json')==j(E/'isoA384-extra-POST-20261108.json')
 c=j(E/'isoA384-final-completion-20261108.json');assert len(c['actual_exits'])==3 and set(c['actual_exits'].values())=={0} and c['actual_exit']==0
 text=(E/'normal-fire-isoA384-s3-coexistence.stdout.log').read_text();err=(E/'normal-fire-isoA384-s3-coexistence.stderr.log').read_text();assert re.findall(r'test result: (\w+)\. (\d+) passed; (\d+) failed;',text)==[('ok','1','0')]*2;assert 'Running tests/normal_fire_query.rs' in err and 'Running tests/physical_flags_collision.rs' in err;assert 'physical flags collision expectations made: 16' in text and 'new hosted identity expectations=3' in text and 'live_model_calls=3041' in text
 P.mkdir();(P/'readers').mkdir();(P/'runs').mkdir()
 for n in READERS:put(P/'readers'/n,(E/n).read_bytes())
 for n in ROOTS+OTHER+['isoA384-extra-PRE-20261108.json','isoA384-extra-POST-20261108.json','isoA384-PRE-utc-20261108.json','isoA384-POST-utc-20261108.json','isoA384-final-process-20261108.json','isoA384-final-completion-20261108.json']:put(P/n,(E/n).read_bytes())
 for label in c['actual_exits']:
  for ext in ['json','stdout.log','stderr.log']:put(P/'runs'/(label+'.'+ext),(E/('normal-fire-isoA384-'+label+'.'+ext)).read_bytes())
 frozen=dict(utc=now(),source_before=BASE,source_rows=post['file_count'],source_fingerprint=post['fingerprint_of_raw_sha256_map'],readers=readers(),extras_count=len(extras()),direct_exits=c['actual_exits'],fresh_Rust_tests=2,legacy_hosted_C_expectations=16,new_hosted_C_expectations=3,new_hosted_provider_entries=3041,new_hosted_callback_interval=[3030,3041],coexistence='Two distinct Cargo integration executables, no name filter/global reset/serial-order assumption.',reuse=audit(),maps={n:dict(path=str(E/('isoA384-inputmap-'+n+'-20261108.json')),sha256=sha((E/('isoA384-inputmap-'+n+'-20261108.json')).read_bytes())) for n in ['PRE','POST']},scope='UNREVIEWED isolation correction, ONE combined source/controller review still required. No C/Lua/controller/physical change or Windows/native/root/cutover/all10 closure. Root owns actual agent completion.')
 dump(E/'D3-NORMAL-FIRE-QUERY-ISOLATION-FROZEN-20261108.json',frozen);dump(P/'D3-NORMAL-FIRE-QUERY-ISOLATION-FROZEN-20261108.json',frozen);put(P/'.gitattributes',b'* -text\n');put(P/'README.md',b'# Normal-fire test-process isolation (UNREVIEWED)\n\nPrior b2a/a384 and nqt2 packet remain literal. New hosted control owns its integration executable; legacy target is preserved. Both full targets run unfiltered in distinct processes. No global reset, C/Lua/controller or physical change. Exact source closure reuses prior controls. ONE combined SAME source/controller review and later Root build/materialized launch review remain required; all10/GenericNative1/prep2/Task49/D3 OPEN.\n');print(json.dumps(frozen));sys.exit()
if mode=='stage':
 for n in j(E/'isoA384-expected-changes-20261108.json'):git('add',n)
 git('add','-A',P.relative_to(W).as_posix());files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'manifest.json',dict(files=files,self_excluded=True));git('add','-A',P.relative_to(W).as_posix());print('manifest_entries',len(files));sys.exit()
assert mode=='join'
code=git('rev-parse','HEAD').decode().strip();post=j(E/'isoA384-inputmap-POST-20261108.json');paths=sorted(post['files']);raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=''.join(code+':'+n+'\n' for n in paths).encode(),capture_output=True,check=True).stdout;offset=0;qualified=[]
for n in paths:
 end=raw.index(b'\n',offset);h=raw[offset:end].decode().split();assert h[1]=='blob';size=int(h[2]);offset=end+1;blob=raw[offset:offset+size];offset+=size+1;current=(W/n).read_bytes();assert sha(current)==post['files'][n]['raw_sha256']
 if current!=blob:qualified.append(n)
assert offset==len(raw) and len(qualified)==3 and all('/tests/lua/tree-geometry-' in n for n in qualified)
for p in P.rglob('*'):
 if p.is_file():assert git('cat-file','blob',code+':'+p.relative_to(W).as_posix())==p.read_bytes()
dump(P/'code-publication.json',dict(CODE=code,prior_CODE='b2a3806a0baf51b498b54a150e5d7aa71c32c62e',prior_META=BASE,source_rows=len(paths),raw_equals_Git=len(paths)-3,inherited_raw_filter_qualifications=qualified,source_fingerprint=post['fingerprint_of_raw_sha256_map'],post_POST_metadata=['STATUS.md','docs/handover/ACTIVE-EXTRACTION.json','manifest.json','code-publication.json','publication-manifest.json'],scope='UNREVIEWED corrected combined source/controller; no native/cutover/all10 closure; Root completion separate.'))
files=[dict(path=p.relative_to(P).as_posix(),sha256=sha(p.read_bytes())) for p in sorted(P.rglob('*')) if p.is_file()];dump(P/'publication-manifest.json',dict(files=files,self_excluded=True,original_manifest_retained=True));git('add','-A',P.relative_to(W).as_posix());print(json.dumps(dict(CODE=code,publication_manifest_entries=len(files),packet_files=len(files)+1)))
