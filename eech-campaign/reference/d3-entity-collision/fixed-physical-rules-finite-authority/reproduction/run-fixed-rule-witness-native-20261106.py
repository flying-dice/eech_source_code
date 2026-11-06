"""One separately bound diagnostic on sealed runtime681, only after Root's reviewed clearance gate."""
from pathlib import Path
import datetime,hashlib,json,os,re,subprocess,sys,traceback
b=Path('R:/d3-evidence'); ev=Path('D:/EECH-Migration-Evidence')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat()
gate_path=b/'fixed-rule-witness-source-clear-gate-20261106.json'
assert len(sys.argv)==2 and re.fullmatch('[0-9a-f]{64}',sys.argv[1])
assert sha(gate_path)==sys.argv[1]
gate=json.loads(gate_path.read_bytes())
assert gate['native_launch_clearance'] is True and gate['findings']==[]
runtime='681ccfe5476a7c0a90036bc01cd054358c751162'
assert gate['runtime_source']==runtime
review=Path(gate['review_path']); assert sha(review)==gate['review_sha256']
assert gate['production_inputs_unchanged'] is True
code,meta=gate['literal_source'],gate['metadata_commit']
assert re.fullmatch('[0-9a-f]{40}',code) and re.fullmatch('[0-9a-f]{40}',meta)
worktree=b/'collision-characterization-20261029'
git=lambda *args:subprocess.check_output(['git','-C',str(worktree),*args])
assert git('rev-parse','HEAD').decode().strip()==meta
assert git('status','--porcelain','--untracked-files=all')==b''
rel='eech-campaign/crates/eech-world/lua/entity-authority.lua'
raw=git('show',code+':'+rel); blob=git('rev-parse',code+':'+rel).decode().strip()
assert blob==gate['literal_git_blob'] and hashlib.sha256(raw).hexdigest()==gate['literal_sha256']
literal=b/('fixed-rule-witness-'+code[:9]+'-literal.lua')
if literal.exists():
 assert literal.read_bytes()==raw
else:
 literal.write_bytes(raw)
prep=json.loads((b/'fixed-rule-witness-input-20261106/preparation.json').read_bytes())
assert prep['actual_exit']==prep['manifest_actual_exit']==0 and prep['root_state'].startswith('VERIFIED UNUSED')
root=Path(prep['root']);out=ev/'fixed-rule-witness-native-20261106'
receipt=b/'fixed-rule-witness-native-20261106-process.json'
marker=ev/'CONSUMED-fixed-rule-witness-negative-20261106.json'
assert root.is_dir() and not out.exists() and not receipt.exists() and not marker.exists()
build=json.loads((b/'fixed-rules-corrected-681ccfe54-windows-20261106-completion.json').read_bytes())
assert build['source']==runtime and build['actual_exit']==0
snapshot=Path(build['snapshot']);binary=Path(build['profiles']['hosted']['binary'])
assert prep['input_tool_sha256']==sha(snapshot/'eech-campaign/tools/input-manifest.py')
assert prep['reference_sha256']==sha(snapshot/'eech-campaign/reference/lebanon-3h/inputs.json')
assert all(sha(binary/x['path'])==x['sha256'] for x in build['profiles']['hosted']['payloads'])
assert sha(b/'fixed-rules-windows-profile-isolation-681cc-20261106.json')=='af7ccb1c808157ef5452595a4c623526f98cade03a9c0024112105e1f290b9e8'
assert sha(b/'fixed-rules-world-objects-681cc-20261106.json')=='1c9fdffb16ec5b058271b269807575ccdc1ef6544338f04c5bbfec157c66a851'
helper=b/'run-entity-public-native-pipe-20261105.py'
assert sha(helper)=='3da2855f8b8b67c73d4ac0e157b40c81acdf633bdf21560daa3e588f024b0582'
args=[sys.executable,'-B','-X','utf8',str(helper),'--source',runtime,'--snapshot',str(snapshot),'--bin',str(binary),'--root',str(root),'--out',str(out),'--script','entity-authority.lua','--scenario','lebanon_retail','--literal-driver',str(literal),'--literal-source',code,'--literal-git-blob',blob,'--literal-sha256',sha(literal),'--arg=mode=changed_fixed_rule','--arg=profile=hosted']
state=dict(runtime_source=runtime,literal_source=code,metadata_commit=meta,review_sha256=sha(review),clearance_gate_sha256=sha(gate_path),controller_pid=os.getpid(),started_utc=now(),state='running',root=str(root),output=str(out),args=args,scope='ONE actual same-payload Host counterfactual, explicitly extra Host evaluation, not a second Core question. Sealed681 runtime reused; no healthy/short/profile replay, no full-pack or whole-D3 acceptance.')
save=lambda:receipt.write_bytes((json.dumps(state,indent=2)+'\n').encode())
marker.write_bytes((json.dumps(dict(runtime=runtime,literal=code,output=str(out),started_utc=now()))+'\n').encode());save()
try:
 with (b/'fixed-rule-witness-native-20261106.stdout.log').open('xb') as stdout,(b/'fixed-rule-witness-native-20261106.stderr.log').open('xb') as stderr:
  child=subprocess.Popen(args,stdout=stdout,stderr=stderr,creationflags=subprocess.CREATE_NO_WINDOW)
  state['pid']=child.pid;save();exitcode=child.wait()
 state['actual_helper_exit']=exitcode
 c=json.loads((out/'controller.json').read_bytes());state['actual_native_exit']=c.get('actual_native_exit');save()
 assert exitcode==c['actual_exit']==c['actual_native_exit']==0
 assert c['verified_payloads']==88 and len(c['stages'])==3 and all(x['actual_exit']==0 for x in c['stages'])
 rows=[json.loads(x) for x in (out/'model.jsonl').read_bytes().splitlines()]
 authorities=[x for x in rows if x.get('event')=='authority'];assert len(authorities)==1
 a=authorities[0]
 assert a['mode']=='changed_fixed_rule' and not a['completed']
 assert a['fixed_rule_eligible']==a['fixed_rule_changed']==1
 assert a['fixed_rule_changed_next']=='placement' and a['fixed_rule_changed_cause']=='fixed_rule_mirror_differs'
 assert a['failure_marker']=='entity_candidate.refused'
 assert '[candidate.mirror]' in a['failure_line'] and 'collision-fixed' in a['failure_line']
 assert a['request_faults']==0 and a['reply_faults']==1 and a['first_fault']=='candidate reply: result refused'
 assert a['physical_after_stop']==-1
 assert (out/'pre-inputs.json').read_bytes()==(out/'post-inputs.json').read_bytes()==(snapshot/'eech-campaign/reference/lebanon-3h/inputs.json').read_bytes()
 state.update(actual_exit=0,verified_payloads=88,pre_post_inputs_actual_exit=[0,0],safe_facts={k:a[k] for k in ['completed','boot_stop','fixed_rule_eligible','fixed_rule_changed','fixed_rule_changed_next','fixed_rule_changed_cause','physical_after_stop','request_faults','reply_faults','first_fault','failure_marker']})
except BaseException:
 state.update(actual_exit=1,error=traceback.format_exc());raise
finally:
 state.update(state='terminal',completed_utc=now());save()
 (b/'fixed-rule-witness-native-20261106-completion.json').write_bytes(receipt.read_bytes())
print(json.dumps({k:state[k] for k in ['runtime_source','literal_source','actual_exit','actual_native_exit','actual_helper_exit','output']}))
