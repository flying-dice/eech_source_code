# Sequential evidence runs of the committed aim fixture build (one native process at a time). Writes R:\d2-evidence\aim-characterization-20261026\out\<name>.
param([string] $Bin = 'R:\eech_source_code\eech-campaign\target\aim-fixture\f9f15f52f')
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-characterization-20261026'
$tools = 'R:\eech_source_code\eech-campaign\tools'
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-final.log" -Append }
foreach ($r in @(@('f-a', 'f-a'), @('f-b', 'f-b'))) {
	Stamp "start 600 s Lebanon witness $($r[0]) root $($r[1])"
	$o = & "$tools\aim-characterization-windows.ps1" -Bin $Bin -Root "$base\roots\$($r[1])" -Out "$base\out\$($r[0])" -Scenario lebanon_retail -Seconds 600
	Stamp "end $($r[0]): script exit=$LASTEXITCODE; $o"
}
# whole-campaign 900 s control with the hooks on: the same ordinary flags as the retained supplied-binary side ab-base
$out = "$base\out\f-ab-hooks"
New-Item -ItemType Directory -Force $out | Out-Null
$world = Join-Path $Bin 'eech-world.exe'
$lua = (Join-Path $Bin 'lua\campaign.lua') -replace '\\', '/'
$f = { param($p) ($p -replace '\\', '/') }
$arguments = @($lua, "root=$(& $f "$base\roots\f-c")", 'scenario=lebanon_retail', 'hours=0.25', 'record_every=10', 'checkpoint_every=3600', 'observe_every=60',
	"acmi=$(& $f "$out\recording.acmi")", "observe=$(& $f "$out\observations.jsonl")", "metrics=$(& $f "$out\metrics.json")", 'observe_supply=0') -join ' '
Stamp "start 900 s whole-campaign control: $arguments"
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError "$out\run.log" -RedirectStandardOutput "$out\run.out" -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
Stamp ("end f-ab-hooks: native exit=$($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
Stamp 'ALL DONE'
