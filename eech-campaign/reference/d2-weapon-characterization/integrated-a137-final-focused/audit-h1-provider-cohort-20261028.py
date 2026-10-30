"""Audit actual provider summaries/refusal reasons and native completion records.

No physics/golden changes. Short altered modes do not establish causality against
the longer reference; zero request counts are coverage gaps, never successes.
"""
import datetime,hashlib,json,pathlib,re,sys
base=pathlib.Path('R:/d2-evidence/weapon-characterization-20261027')
cases=[('h1-pref','reference',600,None),('h1-pnever','reacquire_never',600,None),('h1-pown','own_model',180,None),('h1-pseduce','seduce_never',180,None),('h1-pfast','weapon_fast',180,None),('h1-prelslow','release_slow',180,None),('h1-pdrag','weapon_drag',180,None),('h1-pshort','weapon_short',180,None),
('h1-pbss','bad_step_state',180,'not done, need_draw or need_lookup'),('h1-pbsl','bad_step_lookup_kind',180,'malformed lookup (kind 99)'),('h1-pbsa','bad_step_action_kind',180,'malformed action (kind 99)'),('h1-pbsn','bad_step_nonfinite',180,'must be 12 finite numbers'),('h1-pbsr','bad_step_repeat',180,'again for a lookup it was already given'),('h1-pbse','bad_step_error',180,'provider failed on purpose'),('h1-pbsd','bad_step_draw',180,'more than 8 draws'),('h1-pbba','bad_birth_action',180,'campaign does not apply to a birth'),('h1-pbrb','bad_release_burst',180,'0 rounds and no a last-shot clock'),('h1-pbdf','bad_decoy_flag',600,'weapon_decoy request with a non-boolean decision 2'),('h1-pbsf','bad_seduce_flag',600,'weapon_seduce request with non-boolean decisions 0 and 3'),('h1-pbrf','bad_reacquire_flag',600,'weapon_reacquire request with a non-boolean decision -1'),('h1-pbrn','bad_reacquire_none',600,'not the one decision the campaign applies')]
log=(base/'h1-queue-b.log').read_text(encoding='utf-8-sig')
results=[];errors=[]
for name,mode,seconds,reason in cases:
    p=base/'out'/name/'summary.jsonl'
    if not p.exists() or not p.stat().st_size:
        errors.append(f'{name}: no complete summary');continue
    rows=[json.loads(l) for l in p.read_text().splitlines()]
    if len(rows)!=1 or rows[0].get('event')!='summary':
        errors.append(f'{name}: expected one real summary');continue
    row=rows[0];failure=row.get('failure','')
    end=re.findall(rf'^([^\r\n]+) end {re.escape(name)} : native exit=(\d+) after ([\d,]+) s$',log,re.M)
    start=re.findall(rf'^([^\r\n]+) start {re.escape(name)} \(prov {re.escape(mode)}\).*seconds={seconds} .*mode={re.escape(mode)} ',log,re.M)
    ok=len(end)==1 and end[0][1]=='0' and len(start)==1 and row.get('mode')==mode
    if reason:
        ok=ok and reason in failure and 0<row.get('frames',0)<seconds*10
    else:
        ok=ok and not failure and row.get('frames')==seconds*10+1
    if not ok:errors.append(f'{name}: contract/completion mismatch')
    results.append(dict(name=name,mode=mode,seconds=seconds,native_end=end,passed=ok,summary=row,summary_bytes=p.stat().st_size,summary_sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
queue_complete='PLAN COMPLETE: 21 runs, every actual exit 0' in log
if not queue_complete:errors.append('Queue has no actual all21 completion record')
proof=dict(when_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),scope='Actual provider native exits and exact expected refusals; process outer completion, postinputs and build identity separate',source='a137fc3ec1963d88de5f0b8562145b358b50e111',results=results,queue_complete=queue_complete,errors=errors,passed=not errors,limitations=['180-second altered modes are not causal comparisons to 600-second reference.','seduce_never180 has zero seduction requests; release_slow180 shows no observed effect. Fresh matched activity controls required.','own_model180 has zero births; positive generic birth/release/flight/animate/decoy evidence is the separate controlled h1-case-own.','Native0 in malformed cases means script observed refusal, not successful campaign completion.'])
(base/'root-h1-provider-cohort-audit.json').write_text(json.dumps(proof,indent=2)+'\n')
print(json.dumps(dict(passed=proof['passed'],completed=len(results),errors=errors)))
sys.exit(0 if proof['passed'] else 1)
