"""POST-publication only: committed-byte join and tracked checkpoint, no test replay."""
import datetime,hashlib,json,pathlib,subprocess
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def sha(b):return hashlib.sha256(b).hexdigest()
code=git('rev-parse','HEAD').decode().strip();assert code=='0c7b61bac977075f172bc3d55c2520d2f8aa1f51'
pack='eech-campaign/reference/d3-entity-collision/physical-flags-lifetime'
post=json.loads((E/'fl667-inputmap-POST-20261108.json').read_bytes());freeze=json.loads((E/'D3-FLAG-LIFETIME-FROZEN-20261108.json').read_bytes())
manifest=json.loads((W/pack/'manifest.json').read_bytes())
expected={p:r['raw_sha256'] for p,r in post['files'].items()}
doc='eech-campaign/docs/d3-flag-provider-lifetime.md';expected[doc]=freeze['extra_files'][doc]
expected.update({pack+'/'+r['path']:r['sha256'] for r in manifest['files']});expected[pack+'/manifest.json']=sha((W/pack/'manifest.json').read_bytes())
tree={}
for row in git('ls-tree','-r','-z',code).split(b'\0'):
 if row:
  meta,p=row.split(b'\t',1);tree[p.decode()]=meta.split()[2].decode()
assert {p for p in tree if p.startswith(pack+'/')}=={pack+'/'+r['path'] for r in manifest['files']}|{pack+'/manifest.json'}
ids=sorted({tree[p] for p in expected});raw=subprocess.run(['git','-C',str(W),'cat-file','--batch'],input=('\n'.join(ids)+'\n').encode(),capture_output=True,check=True).stdout
actual={};offset=0
for oid in ids:
 end=raw.index(b'\n',offset);header=raw[offset:end].split();assert header[0].decode()==oid and header[1]==b'blob';n=int(header[2]);actual[oid]=sha(raw[end+1:end+1+n]);offset=end+1+n+1
