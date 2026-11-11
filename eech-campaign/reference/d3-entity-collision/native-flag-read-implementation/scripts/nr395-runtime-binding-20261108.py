import hashlib,json,pathlib
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
p=E/'supply-boolean-native-windows-3abf-20261107-completion.json';b=json.loads(p.read_bytes());o=b['profiles']['ordinary']
assert b['actual_exit']==0 and o['actual_exit']==0 and o['verified_payloads']==91
out=W/'eech-campaign/tools/native-flag-read-runtime.json'
with out.open('x',encoding='utf-8',newline='\n') as f:
 json.dump({'runtime':b['source'],'profile':'ordinary HOSTED x86_64-pc-windows-gnu release','build_receipt':p.name,'build_receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'build_info_sha256':o['build_info_sha256'],'producer_image':o['producer_image'],'payloads':o['payloads'],'scope':'Expected OLD3abf payload identities from actual sealed build receipt. This is NOT verification/materialization of a new bundle or current implementation binaries.'},f,indent=2);f.write('\n')
print('Recorded expected91 payload rows; no binary copied, built or launched')
