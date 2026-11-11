from pathlib import Path
import datetime,hashlib,importlib.util,json,subprocess
B=Path('R:/d3-evidence');W=B/'collision-characterization-20261029';PLAN=B/'native-read-nr602-materialized-plan-v2-20261108.json';REVIEW=B/'independent-native-flag-read-correction-04bf-source-controller-20261108.json'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def load(p):return json.loads(Path(p).read_bytes())
p=load(PLAN);r=load(REVIEW);assert sha(PLAN)=='4ee21a2c6598787c598247f6059f3f1339a49333ad737ac4b6df1857c8e87d53' and sha(REVIEW)=='4cda90703989cde0350a99e3dceea8a98ea768247d8d62a647bd10c5deb04838'
assert r['decision']['native_controller_launch_clearance'] and not r['findings'];outer=B/'native-read-nr602-original-outer-completion-20261108.json';assert load(outer)['actual_outer_exit']==0
T=W/'eech-campaign/tools';paths={'controller':T/'native_flag_read_controller.py','validator':T/'native_flag_read.py','binding':T/'native-flag-read-runtime.json',**{k:Path(p[k]) for k in ('lua','protocol','input_tool','input_reference')}}
for k,path in paths.items():assert sha(path)==p['hashes'][k]
for k in ('controller','validator','binding','lua','protocol'):assert r[k+'_sha256']==p['hashes'][k]
manifest=load(p['bundle_manifest']);assert sha(p['bundle_manifest'])==p['bundle_manifest_sha256'];assert manifest['runtime']==p['runtime']=='3abf03e5133279ae6ed53b657e76d4f7724f7dd3'
assert manifest['files']==load(paths['binding'])['payloads'] and len(manifest['files'])==91
binary=Path(p['binary_dir']);assert {q.relative_to(binary).as_posix() for q in binary.rglob('*') if q.is_file()}=={q['path'] for q in manifest['files']}
for row in manifest['files']:
 path=binary/row['path'];assert path.stat().st_size==row['bytes'] and sha(path)==row['sha256']
literal=Path(p['literal_dir']);assert {q.resolve() for q in literal.rglob('*') if q.is_file()}=={paths['lua'].resolve(),paths['protocol'].resolve()}
spec=importlib.util.spec_from_file_location('nr602_actual_validator',paths['validator']);v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
out=Path(p['out_base']);d=load(out/'completion.json');assert d==load(out/'process.json') and d['state']=='terminal' and d['actual_exit']==0 and d['controller_pid']==7256
assert d['plan_sha256']==sha(PLAN) and d['controller_sha256']==p['hashes']['controller'] and d['review_sha256']==sha(REVIEW) and d['bundle_manifest_sha256']==p['bundle_manifest_sha256']
assert len(d['cases'])==7 and [c['mode'] for c in d['cases']]==p['modes']==v.MODES and len(d['stages'])==21
ref=load(paths['input_reference']);expected={'lua':p['hashes']['lua'],'protocol':p['hashes']['protocol'],'bundle':p['bundle_manifest_sha256'],'reference':'none'}
healthy_path=Path(d['cases'][0]['out'])/'witness.txt';healthy_sha=sha(healthy_path);healthy=v.validate(healthy_path.read_bytes(),expected);assert healthy['verdict']['status']=='PASS'
cases=[];raws=[];seen_roots=set()
for index,c in enumerate(d['cases']):
 assert c['state']=='terminal' and c['native_exit']==0 and c['status']=='PASS';case=Path(c['out']);root=Path(c['root']);assert root.resolve() not in seen_roots;seen_roots.add(root.resolve())
 assert root.resolve()==(Path(p['healthy_root']).resolve() if not index else (Path(p['root_base'])/f'{index+1:02d}-{c["mode"]}').resolve())
 for key in ('pre_inputs','post_inputs'):
  row=c[key];assert sha(row['path'])==row['sha256'];m=load(row['path']);assert m['files']==1179==ref['files'] and m['inputs']==ref['inputs']
 path=case/'witness.txt';assert sha(path)==c['report_sha256'];facts=v.validate(path.read_bytes(),{**expected,'reference':healthy_sha if index else 'none'},c['mode'],healthy if index else None)
 assert facts['verdict']['status']=='PASS';stages=d['stages'][3*index:3*index+3];assert [x['stage'] for x in stages]==['pre-inputs','native','post-inputs']
 for st in stages:
  assert st['state']=='terminal' and st['actual_exit']==0 and st['pid']>0 and st['birth_utc'] and st['argv'] and Path(st['cwd']).resolve()==case.resolve()
  for stream in ('stdout','stderr'):assert sha(case/(st['stage']+'.'+stream+'.log'))==st[stream+'_sha256']
 native=stages[1];assert Path(native['argv'][0]).resolve()==(binary/'eech-world.exe').resolve() and Path(native['argv'][1]).resolve()==paths['lua'].resolve()
 args=dict(x.split('=',1) for x in native['argv'][2:]);assert len(args)==len(native['argv'])-2
 assert args['root']==str(root) and args['out']==str(path) and args['protocol']==str(paths['protocol']) and args['mode']==c['mode'] and args['runtime_code']==p['runtime']
 assert args['request_updated']=='first_eligible' and args['scenario']=='lebanon_retail' and args['hours']=='0.001' and args['frame_ms']=='100' and args['seed']=='1' and args['plan_sha256']==p['source_plan_sha256']
 for k in ('lua','protocol'):assert args[k+'_sha256']==p['hashes'][k]
 assert args['bundle_sha256']==p['bundle_manifest_sha256']
 if index:
  assert args['control']==str(healthy_path) and args['control_sha256']==healthy_sha
  assert stages[0]['birth_utc']>=d['stages'][2]['completed_utc']
 cases.append({'mode':c['mode'],'native_pid':native['pid'],'native_birth_utc':native['birth_utc'],'native_actual_exit':0,'status':'PASS','report_sha256':sha(path),'counts':facts['counts'],'selection':facts.get('selection'),'store':facts.get('store'),'reask':facts.get('reask'),'intervention':facts.get('intervention'),'fatal':facts.get('fatal'),'pre_input_sha256':c['pre_inputs']['sha256'],'post_input_sha256':c['post_inputs']['sha256']})
 for q in sorted(case.iterdir()):
  if q.is_file():raws.append({'path':str(q),'bytes':q.stat().st_size,'sha256':sha(q)})
