import pathlib,json,hashlib
E=pathlib.Path('R:/d3-evidence');rows=[]
for old,new in [('run-physical-flags-nr602-direct-20261108.py','run-core-crt-crt1516-direct-20261108.py'),('check-physical-flags-nr602-linux-20261108.sh','check-core-crt-crt1516-linux-20261108.sh')]:
    raw=(E/old).read_bytes();b=raw.replace(b'nr602',b'crt1516').replace(b'run-physical-flags-crt1516',b'run-core-crt-crt1516').replace(b'check-physical-flags-crt1516',b'check-core-crt-crt1516').replace(b'flags-crt1516-',b'core-crt-crt1516-').replace(b'Root-native-read-feedback',b'Root-core-crt-feedback').replace(b'PUBLIC READ REVIEW CORRECTION',b'CORE LOADER CRT OBSERVER')
    with (E/new).open('xb') as f:f.write(b)
    rows.append({'old':old,'old_sha256':hashlib.sha256(raw).hexdigest(),'new':new,'new_sha256':hashlib.sha256(b).hexdigest()})
with (E/'crt1516-helper-derivation-20261108.json').open('x') as f:json.dump(rows,f,indent=2)
