"""Narrow explicit derivation from existing direct runner/cache helper only.
Old mfr332 bytes are read, never changed. No old final-set/freezer rerun.
"""
import hashlib,json,pathlib
E=pathlib.Path('R:/d3-evidence');W=E/'collision-characterization-20261029'
rows=[]
for old,new in [('run-physical-flags-mfr332-direct-20261107.py','run-physical-flags-fl667-direct-20261108.py'),('check-physical-flags-mfr332-linux-20261107.sh','check-physical-flags-fl667-linux-20261108.sh')]:
    raw=(E/old).read_bytes();s=raw.decode().replace('mfr332','fl667').replace('20261107','20261108').replace('Root-motion-flags-feedback','Root-flag-lifetime-feedback')
    # The old cache helper comment has a dated receipt only; lease names must stay literal unchanged.
    s=s.replace('eech-linux-d3-physical-flags-9d247-20261108','eech-linux-d3-physical-flags-9d247-20261107')
    with (E/new).open('xb') as f:f.write(s.encode())
    rows.append({'source':old,'source_sha256':hashlib.sha256(raw).hexdigest(),'derived':new,'sha256':hashlib.sha256(s.encode()).hexdigest()})
source=W/'eech-campaign/crates/eech-engine-sys/csrc/eech_engine.c';raw=source.read_bytes()
needle=b'const char *eech_engine_fatal_message (void)'
assert raw.count(needle)==1 and raw.count(b'\x00')==1
source.write_bytes(raw.replace(needle,b'#include "eech_flags_lifetime_check.h"\n\n'+needle))
assert source.read_bytes().count(b'\x00')==1
with (E/'fl667-derivation-20261108.json').open('x',encoding='utf-8') as f:json.dump(rows,f,indent=2);f.write('\n')
print('Two derived helpers created exclusively; exact cache/image retained; one C include added, original NUL retained.')