head=subprocess.check_output(['git','-C',str(W),'rev-parse','HEAD'],text=True).strip();assert head=='1ab12d0efc6e088b483ba30e0f12ec74c9ffa2ca' and not subprocess.check_output(['git','-C',str(W),'status','--porcelain=v1','--untracked-files=all'],text=True).strip()
proof={'observed_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runtime':p['runtime'],'source_head':head,'plan':str(PLAN),'plan_sha256':sha(PLAN),'review_sha256':sha(REVIEW),'original_tool_session':27818,'actual_outer_exit':0,'actual_controller_exit':0,'native_direct_exits':[0]*7,'all_21_stage_direct_exits_zero':True,'native_process_identities_from_original_GetProcessTimes_receipts':True,'healthy_first':True,'all_seven_strict_reports_PASS':True,'full_pre_post_input_manifest_observations':14,'input_files_per_root':1179,'unique_retail_roots':len(seen_roots),'all_91_payloads_and_two_Lua_literals_unchanged':True,'source_clean_unchanged':True,'cases':cases,'retained_raw_files':raws,'process_receipt_sha256':sha(out/'process.json'),'completion_receipt_sha256':sha(out/'completion.json'),'Root_checker_sha256':sha(__file__),'limits':'Finite diagnostic extra destructive UPDATED READ on one real eligible candidate at frame16, changed carried fact and five immediate refusals. Derived traffic oracle not independent full-store snapshot. No collision-outcome/default-equivalence/natural-moving/allbits/allarrays/lifetime/FPU/DCS/wholeD3 acceptance. No diagnostic Tacview comparison; both independent fullpacks remain separate and incomplete. Expected caught-refusal native0 does not mean healthy campaign continuation. No rerun or baseline/input/seed/frame/window/tolerance change.'}
target=B/'native-read-nr602-native-Root-binding-20261108.json'
with target.open('x',encoding='utf-8') as f:f.write(json.dumps(proof,indent=2)+'\n')
print(json.dumps({'proof':str(target),'sha256':sha(target),'cases':len(cases),'all_native_direct_exits':0,'all_21_stage_exits':0,'full_input_observations':14,'payloads':91,'retained_raw_files':len(raws),'physical_boundary_scope_only':True}))
