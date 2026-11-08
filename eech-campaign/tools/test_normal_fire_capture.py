"""Synthetic codec/controller controls only. No runtime/retail/boot evidence.
The real normal-fire/D2 and hosted-query collaborators run in Rust/C targets.
"""
import copy
import hashlib
import importlib.util
import tempfile
import unittest
from pathlib import Path

HERE=Path(__file__).resolve().parent
def module(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
R=module('normal_fire_capture');C=module('normal_fire_capture_controller')

def encoded(s):return 'h'+s.encode().hex()
def emit(rows):return ''.join('normal-capture '+name+' '+' '.join(k+'='+str(row[k]) for k in R.SCHEMA[name].split())+'\n' for name,row in rows.items()).encode()
def fixture():
    zero='0x00000000';one='0x3f800000';two='0x40000000';pose=','.join([zero]*3+[one,zero,zero,zero,one,zero,zero,zero,one])
    request=[zero]*44;request[6]=one;request[13:19]=['0x41200000','0x42c80000','0x41a00000',two,zero,zero]
    carry=request.copy();carry[20]=one
    moving=','.join(['0x41000000','0x42c80000','0x41a00000','0x41400000','0x42c80000','0x41a00000',one,zero,zero,zero,one,zero,zero,zero,one,two])
    def event(kind,seq,**values):
        row={k:'0' for k in R.EVENTS[kind].split()};row.update({k:str(v) for k,v in values.items()})
        return f'normal-ev seq={seq} tick=30000 kind={kind} '+' '.join(k+'='+row[k] for k in R.EVENTS[kind].split())
    common=dict(launcher='eech-world:1',target='eech-world:2',weapon=70)
    fire=event('normal_fire',1,fire=1,ready=1,actual=f'{zero},{zero},{one}',intercept=f'{zero},{zero},{one}',angle=zero,maximum=one,**common)
    detach=event('normal_detach',2,fire=1,detach=1,launcher=common['launcher'],weapon=70,config=703,package=1,devices=15,burst=1,ammo_before=2,ammo_after=1,visible_before=1,visible_after=0,flash_present=1,flash_before=zero,flash_after='0xbe800000',pose=pose)
    birth=event('normal_birth',3,fire=1,detach=1,w=1,matched=1,pose=pose,**common)
    step=event('normal_step',4,step=1,w=1,pose=pose,motion=f'{zero},{zero},{zero}',velocity='0x42480000',lifetime='0x40400000',**common)
    query=event('normal_query',5,step=1,w=1,query=1,before=10,segment=','.join([zero]*6))
    models='\n'.join(event('normal_model',6+i,step=1,w=1,query=1,call=11+i,op=op) for i,op in enumerate([16,17,16,18,16]))+'\n'
    end=event('normal_query_end',11,step=1,w=1,query=1,after=15,hit=0,hit_id='none')
    step_end=event('normal_step_end',12,step=1,w=1,pose=pose,motion=f'{zero},{zero},{one}',velocity='0x42480000',lifetime='0x4039999a',**common)
    identity=dict(schema='1',mode='healthy',capture='fixture:healthy',runtime='b'*40,bundle='a'*64,driver='a'*64,collector='a'*64,transaction='a'*64,contract='a'*64,reference='none',profile='reference-hosted',scenario='lebanon_retail',seed='1',frame_ms='100',warmup='30',settle='120')
    counts=dict(frames='1500',fires='1',births='1',steps='1',queries='1',model_calls='15',moving='1',failed='0',changed='0')
    selection={k:'1' for k in R.SCHEMA['selection'].split()};selection.update(frame='301',birth_frame='301',before='10',after='15',need='11',read='12',carry='13',test='15',launcher=common['launcher'],target=common['target'],weapon='70',scene='1',name=encoded('fixture-scene1'),member='2',group='1',bind='2',key='f1',lane='1',word=zero,fact=one,pose=moving,request=','.join(request),reask=','.join(carry),position='10,100,20',motion='2,0,0',base='100',ceiling='10000',previous='30',current='30.100000381469727')
    evidence={k:encoded(value) for k,value in dict(fire=fire,detach=detach,birth=birth,step=step,query=query,models=models,query_end=end,step_end=step_end).items()}
    return dict(identity=identity,counts=counts,selection=selection,evidence=evidence,verdict=dict(status='PASS',reason=encoded('healthy normal cohort')))

class Controls(unittest.TestCase):
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

    def test_single_input_B_requires_healthy_same_cohort_and_real_pose_change(self):
        h=fixture();healthy=R.validate(emit(h),h['identity']);b=copy.deepcopy(h)
        b['identity'].update(mode='terrain_lift',capture='fixture:terrain_lift',reference='c'*64)
        b['counts']['changed']='1';b['verdict']['reason']=encoded('same-cohort terrain effect')
        s=b['selection'];s['position']='10,600,20'
        for k in ('request','reask'):
            w=s[k].split(',');w[14]='0x44160000';s[k]=','.join(w)
        w=s['pose'].split(',');w[1]=w[4]='0x44160000';s['pose']=','.join(w)
        R.validate(emit(b),b['identity'],healthy)
        with self.assertRaises(ValueError):R.validate(emit(b),b['identity'])
        bad=copy.deepcopy(b);bad['selection']['bind']='3'
        with self.assertRaises(ValueError):R.validate(emit(bad),b['identity'],healthy)

if __name__=='__main__':unittest.main()
