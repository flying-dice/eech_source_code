$ErrorActionPreference = 'Continue'
$N = 'E:\eech-runs\m5-d1\aircraft-control-20261023\native'
$W = 'E:\eech_source_code\.claude\worktrees\m5-aircraft-control\eech-campaign'
Set-Location $W
$env:PYTHONIOENCODING = 'utf-8'
& 'C:\Python313\python.exe' -I "$W\tools\regression-pack.py" --georgia E:/eech-runs/roots/georgia --lebanon E:/eech-runs/m2/lebanon-root --lebanon-repeat E:/eech-runs/m2/lebanon-m1-copy --bin "$W\target\aircraft-control" --out "$N\pack" > "$N\pack.out.log" 2> "$N\pack.err.log"
$LASTEXITCODE | Out-File -Encoding ascii "$N\pack.exit"
