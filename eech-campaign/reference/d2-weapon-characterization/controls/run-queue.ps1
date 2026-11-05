# run-natives.ps1 -Bin <fixture build dir> -Plan <comma list of name:kind[:arg]> [-Tag <log name>]
# Sequential evidence runs (ONE native process at a time), each in a fresh root under roots\<name> and a fresh out\<name>.
#   kind w600  : lua/weapon-characterization.lua, Lebanon, seconds = arg (default 600), 100 ms frames, 60 s checkpoints -> out\<name>\trace.jsonl
#   kind case  : lua/weapon-case.lua (controlled launch witness), warmup seconds = arg (default 60); settle cap remains 700 -> out\<name>\trace.jsonl
#   kind ab900 : whole-campaign control with the hooks on: lua/campaign.lua hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 observe_supply=0
#                                                                                                                         -> out\<name>\{recording.acmi,observations.jsonl,metrics.json}
# A PowerShell setup or terminating exception ends the plan too, with an explicit PLAN FAILED line and exit 1 (it records no native exit).
# Each run's ACTUAL exit status is captured immediately after the run, logged, and any nonzero status ends the plan: the remaining runs are NOT started and this script
# exits with that status. "PLAN COMPLETE" is written only when every run exited 0, and it is not evidence by itself: the per-run lines and the traces are.
param([Parameter(Mandatory=$true)][string]$Bin, [Parameter(Mandatory=$true)][string]$Plan, [string]$Tag = 'run')
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\weapon-characterization-20261027'
$tools = 'R:\eech_source_code\eech-campaign\tools'
$log = "$base\$Tag.log"
function Stamp($m) { $line = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') $m"; $line | Out-File -FilePath $log -Append -Encoding utf8; $line }
$f = { param($p) ($p -replace '\\', '/') }
$done = 0
foreach ($item in $Plan.Split(',')) {
	$name = '?'
	try {
		$parts = $item.Split(':'); $name = $parts[0]; $kind = $parts[1]
		$out = "$base\out\$name"
		if (Test-Path $out) { throw "refusing to reuse $out" }
		$root = "$base\roots\$name"
		if (-not (Test-Path $root)) { throw "no root $root" }
		New-Item -ItemType Directory -Force $out | Out-Null
		$exit = $null
		if ($kind -eq 'w600' -or $kind -eq 'case') {
			$arg = if ($parts.Length -gt 2) { [int]$parts[2] } else { if ($kind -eq 'w600') { 600 } else { 60 } }
			Stamp "start ${name} (${kind}, ${arg}) bin=${Bin} root=${root}"
			if ($kind -eq 'w600') {
				$o = & "$tools\weapon-characterization-windows.ps1" -Bin $Bin -Root $root -Out $out -Scenario lebanon_retail -Seconds $arg
			} else {
				$o = & "$tools\weapon-case-windows.ps1" -Bin $Bin -Root $root -Out $out -Warmup $arg
			}
			$exit = $LASTEXITCODE
			Stamp "end ${name} : script/native exit=${exit}; ${o}"
		} elseif ($kind -eq 'ab900') {
			$world = Join-Path $Bin 'eech-world.exe'
			$lua = & $f (Join-Path $Bin 'lua\campaign.lua')
			$arguments = @($lua, "root=$(& $f $root)", 'scenario=lebanon_retail', 'hours=0.25', 'record_every=10', 'checkpoint_every=3600', 'observe_every=60',
				"acmi=$(& $f "$out\recording.acmi")", "observe=$(& $f "$out\observations.jsonl")", "metrics=$(& $f "$out\metrics.json")", 'observe_supply=0') -join ' '
			Stamp "start ${name} (${kind}) bin=${Bin}: ${arguments}"
			$start = Get-Date
			$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError "$out\run.log" -RedirectStandardOutput "$out\run.out" -ArgumentList $arguments
			$null = $proc.Handle
			$proc.WaitForExit()
			$exit = $proc.ExitCode
			Stamp ("end ${name} : native exit=${exit} after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
		} else { throw "unknown kind $kind" }
		if ($exit -ne 0) {
			Stamp "PLAN FAILED at ${name}: exit ${exit}; the remaining runs were NOT started (${done} earlier runs exited 0)"
			exit $(if ($exit) { $exit } else { 1 })
		}
		$done++
	} catch {
		# a setup or terminating exception: no native exit exists for it, none is invented
		Stamp "PLAN FAILED at ${name}: setup/terminating exception before or around the native (no native exit recorded): $($_.Exception.Message)"
		Stamp "exception detail: $($_ | Out-String)"
		exit 1
	}
}
Stamp "PLAN COMPLETE: ${done} runs, every actual exit 0 (not evidence by itself: see the per-run lines and traces)"
exit 0
