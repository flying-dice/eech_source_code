"""Offline validator of the fixed-bound normal-fire public-session report.
No native action. Selected C excerpts are distinct from the complete raw stream.
"""
import math
import re
import struct

SCHEMA = {
    'identity': 'schema mode capture runtime bundle driver collector transaction contract reference profile scenario seed frame_ms warmup settle',
    'counts': 'frames fires births steps queries model_calls moving failed changed',
    'selection': 'frame birth_frame fire detach w step query before after need read carry test launcher target weapon scene name member group bind key lane word fact pose request reask position motion base ceiling previous current',
    'evidence': 'fire detach birth step query models query_end step_end',
    'verdict': 'status reason',
}
EVENTS = {
    'normal_fire': 'fire launcher target weapon ready actual intercept angle maximum',
    'normal_detach': 'fire detach launcher weapon config package devices burst ammo_before ammo_after visible_before visible_after flash_present flash_before flash_after volume ground pose',
    'normal_birth': 'fire detach w launcher target weapon matched pose',
    'normal_step': 'step w launcher target weapon pose motion velocity lifetime code',
    'normal_step_end': 'step w launcher target weapon pose motion velocity lifetime code',
    'normal_query': 'step query w which before segment',
    'normal_query_end': 'step query w after hit hit_id',
    'normal_model': 'step query w call op',
}

def check(ok, why):
    if not ok:
        raise ValueError(why)

def uint(s):
    check(isinstance(s, str) and re.fullmatch(r'0|[1-9][0-9]*', s), 'integer grammar')
    n = int(s); check(n <= 9007199254740991, 'integer range'); return n

def unhex(s):
    check(re.fullmatch(r'h(?:[0-9a-f]{2})*', s), 'encoded text')
    return bytes.fromhex(s[1:]).decode('utf-8')

def words(s, n):
    a = s.split(','); check(len(a) == n and all(re.fullmatch(r'0x[0-9a-f]{8}', w) for w in a), 'raw word grammar'); return a

def number(w):
    v = struct.unpack('<f', bytes.fromhex(words(w, 1)[0][2:])[::-1])[0]
    check(math.isfinite(v), 'nonfinite physical word'); return v

def tokens(body, keys):
    a = {}; parts = body.split(' ')
    for part in parts:
        check(re.fullmatch(r'[a-z_]+=[^\s=]+', part), 'token grammar')
        k, v = part.split('=', 1); check(k not in a, 'duplicate field'); a[k] = v
    check(list(a) == keys.split(), 'field schema/order'); return a

def event(text):
    check(isinstance(text,str) and text.startswith('normal-ev ') and '\n' not in text and '\r' not in text, 'C evidence framing')
    found=re.findall(r'(?:^| )kind=([^ ]+)',text)
    check(len(found)==1 and found[0] in EVENTS,'C evidence kind')
    e=tokens(text[10:],'seq tick kind '+EVENTS[found[0]])
    def integer(k,lo,hi=2147483647):
        s=e[k];check(re.fullmatch(r'0|-?[1-9][0-9]*',s),'integer domain '+k);v=int(s);check(lo<=v<=hi,'integer domain '+k);return v
    def physical(k,n):return [number(x) for x in words(e[k],n)]
    integer('seq',1,9007199254740991);integer('tick',0,4294967295)
    for k in ('launcher','target','hit_id'):
        if k in e:check(e[k]=='none' or (len(e[k])<=128 and re.fullmatch(r'eech-world:[1-9][0-9]*',e[k])),'identity '+k)
    if 'fire' in e:integer('fire',0,9007199254740991)
    if 'weapon' in e:integer('weapon',1,82)
    if 'w' in e:integer('w',1)
    for k in ('step','query'):
        if k in e:integer(k,1,9007199254740991)
    kind=e['kind']
    if kind=='normal_fire':
        integer('fire',1,9007199254740991);integer('ready',1,1)
        physical('actual',3);physical('intercept',3);physical('angle',1);physical('maximum',1)
    elif kind=='normal_detach':
        integer('detach',1,9007199254740991);config=integer('config',-1,740);package=integer('package',-1,31);mask=integer('devices',0,15)
        check(mask in (0,1,3,7,11,15),'device path');burst=integer('burst',1);before=integer('ammo_before',-1);after=integer('ammo_after',-1)
        check(config>=0 and before>0 and after==max(0,before-burst) if package>=0 else config==before==after==-1 and mask==0,'package effect')
        vb,va=integer('visible_before',-1,1),integer('visible_after',-1,1)
        check(vb>=0 and va==0 if mask & 4 else vb==va==-1,'station effect')
        flash=integer('flash_present',0,1);physical('flash_before',1);physical('flash_after',1)
        if not flash:check(e['flash_before']==e['flash_after']=='0x00000000','absent flash effect')
        else:
            check(mask>=8,'flash without muzzle')
            check(int(e['flash_after'][2:],16)>=2147483648 if number(e['flash_before'])==0 else e['flash_after']==e['flash_before'],'conditional flash effect')
        volume=integer('volume',0,1);integer('ground',0,1);check(volume==0 or mask>=3,'volume fallback without device');physical('pose',12)
    elif kind=='normal_birth':
        integer('detach',1,9007199254740991);matched=integer('matched',0,1);check((e['fire']=='0')==(matched==0),'birth match domain');physical('pose',12)
    elif kind in ('normal_step','normal_step_end'):
        physical('pose',12);physical('motion',3);physical('velocity',1);physical('lifetime',1);integer('code',0,7)
    elif kind=='normal_query':integer('which',0);integer('before',0,9007199254740991);physical('segment',6)
    elif kind=='normal_query_end':
        integer('after',0,9007199254740991);hit=integer('hit',0,1);check(hit==1 or e['hit_id']=='none','miss carries hit identity')
    elif kind=='normal_model':integer('call',1,9007199254740991);integer('op',1,18)
    return e

