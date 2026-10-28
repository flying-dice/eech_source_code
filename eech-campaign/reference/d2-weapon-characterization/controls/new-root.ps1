# new-root.ps1 -Name <unique> [-Map lebanon|georgia]: fresh private copy of the verified bundle root, then the repository's own input manifest compared with its reference.
param([Parameter(Mandatory=$true)][string]$Name, [string]$Map = 'lebanon')
$ErrorActionPreference = 'Stop'
$python = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$repo = 'R:\eech_source_code\eech-campaign'
$base = 'R:\d2-evidence\weapon-characterization-20261027'
$src = "R:\eech-regression-20261025\roots\$Map"
$dst = "$base\roots\$Name"
if (Test-Path $dst) { throw "refusing to reuse $dst" }
$ref = if ($Map -eq 'georgia') { "$repo\reference\m4-s3-mutation\inputs-georgia.json" } else { "$repo\reference\lebanon-3h\inputs.json" }
robocopy $src $dst /E /NFL /NDL /NJH /NJS /NP | Out-Null
$rc = $LASTEXITCODE
if ($rc -ge 8) { throw "robocopy exit $rc" }
$out = & $python -B -X utf8 "$repo\tools\input-manifest.py" $dst --compare $ref 2>&1
$mc = $LASTEXITCODE
"$Name robocopy=$rc manifest=$mc : $out"
exit $mc
