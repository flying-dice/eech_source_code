import pathlib,json,hashlib,subprocess
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029';runtime='3abf03e5133279ae6ed53b657e76d4f7724f7dd3'
def git(*a):return subprocess.check_output(['git','-C',str(W),*a])
def sha(b):return hashlib.sha256(b).hexdigest()
module='eech-campaign/crates/eech-dc/src/module.rs';old=git('show',runtime+':'+module);current=(W/module).read_bytes()
suffix=b'\n#[cfg(test)]\n#[path = "native_flag_read_refusal_check.rs"]\nmod native_flag_read_refusal_check;\n'
assert current==old+suffix and current.count(b'\0')==old.count(b'\0')
paths=['aphavoc/source/entity/mobile/mb_hostentity.h','aphavoc/source/entity/mobile/mb_hostoffer.h','eech-campaign/crates/eech-world/csrc/entity_candidate_step.c','eech-campaign/crates/eech-world/src/entity_host.rs','eech-campaign/crates/eech-world/src/model_host.rs']
rows=[]
for n in paths:
    raw=(W/n).read_bytes();assert raw==git('show',runtime+':'+n);rows.append({'path':n,'sha256':sha(raw),'same_whole_file':True})
receipt={'runtime':runtime,'module':{'path':module,'runtime_sha256':sha(old),'current_sha256':sha(current),'same_whole_file':False,'entire_production_prefix_equal':True,'only_delta_utf8':suffix.decode(),'original_NUL_count_preserved':old.count(b'\0')},'other_five_protocol_files':rows,'limits':'No new Windows/runtime build or old binary relabel; actual converter control is a cfg(test) own-crate unit, scripted boot/SCENE, real converter and canonical provider over existing tiny-format installation; not native campaign acceptance.'}
with (E/'nr602-production-prefix-identity-20261108.json').open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2)
print(json.dumps(receipt['module']))
