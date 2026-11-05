# One fresh-root 600 s Lebanon aim-characterization run of a fixture build (one native process). Writes R:\d2-evidence\aim-extraction-20261026\out\<Name>.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Name,
	[int] $Seconds = 600
)
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$py = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$tools = 'R:\eech_source_code\eech-campaign\tools'
$root = "$base\roots\$Name"
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-$Name.log" -Append }
if (Test-Path $root) { throw "root $root exists: one root per run" }
New-Item -ItemType Directory -Force "$base\roots" | Out-Null
Stamp "copy fresh root $root"
Copy-Item -Recurse 'R:\d2-evidence\candidate-roots-d2-aiming\lebanon' $root
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-before-$Name.log"
Stamp "start 600 s Lebanon aim-characterization $Name bin=$Bin"
$o = & "$tools\aim-characterization-windows.ps1" -Bin $Bin -Root $root -Out "$base\out\$Name" -Scenario lebanon_retail -Seconds $Seconds
Stamp "end ${Name}: script exit=$LASTEXITCODE; $o"
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-after-$Name.log"
Stamp "DONE $Name"
