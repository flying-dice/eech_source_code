"""Audit complete recovered original-init facts, independently of wrapper conclusions."""
from pathlib import Path
import datetime,hashlib,json,re,sys
b=Path('R:/d3-evidence');source=sys.argv[1];q=source[:9];out=Path(sys.argv[2]);assert not out.exists()
def pairs(items):
    d={}
    for k,v in items:
        if k in d:raise ValueError('duplicate JSON key '+k)
        d[k]=v
    return d
def parse(text):return json.loads(text,object_pairs_hook=pairs,parse_constant=lambda v:(_ for _ in ()).throw(ValueError(v)))
def read(p):return parse(p.read_text(encoding='utf8'))
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def receipt(p):return dict(file=p.name,bytes=p.stat().st_size,sha256=sha(p))
base=b/f'environment-original-sound-init-recovery-{q}-20261101';aggregate=read(base/'controller.json')
assert aggregate['source']==source and aggregate['state']=='terminal' and aggregate['actual_exit']==0
assert len(aggregate['stages'])==2 and all(s['state']=='terminal' and s['actual_exit']==0 for s in aggregate['stages'])
review_path=b/f'independent-environment-original-sound-init-capacity-source-review-{q}-20261101.json';review=read(review_path)
assert review['source']['head']==source and review['source_review_no_blocking_findings'] and not review['findings']
builds={}
for kind in ['windows','linux']:
    path=b/f'environment-original-sound-init-{q}-{kind}-20261101-completion.json';r=read(path)
    assert r['source']==source and r['state']=='terminal' and r['actual_exit']==0
    builds[kind]=dict(actual_exit=0,completed_utc=r['completed_utc'],receipt=receipt(path))
io_path=b/f'sound-init-exclusive-output-native-{q}-20261101/completion.json';io=read(io_path)
assert io['source']==source and io['state']=='terminal' and io['actual_exit']==io['actual_native_exit']==0
assert io['ascii_bytes_preserved'] and io['binary_nul_and_lf_bytes_preserved'] and io['payloads_and_script_unchanged']
ci_path=b/f'current-CI-{q}-20261101.json';ci=read(ci_path);jobs=[j for group in ci['jobs'] for j in group['jobs']]
assert ci['source']==source and len(jobs)==9 and all(j['status']=='completed' and j['conclusion']=='success' for j in jobs)
snapshot=b/f'environment-original-sound-init-source-{q}';binary=snapshot/f'eech-campaign/target/env-observer/sound-init-original-{q}'
info=binary/'BUILD-INFO.txt';text=info.read_text(encoding='utf8');entries=re.findall(r'^  ([0-9a-f]{64})  (.+)$',text,re.M)
wanted={'eech_dc.dll','eech-world.exe','lua.dll'}|{'lua/'+p.name for p in (snapshot/'eech-campaign/crates/eech-world/lua').glob('*.lua')}
assert len(entries)==len(wanted) and {n for _,n in entries}==wanted and all(sha(binary/n)==h for h,n in entries)
assert 'commit: '+source in text and 'uncommitted changes in the build inputs: 0 files' in text
controllers={};input_checks=0
for name,script in [('probe','env-sound-init.lua'),('observer-off','env-characterization.lua')]:
    c=read(base/name/'controller.json');assert c['source']==source and c['script']==script
    assert c['state']=='terminal' and c['actual_exit']==c['actual_native_exit']==0 and 'controller_error' not in c
    assert 'script_override' not in c and c['verified_payloads']==len(entries) and c['build_info_sha256']==sha(info)
    assert [s['stage'] for s in c['stages']]==['pre-inputs','native','post-inputs'] and all(s['state']=='terminal' and s['actual_exit']==0 for s in c['stages'])
    input_checks+=2
    assert sha(base/name/'environment.jsonl')==c['json_sha256']
    controllers[name]=dict(actual_native_exit=0,actual_controller_exit=0,controller=receipt(base/name/'controller.json'),raw_json=receipt(base/name/'environment.jsonl'))
old=b/'environment-host-pause-defaults-82234a3a3-20261101/natural/environment.jsonl';new=base/'observer-off/environment.jsonl';assert old.read_bytes()==new.read_bytes()
comparison=aggregate['observation_off_complete_comparison'];assert comparison['complete_bytes_and_endings_identical'] and comparison['partial_prefix_only'] is False
assert comparison['original_sha256']==comparison['candidate_sha256']==sha(old)==sha(new)
raw=base/'probe/environment.jsonl';rows=[parse(line) for line in raw.read_text(encoding='utf8').splitlines()]
of=lambda event:[r for r in rows if r.get('event')==event]
def one(event):
    values=of(event);assert len(values)==1,event;return values[0]
