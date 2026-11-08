"""Genuine old diagnostic-gate RED, never a campaign/product RED."""
from pathlib import Path
import importlib.util,copy
E=Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
spec=importlib.util.spec_from_file_location('old_gate',E/'nf3p2-OLD-normal_fire_capture.py');R=importlib.util.module_from_spec(spec);spec.loader.exec_module(R)
raw=(W/'eech-campaign/crates/eech-world/tests/data/synthetic-normal-fire-positive.txt').read_bytes()
r=R.validate(raw,{});print('COHERENT OLD POSITIVE accepted first',flush=True)
def emit(r):return ''.join('normal-capture '+k+' '+' '.join(n+'='+row[n] for n in R.SCHEMA[k].split())+'\n' for k,row in r.items()).encode()
accepted=[]
for role,field,value in [('detach','devices','garbage'),('detach','ammo_after','garbage'),('step_end','target','eech-world:3'),('models','seq','5')]:
 bad=copy.deepcopy(r);text=R.unhex(bad['evidence'][role]);before=next(x for x in text.split() if x.startswith(field+'='));text=text.replace(before,field+'='+value,1);bad['evidence'][role]='h'+text.encode().hex()
 try:R.validate(emit(bad),{});accepted.append(role+'.'+field);print('OLD GATE INCORRECTLY ACCEPTED',role,field,flush=True)
 except (ValueError,AssertionError):print('old rejected',role,field,flush=True)
assert not accepted,'EXPECTED DIAGNOSTIC RED: old healthy gate accepted '+repr(accepted)
