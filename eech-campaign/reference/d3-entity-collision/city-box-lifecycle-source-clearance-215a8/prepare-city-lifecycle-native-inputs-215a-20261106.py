from pathlib import Path
import datetime,hashlib,json,os,shutil,subprocess,sys
b=Path('R:/d3-evidence');repo=Path('D:/EECH-Migration-Evidence/city-lifecycle-witness-215a8f738-20261106-source/eech-campaign')
roots=Path('D:/EECH-Migration-Evidence/city-lifecycle-native-roots-215a8f738-20261106');records=b/'city-lifecycle-native-inputs-215a-20261106'
assert not roots.exists() and not records.exists();roots.mkdir();records.mkdir()
now=lambda:datetime.datetime.now(datetime.timezone.utc).isoformat();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
state=dict(controller_pid=os.getpid(),reference_tool_source='215a8f7389dfc9ab1dac1df2f4e45344392967d9',controller_sha256=sha(Path(__file__)),started_utc=now(),state='preparing',roots=[],input_tool_sha256=sha(repo/'tools/input-manifest.py'),reference_sha256=sha(repo/'reference/lebanon-3h/inputs.json'),scope='Fresh verified retail roots for reviewed finite CITY lifecycle retention native validation. No native launches; eventual exact reviewed source must join unchanged input-tool/reference hashes first. Exact215a frozen source; review clearance permits planned finite controls only.')
save=lambda:(records/'preparation.json').write_bytes((json.dumps(state,indent=2)+'\n').encode());save()
try:
 for name in ['short','healthy','city-box-lifecycle','city-box-lifecycle-missing','city-box-lifecycle-changed']:
  root=roots/name;shutil.copytree(Path('R:/eech-regression-20261025/roots/lebanon'),root)
  args=[sys.executable,'-B','-X','utf8',str(repo/'tools/input-manifest.py'),str(root),'--compare',str(repo/'reference/lebanon-3h/inputs.json'),'--output',str(records/(name+'-inputs.json'))]
  with (records/(name+'.stdout.log')).open('xb') as out,(records/(name+'.stderr.log')).open('xb') as err:
   child=subprocess.Popen(args,stdout=out,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW);row=dict(name=name,root=str(root),pid=child.pid,controller_sha256=sha(Path(__file__)),started_utc=now(),state='verifying');state['roots'].append(row);save();code=child.wait()
  row.update(manifest_actual_exit=code,completed_utc=now(),manifest_sha256=sha(records/(name+'-inputs.json')),state='VERIFIED UNUSED; no native launched' if code==0 else 'FAILED');save();assert code==0,(name,code)
 state['actual_exit']=0
except Exception as error:
 state.update(actual_exit=1,error=repr(error));raise
finally:
 state.update(state='terminal',completed_utc=now());save()
print(json.dumps({'actual_exit':state['actual_exit'],'roots':[{'name':x['name'],'manifest_actual_exit':x['manifest_actual_exit'],'state':x['state']} for x in state['roots']]}))
