"""Read-only fixed26-query context accounting; no retail/candidate search."""
from pathlib import Path
import collections,datetime,hashlib,json
E=Path('R:/d3-evidence');raw=Path('D:/EECH-Migration-Evidence/normal-query-capture-b502-20261108-runs/01-healthy/capture.txt.events')
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
before=sha(raw);fires={};births={};step=None;query=None;queries=[];reasons=collections.Counter();ops=collections.Counter();seq=0;last=0
with raw.open() as stream:
 for line in stream:
  if not line.startswith('normal-ev '):continue
  e=dict(x.split('=',1) for x in line.split()[1:]);assert int(e['seq'])==seq+1;seq+=1;last=int(e['tick']);kind=e['kind']
  if kind=='normal_fire':fires[e['fire']]=e
  elif kind=='normal_birth' and e['fire']!='0':births[e['w']]=e
  elif kind=='normal_step':step={'start':e,'queries':[]}
  elif kind=='normal_query':query={'event':e,'birth':births.get(e['w']),'start':step['start'],'ops':collections.Counter()};step['queries'].append(query)
  elif kind=='normal_model':query['ops'][e['op']]+=1;ops[e['op']]+=1
  elif kind=='normal_query_end':query['end']=e
  elif kind=='normal_step_end':
   for q in step['queries']:
    b=q['birth'];f=fires.get(b['fire']) if b else None;s=q['start'];reason='live_host_motion_and_candidate_answers_missing'
    if not b:reason='no_normal_birth'
    elif not f:reason='no_normal_fire'
    elif f['launcher']=='none' or f['target']=='none':reason='old_noncanonical_fire'
    elif int(f['tick'])//100+1<=300:reason='birth_before_warmup'
    elif s['target']!=b['target']:reason='old_same_birth_target_predicate'
    elif s['launcher']!=b['launcher']:reason='step_launcher_differs_birth'
    elif e['target']!=b['target']:reason='old_end_target_predicate'
    reasons[reason]+=1;queries.append(dict(query=int(q['event']['query']),selector=int(q['event']['which']),tick=int(q['event']['tick']),old_predicate_result=reason,step_target_none=s['target']=='none',end_target_none=e['target']=='none',hit=int(q['end']['hit']),model_ops=dict(q['ops'])))
assert before==sha(raw) and len(queries)==26
result=dict(utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),raw=dict(path=str(raw),bytes=raw.stat().st_size,sha256=before),complete_EOF=True,consecutive_NF_records=seq,last_tick_ms=last,fixed_queries=len(queries),old_stronger_predicate_counts=dict(reasons),model_ops=dict(ops),safe_query_facts=queries,new_contract_missing_fact='Actual registered moving routed candidate/NeedUpdated/READ/reask/TEST facts are absent from the retained C opcode stream for all26 queries; no inferred eligibility or new actor/window.',source_rationale='wn_move scans get_line_of_sight_collision_entity(source,NULL,...), independent of birth/current target. Target already none at24 query step starts; raw lacks unlink cause. No source branch guessed.',scope='Read-only existing26-record analysis, no native/default replay; old1500FAIL staysFAIL. No coordinates/poses/host identities published.')
out=E/'pfb502-fixed-query-analysis-20261108.json'
with out.open('x') as f:json.dump(result,f,indent=2)
print(json.dumps({k:v for k,v in result.items() if k not in ['safe_query_facts','raw']}))
