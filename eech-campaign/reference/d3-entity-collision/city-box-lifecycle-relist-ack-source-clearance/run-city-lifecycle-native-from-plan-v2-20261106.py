from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,traceback
b=Path('R:/d3-evidence');ev=Path('D:/EECH-Migration-Evidence');now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
plan_path=Path(sys.argv[1]);assert sha(plan_path)==sys.argv[2];plan=json.loads(plan_path.read_bytes());source=plan['source'];literal=plan.get('diagnostic_literal');reviewed_source=literal['source'] if literal else source;mode=sys.argv[3];roles={'city_box_lifecycle':'control','city_box_lifecycle_missing':'missing','city_box_lifecycle_changed':'changed'};role=roles[mode]
def artifact(name):
 row=plan['artifacts'][name];p=Path(row['path']);assert p.is_file() and sha(p)==row['sha256'],name;return p
review=artifact('source_review');r=json.loads(review.read_bytes());assert r['decision']['findings']==[] and r['decision']['native_launch_clearance'] is True and reviewed_source in json.dumps(r['source'])
profile=artifact('profile_proof');world=artifact('World_proof');pp=json.loads(profile.read_bytes());wp=json.loads(world.read_bytes());assert pp['source']==wp['source']==source and pp['profile_isolation_passed'] and wp['actual_exit']==0
if role!='control':
 c=json.loads(Path(plan['runs']['city_box_lifecycle']['completion_receipt']).read_bytes());assert c['source']==source and c['actual_exit']==c['actual_native_exit']==c['actual_helper_exit']==c['actual_checker_exit']==0
 if literal:assert c['literal_source']==literal['source'] and c['literal_sha256']==literal['sha256']
build=json.loads(artifact('build_receipt').read_bytes());assert build['actual_exit']==0 and build['source']==source;snapshot=Path(build['snapshot']);binary=Path(build['profiles']['hosted']['binary'])
prep=json.loads(artifact('input_preparation').read_bytes());assert prep['actual_exit']==0 and prep['reference_tool_source']==source
run=plan['runs'][mode];root=Path(run['root']);row=next(x for x in prep['roots'] if Path(x['root'])==root);assert row['manifest_actual_exit']==0 and row['state'].startswith('VERIFIED UNUSED')
assert prep['input_tool_sha256']==sha(snapshot/'eech-campaign/tools/input-manifest.py') and prep['reference_sha256']==sha(snapshot/'eech-campaign/reference/lebanon-3h/inputs.json')
assert all(sha(binary/x['path'])==x['sha256'] for x in build['profiles']['hosted']['payloads'])
helper=artifact('native_helper');checker=artifact('row_checker');assert sha(helper)=='3da2855f8b8b67c73d4ac0e157b40c81acdf633bdf21560daa3e588f024b0582' and sha(checker)=='e18f2609ea317514e1d9866b0d04616b9ff9191bb1e5ac25ae0ad616e4124c2b'
if literal:
 eq=json.loads(artifact('runtime_equivalence').read_bytes());assert eq['runtime_source']==source and eq['literal_source']==literal['source'] and eq['only_diagnostic_and_test_changes'] is True
 literal_path=Path(literal['path']);raw=literal_path.read_bytes();assert sha(literal_path)==literal['sha256'] and hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==literal['git_blob']
stem=run['stem'];out=Path(run['output']);p=Path(run['process_receipt']);marker=Path(run['consumption_marker']);assert out.parent.resolve()==ev.resolve() and p.parent.resolve()==b.resolve() and marker.parent.resolve()==ev.resolve();assert not out.exists() and not p.exists() and not marker.exists()
args=[sys.executable,'-B','-X','utf8',str(helper),'--source',source,'--snapshot',str(snapshot),'--bin',str(binary),'--root',str(root),'--out',str(out),'--script','entity-authority.lua','--scenario','lebanon_retail','--arg=mode='+mode,'--arg=profile=hosted']
if literal:args += ['--literal-driver',literal['path'],'--literal-source',literal['source'],'--literal-git-blob',literal['git_blob'],'--literal-sha256',literal['sha256']]
state=dict(source=source,mode=mode,role=role,controller_pid=os.getpid(),controller_sha256=sha(Path(__file__)),plan_sha256=sha(plan_path),helper_sha256=sha(helper),checker_sha256=sha(checker),profile_proof_sha256=sha(profile),World_proof_sha256=sha(world),snapshot=str(snapshot),binary=str(binary),root=str(root),output=str(out),args=args,started_utc=now(),state='running',scope='Reviewed standalone terminal real-Core selected CITY witness. Control-first gate enforced; no subsequent physical simulation on this root. Thin checker source-bound, no wholeCITY/40all8/Task49/D3 acceptance.')
if literal:state.update(literal_source=literal['source'],literal_git_blob=literal['git_blob'],literal_sha256=literal['sha256'],literal_path=literal['path'],runtime_equivalence_sha256=plan['artifacts']['runtime_equivalence']['sha256'])
save=lambda:p.write_text(json.dumps(state,indent=2)+'\n');marker.write_text(json.dumps({'source':source,'mode':mode,'root':str(root),'output':str(out),'started_utc':now()},indent=2)+'\n');save()
try:
 with (b/(stem+'.stdout.log')).open('xb') as stdout,(b/(stem+'.stderr.log')).open('xb') as stderr:
  child=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW);state['pid']=child.pid;save();code=child.wait()
 state['actual_helper_exit']=code;c=json.loads((out/'controller.json').read_bytes());state['actual_native_exit']=c.get('actual_native_exit');state['helper_controller_actual_exit']=c.get('actual_exit')
 rows=[json.loads(x) for x in (out/'model.jsonl').read_bytes().splitlines()];dedicated=[x for x in rows if x.get('event')=='city_lifecycle_authority'];state['dedicated_rows']=dedicated;save()
 assert code==c['actual_exit']==c['actual_native_exit']==0
 assert c['verified_payloads']==len(build['profiles']['hosted']['payloads']) and len(c['stages'])==3 and all(x['actual_exit']==0 for x in c['stages'])
 with (b/(stem+'-checker.stdout.log')).open('xb') as stdout,(b/(stem+'-checker.stderr.log')).open('xb') as stderr:
  checked=subprocess.run([sys.executable,'-B','-X','utf8',str(checker),'--rows',str(out/'model.jsonl'),'--mode',mode,'--output',str(out/'row-check.json')],stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
 state['actual_checker_exit']=checked.returncode;save();assert checked.returncode==0
 assert (out/'pre-inputs.json').read_bytes()==(out/'post-inputs.json').read_bytes()==(snapshot/'eech-campaign/reference/lebanon-3h/inputs.json').read_bytes()
 assert all(sha(binary/x['path'])==x['sha256'] for x in build['profiles']['hosted']['payloads'])
 state.update(actual_exit=0,payloads_rehashed=len(build['profiles']['hosted']['payloads']),pre_post_full_manifests_literal_reference=True,row_check_sha256=sha(out/'row-check.json'))
except BaseException:
 state.update(actual_exit=1,error=traceback.format_exc());raise
finally:
 state.update(state='terminal',completed_utc=now());save();Path(run['completion_receipt']).write_bytes(p.read_bytes())
print(json.dumps({k:state[k] for k in ['source','mode','actual_exit','actual_helper_exit','actual_native_exit','actual_checker_exit','output']}))
