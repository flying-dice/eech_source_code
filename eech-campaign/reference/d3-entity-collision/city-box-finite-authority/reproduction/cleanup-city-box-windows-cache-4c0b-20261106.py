from pathlib import Path
import datetime,hashlib,json,subprocess
b=Path('R:/d3-evidence');ev=Path('D:/EECH-Migration-Evidence');cache='eech-win-city-box-4c0b89263-20261106'
review=b/'independent-city-box-finite-actual-4c0b-20261106.json';assert hashlib.sha256(review.read_bytes()).hexdigest()=='f267c38d505c4192672014e7cee9caa103a44d6c28c3bb1675bcf434a10b2396'
d=json.loads(review.read_bytes())['decision'];assert d['findings']==[] and d['finite_city_box_constructor_closed']
p=ev/'city-box-profile-artifacts-4c0b-20261106/preservation.json';r=json.loads(p.read_bytes());assert r['actual_exit']==0 and r['retained_objects']==12 and r['retained_files']==30
for f in r['files']:assert hashlib.sha256((p.parent/f['path']).read_bytes()).hexdigest()==f['sha256']
build=json.loads((b/'city-box-corrected-4c0b89263-windows-20261106-completion.json').read_bytes())
for profile in build['profiles'].values():
 for f in profile['payloads']:assert hashlib.sha256((Path(profile['binary'])/f['path']).read_bytes()).hexdigest()==f['sha256']
base=['wsl','-d','EECH-D2-Build','--cd','/','--','docker'];assert subprocess.check_output(base+['ps','-aq','--filter','volume='+cache],text=True).strip()==''
result=subprocess.run(base+['volume','rm',cache],capture_output=True,text=True);assert result.returncode==0 and result.stdout.strip()==cache
receipt=dict(source=build['source'],volume=cache,actual_exit=0,no_container_owners=True,all_three_profiles_rehashed_payloads_each=88,preservation=str(p),preservation_sha256=hashlib.sha256(p.read_bytes()).hexdigest(),retained_objects=12,retained_files=30,scope='Only closed dedicated build cache removed after actual review. Source/bin/roots/raw/COFF retained. Current ordinary4native fullpack untouched and does not use Cargo cache.',completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
out=b/'cleanup-closed-city-box-4c0b-windows-cache-20261106.json';assert not out.exists();out.write_bytes((json.dumps(receipt,indent=2)+'\n').encode());print(json.dumps(receipt))
