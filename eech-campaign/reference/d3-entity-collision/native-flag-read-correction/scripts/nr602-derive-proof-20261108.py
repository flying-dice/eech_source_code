import pathlib,re,ast
E=pathlib.Path('R:/d3-evidence')
old=(E/'nr395-proof-20261108.py').read_text(encoding='utf-8');s=old.replace('nr395','nr602').replace('native-flag-read-implementation','native-flag-read-correction').replace('NATIVE-READ-IMPLEMENTATION','NATIVE-READ-CORRECTION')
def line(name,value):
    global s
    s,n=re.subn('^'+name+'=.*$',name+'='+repr(value),s,count=1,flags=re.M);assert n==1
line('READERS',['nr602-derive-helpers-20261108.py','nr602-add-dc-check-20261108.py','nr602-provenance-20261108.py','nr602-derive-proof-20261108.py','run-physical-flags-nr602-direct-20261108.py','check-physical-flags-nr602-linux-20261108.sh','run-nr602-final-20261108.py','nr602-proof-20261108.py','sector-roster-inputmap-batched-20261106.py'])
line('ROOTS',['codex-d3-native-read-correction-20261108-process.json','independent-native-flag-read-48570-source-controller-20261108.json','independent-native-flag-read-48570-source-controller-20261108-v2.json','native-read-48570-Root-binding-20261108.json','native-read-602f-Root-publication-20261108.json','codex-d3-native-read-implementation-20261108-completion.json','native-read-602f-all9-CI-terminal-20261108.json','native-read-nr602-materialized-plan-v2-20261108.json','native-read-nr602-staging-readonly-preflight-v2-20261108.json','nr602-production-prefix-identity-20261108.json']+['native-read-602f-CI-job-'+n+'-20261108.log' for n in ['128609409742','128609409746','128609409784','128609409851','128609409557']])
line('SOURCES',['eech-campaign/crates/eech-dc/src/module.rs','eech-campaign/crates/eech-dc/src/native_flag_read_refusal_check.rs','eech-campaign/crates/eech-world/lua/physical-flags-read-transaction.lua','eech-campaign/crates/eech-world/lua/physical-flags-read-witness.lua','eech-campaign/crates/eech-world/tests/host_callbacks.rs','eech-campaign/crates/eech-world/tests/physical_flags_read_driver.rs'])
s=s.replace("'eech-campaign/docs/d3-native-flag-read-implementation.md'","'eech-campaign/docs/d3-native-flag-read-implementation.md','eech-campaign/docs/d3-native-flag-read-correction.md'")
line('FINAL',['s1-fmt','s2-clippy','s3-read-driver','s4-converter','s5-census'])
line('DEV',{'d1-refusal-red':101,'d2-refusal-green':0,'d3-lua-and-census':0,'d4-fmt-apply':0,'d5-clippy':0})
needle="        if label=='s3-read-driver':"
s=s.replace(needle,"        if label in ('s4-converter','s5-census'):\n            assert results==[{'passed':1,'failed':0}],results\n"+needle)
s=s.replace("'nr602-driver-derivation-20261108.json'","'nr602-helper-derivation-20261108.json'")
start=s.index("    limits='");end=s.index('\n',start)
s=s[:start]+"    limits="+repr('Diagnostic-only correction: actual MODEL_BRIDGE/call_model refused channel and exact reason/table, scripted boot/SCENE, real canonical refusal over existing tiny-format model root; no successfulEngineboot/native campaign. Nine existing Unix controls incl seven actual-Lua-report validators/five Python controls; all eight Lua controls remain on Windows/Wine, external Python aggregate Unix-only. Census exact66/2 with guards unchanged. Raw module.rs adds only cfg(test) declaration; production converter prefix/NUL unchanged3abf, whole rawfile differs. Runtime same old3abf91payloads; no rebuild/relabel. No native/retail/fullpack launch; V2materialized plan/preflight0 only. All10/NativeAcceptance1/Task49/D3/currentCI/full40all8/fourinputs/review/merge open as applicable.')+s[end:]
s=s.replace('nr602 opt-in READ driver/controller source and offline controls UNREVIEWED','nr602 delegated-refusal/census/Wine harness correction UNREVIEWED')
s=s.replace('Opt-in READ implementation (UNREVIEWED)','READ correction (UNREVIEWED)')
s=s.replace('Developer d1 optional-GROUP fixture action-shape error and d3 synthetic reference-hash mismatch remain literal failed setup controls, not product RED. Old-driver optional-GROUP latch counterexample is an offline diagnostic control, not a campaign semantic defect. No completed old experiments replayed. Source/readers/extras sealed before final3; direct actual counts parsed afterwards. Root writes separate actual agent completion.','d1 genuine diagnostic-adapter RED retains actual callback/refused channel mismatch; d2 same control GREEN. Old exact602f Linux/Wine CI failures retained, no old CI replay. Source/readers/extras sealed before final5; counts parsed from direct logs. Root writes separate actual agent completion.')
s=s.replace('../../../docs/d3-native-flag-read-implementation.md','../../../docs/d3-native-flag-read-correction.md')
ast.parse(s)
with (E/'nr602-proof-20261108.py').open('x',encoding='utf-8') as f:f.write(s)
print('Derived bounded nr602 packet from existing nr395 script; no old file changed')
