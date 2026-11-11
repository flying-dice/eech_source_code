"""Read-only recovery of Root's wrong schema assertion; never rerun native or replace its receipt."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess
b=Path('R:/d3-evidence');out=Path('D:/EECH-Migration-Evidence/fixed-rule-witness-native-20261106')
sha=lambda raw:hashlib.sha256(raw).hexdigest()
snapshot=lambda:{str(p.relative_to(out)):dict(bytes=p.stat().st_size,sha256=sha(p.read_bytes())) for p in sorted(out.rglob('*')) if p.is_file()}
before=snapshot();oldpath=b/'fixed-rule-witness-native-20261106-completion.json';oldraw=oldpath.read_bytes();old=json.loads(oldraw)
assert old['state']=='terminal' and old['actual_exit']==1 and old['actual_native_exit']==old['actual_helper_exit']==0
assert "KeyError: 'physical_after_stop'" in old['error']
runtime='681ccfe5476a7c0a90036bc01cd054358c751162';code='879b0ebde110250169513dfd9fecc4c83f3b7b01'
assert old['runtime_source']==runtime and old['literal_source']==code
c=json.loads((out/'controller.json').read_bytes())
assert c['state']=='terminal' and c['actual_exit']==c['actual_native_exit']==0 and c['verified_payloads']==88
assert [s['stage'] for s in c['stages']]==['pre-inputs','native','post-inputs'] and all(s['state']=='terminal' and s['actual_exit']==0 for s in c['stages'])
assert c['external_diagnostic']['source']==code and c['external_diagnostic']['git_blob']=='e27c2da83ee59dda1ac614a36d0755698f302722'
assert c['external_diagnostic']['sha256']=='8cb88ddf2ab548728abbbd99e64bdf4c41258fe144cf9607e9ba6efbf1867892'
reference=Path(c['snapshot'])/'eech-campaign/reference/lebanon-3h/inputs.json'
assert (out/'pre-inputs.json').read_bytes()==(out/'post-inputs.json').read_bytes()==reference.read_bytes()
binary=Path(c['binary']);info=(binary/'BUILD-INFO.txt').read_text();entries=re.findall(r'^  ([0-9a-f]{64})  (.+)$',info,re.M)
assert len(entries)==88 and all(sha((binary/name).read_bytes())==digest for digest,name in entries)
w=b/'collision-characterization-20261029';rel='eech-campaign/crates/eech-world/lua/entity-authority.lua'
literal=Path(c['external_diagnostic']['path']);gitraw=subprocess.check_output(['git','-C',str(w),'show',code+':'+rel])
assert literal.read_bytes()==gitraw and sha(gitraw)==c['external_diagnostic']['sha256']
rows=[json.loads(line) for line in (out/'model.jsonl').read_bytes().splitlines()]
events=[x.get('event') for x in rows]
assert events==['boot','warmed_up','case_selected','case_launch','cohort','authority']
a=rows[-1]
assert a['mode']=='changed_fixed_rule' and a['profile']=='hosted' and a['completed'] is False
assert a['fixed_rule_eligible']==a['fixed_rule_changed']==1
assert a['fixed_rule_changed_next']=='placement' and a['fixed_rule_changed_cause']=='fixed_rule_mirror_differs'
assert a['failure_marker']=='entity_candidate.refused' and a['expected_marker']=='entity_candidate.refused'
assert '[candidate.mirror]' in a['failure_line'] and 'collision-fixed' in a['failure_line']
assert a['request_faults']==0 and a['reply_faults']==1 and a['first_fault']=='candidate reply: result refused'
# This is the actual post-launch authority schema (Lua2217), not boot_failure_row (1105/1111).
# The driver deliberately does not call pending_members after a failure; -1 is its declared sentinel, not a measured physical result.
assert 'boot_stop' not in a and 'physical_after_stop' not in a and a['case_weapons_pending']==-1
assert 0<=a['frames']<=a['frame_budget'] and a['case_weapons_ended']==0 and a['view_applied']>=1
after=snapshot();assert before==after and oldpath.read_bytes()==oldraw
safe={k:a[k] for k in ['mode','profile','completed','fixed_rule_eligible','fixed_rule_changed','fixed_rule_changed_next','fixed_rule_changed_cause','request_faults','reply_faults','first_fault','failure_marker','frames','frame_budget','case_weapons_pending','case_weapons_ended','view_applied']}
proof=dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),actual_exit=0,scope='Read-only checker recovery only; original outer1 remains1, native/helper0 unchanged. Finite negative evidence pending independent actual review; not new execution, shipping/fullD3 acceptance or native coverage of all15subtypes.',runtime_source=runtime,literal_source=code,metadata_commit=old['metadata_commit'],original_completion_sha256=sha(oldraw),original_native_helper_outer_exits=[0,0,1],recovery_checker_actual_exit=0,checker_schema='post-launch authority row; case_weapons_pending=-1 failure sentinel; no invented BOOT fields or physical measurement',safe_facts=safe,pre_post_inputs_full_literal_reference=True,pre_post_input_actual_exits=[0,0],input_reference_sha256=sha(reference.read_bytes()),verified_payloads=88,literal_git_blob=c['external_diagnostic']['git_blob'],literal_sha256=sha(gitraw),raw_outputs_before_after_literal_unchanged=True,raw_outputs=before,original_completion_preserved=True,no_native_or_input_tool_rerun=True)
p=b/'fixed-rule-witness-checker-recovery-20261106.json';assert not p.exists();p.write_bytes((json.dumps(proof,indent=2)+'\n').encode())
print(json.dumps(dict(proof=str(p),sha256=sha(p.read_bytes()),original_native_helper_outer_exits=[0,0,1],recovery_checker_actual_exit=0,safe_facts=safe,all_retained_raw_unchanged=True)))
