from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys,traceback
b=Path('R:/d3-evidence');ev=Path('D:/EECH-Migration-Evidence');source='f734fdfc08ac36a0feb06efdfac1cb39936a30aa'
snapshot=ev/'fixed-static-corrected-f734-20261106-source';repo=snapshot/'eech-campaign';binary=repo/'target/windows-fixed-static-f734fdfc0'
out=ev/'fixed-static-shipping-f734fdfc0-20261106';preflight=ev/'fixed-static-fullpack-preflight-f734-20261106/preflight.json'
roots=ev/'fixed-static-fullpack-roots-f734-20261106';accepted=Path('R:/eech_source_code/eech-campaign/reference/m5-all-family-loss/final-pack/report.json');baseline=Path('R:/d2-evidence/baseline-host-20261026-shortpath')
skill=Path('C:/Users/starscream/.codex/skills/eech-regression/scripts');sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
pre=json.loads(preflight.read_bytes());assert pre['head']==source and len(pre['expected_checks'])==40 and all(x['exit_code']==0 for x in pre['inputs'].values())
assert sha(accepted)==pre['accepted_report']['sha256'] and sha(skill/'run_regression.py')==pre['launcher']['sha256']
assert not out.exists() and (baseline/'report.json').is_file()
assert subprocess.check_output(['git','-C',str(snapshot),'rev-parse','HEAD'],text=True).strip()==source and not subprocess.check_output(['git','-C',str(snapshot),'status','--porcelain'],text=True).strip()
assert json.loads((b/'fixed-static-ordinary-short-comparison-f734-20261106.json').read_bytes())['all_three_complete_literal_equal']
review=json.loads((b/'independent-fixed-static-corrected-source-f734-20261106.json').read_bytes());assert review['decision']['native_launch_clearance'] is True and review['decision']['findings']==[]
proof=json.loads((b/'fixed-static-windows-profile-isolation-f734-20261106.json').read_bytes());assert proof['source']==source and proof['profile_isolation_passed'] and proof['new_native_prerequisite_met']
args=[sys.executable,'-B','-X','utf8',str(skill/'run_regression.py'),'--repo',str(repo),'--python',sys.executable,'--bin',str(binary),'--georgia',str(roots/'georgia'),'--lebanon',str(roots/'lebanon'),'--lebanon-repeat',str(roots/'lebanon-repeat'),'--accepted-report',str(accepted),'--out',str(out),'--role','candidate','--expected-commit',source]
marker=roots/'CONSUMED-FIXED-STATIC-FULLPACK-f734-20261106.json';assert not marker.exists()
state=dict(controller_pid=os.getpid(),source=source,snapshot=str(snapshot),binary=str(binary),out=str(out),roots=str(roots),baseline=str(baseline),preflight=str(preflight),state='running',started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),args=args,scope='Established accepted40/all8 ordinary shipping regression for this frozen fixed/static canonical-state candidate. Same seed1/frame100/3simh and inputs; no fixture/literal override. Witness-only source edits elsewhere do not mutate source/bin/root. Completion does not close unresolved physical ownership/Task49/D3. Root selected full pack after source/default clearance; same reviewer corrected-source clearance has no findings. Focused healthy/withdrawal actuals and their finite review remain independent unmet gates; this full run grants no authority acceptance.')
receipt=b/'fixed-static-shipping-f734-20261106-process.json';assert not receipt.exists()
def save():receipt.write_bytes((json.dumps(state,indent=2)+'\n').encode())
marker.write_bytes((json.dumps(dict(source=source,out=str(out),reserved_utc=state['started_utc']))+'\n').encode());save()
try:
 with (b/'fixed-static-shipping-f734-20261106-launcher.log').open('xb') as log:
  p=subprocess.Popen(args,stdout=log,stderr=subprocess.STDOUT,cwd='C:/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS',creationflags=subprocess.CREATE_NO_WINDOW)
  state['launcher_pid']=p.pid;save();code=p.wait()
 state['launcher_actual_exit']=code;save();assert code==0,'Inspect actual runner/completion; preserve failed stage, do not restart war'
 done=json.loads((out/'completion.json').read_bytes());run=json.loads((out/'run.json').read_bytes());report=json.loads((out/'report.json').read_bytes());expected=json.loads(accepted.read_bytes())
 assert run['status']=='completed' and run['exit_code']==done['original_pack_exit']==0 and done['expected_check_set_all_pass'] and done['source_build_accepted_identity_unchanged'] and done['launcher_gate_passed']
 assert set(report['checks'])==set(expected['checks']) and len(report['checks'])==40 and all(x['status']=='PASS' for x in report['checks'].values()) and all(x['exit_code']==0 for x in done['post_inputs'].values())
 with (b/'fixed-static-shipping-f734-20261106-internal-input.log').open('xb') as log:
  code=subprocess.run([sys.executable,'-B','-X','utf8',str(repo/'tools/input-manifest.py'),str(out/'roots/lebanon-regress'),'--compare',str(repo/'reference/lebanon-3h/inputs.json')],stdout=log,stderr=subprocess.STDOUT).returncode
 state['internal_fourth_input_actual_exit']=code;save();assert code==0
 comparison=b/'fixed-static-shipping-complete-comparison-f734-20261106.json';assert not comparison.exists()
 with (b/'fixed-static-shipping-f734-20261106-comparison.log').open('xb') as log:
  code=subprocess.run([sys.executable,'-B','-X','utf8',str(skill/'compare_complete_packs.py'),'--baseline',str(baseline),'--candidate',str(out),'--accepted-report',str(accepted),'--out',str(comparison)],stdout=log,stderr=subprocess.STDOUT).returncode
 state['complete_comparison_actual_exit']=code;save();assert code==0
 cmp=json.loads(comparison.read_bytes());assert cmp['passed'] and not cmp['partial_prefix_only'] and len(cmp['results'])==8
 state.update(state='terminal',actual_exit=0,accepted_checks=40,complete_files_literal_equal=8,completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
except Exception:
 state.update(state='terminal',actual_exit=1,error=traceback.format_exc(),completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat());save();raise
finally:
 if state['state']=='terminal':(b/'fixed-static-shipping-f734-20261106-completion.json').write_bytes((json.dumps(state,indent=2)+'\n').encode())
