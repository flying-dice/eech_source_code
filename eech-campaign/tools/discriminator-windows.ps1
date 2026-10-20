# EEDCS-11 helicopter discriminator, TEST ONLY: runs the CONTROLLED build
# (tools/build-observed-windows.sh controlled) on retail scenarios with the
# wingman control on, in parallel, one root each. Each run is a CHANGED
# scenario, never windows-retail v1: it writes its own Tacview recording,
# structured observations (observe_supply=1), metrics and arrival trace, and
# is judged by tools/helicopter-discriminate.py and tools/arrival-check.py, not
# by the regression baselines. See docs/m5-helicopter-discriminator.md.
#
# Usage (PowerShell):
#   tools\discriminator-windows.ps1 -Bin <controlled build> -Out <dir> -Runs @(
#     'lebanon_retail|<root>|hold:four:1500:1000:120', 'lebanon_retail|<root>|hold:navigation:1500:1000:120', ...) [-Hours 3]
#   each root a separate copy; the same root never serves two runs
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Out,
	[Parameter(Mandatory = $true)] [string[]] $Runs,
	[double] $Hours = 3
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path (Join-Path $bin 'OBSERVED.txt'))) { throw "$bin is not a test-only observed/controlled build" }
if (-not (Select-String -Quiet -Path (Join-Path $bin 'BUILD-INFO.txt') -Pattern '^CONTROLLED')) { throw "$bin is not a CONTROLLED build" }
New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path $Out).Path
$python = (Get-Command python3, python -ErrorAction SilentlyContinue | Select-Object -First 1).Source
$roots = @{}
$procs = @()
$start = Get-Date
for ($i = 0; $i -lt $Runs.Count; $i++) {
	$scenario, $root, $control = $Runs[$i].Split('|')
	if ($roots.ContainsKey($root)) { throw "root $root serves two runs" }
	$roots[$root] = 1
	$manifest = if ($scenario -like 'georgia*') { 'reference\m4-s3-mutation\inputs-georgia.json' } else { 'reference\lebanon-3h\inputs.json' }
	& $python (Join-Path $here 'tools\input-manifest.py') $root --compare (Join-Path $here $manifest) | Out-Null
	if ($LASTEXITCODE -ne 0) { throw "root $root differs from $manifest" }
	$dir = Join-Path $out ("run{0}-{1}-{2}" -f ($i + 1), $scenario, ($control -split ':')[1])
	New-Item -ItemType Directory -Force (Join-Path $dir 'trace') | Out-Null
	Set-Content -Path (Join-Path $dir 'run.txt') -Value "scenario=$scenario`nroot=$root`nEECH_ARRIVAL_CONTROL=$control`nhours=$Hours`nbin=$bin"
	$env:EECH_ARRIVAL_TRACE = (Join-Path $dir 'trace') -replace '\\', '/'
	$env:EECH_ARRIVAL_CONTROL = $control
	$procs += Start-Process -FilePath $world -WorkingDirectory $dir -NoNewWindow -PassThru -RedirectStandardError (Join-Path $dir 'run.log') -RedirectStandardOutput (Join-Path $dir 'run.out') -ArgumentList @(
		((Join-Path $bin 'lua\campaign.lua') -replace '\\', '/'), "root=$($root -replace '\\', '/')", "scenario=$scenario", "hours=$Hours",
		'record_every=10', 'checkpoint_every=3600', 'observe_every=60', 'observe_supply=1',
		"acmi=$((Join-Path $dir 'recording.acmi') -replace '\\', '/')", "observe=$((Join-Path $dir 'observations.jsonl') -replace '\\', '/')",
		"metrics=$((Join-Path $dir 'metrics.json') -replace '\\', '/')")
	$null = $procs[-1].Handle
}
Remove-Item Env:EECH_ARRIVAL_CONTROL, Env:EECH_ARRIVAL_TRACE
$status = 0
for ($i = 0; $i -lt $procs.Count; $i++) {
	$procs[$i].WaitForExit()
	Write-Host ("{0}: exit {1}" -f $Runs[$i], $procs[$i].ExitCode)
	if ($procs[$i].ExitCode -ne 0) { $status = 1 }
}
Write-Host ("runs took {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
exit $status
