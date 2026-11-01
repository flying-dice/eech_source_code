"""Read-only audit and safe projection of the new candidate's completed controls."""
from pathlib import Path
import argparse,datetime,hashlib,json,re
p=argparse.ArgumentParser();p.add_argument('--source',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists()
b=Path('R:/d3-evidence');q=a.source[:9];defaults=b/f'environment-host-pause-defaults-{q}-20261101';authority=b/f'environment-host-pause-authority-{q}-20261101'
def pairs(items):
    result={}
    for k,v in items:
        if k in result:raise ValueError('duplicate JSON key '+k)
        result[k]=v
    return result
def read(path):return json.loads(path.read_text(encoding='utf8'),object_pairs_hook=pairs,parse_constant=lambda v:(_ for _ in ()).throw(ValueError(v)))
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def receipt(path):return dict(file=path.name,bytes=path.stat().st_size,sha256=sha(path))
dc,ac,comparison=[read(path) for path in (defaults/'controller.json',authority/'controller.json',defaults/'complete-raw-comparison.json')]
assert dc['state']==ac['state']=='terminal' and dc['actual_exit']==ac['actual_exit']==0
assert dc['source']==ac['source']==a.source and len(dc['stages'])==15 and len(ac['stages'])==7
assert dc['controls_actual_exit']==dc['strict_complete_raw_comparison_actual_exit']==0
assert comparison['expected_cases']==15 and comparison['all_complete_bytes_equal'] and len(comparison['results'])==15 and comparison['partial_prefix_only'] is False
review_path=b/f'independent-environment-host-pause-integrated-source-review-{q}-20261101.json';review=read(review_path)
assert review['source']['head']==a.source and not review['findings'] and review['source_review_no_blocking_findings']
ci_path=b/f'current-CI-{q}-20261101.json';ci=read(ci_path);jobs=[j for group in ci['jobs'] for j in group['jobs']]
assert ci['source']==a.source and len(jobs)==9 and all(j['status']=='completed' and j['conclusion']=='success' for j in jobs)
builds={}
for kind in ['windows','linux']:
    path=b/f'environment-host-sound-pause-{q}-{kind}-20261101-completion.json';r=read(path)
    assert r['source']==a.source and r['state']=='terminal' and r['actual_exit']==0
    builds[kind]=dict(actual_exit=0,completed_utc=r['completed_utc'],receipt=receipt(path))
manifests={}
for native_source,snapshot,native in [
    (a.source,b/f'environment-host-sound-pause-source-{q}',b/f'environment-host-sound-pause-source-{q}/eech-campaign/target/env-observer/host-sound-pause-{q}'),
    ('33c015557fb3fbc26592e20b143ada40ec155e1f',b/'environment-original-sound-pause-resume-source-33c015557',b/'environment-original-sound-pause-resume-source-33c015557/eech-campaign/target/env-observer/sound-pause-resume-original-33c015557')]:
    info=native/'BUILD-INFO.txt';text=info.read_text(encoding='utf8');entries=re.findall(r'^  ([0-9a-f]{64})  (.+)$',text,re.M)
    wanted={'eech_dc.dll','eech-world.exe','lua.dll'}|{'lua/'+p.name for p in (snapshot/'eech-campaign/crates/eech-world/lua').glob('*.lua')}
    assert len(entries)==len(wanted) and {name for _,name in entries}==wanted
    assert 'commit: '+native_source in text and 'uncommitted changes in the build inputs: 0 files' in text
    assert all(sha(native/name)==expected for expected,name in entries)
    manifests[native_source]=dict(payloads=len(entries),build_info_sha256=sha(info),files=sorted(wanted))
assert manifests[a.source]['payloads']==61 and manifests['33c015557fb3fbc26592e20b143ada40ec155e1f']['payloads']==56
new_wrapper_sha256=sha(b/f'environment-host-sound-pause-source-{q}/eech-campaign/target/env-observer/host-sound-pause-{q}/lua/env-sound-host-witness.lua')
input_checks=0
def controller(c,expected):
    global input_checks
    assert c['state']=='terminal' and c['actual_exit']==c['actual_native_exit']==expected and 'controller_error' not in c
    assert [s['stage'] for s in c['stages']]==['pre-inputs','native','post-inputs']
    assert c['verified_payloads']==manifests[c['source']]['payloads'] and c['build_info_sha256']==manifests[c['source']]['build_info_sha256']
    for stage in c['stages']:
        assert stage['state']=='terminal' and stage['actual_exit']==(expected if stage['stage']=='native' else 0)
        if stage['stage']!='native':input_checks+=1
default_rows=[]
for item in comparison['results']:
    name=item['name'];c=read(defaults/name/'controller.json');controller(c,0)
    assert c['source']==a.source and item['candidate_source']==a.source and item['complete_bytes_and_endings_identical']
    old,new=Path(item['original_file']),Path(item['candidate_file']);assert new.read_bytes()==old.read_bytes()
    assert sha(new)==c['json_sha256']==item['candidate_sha256']==item['original_sha256']
    if name!='natural':assert c['script_override_source']=='299bcf05128da68e85c8490025594f99969335b8' and c['script_override_sha256']==c['script_override_sha256_after']=='7b1f3d491a200b7cd2fd18f217bb120fab43971be39ae6890186fc5ca074a24e'
    default_rows.append(item)
authority_rows=[]
safe_keys=['mode','op','case','completed','delegate_ran','requests','boot_requests','forwarded_other_requests','altered','verdict','calls','creation_life_w','kills','life_words','returns','state_words','marker','stopped']
for stage in ac['stages']:
    mode=stage['mode'];old=mode=='old-capability-red';expected=1 if old else 0;c=read(authority/mode/'controller.json');controller(c,expected)
    assert stage['state']=='terminal' and stage['actual_exit']==expected
    assert c['source']==('33c015557fb3fbc26592e20b143ada40ec155e1f' if old else a.source)
    assert stage['delegate_source']=='299bcf05128da68e85c8490025594f99969335b8'
    assert stage['delegate_sha256_before']==stage['delegate_sha256_after']=='7b1f3d491a200b7cd2fd18f217bb120fab43971be39ae6890186fc5ca074a24e'
    assert stage['wrapper_sha256']==stage['wrapper_sha256_after']==new_wrapper_sha256
    path=authority/mode/'environment.jsonl.authority.jsonl';answer=read(path)
    assert answer==stage['authority'] and sha(path)==stage['authority_sha256'] and answer['op']=='sound_pause_resume'
    assert answer['requests']==0 if old else answer['requests']>0
    assert answer['boot_requests']==0
    raw_path=authority/mode/'environment.jsonl';raw=raw_path.read_bytes();rows=[json.loads(line,object_pairs_hook=pairs) for line in raw.decode('utf8').splitlines()]
    failures=[r for r in rows if r.get('event')=='engine_failure'];results=[r for r in rows if r.get('event')=='result']
    if old or mode=='pause_healthy':
        original=b/'environment-original-sound-pause-resume-controls-native33-driver299bcf051-20261031/case-12/environment.jsonl'
        assert raw==original.read_bytes() and stage['complete_original_case12_bytes_equal']
        assert len(results)==1 and not failures and results[0]['case']==12
    if mode in ['pause_withheld','pause_malformed','pause_incoherent']:
        suffix='callback' if mode=='pause_withheld' else 'answer';marker=f'[env-provider:sound_pause_resume.{suffix}]'
        assert len(failures)==1 and not results and marker in failures[0]['reason'] and answer['marker']==marker and answer['stopped'] is True
        assert failures[0]['i']==(13 if mode=='pause_incoherent' else 11)
    if mode in ['pause_healthy','pause_changed','resume_changed']:
        states=[r for r in rows if r.get('event')=='env-sound-state' and r.get('ent')=='0']
        steps=[r for r in rows if r.get('event')=='env-sound-step' and r.get('op') in ['1','2']]
        observed_states=','.join(r['valid']+r['member']+':'+r['vlife_w'] for r in states)
        observed_lives=','.join(r['life_w'] for r in states);observed_returns=','.join(r['returned'] for r in steps)
        assert answer['state_words']==observed_states and answer['life_words']==observed_lives and answer['returns']==observed_returns
        if mode=='pause_changed':assert observed_states=='11:00000000,01:40400000,11:40400000' and answer['altered']==1
        if mode=='resume_changed':assert observed_states=='11:00000000,01:3fc00000,01:3f000000,01:00000000,00:00000000,10:00000000,10:00000000' and answer['altered']==1 and observed_lives==','.join(['40a00000']*7)
    safe={k:answer[k] for k in safe_keys if k in answer}
    if safe.get('state_words'):assert re.fullmatch(r'[01]{2}:[0-9a-f]{8}(,[01]{2}:[0-9a-f]{8})*',safe['state_words'])
    authority_rows.append(dict(mode=mode,actual_native_exit=expected,actual_controller_exit=expected,verified_payloads=c['verified_payloads'],raw=receipt(raw_path),authority_receipt=receipt(path),semantic_authority=safe,actual_delegate_sha256=stage['delegate_sha256_after'],actual_wrapper_sha256=stage['wrapper_sha256_after']))
assert input_checks==44
result=dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),source=a.source,scope='Finite host pause/resume extraction on accepted ground integration: all15same-original COMPLETE diagnostic byte comparisons and7public ownership/refusal controls. No normalization or baseline replacement. Private raw/vector/entity-index data excluded from this safe projection.',all_measured_gates_pass=True,independent_new_actual_review_required=True,builds=builds,payload_manifests=manifests,source_review=receipt(review_path),current9_ci=receipt(ci_path),native_controls=22,pre_post_input_checks0=input_checks,all15_complete_original_raw_bytes_equal=True,complete_comparison=receipt(defaults/'complete-raw-comparison.json'),defaults=default_rows,ownership=authority_rows,retained_failures='Second Root auditor actual1 counted update op3 in pause/resume returns; v3 derives pause/resume returns only from actual tagged ops1/2, matching frozen299 result semantics, while all raw bytes/updates remain retained. First Root auditor actual1 assumed56 for new combined native; it has61 verified payloads (five newly integrated ground scripts), old33 retains56. Original script70b66c179fa254a28bcb88d7c821e8e9edd4cbbe23bc126df2a4c5457d83a79c and after-call failure receipt retained; no native rerun or raw modification. Original hostpause d1engine101/e1fmt1; integration b1fmt1/missingDLL, d1undefinedsymbolOPT0, e1missingDLL in newOPT1target and fmtc1shellsyntax2 all retained failed. Attribution ofOPT0failure to prior inline linkage remains unverified baseline inference. Earlier firstsoundd1 raw unavailable/partialonly; oldd2/d3/c2tags untrusted.',remaining_core_coupling=['metadata/assets and sound database','private mirrors and list/heap adapters','non-session range and per-family playback','backend/device/handles','independent destroy/network/pitch/amplitude','remaining physical time and required terrain/spatial/model services'],delivery_limits='No Task51 shipping40/all8complete Tacview/observation/metric or wholeD3/normaldelivery acceptance. Separate accepted ground pack is not this source. D4/D5 unstarted.')
with a.out.open('x',encoding='utf8',newline='\n') as f:f.write(json.dumps(result,indent=2)+'\n')
print(json.dumps(dict(source=a.source,controls=22,input_checks0=44,all15_complete_raw_equal=True,actual_review_required=True)))
