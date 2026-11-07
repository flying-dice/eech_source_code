"""Read-only final binding of actual native receipts/reports. Runs no campaign."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess
B=Path('R:/d3-evidence');load=lambda p:json.loads(Path(p).read_bytes());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
P=B/'physical-flags-witness-5d2c-native-plan-20261107.json';p=load(P);C=B/'physical-flags-witness-5d2c-native-20261107-completion.json';s=load(C);assert s['state']=='terminal' and s['actual_exit']==0 and s['plan_sha256']==sha(P)
assert sha(B/'run-physical-flags-witness-5d2c-native-20261107.py')==s['controller_sha256'];assert sha(s['launch_review'])==s['launch_review_sha256'];assert sha(p['driver'])==s['driver_sha256']==p['native_plan']['driver_literal']['sha256']
for row in p['bindings']:assert sha(row['path'])==row['sha256'],row['path']
payloads=[]
for role,profile in p['profiles'].items():
 assert len(profile['payloads'])==90
 for row in profile['payloads']:
  path=Path(profile['binary'])/row['path'];assert path.stat().st_size==row['bytes'] and sha(path)==row['sha256'];payloads.append(dict(row,role=role,absolute_path=str(path)))
git=lambda *args:subprocess.check_output(['git','-C',p['snapshot'],*args]).decode().strip();assert git('rev-parse','HEAD')==p['runtime_source'] and not git('status','--porcelain=v1','--untracked-files=all')
def report(path):
 lines=Path(path).read_text(encoding='utf-8').splitlines();result={};first={}
 for line in lines:
  name,*tokens=line.split()
  if name=='pf-witness-first':
   row=dict(x.split('=',1) for x in tokens);assert row['event'] not in first;first[row['event']]=row
  elif name in ['pf-witness-intervention','pf-witness-identity','pf-witness-counts','pf-witness-oracle']:
   assert name not in result;result[name]=dict(x.split('=',1) for x in tokens)
  elif name=='pf-witness-fatal':
   assert name not in result;result[name]=dict(x.split('=',1) for x in tokens[:3]);result['fatal_line']=line
  elif name=='pf-witness-verdict':
   assert name not in result and line==lines[-1];result[name]=dict(x.split('=',1) for x in tokens[:2])
  else:raise AssertionError('Unexpected report record '+name)
 result['first']=first;return result
assert len(s['cases'])==23
reference=load(p['input_reference']);rows=[];original=None;control=None;all_raw=[]
for case,step in zip(s['cases'],p['native_plan']['steps'],strict=True):
 assert case['step']==step['step'] and case['mode']==step['mode'];n=case['step'];mode=case['mode']
 event='read_consumable' if mode=='read_changed' else 'group_second' if mode=='group_duplicate_key' else mode.split('_',1)[0]
 if case['state']=='not_launched':
  assert case['status']=='UNREACHED_PREREQUISITE' and n>=3 and control is not None
  assert event not in control['first'] or (mode=='read_changed' and not s['cases'][1]['complete_consume'])
  assert not (Path(p['root_base'])/f'{n:02d}-{mode}').exists() and not (Path(p['out_base'])/f'{n:02d}-{mode}').exists()
  rows.append(dict(step=n,mode=mode,status=case['status'],native_executed=False,reason=case['detail']));continue
 assert case['state']=='terminal' and case['native_exit']==case['post_inputs_exit']==0 and case['status']=='PASS';out=Path(case['output']);r=report(case['path']);assert sha(case['path'])==case['sha256']
 identity=r['pf-witness-identity'];assert identity['mode']==mode and identity['scenario']=='lebanon_retail' and identity['seed']=='1' and identity['hours']=='0.001' and identity['frame_ms']=='100'
 stages=[v for v in s['stages'] if v['cwd']==str(out)];assert len(stages)==3 and [v['stage'] for v in stages]==['pre-inputs','native','post-inputs'] and all(v['state']=='terminal' and v['actual_exit']==0 for v in stages)
 for stage in stages:
  assert stage['pid']>0 and stage['birth_utc'] and stage['argv'];assert datetime.datetime.fromisoformat(stage['started_utc'])<=datetime.datetime.fromisoformat(stage['completed_utc'])
  for stream in ['stdout','stderr']:assert sha(out/(stage['stage']+'.'+stream+'.log'))==stage[stream+'_sha256']
 for name in ['pre-inputs','post-inputs']:
  m=load(out/(name+'.json'));assert m['inputs']==reference['inputs'] and m['combined_sha256']==reference['combined_sha256'] and m['files']==reference['files']==1179
 for path in out.iterdir():
  if path.is_file():all_raw.append(dict(path=str(path),bytes=path.stat().st_size,sha256=sha(path)))
 for key,path in case['reference_reports']['paths'].items():assert sha(path)==case['reference_reports']['sha256'][key]
 counts=r['pf-witness-counts'];assert r['pf-witness-oracle']['mismatches']==counts['oracle_mismatches']=='0'
 assert r['pf-witness-verdict']['status']=='PASS';evidence=dict(step=n,mode=mode,status='PASS',native_executed=True,native_exit=0,pre_post_inputs=0,report_sha256=case['sha256'],counts=counts)
 if n==1:original=r;assert all(int(value)==0 for key,value in counts.items() if key!='frames') and int(counts['frames'])==35
 elif n==2:
  control=r;assert original is not None and identity['paired']=='1'
  assert all(int(counts['accepted_'+event])>0 for event in ['group','set','reset'])
  assert int(counts['altered'])==0 and int(counts['frames'])==35
 else:
  assert control is not None and r['first'][event]==control['first'][event]
  assert counts['altered']=='1';intervention=r['pf-witness-intervention'];fatal=r['pf-witness-fatal'];assert intervention['callback_after']=='none' and intervention['entry']==fatal['entry']
  ordinal=control['first'][event]['ordinal'];assert intervention['at']==('group' if event=='group_second' else event)+'_ordinal_'+ordinal
  marker='entity_scan.callback' if mode.endswith('_withheld') else 'entity_scan.answer' if mode.endswith('_malformed') else 'entity_scan.state' if mode.endswith('_wrong_state') else 'flag.key' if mode in ['group_bad_key','group_duplicate_key'] else 'flag.word' if mode.endswith('_wrong_word') else 'flag.answer'
  assert fatal['marker']==marker and '[model-provider:'+marker+']' in r['fatal_line'];evidence.update(matched_positive_event=event,matched_ordinal=int(ordinal),immediate_same_entry=fatal['entry'],expected_marker=marker)
 rows.append(evidence)
out=B/'physical-flags-witness-5d2c-native-Root-binding-20261107.json';assert not out.exists();v=dict(kind='Root read-only finite actual native witness binding',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),controller_completion=str(C),completion_sha256=sha(C),actual_controller_exit=0,plan_sha256=sha(P),launch_review_sha256=s['launch_review_sha256'],driver_source=p['driver_source'],runtime_source=p['runtime_source'],driver_literal=p['driver'],driver_sha256=s['driver_sha256'],unchanged_payloads=180,retail_manifests=sum(r['native_executed'] for r in rows)*2,all_manifest_rows_match_accepted_reference=True,cases=rows,raw_files=all_raw,native_pass=sum(r['native_executed'] for r in rows),unreached=sum(not r['native_executed'] for r in rows),scope='Finite group/set/reset/identity refusals ONLY; fixed35frames no READ or carried changed read reached, no read case executed, no generic whole flags/nativeconsumer/lifetime/FPU/DCS/fullpack/wholeheight/Task49/D3 acceptance. Previous020 six complete defaultfiles remain literal80fd at own accepted scope; no witness recording comparison claimed. Original764 shipping untouched. SAME actuals review still required.')
out.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(dict(receipt=str(out),sha256=sha(out),native_pass=v['native_pass'],unreached=v['unreached'],manifests=v['retail_manifests'],raw_files=len(all_raw))))
