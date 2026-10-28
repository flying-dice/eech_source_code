# One fresh-root controlled person case (one native process): tools/aim-person-case-windows.ps1 on a private copy of the Lebanon inputs.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Name,
	[Parameter(Mandatory = $true)] [string] $Case
)
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$py = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$tools = 'R:\eech_source_code\eech-campaign\tools'
$root = "$base\roots\$Name"
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-$Name.log" -Append }
if (Test-Path $root) { throw "root $root exists: one root per run" }
Stamp "copy fresh root $root"
Copy-Item -Recurse 'R:\d2-evidence\candidate-roots-d2-aiming\lebanon' $root
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-before-$Name.log"
Stamp "start person case $Case $Name bin=$Bin"
$o = & "$tools\aim-person-case-windows.ps1" -Bin $Bin -Root $root -Out "$base\out\$Name" -Scenario lebanon_retail -Case $Case
$code = $LASTEXITCODE
Stamp "end ${Name}: script exit=$code; $o"
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-after-$Name.log"
Stamp "DONE $Name exit=$code"
