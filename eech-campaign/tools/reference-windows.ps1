# The M2 reference scenario on Windows: retail Lebanon, recorded as Tacview
# and as structured observations from the same samples, checked against each
# other and against the run's metrics, and repeated (docs/m2-reference.md).
#
#   1. inputs      tools/input-manifest.py: SHA-256 of every input file
#   2. two runs    eech-world.exe lua/campaign.lua scenario=lebanon_retail
#                  with acmi=, observe= and metrics=; run 2 on -RepeatRoot
#                  (a second, separately assembled root) in parallel, or on
#                  -Root after run 1
#   3. checks      tools/observation-check.py on each run
#   4. repeat      the two runs' recording, observations and metrics must be
#                  byte-identical
#   5. unperturbed the metrics must equal the Windows regression baseline
#                  (which was recorded with no recording and no observations)
#                  and keep every campaign expectation
#
# Usage (PowerShell):
#   tools\reference-windows.ps1 -Root <Lebanon root> [-RepeatRoot <second Lebanon root>] [-Bin <dir>] [-Out <dir>] [-Hours 3]
#   roots: tools/retail-cvh.sh <Comanche vs Hokum install> map5 <root>
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $RepeatRoot,
	[string] $Bin,
	[string] $Out,
	[double] $Hours = 3
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { (Resolve-Path $Bin).Path } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\reference-windows' }
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found: run tools/build-windows.sh" }
$python = (Get-Command python3, python -ErrorAction SilentlyContinue | Select-Object -First 1).Source
New-Item -ItemType Directory -Force $out | Out-Null
$out = (Resolve-Path $out).Path
$status = 0
function Say ($text) { Write-Host $text; Add-Content -Path (Join-Path $out 'summary.txt') -Value $text }
Set-Content -Path (Join-Path $out 'summary.txt') -Value "M2 reference scenario: retail Lebanon, $Hours simulated hours ($(Get-Date -Format s))"
if (Test-Path (Join-Path $bin 'BUILD-INFO.txt')) { Copy-Item (Join-Path $bin 'BUILD-INFO.txt') $out -Force; Say ((Get-Content (Join-Path $bin 'BUILD-INFO.txt') -TotalCount 1)) }

# 1. inputs
$roots = @($Root) + $(if ($RepeatRoot) { @($RepeatRoot) } else { @() })
for ($i = 0; $i -lt $roots.Count; $i++) {
	$line = & $python (Join-Path $here 'tools\input-manifest.py') $roots[$i] -o (Join-Path $out "inputs-$($i + 1).json")
	Say "inputs, run $($i + 1): $line"
}
if ($RepeatRoot) {
	& $python (Join-Path $here 'tools\input-manifest.py') $RepeatRoot --compare (Join-Path $out 'inputs-1.json') | Out-Null
	if ($LASTEXITCODE -ne 0) { Say 'the two roots hold different inputs'; $status = 1 } else { Say 'the two roots hold identical inputs' }
}

# 2. the runs
function Start-Run ($n, $root) {
	$dir = Join-Path $out "run$n"
	New-Item -ItemType Directory -Force $dir | Out-Null
	Remove-Item (Join-Path $dir '*') -Force -ErrorAction SilentlyContinue
	$p = Start-Process -FilePath $world -WorkingDirectory $dir -NoNewWindow -PassThru -RedirectStandardError (Join-Path $dir 'run.log') -RedirectStandardOutput (Join-Path $dir 'run.out') -ArgumentList @(
		((Join-Path $bin 'lua\campaign.lua') -replace '\\', '/'), "root=$($root -replace '\\', '/')", 'scenario=lebanon_retail', "hours=$Hours",
		'record_every=10', 'checkpoint_every=3600', 'observe_every=60',
		"acmi=$((Join-Path $dir 'recording.acmi') -replace '\\', '/')", "observe=$((Join-Path $dir 'observations.jsonl') -replace '\\', '/')",
		"metrics=$((Join-Path $dir 'metrics.json') -replace '\\', '/')")
	# read the handle now: Start-Process -PassThru reports no ExitCode otherwise
	$null = $p.Handle
	return $p
}
$start = Get-Date
if ($RepeatRoot) {
	$runs = @((Start-Run 1 $Root), (Start-Run 2 $RepeatRoot))
	$runs | ForEach-Object { $_.WaitForExit() }
} else {
	$runs = @((Start-Run 1 $Root)); $runs[0].WaitForExit()
	$runs += Start-Run 2 $Root; $runs[1].WaitForExit()
}
Say ("runs took {0:N0} s" -f ((Get-Date) - $start).TotalSeconds)
for ($i = 0; $i -lt 2; $i++) { if ($runs[$i].ExitCode -ne 0) { Say "run $($i + 1) failed (exit $($runs[$i].ExitCode)): see run$($i + 1)\run.log"; exit 1 } }

# 3. each run's recording against its observations and metrics
foreach ($n in 1, 2) {
	$dir = Join-Path $out "run$n"
	& $python (Join-Path $here 'tools\observation-check.py') (Join-Path $dir 'recording.acmi') (Join-Path $dir 'observations.jsonl') (Join-Path $dir 'metrics.json') --report (Join-Path $dir 'check.json') > (Join-Path $dir 'check.txt')
	if ($LASTEXITCODE -ne 0) { Say "run ${n}: observation check FAIL (run$n\check.json)"; $status = 1 } else { Say "run ${n}: observation check PASS" }
}

# 4. repeatability
foreach ($f in 'recording.acmi', 'observations.jsonl', 'metrics.json') {
	$a = (Get-FileHash (Join-Path $out "run1\$f")).Hash; $b = (Get-FileHash (Join-Path $out "run2\$f")).Hash
	$size = (Get-Item (Join-Path $out "run1\$f")).Length
	if ($a -eq $b) { Say ("{0}: byte-identical in both runs ({1:N0} bytes, SHA-256 {2})" -f $f, $size, $a.ToLower()) } else { Say "${f}: the runs DIFFER"; $status = 1 }
}

# 5. observation does not perturb the campaign: the metrics equal the regression baseline
$baseline = Join-Path $here 'regression\windows\lebanon_retail.json'
if ($Hours -eq 3) {
	$verdict = & $python (Join-Path $here 'tools\regress-compare.py') $baseline (Join-Path $out 'run1\metrics.json') --exact
	if ($LASTEXITCODE -ne 0) { $status = 1 }
	$verdict | Select-String 'IDENTICAL|OUTSIDE' | ForEach-Object { Say "metrics against the regression baseline: $($_.Line)" }
}
& $python (Join-Path $here 'tools\campaign-expectations.py') (Join-Path $out 'run1\metrics.json') | Select-String 'campaign expectations' | ForEach-Object { Say $_.Line }
if ($LASTEXITCODE -ne 0) { $status = 1 }
Say $(if ($status -eq 0) { 'REFERENCE: PASS' } else { 'REFERENCE: FAIL' })
exit $status
