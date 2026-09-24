# The campaign regression test, natively on Windows: eech-world.exe and
# eech_dc.dll (tools/build-windows.sh) run the retail Lebanon and Georgia
# campaigns for 3 simulated hours each, in parallel. Each run must:
#   1. play out as an EECH campaign should (tools/campaign-expectations.py:
#      tasking, combat, attrition, regen, supply, production, captures);
#   2. match its baseline in regression/windows/ (tools/regress-compare.py):
#      identical, as runs are deterministic, or within tolerance.
# See regression/README.md.
#
# Usage (PowerShell):
#   tools\regress-windows.ps1 -Georgia <root> -Lebanon <root> [-Update] [-Exact] [-Build]
#   roots: tools/retail-map3-installs.sh and tools/retail-cvh.sh (or tools/regress-docker.sh's, copied out)
#   -Build  runs tools/build-windows.sh (Docker) first
param(
	[Parameter(Mandatory = $true)] [string] $Georgia,
	[Parameter(Mandatory = $true)] [string] $Lebanon,
	[switch] $Update,
	[switch] $Exact,
	[switch] $Build,
	[double] $Hours = 3
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = Join-Path $here 'target\windows'
$out = Join-Path $here 'target\regress-windows'
$baselines = Join-Path $here 'regression\windows'
New-Item -ItemType Directory -Force $out, $baselines | Out-Null

if ($Build) {
	& sh (Join-Path $here 'tools/build-windows.sh')
	if ($LASTEXITCODE -ne 0) { throw 'tools/build-windows.sh failed' }
}
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found: run with -Build, or tools/build-windows.sh" }

$python = (Get-Command python3, python -ErrorAction SilentlyContinue | Select-Object -First 1).Source
if (-not $python) { throw 'Python 3 is needed for tools/regress-compare.py' }

$start = Get-Date
Write-Host "running georgia_retail and lebanon_retail for $Hours simulated hours each ..."
$runs = @{}
foreach ($s in @(@{ Name = 'georgia_retail'; Root = $Georgia }, @{ Name = 'lebanon_retail'; Root = $Lebanon })) {
	$metrics = Join-Path $out "$($s.Name).json"
	$log = Join-Path $out "$($s.Name).log"
	$runs[$s.Name] = Start-Process -FilePath $world -NoNewWindow -PassThru -RedirectStandardError $log -RedirectStandardOutput "$log.out" -ArgumentList @(
		(Join-Path $bin 'lua\campaign.lua'), "root=$($s.Root -replace '\\', '/')", "scenario=$($s.Name)", "hours=$Hours",
		'record=0', 'checkpoint_every=3600', "metrics=$($metrics -replace '\\', '/')")
}
$failed = $false
foreach ($name in $runs.Keys) {
	$p = $runs[$name]
	$p.WaitForExit()
	if ($p.ExitCode -ne 0) { Write-Host "$name failed (exit $($p.ExitCode)): see $out\$name.log"; $failed = $true }
}
Write-Host ("runs took {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
if ($failed) { exit 1 }

$status = 0
foreach ($name in 'georgia_retail', 'lebanon_retail') {
	$current = Join-Path $out "$name.json"
	$base = Join-Path $baselines "$name.json"
	Write-Host ''
	# the campaign played out as EECH's should: every mechanic present (no reference needed)
	& $python (Join-Path $here 'tools\campaign-expectations.py') $current
	if ($LASTEXITCODE -ne 0) { Write-Host "${name}: FAIL (campaign expectations)"; $status = 1 }
	if ($Update -or -not (Test-Path $base)) {
		Copy-Item $current $base -Force
		& $python (Join-Path $here 'tools\regress-compare.py') $base
		Write-Host "baseline written: regression/windows/$name.json"
	} else {
		$arguments = @((Join-Path $here 'tools\regress-compare.py'), $base, $current)
		if ($Exact) { $arguments += '--exact' }
		& $python @arguments
		if ($LASTEXITCODE -eq 0) { Write-Host "${name}: PASS" } else { Write-Host "${name}: FAIL"; $status = 1 }
	}
}
exit $status
