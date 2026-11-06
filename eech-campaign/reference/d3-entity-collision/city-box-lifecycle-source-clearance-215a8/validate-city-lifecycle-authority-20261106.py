"""Read-only checker of the dedicated standalone CITY row; no native/Core access."""
from pathlib import Path
import argparse,copy,datetime,hashlib,json,re
MODES={'city_box_lifecycle':('none',0),'city_box_lifecycle_missing':('host_retention_release',0),'city_box_lifecycle_changed':('mirror_conformance_negative',1)}
REPORT_NUMERIC=['buildings','host_keyed','query_before_hit','query_after_hit','query_equal','list_order_restored','mirror_box_equal','empty_keyed','empty_key_changed','restored_keyed','restored_key_changed']
REPORT_TEXT=['box_min','box_max','query_before_kind','query_after_kind']
COUNTS=['consumed_before_selected','consumed_after_selected','city_stage_lookups_in_window','retained_answers','continuation_links','releases_in_window','expansions_selected','expansions_selected_answered','request_faults','reply_faults','frames']
def require(condition,message):
 if not condition:raise ValueError(message)
def finite_box_triple(text):
 if not isinstance(text,str) or re.fullmatch(r'0x[0-9a-fA-F]{8},0x[0-9a-fA-F]{8},0x[0-9a-fA-F]{8}',text) is None:return False
 return all(((int(word,16)>>23)&255)!=255 for word in text.split(','))
def check(row,mode):
 require(mode in MODES,'unknown mode');require(row['event']=='city_lifecycle_authority','wrong event')
 require(row['mode']==mode and row['profile']=='hosted','wrong mode/profile')
 require(row['row_phase']=='after_boot_warmup_standalone_fixture','wrong row phase')
 require(row['natural_weapon_engagement']=='not_claimed_terminal_witness','wrong physical-claim domain')
 require(row['unreached']=='','UNREACHED is not acceptance')
 require(isinstance(row['selected_record'],str) and bool(row['selected_record']),'no selected record')
 require(all(type(row[k]) is int and row[k]>=0 for k in COUNTS),'bad counter domain')
 require(all(type(row[k]) is int for k in REPORT_NUMERIC+['report_bound','intervened','altered_request']),'bad integer flag/report domain')
 require(row['consumed_before_selected']>=1,'selected record not consumed BEFORE')
 intervention,altered=MODES[mode];require(row['intervention']==intervention and row['altered_request']==altered,'wrong intervention domain')
 if mode=='city_box_lifecycle':
  require(row['completed'] is True and row['intervened']==0,'control incomplete/intervened')
  require(row['failure_marker']==row['failure_line']==row['first_fault']=='','control failure')
  require(row['request_faults']==row['reply_faults']==0,'control fault')
  require(row['report_bound']==1 and row['consumed_after_selected']>=1,'selected record not bound/consumed AFTER')
  require(type(row['buildings']) is int and 1<=row['buildings']<=256,'bad building count')
  require(all(row[k]==1 for k in ['host_keyed','query_equal','list_order_restored','mirror_box_equal','empty_keyed','empty_key_changed','restored_keyed','restored_key_changed']),'control proof flag missing')
  require(row['query_before_hit'] in (0,1) and row['query_after_hit']==row['query_before_hit'],'query hit mismatch')
  require(row['query_before_kind']==row['query_after_kind'],'query kind mismatch')
  require((row['query_before_kind']=='none') if row['query_before_hit']==0 else isinstance(row['query_before_kind'],str) and re.fullmatch(r'ENTITY_TYPE_[A-Z_]+',row['query_before_kind']) is not None,'bad hit/kind domain')
  require(all(finite_box_triple(row[k]) for k in ['box_min','box_max']),'box is not a finite raw triple')
  require(row['retained_answers']==row['continuation_links']==2 and row['releases_in_window']==0,'retention/relink mismatch')
  require(row['expansions_selected']==row['expansions_selected_answered']==row['buildings'],'relist expansion mismatch')
 else:
  require(row['completed'] is False and row['intervened']==1,'negative not reached exactly once')
  require(row['failure_marker']=='entity_scan.refused','wrong refusal marker')
  wanted=('[roster.link]','has ended') if mode.endswith('_missing') else ('[roster.city]','not the one the host holds')
  require(all(x in row['failure_line'] for x in wanted),'wrong named refusal')
  require(row['request_faults']==0 and row['reply_faults']==1 and 'result refused' in row['first_fault'],'wrong fault category')
  require(row['report_bound']==0 and all(row[k]==-1 for k in REPORT_NUMERIC),'postfatal report invented')
  require(all(row[k]=='' for k in REPORT_TEXT),'postfatal report strings invented')
  require(row['consumed_after_selected']==0,'postfatal selected consumption')
 return {'mode':mode,'selected_record':row['selected_record'],'completed':row['completed'],'consumed_before_selected':row['consumed_before_selected'],'consumed_after_selected':row['consumed_after_selected'],'intervention':row['intervention'],'intervened':row['intervened'],'failure_marker':row['failure_marker'],'request_faults':row['request_faults'],'reply_faults':row['reply_faults'],'phase':row['row_phase'],'frames':row['frames']}
