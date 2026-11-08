"""Offline full selected-event codec scan; private raw bytes never published."""
from pathlib import Path
import collections,datetime,hashlib,importlib.util,json,sys
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
raw=Path('D:/EECH-Migration-Evidence/normal-fire-capture-56d1b-20261108-runs/01-healthy/capture.txt.events')
validator=W/'eech-campaign/tools/normal_fire_capture.py';label=sys.argv[1];assert label in ['RED','GREEN']
def sha(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1048576),b''):h.update(b)
 return h.hexdigest()
before=sha(raw);spec=importlib.util.spec_from_file_location('codec',validator);R=importlib.util.module_from_spec(spec);spec.loader.exec_module(R)
kinds=collections.Counter();selectors=collections.Counter();faults=collections.Counter();examples=[];selected=0;lines=0;sequence=0;last_tick=0
with raw.open('r',encoding='utf-8',newline='') as f:
 for lines,line in enumerate(f,1):
  assert line.endswith('\n') and '\r' not in line;line=line[:-1]
  if line.startswith('wpn-ev '):kinds['legacy_wpn']+=1;continue
  assert line.startswith('normal-ev ');e=dict(x.split('=',1) for x in line.split()[1:]);kind=e['kind'];kinds[kind]+=1
  assert int(e['seq'])==sequence+1;sequence+=1;last_tick=int(e['tick'])
  if kind=='normal_query':selectors[e['which']]+=1
  if kind not in R.EVENTS:
   assert kind in ['normal_end','normal_error']
   if kind=='normal_error':faults['native_observer_error']+=1
   continue
  selected+=1
  try:R.event(line)
  except (ValueError,AssertionError) as error:
   faults[str(error)]+=1
   if len(examples)<5:examples.append(dict(file_line=lines,seq=sequence,tick=last_tick,kind=kind,which=e.get('which'),error=str(error)))
assert before==sha(raw)
code=1 if faults else 0
result=dict(utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),label=label,validator=dict(path=str(validator),sha256=sha(validator)),raw=dict(path=str(raw),bytes=raw.stat().st_size,sha256=before),complete_EOF=True,total_lines=lines,normal_records=sequence,selected_codec_records=selected,kinds=dict(kinds),query_selector_counts=dict(selectors),fault_counts=dict(faults),first_safe_fault_projections=examples,last_tick_ms=last_tick,result='FAIL' if code else 'PASS',scope='Read-only complete retained raw selected-event codec scan, not N.feed/provider-order replay or native acceptance. normal_end/error intentionally outside Python EVENTS; observer errors separately fail. No coordinates/physical ids copied.')
out=E/('nq56-retained-'+label+'-20261108.json')
with out.open('x') as f:json.dump(result,f,indent=2)
print(json.dumps({k:v for k,v in result.items() if k not in ['raw','validator']}));raise SystemExit(code)