assert not any(of(e) for e in ['witness_failure','engine_failure','refusal'])
result=one('result');assert result['frames']==1200 and result['changed']=='0' and result['read_only']=='1'
raw_report=one('raw_report');sidecar=Path(raw_report['sidecar']);assert sidecar==base/'probe/environment.jsonl.case15.raw'
native_rows=[r for r in rows if 'raw' in r]
returned=b''.join((r['raw']+'\n').encode('ascii') for r in native_rows)
assert returned==sidecar.read_bytes() and len(returned)==raw_report['bytes']
for row in native_rows:
    words=row['raw'].split();assert words[0]==row['event']
    for word in words[1:]:
        k,v=word.split('=',1);assert row[k]==v
store=one('env-sndinit-store');assert store['dropped']==store['orphans']==store['saturated']=='0'
fingerprint=one('env-sndinit-fingerprint');assert fingerprint['unchanged']=='1'
for stem in ['rng','entity','local']:
    suffix='_w' if stem=='rng' else '';assert fingerprint[stem+'_before'+suffix]==fingerprint[stem+'_after'+suffix]
ctor=one('env-sndinit-ctor');samples=of('env-sndinit-sample');keys=[int(r['index']) for r in samples]
assert len(keys)==len(set(keys))==int(ctor['distinct']) and all(0<=k<int(store['catalogue_entries']) for k in keys)
assert sum(int(r['uses']) for r in samples)==int(ctor['sample_reads']) and ctor['out_of_domain']=='0'
prior_path=b/'original-sound-init-attempt1-failure-and-control-20261101.json';prior=read(prior_path)
assert prior['actual_pair_exit']==1 and prior['controls']['probe']['actual_native_exit']==1 and prior['controls']['observer-off']['actual_native_exit']==0
observed_old=dict(re.findall(r'\b(calls|looping|max_count|sample_reads|digest_w)=([^ ]+)',prior['probe_failure']['reason']))
assert set(observed_old)=={'calls','looping','max_count','sample_reads','digest_w'} and all(ctor[k]==v for k,v in observed_old.items())
safe=[]
for r in rows:
    if r['event']=='env-sndinit-fingerprint':safe.append(dict(event=r['event'],unchanged=True));continue
    if r['event']=='raw_report':safe.append(dict(event=r['event'],bytes=r['bytes']));continue
    safe.append({k:v for k,v in r.items() if k!='raw'})
answer=dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source=source,scope='Recovered ORIGINAL initialization/sample-data characterization, no host metadata/backend ownership move or shipping/Tacview/Task51/D3 acceptance.',all_measured_gates_pass=True,independent_actual_review_required=True,builds=builds,source_review=receipt(review_path),windows_output_probe=receipt(io_path),current9_ci=receipt(ci_path),native_payloads_verified=len(entries),build_info_sha256=sha(info),controls=controllers,pre_post_input_checks0=input_checks,complete_native_report_sidecar=receipt(sidecar),all_returned_native_bytes_join_json_raw_rows=True,observation_off_complete_byte_comparison=comparison,old_failed_capture=receipt(prior_path),retained_old_ctor_totals_and_digest_equal=observed_old,measured_probe_projection=safe,limitations=['Original full attempt1 in-memory report remains UNAVAILABLE; recovery does not recover it. Only retained old ctor summary/digest are compared.','New JSON includes sidecar path provenance; no claim that whole new wrapper JSON equals unavailable old capture. Complete native sidecar and oldoff whole bytes have separate proofs.','No positive device/playing handle, loader data reads, other compiled consumer liveness or reinitialization measured. No fake handle or absence-of-assets inference.','Earlier development/raw gaps, failed red/compile/lint/command/preprocess attempts retained.','Metadata/assets/per-family/backend/destroy/network/pitch/amplitude/time/terrain/spatial/model duties remain D3; campaign policy/calendar/timers/scheduling stay Core.'])
with out.open('x',encoding='utf8') as stream:stream.write(json.dumps(answer,indent=2)+'\n')
print(json.dumps(dict(source=source,controls=2,input_checks0=input_checks,oldoff_complete_bytes_equal=True,native_sidecar_bytes=sidecar.stat().st_size,old_ctor_digest_equal=True,probe_rows=len(rows),actual_review_required=True)))
