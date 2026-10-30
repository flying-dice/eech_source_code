$ErrorActionPreference='Stop'
$out='R:\d2-evidence\candidate-pack-sensing-a2a041fec-postcheck-recovery\regress'
$baselines='R:\d2-evidence\shipping-source-d2-sensing-a2a041fec\eech-campaign\regression\windows'
$here='R:\d2-evidence\shipping-source-d2-sensing-a2a041fec\eech-campaign'
$python='C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$Exact=$true
$status = 0
foreach ($name in 'georgia_retail', 'lebanon_retail') {
	$current = Join-Path $out "$name.json"
	$base = Join-Path $baselines "$name.json"
	Write-Host ''
	# the campaign played out as EECH's should: every mechanic present (no reference needed)
	& $python (Join-Path $here 'tools\campaign-expectations.py') $current
	if ($LASTEXITCODE -ne 0) { Write-Host "${name}: FAIL (campaign expectations)"; $status = 1 }
	if (-not (Test-Path $base)) {
		# never write a missing baseline: that would accept whatever this run did
		Write-Host "${name}: FAIL (no accepted baseline regression/windows/$name.json; see regression/baselines/lineages.json)"
		$status = 1
	} else {
		$arguments = @((Join-Path $here 'tools\regress-compare.py'), $base, $current)
		if ($Exact) { $arguments += '--exact' }
		& $python @arguments
		if ($LASTEXITCODE -eq 0) { Write-Host "${name}: PASS" } else { Write-Host "${name}: FAIL"; $status = 1 }
	}
}
exit $status
