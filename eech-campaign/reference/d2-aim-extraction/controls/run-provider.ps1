# One fresh-root run of lua/aim-provider.lua (one native process). Writes R:\d2-evidence\aim-extraction-20261026\out\<Name>.
# -Bin may be a fixture build (the aiming report is written to trace.jsonl) or a shipping build (no report). -LuaDir overrides the directory of the script
# (used to run the NEW provider script against a private copy of an original build).
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Name,
	[Parameter(Mandatory = $true)] [string] $Mode,
	[int] $Seconds = 300,
	[string] $Scenario = 'lebanon_retail'
)
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$py = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$tools = 'R:\eech_source_code\eech-campaign\tools'
$root = "$base\roots\$Name"
$out = "$base\out\$Name"
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-$Name.log" -Append }
if (Test-Path $root) { throw "root $root exists: one root per run" }
New-Item -ItemType Directory -Force "$base\roots" | Out-Null
Stamp "copy fresh root $root"
Copy-Item -Recurse 'R:\d2-evidence\candidate-roots-d2-aiming\lebanon' $root
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-before-$Name.log"
New-Item -ItemType Directory -Force $out | Out-Null
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
$f = { param($p) ($p -replace '\\', '/') }
$arguments = @((& $f (Join-Path $bin 'lua\aim-provider.lua')), "root=$(& $f $root)", "scenario=$Scenario", "seconds=$Seconds", "mode=$Mode",
	"out=$(& $f "$out\summary.jsonl")", "trace=$(& $f "$out\trace.jsonl")") -join ' '
Stamp "start provider run $Name mode=$Mode seconds=$Seconds bin=$bin"
Stamp "arguments: $arguments"
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError "$out\run.log" -RedirectStandardOutput "$out\run.out" -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
Stamp ("end ${Name}: native exit=$($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
Select-String -Path "$out\run.log" -Pattern '^aim-provider:|EECH fatal|fatal error|host aim' | Select-Object -Last 3 | ForEach-Object { Stamp $_.Line }
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-after-$Name.log"
Stamp "DONE $Name exit=$($proc.ExitCode)"
