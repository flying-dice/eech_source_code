"""Actual committed-byte join after CODE; metadata-only, no checks replayed."""
import hashlib,json,pathlib,subprocess,datetime
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
PACK='eech-campaign/reference/d3-entity-collision/native-flag-read-correction';P=W/PACK
DEST=PACK+'-publication';D=W/DEST
def sha(b):return hashlib.sha256(b).hexdigest()
def j(p):return json.loads(p.read_bytes())
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def new(p,b):
    with p.open('xb') as f:f.write(b)
def dump(p,d):new(p,(json.dumps(d,indent=2)+'\n').encode())
assert not git('status','--porcelain=v1').strip()
code=git('rev-parse','HEAD').decode().strip();tree={}
for row in git('ls-tree','-r','-z',code).decode().split('\0'):
    if row:
        meta,path=row.split('\t',1);tree[path]=meta.split()[2]
post=j(E/'nr602-inputmap-POST-20261108.json');extra=j(E/'nr602-extra-POST-20261108.json');frozen=j(P/'D3-NATIVE-READ-CORRECTION-FROZEN-20261108.json')
paths=set(post['files'])|{p for p in tree if p.startswith(PACK+'/')}|{p for p in extra['files'] if (W/p).is_file()}
objects=sorted({tree[p] for p in paths});data=git('cat-file','--batch') if not objects else subprocess.check_output(['git','-C',str(W),'cat-file','--batch'],input=('\n'.join(objects)+'\n').encode())
offset=0;blobs={}
for obj in objects:
    end=data.index(b'\n',offset);h,kind,size=data[offset:end].decode().split();size=int(size);start=end+1;raw=data[start:start+size]
    assert h==obj and kind=='blob' and len(raw)==size and data[start+size:start+size+1]==b'\n'
    blobs[obj]=(sha(raw),size);offset=start+size+1
assert offset==len(data)
rows=[];qualified=[]
for path,row in post['files'].items():
    digest,size=blobs[tree[path]]
    entry={'path':path,'git_blob':tree[path],'committed_sha256':digest,'working_raw_sha256':row['raw_sha256'],'bytes_equal':digest==row['raw_sha256']}
    if not entry['bytes_equal']:
        assert path in frozen['inherited_Lua_qualifications'] and digest==row['head_blob_bytes_sha256']
        entry['qualification']='Inherited working CRLF versus committed LF Lua; unchanged from PRE/POST and declared before commands.';qualified.append(path)
    rows.append(entry)
assert sorted(qualified)==sorted(frozen['inherited_Lua_qualifications'])
packet=[]
for path in sorted(p for p in tree if p.startswith(PACK+'/')):
    raw=(W/path).read_bytes();digest,size=blobs[tree[path]];assert digest==sha(raw) and size==len(raw)
    packet.append({'path':path,'git_blob':tree[path],'sha256':digest})
extras=[]
for path,digest in extra['files'].items():
    location=W/path if (W/path).is_file() else E/path
    assert sha(location.read_bytes())==digest,path
    if path in tree:assert blobs[tree[path]][0]==digest
    extras.append({'path':path,'sha256':digest,'current_bytes_equal':True,'committed_source':path in tree})
assert not D.exists();D.mkdir()
new(D/'.gitattributes',b'* -text\n')
result={'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'code_commit':code,'source_rows':len(rows),'literal_source_rows':len(rows)-len(qualified),'qualified_source_rows':qualified,'source_rows_join':rows,'packet_files':len(packet),'literal_packet_join':packet,'extras':extras,'freeze_sha256':sha((P/'D3-NATIVE-READ-CORRECTION-FROZEN-20261108.json').read_bytes()),'materialized_plan_sha256':sha((P/'native-read-nr602-materialized-plan-v2-20261108.json').read_bytes()),'test_actuals':{'final_direct_commands':5,'final_direct_exits':[0,0,0,0,0],'rust_tests':11,'Unix_driver_tests':9,'actual_converter_tests':1,'profile_census_tests':1,'external_Python_scope':'Unix only; all eight Lua controls remain executable under Wine, new-head CI required','independent_actual_Lua_report_validations':7,'python_offline_controls':5},'post_POST_deltas':'Only packet freeze/publication, one top STATUS, active card and this metadata; tested source/helper/extras unchanged. Raw Cargo logs retain original blank EOF/trailing whitespace; code diff check excluding literal packet passed. No receipt normalization.','limits':frozen['limits'],'agent_completion':'Root will record actual service-agent completion separately, never a fabricated CLI exit.'}
dump(D/'committed-bytes-and-checkpoint.json',result)
new(D/'README.md',('# Committed-byte publication (UNREVIEWED)\n\nCode: `'+code+'`. Actual Git blob bytes join4004 source rows:4001 exact plus three inherited Lua CRLF/LF qualifications; all70 packet files literal. Final5 direct0; eleven Rust tests (nine Unix driver controls, actual converter1, existing census1), seven independently validated Lua-produced reports and five Python controls. Root materialized V2plan4ee21a2c and preflight0 are copy/read-only prerequisites, not native launch clearance. No Windows/native/retail execution or production semantic change.\n\n'+frozen['limits']+'\n\nSTATUS/card and this publication follow POST, no tested input changes. Raw log whitespace is retained literally. STOP for SAME source/controller review.\n').encode())
new(D/'publisher.py',pathlib.Path(__file__).read_bytes())
files=[{'path':str(p.relative_to(D)).replace('\\','/'),'sha256':sha(p.read_bytes())} for p in sorted(D.rglob('*')) if p.is_file()]
dump(D/'manifest.json',{'files':files,'scope':'Post-CODE actual Git byte publication only; no new tests/native/acceptance'})
print(json.dumps({'code':code,'source_rows':len(rows),'exact':len(rows)-len(qualified),'packet_files':len(packet),'extras':len(extras),'metadata_files':len(files)+1}))
