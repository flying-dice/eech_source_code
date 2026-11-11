"""Synthetic codec/controller controls only. No runtime/retail/boot evidence.
The real normal-fire/D2 and hosted-query collaborators run in Rust/C targets.
Schema4 (lead216): the paired counterfactual is ONE zero physical step time.
"""
import copy
import hashlib
import importlib.util
import tempfile
import unittest
import re
from pathlib import Path

HERE=Path(__file__).resolve().parent
def module(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
R=module('normal_fire_capture');C=module('normal_fire_capture_controller')

def encoded(s):return 'h'+s.encode().hex()

def sync_candidate(r):
    s=r['selection'];equal=dict(call='need',scene='scene',identity='target',member='member',group='group',bind='bind',move_identity='target',move_call='move_call',move_before='move_before',move_after='move_after',latest='latest',move_frame='move_frame',prestate='prestate',delegated='delegated',query_frame='query_frame',name='name',read_call='read',read_key='key',lane='lane',word='word',carry_call='carry',carry_identity='target',carry_scene='scene',request='request',reask='reask',test_call='test',test_identity='target',test_scene='scene',pose='pose')
    placed=s['reask'].split(',');placed[43]=placed[27]=placed[31]=placed[35]='0x3f800000'
    row={k:('move' if k=='mode' else ','.join(placed) if k=='test_request' else s[equal[k]]) for k in R.CANDIDATE_FIELDS.split()}
    r['evidence']['candidate']=encoded('normal-candidate '+' '.join(k+'='+row[k] for k in R.CANDIDATE_FIELDS.split()))
    return r
def emit(rows):return ''.join('normal-capture '+name+' '+' '.join(k+'='+str(row[k]) for k in R.SCHEMA[name].split())+'\n' for name,row in rows.items()).encode()
def fixture():
    zero='0x00000000';one='0x3f800000';two='0x40000000';pose=','.join([zero]*3+[one,zero,zero,zero,one,zero,zero,zero,one])
    request=[zero]*44;request[6]=one;request[13:19]=['0x41200000','0x42c80000','0x41a00000',two,zero,zero]
    carry=request.copy();carry[20]=one
    moving=','.join(['0x41000000','0x42c80000','0x41a00000','0x41400000','0x42c80000','0x41a00000',one,zero,zero,zero,one,zero,zero,zero,one,two])
    def event(kind,seq,**values):
        row={k:'0' for k in R.EVENTS[kind].split()};row.update({k:str(v) for k,v in values.items()})
        return f'normal-ev seq={seq} tick={30000 if kind in ("normal_fire","normal_detach","normal_birth") else 30100} kind={kind} '+' '.join(k+'='+row[k] for k in R.EVENTS[kind].split())
    common=dict(launcher='eech-world:1',target='eech-world:2',weapon=70)
    fire=event('normal_fire',1,fire=1,ready=1,actual=f'{zero},{zero},{one}',intercept=f'{zero},{zero},{one}',angle=zero,maximum=one,**common)
    detach=event('normal_detach',2,fire=1,detach=1,launcher=common['launcher'],weapon=70,config=703,package=1,devices=15,burst=1,ammo_before=2,ammo_after=1,visible_before=1,visible_after=0,flash_present=1,flash_before=zero,flash_after='0xbe800000',pose=pose)
    birth=event('normal_birth',3,fire=1,detach=1,w=1,matched=1,pose=pose,**common)
    step=event('normal_step',5,step=1,w=1,pose=pose,motion=f'{zero},{zero},{zero}',velocity='0x42480000',lifetime='0x40400000',**common)
    query=event('normal_query',6,step=1,w=1,query=1,before=10,segment=','.join([zero]*6))
    models='\n'.join(event('normal_model',7+i,step=1,w=1,query=1,call=11+i,op=op) for i,op in enumerate([16,17,16,18,16]))+'\n'
    end=event('normal_query_end',12,step=1,w=1,query=1,after=15,hit=0,hit_id='none')
    step_end=event('normal_step_end',13,step=1,w=1,pose=pose,motion=f'{zero},{zero},{one}',velocity='0x42480000',lifetime='0x4039999a',**common)
    identity=dict(schema='4',mode='healthy',capture='fixture:healthy',runtime='b'*40,bundle='a'*64,driver='a'*64,collector='a'*64,transaction='a'*64,contract='a'*64,reference='none',profile='reference-hosted',scenario='lebanon_retail',seed='1',frame_ms='100',warmup='30',settle='120')
    counts=dict(frames='1500',fires='1',births='1',steps='1',queries='1',model_calls='15',moving='1',failed='0',changed='0')
    selection={k:'1' for k in R.SCHEMA['selection'].split()};selection.update(query_frame='302',move_frame='302',birth_frame='301',before='10',after='15',need='11',read='12',carry='13',test='15',launcher=common['launcher'],birth_target=common['target'],step_target=common['target'],end_target=common['target'],target=common['target'],move_call='1',move_before='10',move_after='10',weapon='70',scene='1',name=encoded('fixture-scene1'),member='2',group='1',bind='2',key='f1',lane='1',word=zero,fact=one,pose=moving,request=','.join(request),reask=','.join(carry),position='10,100,20',motion='2,0,0',latest='1',prestate=PRESTATE,tacview='0',terrain='absent',delegated='1',previous='30.100000381469727',current='30.200000762939453')
    evidence={k:encoded(value) for k,value in dict(fire=fire,detach=detach,birth=birth,step=step,query=query,models=models,query_end=end,step_end=step_end).items()}
    outcomes=['0']*27;outcomes[20]='1'
    account=dict(processed='1',decisions='1',outcomes=','.join(outcomes),first_query='1',first_code='21',first_frame='302',first_selector='0',first_models='5',first_candidates='3',first_need='1',first_reads='1',first_reasks='1',first_tests='1',first_mode='1',first_registered='1',first_type='0',first_birth_frame='301',first_move_frame='302',first_move_call='1',first_latest_call='1',first_later_call='0',first_receipt='1')
    return sync_candidate(dict(identity=identity,counts=counts,accounting=account,selection=selection,evidence=evidence,verdict=dict(status='PASS',reason=encoded('healthy normal cohort'))))

# Consumed routed-MOVE input in the host's read order: pose position3, velocity,
# attitude9, delta_seconds, desired_velocity, max_acceleration, waypoint3,
# map_min3, map_max3. Pre-position 8,100,20 steps to the healthy 10,100,20.
PRESTATE='8,100,20,2,1,0,0,0,1,0,0,0,1,1,2,1,100,100,20,0,-1000,0,4095,10000,4095'

def prestate(report,index,value):
    bad=copy.deepcopy(report);t=bad['selection']['prestate'].split(',');t[index]=value;bad['selection']['prestate']=','.join(t);return sync_candidate(bad)

def zero_step(h):
    """The paired case: same cohort and consumed input, ONLY delegated step time
    0, genuine result at the pre-position and a different TEST pose."""
    b=copy.deepcopy(h);b['identity'].update(mode='zero_step',capture='fixture:zero_step',reference='c'*64)
    b['counts']['changed']='1';b['verdict']['reason']=encoded('same-cohort zero-step effect')
    s=b['selection'];s['position']='8,100,20';s['delegated']='0'
    for key in ('request','reask'):
        w=s[key].split(',');w[13]='0x41000000';s[key]=','.join(w)
    w=s['pose'].split(',');w[0]='0x40c00000';s['pose']=','.join(w)
    return sync_candidate(b)

def damage_event(report,role,field,value):
    bad=copy.deepcopy(report);text=R.unhex(bad['evidence'][role]);token=next(x for x in text.split() if x.startswith(field+'='))
    bad['evidence'][role]=encoded(text.replace(token,field+'='+value,1));return bad

def damage_test_request(report,index,value):
    bad=copy.deepcopy(report);ce=R.tokens(R.unhex(bad['evidence']['candidate'])[17:],R.CANDIDATE_FIELDS)
    words=ce['test_request'].split(',');words[index]=value;ce['test_request']=','.join(words)
    bad['evidence']['candidate']=encoded('normal-candidate '+' '.join(k+'='+ce[k] for k in R.CANDIDATE_FIELDS.split()))
    return bad

SEMANTIC_FAULTS=[
    ('detach','devices','garbage'),('detach','devices','2'),('detach','ammo_after','garbage'),('detach','ammo_after','2'),
    ('detach','config','741'),('detach','package','32'),('detach','burst','0'),('detach','visible_after','1'),
    ('detach','flash_present','0'),('detach','flash_before','0x3f800000'),('detach','ground','2'),
    ('fire','actual','0x7fc00000,0x00000000,0x3f800000'),('fire','ready','0'),
    ('birth','matched','0'),('step','code','8'),('step','velocity','0x7f800000'),
    ('step_end','target','eech-world:3'),('step_end','launcher','eech-world:3'),('step_end','weapon','71'),
    ('query','which','-2'),('query_end','hit_id','eech-world:2'),
    ('models','seq','6'),('models','seq','9999'),('models','seq','garbage'),('models','op','19'),
]

class Controls(unittest.TestCase):
    def test_completed_cross_frame_receipt_and_pre_child_refusals(self):
        f=fixture();f['selection'].update(query_frame='303',move_frame='302',previous='30.2',current='30.3')
        f['accounting']['first_frame']='303'
        for role in ('step','query','models','query_end','step_end'):
            f['evidence'][role]=encoded(R.unhex(f['evidence'][role]).replace('tick=30100','tick=30200'))
        sync_candidate(f);R.validate(emit(f),f['identity'])
        with tempfile.TemporaryDirectory(prefix='persistent-pre-child-') as d:
            p=Path(d)/'healthy.txt';p.write_bytes(emit(f));C.healthy_reference(p,C.sha(p),f['identity'])
            for key,value in [('move_frame','301'),('move_frame','304'),('query_frame','302'),('move_call','0'),('move_after','11')]:
                bad=copy.deepcopy(f);bad['selection'][key]=value;sync_candidate(bad);p.write_bytes(emit(bad))
                with self.subTest(key=key,value=value):
                    with self.assertRaises(ValueError):C.healthy_reference(p,C.sha(p),f['identity'])
                    self.assertEqual([x.name for x in Path(d).iterdir()],['healthy.txt'])
        old=copy.deepcopy(f);old['identity']['schema']='3'
        with self.assertRaisesRegex(ValueError,'fixed setup'):R.validate(emit(old),old['identity'])

    def test_actual_TEST_domain_before_any_child_effect(self):
        f=fixture();R.validate(emit(f),f['identity'])
        # Existing domain accepts signed zero and raw unknown height. These are
        # TEST inputs, separate from the earlier NEED request and carried fact.
        for value in ('0x00000000','0x80000000','0x3f800000','0x40000000'):
            good=damage_test_request(f,20,value);good=damage_test_request(good,12,'0x7fc00000')
            R.validate(emit(good),f['identity'])
        role=damage_test_request(f,9,'0x3f800000');R.validate(emit(role),f['identity'])
        large_scene=copy.deepcopy(f);large_scene['selection']['scene']='2147483648';sync_candidate(large_scene)
        with self.assertRaisesRegex(ValueError,'scene native index domain'):R.validate(emit(large_scene),f['identity'])
        faults=[(f,20,'0x40400000','UPDATED'),(f,6,'0x00000000','nonmobile'),
                (f,8,'0x3f800000','nonzero scene'),(role,8,'0x3f800000','incompatible city roles')]
        with tempfile.TemporaryDirectory(prefix='normal-TEST-domain-') as d:
            p=Path(d)/'healthy.txt';p.write_bytes(emit(f));C.healthy_reference(p,C.sha(p),f['identity'])
            for positive,index,value,reason in faults:
                bad=damage_test_request(positive,index,value);p.write_bytes(emit(bad))
                with self.subTest(index=index,reason=reason):
                    with self.assertRaisesRegex(ValueError,reason):R.validate(emit(bad),f['identity'])
                    with self.assertRaisesRegex(ValueError,reason):C.healthy_reference(p,C.sha(p),f['identity'])
                    self.assertEqual(sorted(x.name for x in Path(d).iterdir()),['healthy.txt'])

    def test_scanned_candidate_is_distinct_from_immutable_target_history(self):
        f=fixture();R.validate(emit(f),f['identity'])
        f['selection'].update(target='eech-world:3',member='3',bind='3',step_target='none',end_target='none')
        for role in ('step','step_end'):f=damage_event(f,role,'target','none')
        f=damage_event(f,'query','which','-1');f['accounting']['first_selector']='-1'
        sync_candidate(f);R.validate(emit(f),f['identity'])
        self.assertEqual(f['selection']['birth_target'],'eech-world:2')
        for key,value in [('target','eech-world:4'),('birth_target','none'),('step_target','eech-world:2'),('member','4'),('bind','4')]:
            bad=copy.deepcopy(f);bad['selection'][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):R.validate(emit(bad),f['identity'])

    def test_rejection_accounting_is_bounded_and_cannot_credit_a_missing_witness(self):
        f=fixture();R.validate(emit(f),f['identity'])
        for key,value in [('processed','2'),('first_code','0'),('first_tests','2'),('first_mode','12'),('first_registered','2'),('first_selector','2'),('outcomes',','.join(['0']*27)),('outcomes',','.join(['0']*20+['1'])),('outcomes',','.join(['0']*10+['1']+['0']*16))]:
            bad=copy.deepcopy(f);bad['accounting'][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):R.validate(emit(bad),f['identity'])
        under=copy.deepcopy(f);under.pop('selection');under.pop('evidence');under['counts']['moving']='0'
        under['verdict'].update(status='UNREACHED',reason=encoded('no eligible normal moving cohort'))
        outcomes=['0']*27;outcomes[13]='1';under['accounting'].update(outcomes=','.join(outcomes),first_code='14',first_need='0',first_reads='0',first_reasks='0',first_tests='0')
        R.validate(emit(under),under['identity'])
        with tempfile.TemporaryDirectory(prefix='normal-accounting-unreached-') as d:
            p=Path(d)/'under.txt';p.write_bytes(emit(under))
            with self.assertRaises(ValueError):C.healthy_reference(p,C.sha(p),under['identity'])

    def test_query_selector_exact_signed_source_domain(self):
        f=fixture();text=R.unhex(f['evidence']['query'])
        for which in ('0','1','-1'):
            line=text.replace('which=0','which='+which);self.assertEqual(R.event(line)['which'],which)
            r=damage_event(f,'query','which',which);R.validate(emit(r),r['identity'])
        for which in ('-2','2','-2147483648','2147483647','1.5','garbage','-0'):
            with self.subTest(which=which),self.assertRaises(ValueError):R.event(text.replace('which=0','which='+which))

    def test_boot_ordinals_are_global_not_a_per_frame_call_limit(self):
        f=fixture();R.validate(emit(f),f['identity'])
        offset=100001;f['counts']['model_calls']=str(int(f['counts']['model_calls'])+offset)
        for key in ('before','after','need','read','carry','test','move_before','move_after'):
            f['selection'][key]=str(int(f['selection'][key])+offset)
        for role,field in [('query','before'),('models','call'),('query_end','after')]:
            text=R.unhex(f['evidence'][role])
            text=re.sub(r'\b'+field+r'=(\d+)',lambda m:field+'='+str(int(m[1])+offset),text)
            f['evidence'][role]=encoded(text)
        sync_candidate(f);raw=emit(f);R.validate(raw,f['identity'])
        with tempfile.TemporaryDirectory(prefix='normal-boot-ordinal-') as d:
            p=Path(d)/'synthetic.txt';p.write_bytes(raw);C.healthy_reference(p,C.sha(p),f['identity'])

    def test_complete_event_semantics_before_child_effect(self):
        f=fixture();R.validate(emit(f),f['identity'])
        with tempfile.TemporaryDirectory(prefix='normal-semantic-gate-') as d:
            p=Path(d)/'synthetic.txt';p.write_bytes(emit(f));C.healthy_reference(p,C.sha(p),f['identity'])
            for role,key,value in SEMANTIC_FAULTS:
                with self.subTest(role=role,field=key,value=value):
                    bad=damage_event(f,role,key,value);p.write_bytes(emit(bad))
                    with self.assertRaises(ValueError):R.validate(emit(bad),f['identity'])
                    # Recomputed report hash intentionally passes the byte gate:
                    # rejection must be semantic, before any child effect.
                    with self.assertRaises(ValueError):C.healthy_reference(p,C.sha(p),f['identity'])
                    self.assertEqual(sorted(x.name for x in Path(d).iterdir()),['synthetic.txt'])

    def test_earlier_birth_and_unchanged_common_prefix(self):
        f=fixture();healthy=R.validate(emit(f),f['identity'])
        same=copy.deepcopy(f);same['selection']['birth_frame']=same['selection']['query_frame']
        with self.assertRaisesRegex(ValueError,'earlier observed birth'):R.validate(emit(same),f['identity'])
        b=zero_step(f);R.validate(emit(b),b['identity'],healthy)
        changed=damage_event(b,'birth','pose','0x3f800000,'+R.unhex(b['evidence']['birth']).split('pose=')[1].split(', ',1)[0].split(',',1)[1])
        with self.assertRaisesRegex(ValueError,'paired common prefix'):R.validate(emit(changed),b['identity'],healthy)

    def test_pinned_event_enum_domains_match_unchanged_compiled_headers(self):
        base=HERE.parents[1]
        for name,last,count in [('aphavoc/source/entity/system/en_types/en_sbtyp.h','NUM_ENTITY_SUB_TYPE_WEAPONS',83),('aphavoc/source/entity/mobile/weapon/weapon.h','NUM_WEAPON_CONFIG_TYPES',741),('aphavoc/source/entity/mobile/weapon/weapon.h','NUM_WEAPON_KILL_CODES',8)]:
            text=re.sub(r'/\*.*?\*/|//[^\n]*','',(base/name).read_text(),flags=re.S);end=text.index(last);body=text[text.rfind('{',0,end)+1:end];value=-1
            for row in body.split(','):
                if row.strip():
                    parts=row.strip().split('=');value=int(parts[1].strip(),0) if len(parts)>1 else value+1
            self.assertEqual(value+1,count)

    def test_coherent_positive_then_single_field_damage(self):
        f=fixture();R.validate(emit(f),f['identity'])
        for row,key,value in [('selection','target','eech-world:3'),('selection','carry','14'),('selection','word','0x00000002'),('selection','position','11,100,20'),('counts','failed','1'),('counts','frames','1499'),('identity','seed','2'),('selection','before','9')]:
            bad=copy.deepcopy(f);bad[row][key]=value
            with self.subTest(row=row,key=key),self.assertRaises(ValueError):R.validate(emit(bad),f['identity'])
        with self.assertRaises(ValueError):R.validate(emit(f)+b'extra\n',f['identity'])

    def test_healthy_hash_and_full_positive_gate(self):
        f=fixture();raw=emit(f)
        with tempfile.TemporaryDirectory(prefix='normal-codec-') as d:
            p=Path(d)/'synthetic.txt';p.write_bytes(raw);h=hashlib.sha256(raw).hexdigest()
            C.healthy_reference(p,h,f['identity'])
            p.write_bytes(raw.replace(b'carry=13',b'carry=14'))
            with self.assertRaises(ValueError):C.healthy_reference(p,h,f['identity'])
            with self.assertRaises(ValueError):C.healthy_reference(p,C.sha(p),f['identity'])

    def test_unreached_is_not_healthy_and_failure_precedes_it(self):
        f=fixture();f.pop('selection');f.pop('evidence');f['counts'].update(fires='0',births='0',steps='0',queries='0',moving='0');f['verdict']=dict(status='UNREACHED',reason=encoded('no eligible normal moving cohort'))
        f['accounting']={k:('0,'*26+'0' if k=='outcomes' else '0') for k in R.SCHEMA['accounting'].split()}
        self.assertEqual(R.validate(emit(f),f['identity'])['verdict']['status'],'UNREACHED')
        f['counts']['failed']='1'
        with self.assertRaises(ValueError):R.validate(emit(f),f['identity'])
        f['verdict']=dict(status='FAIL',reason=encoded('unrelated native error'))
        self.assertEqual(R.validate(emit(f),f['identity'])['verdict']['status'],'FAIL')

    def test_no_unmaterialized_plan_effects(self):
        with tempfile.TemporaryDirectory(prefix='normal-plan-') as d:
            p=Path(d)/'plan.json';p.write_text('{"schema":1,"state":"UNMATERIALIZED"}')
            with self.assertRaises(ValueError):C.preflight(p)
            self.assertEqual(sorted(x.name for x in Path(d).iterdir()),['plan.json'])

    def test_zero_step_B_requires_healthy_same_cohort_input_and_genuine_pre_position(self):
        h=fixture();healthy=R.validate(emit(h),h['identity']);b=zero_step(h)
        R.validate(emit(b),b['identity'],healthy)
        with self.assertRaises(ValueError):R.validate(emit(b),b['identity'])
        bad=copy.deepcopy(b);bad['selection']['bind']='3'
        with self.assertRaises(ValueError):R.validate(emit(bad),b['identity'],healthy)
        # Single-field B faults: delegated step time, a result that moved, a
        # different consumed input, and the retired schema3 terrain mode.
        for key,value,reason in [('delegated','1','delegated physical step time'),('position','9,100,20','eligibility code 27'),('tacview','1','consumed MOVE input')]:
            bad=copy.deepcopy(b);bad['selection'][key]=value
            if key=='position':
                for k in ('request','reask'):
                    w=bad['selection'][k].split(',');w[13]='0x41100000';bad['selection'][k]=','.join(w)
            sync_candidate(bad)
            with self.subTest(key=key),self.assertRaisesRegex(ValueError,reason):R.validate(emit(bad),b['identity'],healthy)
        bad=prestate(b,14,'3')
        with self.assertRaisesRegex(ValueError,'consumed MOVE input'):R.validate(emit(bad),b['identity'],healthy)
        old=copy.deepcopy(b);old['identity']['mode']='terrain_lift'
        with self.assertRaisesRegex(ValueError,'mode'):R.validate(emit(old),old['identity'],healthy)

    def test_schema4_healthy_eligibility_single_field_faults_before_child(self):
        f=fixture();R.validate(emit(f),f['identity'])
        faults=[('terrain',lambda r:dict(r,terrain='present'),'code 22'),
                ('nonfinite input',lambda r:None,'code 23'),('short input',lambda r:None,'code 23'),('noncanonical input',lambda r:None,'code 23'),
                ('zero dt',lambda r:None,'code 24'),('negative dt',lambda r:None,'code 24'),
                ('negative speed',lambda r:None,'code 25'),('outside bounds',lambda r:None,'code 26'),
                ('no displacement',lambda r:None,'code 27'),('dt mismatch',lambda r:dict(r,delegated='0.5'),'delegated physical step time'),
                ('newer writer',lambda r:dict(r,latest='2'),'latest same-ID writer'),('model ordinal',lambda r:dict(r,move_before='11'),'existing movement callback order'),
                ('callback ordinal',lambda r:dict(r,move_call='0',latest='0'),'existing movement callback order')]
        edits={'nonfinite input':(0,'nan'),'noncanonical input':(0,'8.0'),'zero dt':(13,'0'),'negative dt':(13,'-1'),'negative speed':(3,'-1'),'outside bounds':(22,'7'),'no displacement':(0,'10')}
        with tempfile.TemporaryDirectory(prefix='zero-step-eligibility-') as d:
            p=Path(d)/'healthy.txt'
            for name,change,reason in faults:
                if name in edits:bad=prestate(f,*edits[name])
                elif name=='short input':bad=copy.deepcopy(f);bad['selection']['prestate']=PRESTATE.rsplit(',',1)[0];sync_candidate(bad)
                else:bad=copy.deepcopy(f);bad['selection']=change(bad['selection']);sync_candidate(bad)
                if name in ('zero dt','negative dt'):bad['selection']['delegated']=bad['selection']['prestate'].split(',')[13];sync_candidate(bad)
                p.write_bytes(emit(bad))
                with self.subTest(fault=name):
                    with self.assertRaisesRegex(ValueError,reason):R.validate(emit(bad),f['identity'])
                    with self.assertRaisesRegex(ValueError,reason):C.healthy_reference(p,C.sha(p),f['identity'])
                    self.assertEqual([x.name for x in Path(d).iterdir()],['healthy.txt'])

if __name__=='__main__':unittest.main()