def no_duplicates(pairs):
 d={}
 for k,v in pairs:
  require(k not in d,'duplicate JSON field '+k);d[k]=v
 return d
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--rows',type=Path);ap.add_argument('--mode',choices=list(MODES));ap.add_argument('--output',type=Path,required=True);ap.add_argument('--self-check',action='store_true');a=ap.parse_args();require(not a.output.exists(),'output already exists')
 state={'created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'checker':str(Path(__file__)),'checker_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'scope':'Read-only aggregate standalone-row validator. Full Lua report and raw query-tuple consistency rely on the reviewed driver; the row omits raw query tuples, so this is not independent reconstruction of the full report. No Core getters, native launch or whole-D3 acceptance.'}
 if a.self_check:
  base=dict(event='city_lifecycle_authority',mode='city_box_lifecycle',profile='hosted',row_phase='after_boot_warmup_standalone_fixture',natural_weapon_engagement='not_claimed_terminal_witness',unreached='',selected_record='9',intervention='none',altered_request=0,completed=True,intervened=0,failure_marker='',failure_line='',first_fault='',report_bound=1,buildings=3,host_keyed=1,query_before_hit=0,query_after_hit=0,query_before_kind='none',query_after_kind='none',query_equal=1,list_order_restored=1,mirror_box_equal=1,empty_keyed=1,empty_key_changed=1,restored_keyed=1,restored_key_changed=1,box_min='0x00000000,0x00000000,0x00000000',box_max='0x3f800000,0x3f800000,0x3f800000',consumed_before_selected=1,consumed_after_selected=1,city_stage_lookups_in_window=2,retained_answers=2,continuation_links=2,releases_in_window=0,expansions_selected=3,expansions_selected_answered=3,request_faults=0,reply_faults=0,frames=300)
  controls=[]
  for mode in MODES:
   r=copy.deepcopy(base);r['mode']=mode
   if mode!='city_box_lifecycle':
    r.update(intervention=MODES[mode][0],altered_request=MODES[mode][1],completed=False,intervened=1,report_bound=0,consumed_after_selected=0,reply_faults=1,first_fault='scan reply: result refused',failure_marker='entity_scan.refused',failure_line='[roster.link] retained record has ended' if mode.endswith('_missing') else '[roster.city] built box is not the one the host holds')
    r.update({k:-1 for k in REPORT_NUMERIC});r.update({k:'' for k in REPORT_TEXT})
   check(r,mode);controls.append(r)
  rejected=[]
  for index,field,value in [(0,'row_phase','boot'),(0,'report_bound',0),(0,'consumed_after_selected',0),(0,'retained_answers',1),(0,'box_min','garbage'),(0,'box_max','0x7f800000,0x00000000,0x00000000'),(0,'host_keyed',True),(1,'consumed_before_selected',0),(1,'unreached','not_consumed'),(1,'reply_faults',0),(1,'buildings',3),(2,'intervention','host_retention_release'),(2,'altered_request',0),(2,'failure_line','unrelated failure')]:
   r=copy.deepcopy(controls[index]);r[field]=value
   try:check(r,r['mode'])
   except ValueError:rejected.append({'mode':r['mode'],'changed_field':field,'value':value})
   else:raise ValueError('invalid checker example accepted: '+field)
  r=copy.deepcopy(controls[0]);r['query_before_kind']=r['query_after_kind']='-'
  try:check(r,r['mode'])
  except ValueError:rejected.append({'mode':r['mode'],'changed_fields':['query_before_kind','query_after_kind'],'value':'-'})
  else:raise ValueError('malformed consistent MISS kinds accepted')
  state.update(actual_exit=0,synthetic_examples_only=True,source_contract='215a dedicated row; MISS kind none and finite box word grammar taken from reviewed Lua parser',accepted_source_derived_examples=controls,rejected_examples=rejected,native_execution=False)
 else:
  require(a.rows is not None and a.mode is not None,'rows/mode required');raw=a.rows.read_bytes();rows=[json.loads(line,object_pairs_hook=no_duplicates,parse_constant=lambda x:(_ for _ in ()).throw(ValueError('non-JSON constant '+x))) for line in raw.splitlines()];selected=[r for r in rows if r.get('event')=='city_lifecycle_authority'];require(len(selected)==1,'not exactly one dedicated row');facts=check(selected[0],a.mode)
  state.update(actual_exit=0,rows=str(a.rows),raw_rows_sha256=hashlib.sha256(raw).hexdigest(),raw_rows_bytes=len(raw),facts=facts,synthetic_examples_only=False)
 a.output.write_text(json.dumps(state,indent=2)+'\n');print(json.dumps({k:state[k] for k in ['actual_exit','checker_sha256','synthetic_examples_only']}))
if __name__=='__main__':main()
