"""Strict offline report validator for the seven-case nr395 public READ witness.
No engine/binary/world action. Raw reports stay immutable; parsing never executes
their contents. The Lua driver independently validates before any child effect.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path

RUNTIME = '3abf03e5133279ae6ed53b657e76d4f7724f7dd3'
PLAN = '1c472eec613c09041491617d692c91b957568a21d18d9137c316ea1cfebe7352'
MODES = ['read_probe_control', 'read_changed', 'read_withheld', 'read_malformed', 'read_wrong_state', 'read_wrong_array', 'read_wrong_word']
MARKERS = dict(zip(MODES[2:], ['entity_scan.callback','entity_scan.answer','entity_scan.state','flag.answer','flag.word']))
SCHEMA = {
 'identity':'schema mode profile scenario seed frame_ms hours campaign map paired runtime lua protocol plan bundle reference strategy',
 'counts':'frames attempts_group attempts_set attempts_read attempts_reset answered_group answered_set answered_read answered_reset accepted_group accepted_set accepted_read accepted_reset altered changed_accepted keys_issued reads_zero reads_mask reads_consumable resets_cleared oracle_sets oracle_reads oracle_resets oracle_mismatches candidate_questions bind_calls',
 'oracle':'sets reads resets mismatches',
 'first':'event ordinal frame before_group before_set before_read before_reset',
 'consume':'fact expected host real_word answered_word changed',
 'selection':'tag scene id name member group bind candidate frame entry original pose request group_opened group_acked key array lane ordinal before_group before_set before_read before_reset words',
 'store':'key array lane before mask real answered after',
 'reask':'words fact',
}
ORDER = {name:i for i,name in enumerate(['identity','counts','oracle','first','consume','selection','store','reask','intervention','fatal','reply-fault','request-fault','verdict'])}
ZERO, ONE, TWO = '0x00000000','0x3f800000','0x40000000'

class Invalid(ValueError): pass
def check(ok, why):
    if not ok: raise Invalid(why)
def uint(s):
    check(isinstance(s,str) and re.fullmatch(r'0|[1-9][0-9]*',s),'noncanonical whole number')
    n=int(s);check(n<=9007199254740991,'whole number too large');return n
def word(s):
    check(isinstance(s,str) and re.fullmatch(r'0x[0-9a-f]{8}',s),'noncanonical raw word');return int(s[2:],16)
def words(s,count=None):
    out=[word(x) for x in s.split(',')];check(count is None or len(out)==count,'wrong raw word count');return out
def unhex(s):
    check(re.fullmatch(r'h(?:[0-9a-f]{2})*',s) is not None,'noncanonical encoded text')
    return bytes.fromhex(s[1:]).decode('utf-8')
def fact(raw): return ONE if raw==ZERO else TWO
def parse(raw):
    check(isinstance(raw,bytes) and len(raw)<=131072 and raw.endswith(b'\n') and b'\r' not in raw and b'\n\n' not in raw,'noncanonical report framing')
    result={};rank=-1
    for line in raw.decode('utf-8').splitlines():
        m=re.fullmatch(r'pf-read-([a-z-]+) (.+)',line);check(m is not None,'unknown report line')
        kind,body=m.groups();check(kind in ORDER and ORDER[kind]>=rank,'unknown/out-of-order record');rank=ORDER[kind]
        if kind=='verdict':
            v=re.fullmatch(r'mode=(\S+) status=(PASS|FAILURE|UNREACHED) detail=(.+)',body);check(v is not None,'malformed verdict')
            fields=dict(zip(['mode','status','detail'],v.groups()))
        elif kind in ('reply-fault','request-fault'): fields={'detail':body}
        else:
            keys=SCHEMA.get(kind, 'at entry callback_after' if kind=='intervention' else 'where marker entry line').split()
            if kind=='fatal':
                v=re.fullmatch(r'where=(\S+) marker=(\S+) entry=(\S+) line=(.+)',body);check(v is not None,'malformed fatal');fields=dict(zip(keys,v.groups()))
            else:
                tokens=body.split(' ');check(len(tokens)==len(keys),'wrong field count')
                fields={}
                for k,t in zip(keys,tokens):
                    pair=t.split('=',1);check(len(pair)==2 and pair[0]==k and pair[1] and k not in fields,'noncanonical key/order/spacing');fields[k]=pair[1]
        if kind=='first': result.setdefault(kind,[]).append(fields)
        elif kind in ('reply-fault','request-fault'): result.setdefault(kind,[]).append(fields)
        else: check(kind not in result,'duplicate report record');result[kind]=fields
    check(list(result)[0]=='identity' and list(result)[-1]=='verdict','report endpoints')
    check(all(k in result for k in ('identity','counts','oracle','verdict')),'incomplete report')
    return result

def validate_request(w):
    check(len(w)==44,'request44')
    check(all(i==12 or v&0x7f800000!=0x7f800000 for i,v in enumerate(w)),'nonfinite request')
    z=lambda v:v&0x7fffffff==0
    check(all(z(w[i]) or w[i]==0x3f800000 for i in [6,7,8,9,10,11,22,23,42,43]),'invalid request flag')
    check(z(w[20]) or w[20] in (0x3f800000,0x40000000),'invalid UPDATED')
    check(not(w[8]==0x3f800000 and (w[9]==0x3f800000 or w[10]==0x3f800000)),'invalid city roles')

def validate(raw, expected, mode='read_probe_control', reference=None):
    check(set(expected)=={'lua','protocol','bundle','reference'},'unexpected expected-identity fields')
    check(mode in MODES,'unsupported mode')
    check(mode=='read_probe_control' or reference is not None,'child requires complete healthy reference')
    r=parse(raw);i,n,o,v=r['identity'],r['counts'],r['oracle'],r['verdict']
    wanted={'schema':'1','mode':mode,'profile':'hosted','scenario':'lebanon_retail','seed':'1','frame_ms':'100','hours':'0.001','campaign':'LEBANON.CHC','map':'..\\common\\maps\\map5','paired':'1','runtime':RUNTIME,'plan':PLAN,'strategy':'first_eligible',**expected}
    for k,value in wanted.items():check(i.get(k)==value,'identity mismatch '+k)
    for k in ('lua','protocol','bundle'):check(re.fullmatch('[0-9a-f]{64}',i[k]) is not None,'invalid artifact hash')
    check(v['mode']==mode,'verdict mode mismatch')
    n={k:uint(value) for k,value in n.items()};o={k:uint(value) for k,value in o.items()}
    check(1<=n['frames']<=35,'fixed frame bound')
    check(o=={k:n['oracle_'+k] for k in ('sets','reads','resets','mismatches')},'oracle record differs from counts')
    check(not r.get('reply-fault') and not r.get('request-fault') and o['mismatches']==0,'request/reply/oracle fault')
    for e in ('group','set','read','reset'):
        check(n['accepted_'+e]<=n['answered_'+e]<=n['attempts_'+e],'invalid phase accounting')
    check(n['keys_issued']==n['answered_group'] and n['oracle_sets']==n['answered_set'] and n['oracle_reads']==n['answered_read'] and n['oracle_resets']==n['answered_reset'],'oracle phase accounting')
    check(n['reads_zero']+n['reads_mask']==n['oracle_reads'] and n['reads_consumable']<=n['answered_read'] and n['resets_cleared']<=n['oracle_resets']*n['keys_issued'],'invalid read/reset accounting')
    first={};previous=None;total=sum(n['attempts_'+e] for e in ('group','set','read','reset'))
    for row in r.get('first',[]):
        event=row['event'];check(event in ('group','group_second','set','read','reset','read_consumable') and event not in first,'invalid duplicate first event')
        f={k:uint(value) for k,value in row.items() if k!='event'}
        own='group' if event in ('group','group_second') else 'read' if event=='read_consumable' else event
        check(1<=f['ordinal']<=total and f['frame']<=n['frames'] and sum(f['before_'+e] for e in ('group','set','read','reset'))==f['ordinal']-1,'first event bounds/prefix')
        for e in ('group','set','read','reset'):check(f['before_'+e]<=n['attempts_'+e]-(own==e),'first prefix beyond counts')
        if event in ('set','read','read_consumable'):check(f['before_group']>=1,'key before group')
        if event=='group':check(f['before_group']==0,'first group prefix')
        if event=='group_second':check(f['before_group']==1,'second group prefix')
        if event in ('set','read','reset'):check(f['before_'+event]==0,'first own prefix')
        if previous:
            pe,pf=previous;check(all(f[k]>=pf[k] for k in f),'nonmonotonic first rows')
            if f['ordinal']==pf['ordinal']:check(pe=='read' and event=='read_consumable' and f==pf,'invalid equal first ordinal')
        first[event]=f;previous=(event,f)
    for event,e,limit in [('group','group',1),('group_second','group',2),('set','set',1),('read','read',1),('reset','reset',1),('read_consumable','read',1)]:
        happened=n['reads_consumable']>=1 if event=='read_consumable' else n['attempts_'+e]>=limit
        check((event in first)==happened,'first event/count disagreement '+event)
    for a,b in [('group','set'),('group','read'),('group','group_second')]:
        if a in first and b in first:check(first[a]['ordinal']<first[b]['ordinal'],'group ordering')
    if mode=='read_probe_control' and v['status']=='UNREACHED':
        check(n['frames']==35,'clean absence must exhaust the fixed window')
        check(not any(k in r for k in ('selection','store','reask','consume','fatal','intervention')) and n['altered']==0 and n['reads_consumable']==0,'false clean absence')
        return r
    check(v['status']=='PASS','not a passing diagnostic')
    check(all(k in r for k in ('selection','store')),'selected transaction absent')
    s,store=r['selection'],r['store'];sw=words(s['words'],44);validate_request(sw)
    check(s['tag']=='1' and s['array']=='0' and s['group_opened']==s['group_acked'] and s['group_opened'] in ('0','1'),'selection/group ACK')
    check(all(uint(s[k])>=1 for k in ('scene','member','group','bind','candidate','ordinal')),'positive selection fields')
    check(uint(s['scene'])<=2147483647,'scene outside native integer domain')
    check(uint(s['frame'])<=n['frames'] and uint(s['bind'])<=n['bind_calls'] and uint(s['candidate'])<=n['candidate_questions'],'selection count bounds')
    check(unhex(s['entry'])==('boot' if s['frame']=='0' else 'frame:'+s['frame']),'selection entry/frame')
    check(unhex(s['id'])!='' and len(unhex(s['id']).encode())<=128 and '\0' not in unhex(s['id']),'selected identity')
    check(unhex(s['name'])!='' and len(unhex(s['name']).encode())<=127 and '\0' not in unhex(s['name']),'selected scene name')
    check(sw[6]==0x3f800000 and sw[10]&0x7fffffff==0 and sw[20]&0x7fffffff==0,'selection eligibility')
    check(s['original'] in ('skip','need_updated','need_placement','test') and s['request']==('natural' if s['original']=='need_updated' else 'diagnostic'),'selected delegated result')
    if s['original']=='test':
        pose=words(s['pose']);check(len(pose) in (12,16) and all(x&0x7f800000!=0x7f800000 for x in pose),'delegated pose')
    else:check(s['pose']=='none','pose on non-test')
    if s['original']=='need_placement':check(sw[43]&0x7fffffff==0,'illegal delegated placement')
    check(re.fullmatch(r'f[1-9][0-9]*',s['key']) is not None and len(s['key'])<=20,'issued key grammar')
    check(store['key']==s['key'] and store['array']==s['array'] and store['lane']==s['lane'],'selected READ correlation')
    lane=uint(s['lane']);check(lane<=31,'lane domain');mask=1<<lane
    check(word(store['mask'])==mask and word(store['real'])==word(store['before'])&mask and store['after']==store['real'],'real destructive store effect')
    selected=first.get('read_consumable');check(selected is not None,'selected READ first event absent')
    check(all(uint(s[k])==selected[k] for k in selected),'selected first tuple differs')
    if reference:
        for k in SCHEMA['selection'].split():check(s[k]==reference['selection'][k],'child selected trajectory differs '+k)
        check(store['before']==reference['store']['before'] and store['real']==reference['store']['real'],'child store precondition differs')
    if mode in ('read_probe_control','read_changed'):
        check('fatal' not in r and 'reask' in r and 'consume' in r,'positive carried transaction incomplete')
        after=words(r['reask']['words'],44);check(all(a==b for j,(a,b) in enumerate(zip(sw,after)) if j!=20),'re-ask changed another fact')
        answered=store['real'] if mode=='read_probe_control' else (f'0x{mask:08x}' if store['real']==ZERO else ZERO)
        check(store['answered']==answered,'answered raw word is wrong')
        carried=fact(answered);c=r['consume'];check(r['reask']['fact']==carried and after[20]==int(carried[2:],16),'re-ask carried wrong fact')
        check(c==dict(fact=carried,expected=carried,host=fact(store['real']),real_word=store['real'],answered_word=answered,changed='0' if mode=='read_probe_control' else '1'),'consume relation')
    if mode=='read_probe_control':
        check('intervention' not in r and n['altered']==0 and n['changed_accepted']==0,'altered healthy reference')
        check(all(n['attempts_'+e]==n['answered_'+e]==n['accepted_'+e] for e in ('group','set','read','reset')),'healthy phase mismatch')
        check(all(n['accepted_'+e]>=1 for e in ('group','set','read','reset')),'healthy positive phase absent')
    else:
        check(n['altered']==1 and 'intervention' in r,'missing/duplicate intervention')
        check(r['intervention']['at']=='read_ordinal_'+s['ordinal'] and r['intervention']['entry']==unhex(s['entry']),'intervention differs from selected READ')
        if mode=='read_changed':check(n['changed_accepted']==1 and fact(store['real'])!=fact(store['answered']),'changed reply not consumed')
        else:
            check('fatal' in r and 'reask' not in r and 'consume' not in r,'refusal continued into a carried result')
            inj,fatal=r['intervention'],r['fatal'];check(fatal['marker']==MARKERS[mode] and fatal['entry']==inj['entry']==unhex(s['entry']) and inj['callback_after']=='none','refusal not immediate in same entry')
            check(n['changed_accepted']==0 and store['answered']=='none','refusal credited as changed success')
    return r

def main():
    p=argparse.ArgumentParser();p.add_argument('report');p.add_argument('--mode',default=MODES[0],choices=MODES);p.add_argument('--identities',required=True);p.add_argument('--reference');a=p.parse_args()
    expected=json.loads(Path(a.identities).read_text());ref=None
    if a.reference:
        raw=Path(a.reference).read_bytes();ref=validate(raw,{**expected,'reference':'none'});check(ref['verdict']['status']=='PASS','reference unreached');expected={**expected,'reference':hashlib.sha256(raw).hexdigest()}
    else:expected={**expected,'reference':'none'}
    r=validate(Path(a.report).read_bytes(),expected,a.mode,ref);print(json.dumps({'status':r['verdict']['status'],'mode':a.mode}))
if __name__=='__main__':main()
