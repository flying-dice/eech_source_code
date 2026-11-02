"""Compare complete candidate diagnostics to retained closed originals, without normalization."""
from pathlib import Path
import argparse,datetime,hashlib,json
p=argparse.ArgumentParser();p.add_argument('--candidate',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();assert not a.out.exists()
b=Path('R:/d3-evidence');old_life=b/'environment-host-sound-lifetime-defaults-e50205871-20261031';old_pause=b/'environment-original-sound-pause-resume-controls-native33-driver299bcf051-20261031';rows=[]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
for name in ['natural']+[f'case-{i}' for i in range(1,15)]:
    number=0 if name=='natural' else int(name.split('-')[1]);original=(old_life if number<=10 else old_pause)/name
    old=original/'environment.jsonl';new=a.candidate/name/'environment.jsonl';assert old.is_file() and new.is_file(),name
    c=json.loads((a.candidate/name/'controller.json').read_text(encoding='utf8'))
    assert c['state']=='terminal' and c['actual_exit']==c['actual_native_exit']==0,(name,c.get('actual_exit'))
    assert [s['stage'] for s in c['stages']]==['pre-inputs','native','post-inputs'] and all(s['actual_exit']==0 and s['state']=='terminal' for s in c['stages'])
    left,right=old.read_bytes(),new.read_bytes();first=next((i for i,(x,y) in enumerate(zip(left,right)) if x!=y),None)
    if first is None and len(left)!=len(right):first=min(len(left),len(right))
    rows.append({'name':name,'original_file':str(old),'candidate_file':str(new),'original_bytes':len(left),'candidate_bytes':len(right),'original_sha256':digest(old),'candidate_sha256':digest(new),'complete_bytes_and_endings_identical':left==right,'first_difference_offset':first,'original_rows':len(left.splitlines()),'candidate_rows':len(right.splitlines()),'candidate_source':c['source'],'candidate_payloads_verified':c['verified_payloads']})
r={'when_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'scope':'All15 COMPLETE retained original diagnostic files including endings; original0..10 reused through independently closed exact-original e502 controls; original11..14 are33native/299driver. No normalization, baseline replacement, native rerun, shipping or wholeD3 claim. Input/payload/actual exits checked through candidate controllers; provenance/public authority reviewed separately.','partial_prefix_only':False,'expected_cases':15,'post_and_pre_input_checks0':30,'all_complete_bytes_equal':all(x['complete_bytes_and_endings_identical'] for x in rows),'results':rows}
with a.out.open('x',encoding='utf8',newline='\n') as f:f.write(json.dumps(r,indent=2)+'\n')
print(f'{sum(x["complete_bytes_and_endings_identical"] for x in rows)}/15 complete raw files and endings equal');raise SystemExit(0 if r['all_complete_bytes_equal'] else 1)