assert offset==len(raw)
different=[p for p,s in expected.items() if actual[tree[p]]!=s];assert sorted(different)==sorted(freeze['inherited_Lua_qualifications'])
assert all(sha((W/p).read_bytes())==s for p,s in expected.items())
meta=W/'eech-campaign/reference/d3-entity-collision/physical-flags-lifetime-publication';meta.mkdir()
receipt={'kind':'Post-POST actual committed Git blob byte join; no experiment replay','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'code':code,'source_rows':post['file_count'],'source_raw_Git_exact':post['file_count']-len(different),'qualified_inherited_Lua':different,'packet_manifest_entries':len(manifest['files']),'packet_tracked':len(manifest['files'])+1,'packet_all_actual_Git_bytes_literal':True,'all_current_source_equal_POST':True,'new_contract_actual_Git_bytes_literal':True,'freeze_sha256':sha((E/'D3-FLAG-LIFETIME-FROZEN-20261108.json').read_bytes()),'final_actual_exits':{k:v['actual_exit'] for k,v in freeze['runs'].items() if k.startswith('s')},'final_selected_tests':4,'qualification':'This metadata helper was created after POST only for committed-byte publication/state. No sealed test source/helper changed. Three inherited Lua CRLF qualifications distinct from four Core filter-identity disagreements whose raw bytes are Git-exact. Raw receipts/log EOF bytes retained; source-only whitespace and per-command cr-at-eol,-blank-at-eof receipt checks each passed. Native UNRUN means no Windows/retail public campaign; compiled Linux C test entries executed and default Linux module built. NUL boot attempt stops before C/retail; private preboot entry is not successful boot/publicnative READ or broad lifetime/FPU/DCS acceptance.','helper_sha256':sha(pathlib.Path(__file__).read_bytes()),'state':'UNREVIEWED; STOP for SAME reviewer; all10/NativeAcceptance1/Task49/D3 OPEN'}
for name,data in [('.gitattributes',b'* -text\n'),('committed-bytes-and-checkpoint.json',(json.dumps(receipt,indent=2)+'\n').encode()),('publication-helper.py',pathlib.Path(__file__).read_bytes())]:
 with (meta/name).open('xb') as f:f.write(data)
status=W/'STATUS.md';old=status.read_text(encoding='utf-8');at=old.index('\n# HISTORICAL');prior=old[:at].replace('# CURRENT','# HISTORICAL (flag lifetime candidate superseded this summary)',1)
new='''# CURRENT — D3 Task49: supported flag provider/entry lifetime characterised; frozen UNREVIEWED (8 November 2026)

**Candidate:** code/proof `0c7b61bac977075f172bc3d55c2520d2f8aa1f51`; [contract](eech-campaign/docs/d3-flag-provider-lifetime.md), [packet](eech-campaign/reference/d3-entity-collision/physical-flags-lifetime/README.md), [committed-byte join](eech-campaign/reference/d3-entity-collision/physical-flags-lifetime-publication/committed-bytes-and-checkpoint.json). Safe boot consumes an owned provider into thread-local MODEL and rejects a second attempt before replacement; Engine is not Send/Sync. C retains model callback with NULL user across normal/error entries. Reference SharedRoster/FlagStore and C group keys are one persistent binding; replacement remains unsupported/unproved, with possible coincident fN alias, no fresh-store refusal guarantee.

**Finite actuals:** NUL-argument safe boot attempt fails BEFORE C/retail boot; one private preboot C entry uses exact production ENTER/LEAVE, owning engine unit's real with_host/model_callback, real C flag binding and reference flag_answer/store. Six real callback calls/one group/word32; post-poison zero calls. Normal NULL-host and argument-error retention, second-provider rejection/drop, withheld-answer fatal and poisoned refusal pass in ONE isolated test process without BOOTED/poison/key/store resets. No successfulEngineboot, Lua, native/publicREAD, host/session/VM/thread replacement, fullglobal or FPU acceptance. Prior Task50 wrapper/FPU evidence remains at its own historical scope; old accepted tests not replayed.

**Verification:** six final direct commands0, outer session80196 actual0; lifetime1 + sourceguards3 = four selected tests. REAL PRE/POST4000 inputs/extras/sevenreaders equal. Actual committed bytes:3997/4000 source rows exact, only three inherited Lua CRLF/Git-LF qualifications;63packetfiles literal. Original engine C NUL retained; raw source adds one gated test-header include, no MODEL/BOOTED/ENTER/LEAVE behavior change. Default Linux module built0; no 3abf Windows binary relabel. Development compile101 and wrong-refusal-marker assertion101 retained as fixture errors, not product RED. Frozen candidate is not acceptance.

**STOP:** SAME independent changed-material review next. Both Root-owned immutable original fullpacks continue; prefix equality PARTIAL only, current40/all8/fourinputs pending. NativeAcceptance1/publicnativeREAD/changedanswer/fullLifetimes/FPU/neededDCS, wholeheight/sharedterrain/RNG/hierarchy/services/naturalmoving/finalcensus/currentCI/formalreview/normalmerge/all10/Task49/D3 OPEN. Motion SAME774, Tacview7a4/873c and supply43fe retain exact finite closures. Prep2 retained; no D4/D5/TypeScript. Root owns live D3-CURRENT, services and completion receipt.
'''
status.write_text(new+'\n'+prior+old[at:],encoding='utf-8',newline='\n')
card=W/'docs/handover/ACTIVE-EXTRACTION.json';c=json.loads(card.read_bytes())
c['phase']='Flag supported provider/entry lifetime finite candidate code0c7b61bac FROZEN UNREVIEWED; STOP for SAME reviewer. Both fullpacks running; all10/NativeAcceptance1/Task49/D3 OPEN.'
c['next_action']='STOP for SAME independent changed-material review of code0c7b61bac flag lifetime candidate. Root owns appropriate native/runtime qualification and remaining required whole-D3 gates; no closed-proof replay.'
c['ready_evidence'].append('UNREVIEWED flag lifetime candidate0c7b61bac: owned TLSprovider/oneboot supported source contract; actual NUL safe boot fails beforeC/retail, secondattempt rejects provider withoutreplacement; private preboot sharedENTER/LEAVE entry->ownRustcallback->realCflagbinding->realhoststore across normal/NULLhost/argumenterror/fatalpoison. Final6direct0/fourRusttests, real6callbacks1groupword32/poison0calls; stable4000/extras7readers. RawoneCinclude/NULretained; no3abfWindowsrelabel. Devcompile101/wrongexpectedearlyrefusal101 retained asfixtureerrors, notproductRED. Replacementunsupported/unproved possiblefNalias; no boot/nativeREAD/Lua/VMthread/fullFPU/lifetimeacceptance. Packet physical-flags-lifetime and publication beside it.')
for b in c['blockers']:
 if b['id']=='ENV-PHYSICAL-FLAGS-HOST-STORE-NATIVE-ACCEPTANCE-1':
  b['evidence']+=' Flag lifetime finite candidate0c7b61bac frozen UNREVIEWED, sixactualchecks0/fourRusttests/stable4000; explicit private preboot entry limits, no current successfulboot/nativeREAD/FPU/full lifetime acceptance.'
  b['missing_fact']='SAME review of supported-provider/entry-lifetime finite candidate; generic public native READ/changed fact, complete real successful callback/session/VM/thread lifetime and current whole-FPU facts, neededDCS disposition and complete CURRENT shipping40/all8/fourinputs remain required. Unsupported replacement remains explicit, possible fN alias, no refusal guarantee.'
  b['next_fix']='STOP for SAME review of bounded flag lifetime candidate0c7b61bac; Root then appropriate remaining runtime/whole-D3 gates, no automatic replacement/reset framework.'
card.write_text(json.dumps(c,indent=2)+'\n',encoding='utf-8',newline='\n')
print(json.dumps(receipt))
