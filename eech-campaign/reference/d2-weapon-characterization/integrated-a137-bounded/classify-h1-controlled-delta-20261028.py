"""Describe the complete raw difference; does not alter goldens or comparators."""
import datetime,hashlib,json
from pathlib import Path
w=Path('R:/d2-evidence/weapon-characterization-20261027')
a=w/'out/w4-case-a/trace.jsonl'
b=w/'out/h1-case-a/trace.jsonl'
x=a.read_bytes().splitlines(keepends=True)
y=b.read_bytes().splitlines(keepends=True)
assert len(x)==len(y)==5762
differences=[]
for index,(old,new) in enumerate(zip(x,y),1):
    if old==new: continue
    left,right=json.loads(old),json.loads(new)
    assert left.keys()==right.keys()
    fields=[k for k in left if left[k]!=right[k]]
    assert fields in (['text'],['detail'])
    field=fields[0]
    assert (index,left['event'],field) in ((519,'case_launch','text'),(520,'check','detail'))
    # Exact, source-specific diagnostic classification; this is NOT a tolerance or
    # a normalized acceptance artifact. Raw original/candidate hashes remain unequal.
    assert right[field]==left[field].replace(':161:',':178:').replace(':240:',':257:')
    differences.append(dict(line=index,event=left['event'],field=field,old=left[field],new=right[field]))
assert len(differences)==2
physical_old=[s for s in x if str(json.loads(s).get('event','')).startswith('wpn-')]
physical_new=[s for s in y if str(json.loads(s).get('event','')).startswith('wpn-')]
assert physical_old==physical_new
before=json.loads((w/'root-independent-a137-clean-build.json').read_text(encoding='utf-8-sig'))
after=json.loads((w/'root-post-h1-case-build.json').read_text(encoding='utf-8-sig'))
assert after['passed'] and before['manifest_sha256']==after['manifest_sha256']
assert 'inputs IDENTICAL' in (w/'root-h1-case-post-input.log').read_text(encoding='utf-8-sig')
record=dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    source='a137fc3ec1963d88de5f0b8562145b358b50e111',build='a137fc3ec-clean fixture dirty0',
    actual_native_and_driver_exit=0,completed_utc='2026-10-28T02:30:52Z',wall_seconds=1248,
    configuration=dict(warmup=60,settle=700,frame_ms=100,seed=1),
    strict_wpn_case_verifier_actual_exit=0,checks=11,failed=0,
    raw_entire_trace_equal=False,bytes_each=b.stat().st_size,lines_each=len(x),
    original_sha256=hashlib.sha256(a.read_bytes()).hexdigest(),candidate_sha256=hashlib.sha256(b.read_bytes()).hexdigest(),
    raw_equal_lines=5760,physical_wpn_events_each=len(physical_old),physical_wpn_raw_equal=True,
    differences=differences,classification='Two unsupported-launch rejection diagnostics have Lua source positions shifted +17 by added own-model setup; all other raw lines exact. Classification awaits same independent final-evidence review.',
    unchanged='No golden, tolerance, comparator, original/candidate raw or product source changed for classification; no healthy campaign rerun.',
    post1179_input_manifest_actual_exit=0,post35_build_manifest_actual_exit=0,BUILD_INFO_unchanged=True,
    remaining='Generic-host positive activity/refusals; full ordinary40/all8/post3; independent proof/CI/normalmerge45. Outer cohort74298 still running.')
(w/'root-h1-controlled-completion.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:record[k] for k in ['actual_native_and_driver_exit','checks','failed','raw_entire_trace_equal','raw_equal_lines','physical_wpn_events_each','physical_wpn_raw_equal']},indent=2))
