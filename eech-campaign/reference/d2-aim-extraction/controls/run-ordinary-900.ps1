# One fresh-root 900 s whole-campaign run with ORDINARY (shipping) binaries (one native process), the flags of the retained supplied-binary side ab-base:
# hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 observe_supply=0, seed 1, 100 ms frames (campaign.lua defaults).
param([Parameter(Mandatory = $true)] [string] $Bin, [Parameter(Mandatory = $true)] [string] $Name)
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$py = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$tools = 'R:\eech_source_code\eech-campaign\tools'
$root = "$base\roots\$Name"
$out = "$base\out\$Name"
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-$Name.log" -Append }
if (Test-Path $root) { throw "root $root exists: one root per run" }
Stamp "copy fresh root $root"
Copy-Item -Recurse 'R:\d2-evidence\candidate-roots-d2-aiming\lebanon' $root
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-before-$Name.log"
New-Item -ItemType Directory -Force $out | Out-Null
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
$f = { param($p) ($p -replace '\\', '/') }
$arguments = @((& $f (Join-Path $bin 'lua\campaign.lua')), "root=$(& $f $root)", 'scenario=lebanon_retail', 'hours=0.25', 'record_every=10', 'checkpoint_every=3600', 'observe_every=60',
	"acmi=$(& $f "$out\recording.acmi")", "observe=$(& $f "$out\observations.jsonl")", "metrics=$(& $f "$out\metrics.json")", 'observe_supply=0') -join ' '
Stamp "start 900 s ordinary run: $arguments"
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError "$out\run.log" -RedirectStandardOutput "$out\run.out" -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
Stamp ("end ${Name}: native exit=$($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-after-$Name.log"
Stamp "DONE $Name exit=$($proc.ExitCode)"
