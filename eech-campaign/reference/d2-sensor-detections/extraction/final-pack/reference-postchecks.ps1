$ErrorActionPreference = 'Stop'
$here = 'R:\d2-evidence\shipping-source-d2-sensing-a2a041fec\eech-campaign'
$out = 'R:\d2-evidence\candidate-pack-sensing-a2a041fec-postcheck-recovery\reference'
$python = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$Hours = 3
$ObserveSupply = $true
$status = 0
function Say ($text) { Write-Host $text; Add-Content -Path (Join-Path $out 'summary.txt') -Value $text }
Set-Content -Path (Join-Path $out 'summary.txt') -Value 'OFFLINE POSTCHECK RECOVERY ONLY: original native runs reused; original alias failures preserved separately'
# 3. each run's recording against its observations and metrics
foreach ($n in 1, 2) {
	$dir = Join-Path $out "run$n"
	& $python (Join-Path $here 'tools\observation-check.py') (Join-Path $dir 'recording.acmi') (Join-Path $dir 'observations.jsonl') (Join-Path $dir 'metrics.json') --report (Join-Path $dir 'check.json') > (Join-Path $dir 'check.txt')
	if ($LASTEXITCODE -ne 0) { Say "run ${n}: observation check FAIL (run$n\check.json)"; $status = 1 } else { Say "run ${n}: observation check PASS" }
}

# 3b. the resupply chains
if ($ObserveSupply) {
	foreach ($n in 1, 2) {
		$dir = Join-Path $out "run$n"
		& $python (Join-Path $here 'tools\supply-chain-check.py') (Join-Path $dir 'observations.jsonl') (Join-Path $dir 'metrics.json') --report (Join-Path $dir 'chains.json') > (Join-Path $dir 'chains.txt') 2> (Join-Path $dir 'chains.log')
		$line = Get-Content (Join-Path $dir 'chains.log') -Tail 1
		if ($LASTEXITCODE -ne 0) { Say "run ${n}: $line"; $status = 1 } else { Say "run ${n}: $line" }
	}
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
