from pathlib import Path
import datetime, hashlib, json, subprocess, sys

b = Path('R:/d3-evidence')
repo = Path('R:/eech_source_code')
src = b/'ground-production-source-aafb21775/eech-campaign'
pack = b/'shipping-ground-aafb21775-recovery-pack-20261031-0945'
binary = src/'target/windows-ground-production-aafb21775'
out = b/'ground-completed-shipping-audit-20261101.json'
assert not out.exists()
def sha(p):
    h = hashlib.sha256()
    with p.open('rb') as f:
        for block in iter(lambda: f.read(1048576), b''): h.update(block)
    return h.hexdigest()
def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
run, pre, done, report = [read(pack/n) for n in ('run.json','preflight.json','completion.json','report.json')]
comparison = read(b/'ground-aafb21775-complete-comparison-20261101.json')
assert run['status'] == 'completed' and run['exit_code'] == 0
assert done['original_pack_exit'] == 0 and done['launcher_gate_passed']
assert len(report['checks']) == 40 and set(report['checks']) == set(pre['expected_checks'])
assert report['head']['verdict'] == 'PASS' and all(v['status']=='PASS' for v in report['checks'].values())
assert comparison['passed'] and comparison['accepted_checks_each']==40 and len(comparison['results'])==8
payloads = {n: {'expected': v, 'actual': {'bytes': (binary/n).stat().st_size, 'sha256': sha(binary/n)}} for n,v in pre['files'].items()}
assert len(payloads)==50 and all(v['actual']==v['expected'] for v in payloads.values())
assert sha(binary/'BUILD-INFO.txt') == pre['build_info']['sha256']
assert sha(src/'tools/regression-pack.py') == pre['runner']['sha256']
assert sha(Path('C:/Users/starscream/.codex/skills/eech-regression/scripts/run_regression.py')) == pre['launcher']['sha256']
post=[]
roots=[(n,Path(p)) for n,p in pre['roots'].items()]+[('internal_lebanon',pack/'roots/lebanon-regress')]
for name,root in roots:
    assert root.is_dir(),root
    manifest=src/('reference/m4-s3-mutation/inputs-georgia.json' if name=='georgia' else 'reference/lebanon-3h/inputs.json')
    cmd=[sys.executable,'-B','-X','utf8',str(src/'tools/input-manifest.py'),str(root),'--compare',str(manifest)]
    log=b/f'ground-final-post-inputs-{name}-20261101.log'
    with log.open('xb') as f: result=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,creationflags=subprocess.CREATE_NO_WINDOW)
    post.append({'name':name,'root':str(root),'manifest_sha256':sha(manifest),'command':cmd,'actual_exit':result.returncode,'log':str(log),'log_sha256':sha(log)})
    assert result.returncode==0,post[-1]
changed=subprocess.check_output(['git','-C',str(repo),'diff','--name-only',pre['head'],'HEAD'],text=True).splitlines()
allowed=lambda n:n=='STATUS.md' or n=='eech-campaign/crates/eech-world/tests/terrain_backend.rs' or n.startswith(('eech-campaign/docs/','eech-campaign/reference/'))
assert all(map(allowed,changed)),changed
records={n:{'sha256':sha(pack/n),'bytes':(pack/n).stat().st_size} for n in ('run.json','completion.json','preflight.json','report.json','report.md','pack.stdout.log','pack.stderr.log')}
audit={'when_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'Completed ordinary shipping pack; no native re-execution. Reuses prior original/host/FP/authority evidence. Independent review/current final CI/merge still required.','runtime_source':pre['head'],'publication_head_before_final_evidence':subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip(),'run':run,'actual_runner_exit':0,'accepted_check_count':40,'accepted_check_ids':sorted(report['checks']),'all_checks_pass':True,'complete_comparison_file':'ground-aafb21775-complete-comparison-20261101.json','complete_comparison_sha256':sha(b/'ground-aafb21775-complete-comparison-20261101.json'),'all_eight_complete_artifacts_equal':True,'payloads':payloads,'all_50_payloads_equal':True,'runner_sha256':sha(src/'tools/regression-pack.py'),'launcher_sha256':pre['launcher']['sha256'],'build_info_sha256':sha(binary/'BUILD-INFO.txt'),'post_inputs':post,'publication_changes_since_runtime_source':changed,'production_inputs_unchanged':True,'pack_record_identities':records,'limits':['Complete byte comparison and established structural checks; no new visual Tacview inspection.','Source changes since aafb are metadata and independently reviewed FP test-only recovery; no new shipping binary is inferred.','Finite ground predicates/terrain-impact extraction only; remaining D3 terrain/spatial/model/environment/time responsibilities stay open.','Historical interrupted pack remains interrupted; only this fresh-root recovery has actual exit0.']}
out.write_text(json.dumps(audit,indent=2)+'\n',encoding='utf8')
print(json.dumps({'audit':str(out),'checks':40,'complete_files_equal':8,'post_input_checks':len(post),'payloads':len(payloads)}))