def parse(raw):
    check(isinstance(raw, bytes) and len(raw) <= 262144 and raw.endswith(b'\n') and b'\r' not in raw and b'\n\n' not in raw, 'report framing')
    rows = {}
    for line in raw.decode('utf-8').splitlines():
        p, name, body = line.split(' ', 2)
        check(p == 'normal-capture' and name in SCHEMA and name not in rows, 'report family')
        rows[name] = tokens(body, SCHEMA[name])
    check(list(rows) in (['identity', 'counts', 'verdict'], list(SCHEMA)), 'report order')
    return rows

def validate(raw, expected, healthy=None):
    r = parse(raw); a, c, v = r['identity'], r['counts'], r['verdict']
    check(all(a.get(k) == str(value) for k, value in expected.items()), 'artifact/setup identity')
    check(a['mode'] in ('healthy', 'terrain_lift'), 'mode')
    fixed = dict(schema='1', profile='reference-hosted', scenario='lebanon_retail', seed='1', frame_ms='100', warmup='30', settle='120')
    check(all(a[k] == value for k, value in fixed.items()), 'fixed setup')
    check(re.fullmatch('[0-9a-f]{40}', a['runtime']), 'runtime source')
    check(all(re.fullmatch('[0-9a-f]{64}', a[k]) for k in ('bundle', 'driver', 'collector', 'transaction', 'contract')), 'artifact hash')
    check(re.fullmatch(r'[\w.:-]{1,128}', a['capture']), 'capture identity')
    check(a['reference'] == 'none' if a['mode'] == 'healthy' else re.fullmatch('[0-9a-f]{64}', a['reference']), 'reference identity')
    counts = {k: uint(value) for k, value in c.items()}
    reason = unhex(v['reason'])
    check(v['status'] in ('PASS', 'UNREACHED', 'FAIL'), 'verdict')
    if v['status'] == 'FAIL':
        check(counts['failed'] > 0 and reason, 'failure precedence'); return r
    check(counts['frames'] == 1500 and counts['failed'] == 0, 'complete healthy frames')
    if v['status'] == 'UNREACHED':
        check(a['mode'] == 'healthy' and 'selection' not in r and counts['moving'] == 0 and counts['changed'] == 0 and reason in ('no eligible normal moving cohort','locked cohort lacks earlier observed eligible query'), 'unreached domain'); return r
    s, e = r['selection'], r['evidence']
    check(counts['changed'] == (0 if a['mode'] == 'healthy' else 1), 'input intervention count')
    check(all(counts[k] > 0 for k in ('fires', 'births', 'steps', 'queries', 'model_calls', 'moving')), 'positive traffic')
    check(reason == ('healthy normal cohort' if a['mode'] == 'healthy' else 'same-cohort terrain effect'), 'positive reason')
    n = {k: uint(s[k]) for k in 'frame birth_frame fire detach w step query before after need read carry test weapon scene member group bind lane'.split()}
    check(300 < n['birth_frame'] < n['frame'] <= 1500 and n['before'] < n['need'] < n['read'] and n['read'] + 1 == n['carry'] <= n['test'] <= n['after'] <= counts['model_calls'], 'earlier observed birth/window/callback order')
    check(counts['moving']<=counts['queries'] and n['step']<=counts['steps'] and n['query']<=counts['queries'],'capture count bounds')
    check(all(n[k]>0 for k in ('fire','detach','w','step','query','weapon','scene','member','group','bind')),'positive selected indices')
    check(all(re.fullmatch(r'eech-world:[1-9][0-9]*', s[k]) for k in ('launcher', 'target')) and s['launcher'] != s['target'], 'canonical cohort')
    check(re.fullmatch(r'f[1-9][0-9]{0,18}', s['key']) and n['lane'] <= 31, 'store identity')
    mask = f'0x{1 << n["lane"]:08x}'; check(s['word'] in ('0x00000000', mask), 'lawful raw READ')
    check(s['fact'] == ('0x3f800000' if s['word'] == '0x00000000' else '0x40000000'), 'carried fact')
    request, carry, pose = words(s['request'], 44), words(s['reask'], 44), words(s['pose'], 16)
    name=unhex(s['name']);check(len(name.encode())<=127 and '\0' not in name, 'scene name')
    check(all(i==12 or math.isfinite(number(x)) for i,x in enumerate(request)), 'finite candidate facts')
    check(all(request[i] in ('0x00000000','0x80000000','0x3f800000') for i in (6,7,8,9,10,11,22,23,42,43)), 'candidate boolean domains')
    check(request[6] == '0x3f800000' and request[10] in ('0x00000000', '0x80000000') and request[20] in ('0x00000000', '0x80000000'), 'lawful NEED_UPDATED')
    check(carry[20] == s['fact'] and all(x == y for i, (x, y) in enumerate(zip(request, carry)) if i != 20), 'same-candidate re-ask')
    vectors = {k: list(map(float, s[k].split(','))) for k in ('position', 'motion')}
    check(all(len(x) == 3 and all(map(math.isfinite, x)) for x in vectors.values()), 'real motion vectors')
    check(sum(x*x for x in vectors['motion']) > 1 and number(pose[15]) > 1 and all(math.isfinite(number(x)) for x in pose), 'moving pose')
    check(all(number(request[13+i]) == vectors['position'][i] and number(request[16+i]) == vectors['motion'][i] for i in range(3)), 'same host physical record')
    scalar = {k: float(s[k]) for k in ('base', 'ceiling', 'previous', 'current')}
    check(all(map(math.isfinite, scalar.values())) and scalar['base'] + 500 <= scalar['ceiling'] and scalar['current'] > scalar['previous'], 'physical input/frame clock')
    events = {}
    for k, value in e.items():
        text = unhex(value)
        if k == 'models': continue
        parsed=event(text);kind=parsed['kind']
        check(kind == {'fire':'normal_fire', 'detach':'normal_detach', 'birth':'normal_birth', 'step':'normal_step', 'query':'normal_query', 'query_end':'normal_query_end', 'step_end':'normal_step_end'}[k], 'C evidence kind')
        events[k] = parsed
    f, d, b, st, q, qe, se = (events[k] for k in ('fire', 'detach', 'birth', 'step', 'query', 'query_end', 'step_end'))
    check(all(x['fire'] == s['fire'] for x in (f,d,b)) and d['detach'] == b['detach'] == s['detach'] and b['w'] == s['w'] and b['matched'] == '1', 'normal birth chain')
    check(int(d['package'])>=0,'selected package absent')
    check(all(x[k] == s[k] for x in (f,b,st,se) for k in ('launcher','target','weapon')), 'normal source/target/weapon')
    check(d['launcher']==s['launcher'] and d['weapon']==s['weapon'] and f['ready']=='1' and 0<=number(f['angle'])<=number(f['maximum']), 'actual firing/detachment')
    check(uint(f['seq'])<uint(d['seq'])<uint(b['seq'])<uint(st['seq'])<uint(q['seq'])<uint(qe['seq'])<uint(se['seq']), 'C event order')
    check(all(math.isfinite(number(x)) for event in (d,b,st,se) for x in words(event['pose'],12)), 'finite C poses')
    check(all(uint(x['tick']) == (n['birth_frame'] - 1) * 100 for x in (f,d,b)), 'normal birth frame')
    check(all(x['w'] == s['w'] and x['step'] == s['step'] and uint(x['tick']) == (n['frame']-1)*100 for x in (st,q,qe,se)), 'step/query frame')
    check(q['query'] == qe['query'] == s['query'] and q['before'] == s['before'] and qe['after'] == s['after'], 'query interval')
    models = []
    model_text=unhex(e['models']);check(model_text.endswith('\n') and '\n\n' not in model_text,'model framing')
    for line in model_text.splitlines():
        x=event(line);models.append(x)
        check(x['kind'] == 'normal_model' and all(x[k] == s[k] for k in ('step','query','w')) and uint(x['call']) == n['before']+len(models) and 1 <= uint(x['op']) <= 18 and uint(x['tick']) == (n['frame']-1)*100, 'complete model callback interval')
        check(uint(x['seq'])==uint(q['seq'])+len(models) and uint(x['seq'])<uint(qe['seq']),'model sequence envelope')
    check(len(models) == n['after'] - n['before'] > 0, 'model count')
    check(uint(qe['seq'])==uint(q['seq'])+len(models)+1,'query sequence closure')
    check(all(x['op'] == op for x, op in ((models[n['need']-n['before']-1], '16'), (models[n['read']-n['before']-1], '17'), (models[n['carry']-n['before']-1], '16'), (models[n['test']-n['before']-1], '16'))), 'actual callback roles')
    if a['mode'] == 'terrain_lift':
        check(healthy is not None and healthy['verdict']['status'] == 'PASS' and healthy['identity']['mode'] == 'healthy', 'healthy first')
        old = healthy['selection']
        check(all(s[k] == old[k] for k in 'frame birth_frame fire detach w step query launcher target weapon scene member group bind'.split()), 'same selected cohort')
        check(all(e[k]==healthy['evidence'][k] for k in ('fire','detach','birth')),'paired common prefix')
        check(s['pose'] != old['pose'] and s['position'] != old['position'], 'real pose effect')
    return r
