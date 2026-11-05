# One fresh-root run of lua/aim-lead-probe.lua (one native process, about a minute). Writes R:\d2-evidence\aim-extraction-20261026\out\<Name>.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Name,
	[string] $Mode = 'all'
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
Stamp "copy fresh root $root"
Copy-Item -Recurse 'R:\d2-evidence\candidate-roots-d2-aiming\lebanon' $root
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-before-$Name.log"
New-Item -ItemType Directory -Force $out | Out-Null
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
$f = { param($p) ($p -replace '\\', '/') }
$arguments = @((& $f (Join-Path $bin 'lua\aim-lead-probe.lua')), "root=$(& $f $root)", 'scenario=lebanon_retail', "mode=$Mode", "out=$(& $f "$out\trace.jsonl")") -join ' '
Stamp "start lead probe $Name mode=$Mode bin=$bin"
Stamp "arguments: $arguments"
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError "$out\run.log" -RedirectStandardOutput "$out\run.out" -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
Stamp ("end ${Name}: native exit=$($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
Select-String -Path "$out\run.log" -Pattern '^aim-lead-probe:|fatal|host aim' | Select-Object -Last 3 | ForEach-Object { Stamp $_.Line }
& $py -X utf8 "$tools\input-manifest.py" --compare 'R:\eech_source_code\eech-campaign\reference\lebanon-3h\inputs.json' $root 2>&1 | Tee-Object -FilePath "$base\roots-manifest-after-$Name.log"
Stamp "DONE $Name exit=$($proc.ExitCode)"
