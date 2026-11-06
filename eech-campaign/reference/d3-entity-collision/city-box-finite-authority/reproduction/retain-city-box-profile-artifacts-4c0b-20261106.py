from pathlib import Path
import datetime,hashlib,json,re,subprocess
b=Path('R:/d3-evidence');ev=Path('D:/EECH-Migration-Evidence')
dest=ev/'city-box-profile-artifacts-4c0b-20261106';assert not dest.exists();dest.mkdir()
proof=json.loads((b/'city-box-windows-profile-isolation-4c0b-20261106.json').read_bytes())
build=json.loads((b/'city-box-corrected-4c0b89263-windows-20261106-completion.json').read_bytes());cache=build['cache']
base=['wsl','-d','EECH-D2-Build','--cd','/','--','docker']
assert subprocess.check_output(base+['ps','-aq','--filter','volume='+cache],text=True).strip()==''
lines=['#!/bin/sh','set -eu','base=/target/x86_64-pc-windows-gnu/release','for profile in ordinary hosted original; do',' case "$profile" in']
for role in ['ordinary','hosted','original']:
    suffix=Path(proof['profiles'][role]['rlib']).name.removeprefix('libeech_engine_sys-').removesuffix('.rlib')
    root='/src/eech-campaign/'+Path(build['profiles'][role]['binary']).as_posix().split('/eech-campaign/',1)[1]
    lines.append(f' {role}) suffix={suffix}; root={root} ;;')
lines+=[' esac',' mkdir "/out/$profile"',' cp "$base/.fingerprint/eech-engine-sys-$suffix/lib-eech_engine_sys.json" "/out/$profile/cargo-feature-manifest.json"',' lib="$base/deps/libeech_engine_sys-$suffix.rlib"',' x86_64-w64-mingw32-ar t "$lib" >/tmp/members.txt'," sed -n '/eech_engine\\.o$/p;/mb_hostplace\\.o$/p;/eech_wpn_fixture\\.o$/p;/eech_mq_report_check\\.o$/p' /tmp/members.txt >/tmp/selected.txt",' while IFS= read -r member; do','  x86_64-w64-mingw32-ar p "$lib" "$member" >"/out/$profile/$member"','  x86_64-w64-mingw32-nm -g --defined-only "/out/$profile/$member" >"/out/$profile/$member.symbols.txt"',' done </tmp/selected.txt',' x86_64-w64-mingw32-objdump -p "$root/eech_dc.dll" >"/out/$profile/dll-exports.txt"','done']
helper=b/'retain-city-box-profile-artifacts-4c0b-20261106.sh';assert not helper.exists();helper.write_bytes(('\n'.join(lines)+'\n').encode())
image=re.search(r'^image: (.+)$',build['profiles']['original']['build_info_header'],re.M).group(1)
args=base+['run','--rm','-v',cache+':/target:ro','-v','/mnt/d/EECH-Migration-Evidence/city-box-corrected-4c0b89263-20261106-source:/src:ro','-v','/mnt/d/EECH-Migration-Evidence/city-box-profile-artifacts-4c0b-20261106:/out','-v','/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d3-evidence:/audit:ro',image,'sh','/audit/'+helper.name]
with (b/'retain-city-box-profile-artifacts-4c0b-20261106.stdout.log').open('xb') as o,(b/'retain-city-box-profile-artifacts-4c0b-20261106.stderr.log').open('xb') as e:code=subprocess.run(args,stdout=o,stderr=e).returncode
assert code==0
log=(b/'city-box-windows-rlib-binding-v2-4c0b-20261106.log').read_text();expected={}
for section in re.split(r'(?m)^=== BOUND_RLIB ',log)[1:]:
    manifest=section.splitlines()[0].split()[0];role=next(k for k,v in proof['profiles'].items() if v['manifest_path']==manifest)
    for match in re.finditer(r'(?m)^=== BOUND_MEMBER (.+)\n([0-9a-f]{64})\s',section):expected[(role,match[1])]=match[2]
assert len(expected)==12
for (role,member),want in expected.items():assert hashlib.sha256((dest/role/member).read_bytes()).hexdigest()==want
files=[{'path':p.relative_to(dest).as_posix(),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(dest.rglob('*')) if p.is_file()];assert len(files)==30
for profile in build['profiles'].values():
    for payload in profile['payloads']:assert hashlib.sha256((Path(profile['binary'])/payload['path']).read_bytes()).hexdigest()==payload['sha256']
receipt=dict(source=build['source'],cache=cache,actual_exit=0,retained_objects=12,retained_files=30,files=files,object_bytes_match_original_bound_audit=True,all_three_profiles_rehashed_payloads_each=88,cache_removed=False,scope='Retained actual bound COFF/features/exports. No new source/build/native test or acceptance. Full RLIB not retained; cache not yet removed.',completed_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
(dest/'preservation.json').write_bytes((json.dumps(receipt,indent=2)+'\n').encode());print(json.dumps({'actual_exit':0,'objects':12,'files':30,'sha256':hashlib.sha256((dest/'preservation.json').read_bytes()).hexdigest()}))
