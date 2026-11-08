import pathlib,json,datetime,hashlib
E=pathlib.Path('R:/d3-evidence')
rows=[]
for old,new in [('run-physical-flags-nr395-direct-20261108.py','run-physical-flags-nr602-direct-20261108.py'),('check-physical-flags-nr395-linux-20261108.sh','check-physical-flags-nr602-linux-20261108.sh')]:
    raw=(E/old).read_bytes();out=raw.replace(b'nr395',b'nr602').replace(b'FLAG PROVIDER LIFETIME batch',b'PUBLIC READ REVIEW CORRECTION batch')
    with (E/new).open('xb') as f:f.write(out)
    rows.append({'old':old,'old_sha256':hashlib.sha256(raw).hexdigest(),'new':new,'sha256':hashlib.sha256(out).hexdigest()})
with (E/'nr602-helper-derivation-20261108.json').open('x') as f:json.dump({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'rows':rows,'scope':'New family only; exact old cache/image retained, no old helper changed'},f,indent=2)
