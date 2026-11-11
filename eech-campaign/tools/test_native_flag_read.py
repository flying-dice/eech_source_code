"""Offline validator/controller controls over a report produced by the REAL Lua
driver in the scripted-campaign test. No engine/native process or retail copy.
"""
import argparse,hashlib,importlib.util,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
def module(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
v=module('native_flag_read');c=module('native_flag_read_controller')
CONTROL=None;EXPECTED=None

class Controls(unittest.TestCase):
    def test_real_lua_positive_and_every_new_record_join(self):
        r=v.validate(CONTROL,EXPECTED)
        self.assertEqual(r['verdict']['status'],'PASS')
        for before,after in [('frames=1','frames=36'),('member=1','member=0'),('candidate=1','candidate=0'),('tag=1','tag=2'),('array=0','array=1'),('lane=5','lane=32'),('before=0x00000060','before=0x00000000'),('after=0x00000020','after=0x00000000'),('pose=none','pose=0x00000000'),('original=skip','original=test'),('group_acked=0','group_acked=1')]:
            self.assertIn(before,CONTROL.decode())
            with self.assertRaises(ValueError,msg=before):v.validate(CONTROL.replace(before.encode(),after.encode(),1),EXPECTED)

    def test_reference_hash_checked_before_child_work_even_for_plausible_edit(self):
        with tempfile.TemporaryDirectory(prefix='nr395-offline-') as d:
            p=Path(d)/'control';p.write_bytes(CONTROL);digest=hashlib.sha256(CONTROL).hexdigest()
            self.assertEqual(c.child_reference(p,digest,EXPECTED)['verdict']['status'],'PASS')
            p.write_bytes(CONTROL.replace(b'bind_calls=3',b'bind_calls=4',1))
            with patch.object(c.shutil,'copytree',side_effect=AssertionError('child root created')),patch.object(c.subprocess,'Popen',side_effect=AssertionError('process launched')):
                with self.assertRaisesRegex(ValueError,'healthy file changed'):c.child_reference(p,digest,EXPECTED)

    def test_unmaterialized_plan_refuses_before_any_path_or_process_action(self):
        with tempfile.TemporaryDirectory(prefix='nr395-offline-') as d:
            p=Path(d)/'plan.json';p.write_text(json.dumps({'schema':1,'state':'UNMATERIALIZED_NO_LAUNCH'}))
            with patch.object(c.shutil,'copytree',side_effect=AssertionError('root created')),patch.object(c.subprocess,'Popen',side_effect=AssertionError('process launched')):
                with self.assertRaisesRegex(ValueError,'UNMATERIALIZED'):c.preflight(p)

    def test_manifest_paths_do_not_escape_and_relative_write_targets_are_refused(self):
        with tempfile.TemporaryDirectory(prefix='nr395-offline-') as d:
            for name in ['../escape','/absolute','a\\b','C:/escaped']:
                with self.assertRaises(ValueError,msg=name):c.child_path(Path(d),name)
        with self.assertRaisesRegex(ValueError,'absolute'):c.safe_new_target('relative-output')

    def test_full_bundle_inventory_and_every_one_of_91_bytes_are_required(self):
        with tempfile.TemporaryDirectory(prefix='nr395-offline-') as d:
            root=Path(d);base=root/'synthetic-data-bundle';base.mkdir();literal=root/'literal';literal.mkdir()
            rows=[]
            for i in range(91):
                p=base/f'payload-{i}.data';p.write_bytes(str(i).encode());rows.append({'path':p.name,'bytes':p.stat().st_size,'sha256':c.sha(p)})
            for name in ['driver.lua','transaction.lua']:(literal/name).write_text('-- offline data only\n')
            manifest=root/'manifest.json';manifest.write_text(json.dumps({'runtime':c.RUNTIME,'files':rows}))
            p={'binary_dir':str(base),'bundle_manifest':str(manifest),'bundle_manifest_sha256':c.sha(manifest),'literal_dir':str(literal),'lua':str(literal/'driver.lua'),'protocol':str(literal/'transaction.lua'),'hashes':{'lua':c.sha(literal/'driver.lua'),'protocol':c.sha(literal/'transaction.lua')}}
            self.assertEqual(c.bundle_check(p,{'payloads':rows}),base)
            (base/'payload-37.data').write_text('damaged')
            with self.assertRaisesRegex(ValueError,'bundle bytes differ'):c.bundle_check(p,{'payloads':rows})

if __name__=='__main__':
    q=argparse.ArgumentParser();q.add_argument('--control',required=True);q.add_argument('--identities',required=True);a=q.parse_args()
    CONTROL=Path(a.control).read_bytes();EXPECTED={**json.loads(Path(a.identities).read_text()),'reference':'none'}
    unittest.main(argv=['nr395-offline-controls'],verbosity=2)
