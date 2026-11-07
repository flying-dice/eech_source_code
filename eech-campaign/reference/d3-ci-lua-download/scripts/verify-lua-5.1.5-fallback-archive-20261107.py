from pathlib import Path
import datetime,hashlib,json,urllib.request
B=Path('R:/d3-evidence');archive=B/'lua-5.1.5-MacPorts-verified-20261107.tar.gz';receipt=B/'lua-5.1.5-MacPorts-verification-20261107.json';assert not archive.exists() and not receipt.exists()
url='https://distfiles.macports.org/lua/lua-5.1.5.tar.gz';expected='2640fc56a795f29d28ef15e13c34a47e223960b0240e8cb0a82d9b0738695333'
with urllib.request.urlopen(url,timeout=30) as response:raw=response.read();status=response.status
archive.write_bytes(raw);digest=hashlib.sha256(raw).hexdigest();assert status==200 and digest==expected,(status,digest)
proof=dict(kind='Actual HTTPS fallback Lua archive identity verification; no repository/CI/source change',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),url=url,http_status=status,bytes=len(raw),sha256=digest,expected_existing_workflow_sha256=expected,archive=str(archive),scope='Identical pinned Lua5.1.5 official-source archive; mirror is MacPorts project distfiles, source listing read live. No credential/auth/baseline/dependency version change. Not retail data.')
receipt.write_text(json.dumps(proof,indent=2)+'\n',encoding='utf-8');print(json.dumps(proof))
