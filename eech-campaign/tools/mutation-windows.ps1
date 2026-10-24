# Does the accepted public-path regression detect a real behavioural
# regression? Builds a control and a test-only mutant with one source patch
# removed (tools/build-mutant-windows.sh, from the same clean commit, each in
# a temporary worktree), shows the two builds differ only in that patch, runs
# the accepted Windows regression (tools/regress-windows.ps1 -Exact: retail
# Georgia and Lebanon, 3 simulated hours, eech-world.exe -> campaign.lua ->
# require ("eech_dc")) on both with identical inputs, and reports:
#
#   control   must PASS: every campaign expectation, IDENTICAL to the baselines
#   mutant    DETECTED if a run fails its exact baseline comparison or a
#             campaign expectation; the failing checks and the metrics that
#             differ from the control are listed (difference.txt)
#   -RepeatMutant  runs the mutant twice, on two copies of the inputs, and
#             requires byte-identical metrics (the difference is not noise)
#
# The mutant exists only under target/mutants/; the normal build is untouched.
# Exit 0 when the control passes and the mutant is detected.
#
# Usage (PowerShell):
#   tools\mutation-windows.ps1 -Georgia <root> -Lebanon <root> [-Patch S3-pick-up-the-tasks-cargo] [-Out target\mutants\<dir>] [-RepeatMutant]
param(
	[Parameter(Mandatory = $true)] [string] $Georgia,
	[Parameter(Mandatory = $true)] [string] $Lebanon,
	[string] $Patch = 'S3-pick-up-the-tasks-cargo',
	[string] $Out,
	[switch] $RepeatMutant
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$short = ($Patch -split '-')[0]
$rel = if ($Out) { $Out -replace '\\', '/' } else { "target/mutants/$($short.ToLower())" }
if ($rel -notlike 'target/mutants/*') { throw 'the output must be under target/mutants/' }
$out = Join-Path $here ($rel -replace '/', '\')
New-Item -ItemType Directory -Force $out | Out-Null
$python = (Get-Command python3, python -ErrorAction SilentlyContinue | Select-Object -First 1).Source
# Git for Windows' sh, for the build scripts: not PATH's bash, which can be WSL's
$sh = @("$env:ProgramFiles\Git\bin\sh.exe", "$env:ProgramFiles\Git\usr\bin\sh.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $sh) { $sh = (Get-Command sh -ErrorAction SilentlyContinue | Where-Object { $_.Source -notlike '*\System32\*' -and $_.Source -notlike '*\WindowsApps\*' } | Select-Object -First 1).Source }
if (-not $sh) { throw 'Git for Windows sh is needed for tools/build-mutant-windows.sh' }
$summary = Join-Path $out 'summary.txt'
Set-Content $summary "Mutation check: $Patch ($(Get-Date -Format s))"
function Say ($t) { Write-Host $t; Add-Content $summary $t }
$status = 0

# 1. the two builds, from the same clean commit
Push-Location $here
try {
	foreach ($b in @(@('control', "$rel/control-bin"), @($Patch, "$rel/mutant-bin"))) {
		& $sh tools/build-mutant-windows.sh $b[0] $b[1] | Out-Null
		if ($LASTEXITCODE -ne 0) { throw "build of $($b[0]) failed" }
	}
} finally { Pop-Location }
$cb, $mb = (Join-Path $out 'control-bin'), (Join-Path $out 'mutant-bin')

# 2. attribution: the builds differ only in the patch
$ci, $mi = (Get-Content (Join-Path $cb 'BUILD-INFO.txt')), (Get-Content (Join-Path $mb 'BUILD-INFO.txt'))
$same = { param($prefix) (($ci | Where-Object { $_ -like "$prefix*" }) -join '|') -eq (($mi | Where-Object { $_ -like "$prefix*" }) -join '|') }
foreach ($p in 'commit:', 'image:', 'rustc:', 'cargo:', 'mingw:', 'lua.dll:', 'target:') {
	if (-not (& $same $p)) { Say "attribution: '$p' lines differ between control and mutant"; $status = 1 }
}
# lua.dll and the Lua scripts must be identical (eech_dc.dll and eech-world.exe differ in their link timestamps at least)
$scripts = { param($info) ($info | Where-Object { $_ -match '^\s+\S+\s+\*?(lua\.dll|lua/.+)$' }) -join '|' }
if ((& $scripts $ci) -ne (& $scripts $mi) -or -not (& $scripts $ci)) { Say 'attribution: lua.dll or the Lua scripts differ'; $status = 1 }
Say ("builds: " + ($ci[0]) + " / " + ($mi[0]))
Say ("the same commit, image, toolchain, lua.dll and Lua scripts: " + $(if ($status -eq 0) { 'yes' } else { 'NO' }))
$diff = Get-Content (Join-Path $mb 'mutation.diff')
$files = $diff | Where-Object { $_ -like 'diff --git*' }
$added = $diff | Where-Object { $_ -match '^\+[^+]' }
$removedId = $diff | Where-Object { $_ -like "-*id: `"$Patch`"*" }
if ($files.Count -ne 1 -or $files -notlike '*build/patches.rs*' -or $added -or -not $removedId) { Say 'attribution: the mutation is not exactly one patch entry removed'; $status = 1 }
else { Say "source difference: patches.rs, the '$Patch' entry removed ($(($diff | Where-Object { $_ -match '^-[^-]' }).Count) lines), nothing added" }
$cm, $mm = (Get-Content (Join-Path $cb 'patch-markers.txt')), (Get-Content (Join-Path $mb 'patch-markers.txt'))
$onlyControl = @($cm | Where-Object { $mm -notcontains $_ }); $onlyMutant = @($mm | Where-Object { $cm -notcontains $_ })
if ($onlyControl.Count -ne 1 -or $onlyControl[0] -ne "EECH headless ($short)" -or $onlyMutant.Count -ne 0) { Say "attribution: compiled patch markers differ by $($onlyControl -join ',') / $($onlyMutant -join ',')"; $status = 1 }
else { Say "compiled patch markers: the control's $($cm.Count), the mutant's $($mm.Count): only '$($onlyControl[0])' is missing from the mutant" }
if ($status -ne 0) { Say 'MUTATION CHECK: the builds are not a clean control/mutant pair'; exit 1 }

# 3. identical inputs: the control uses the roots given, each mutant run a copy
$roots = @{ control = @($Georgia, $Lebanon) }
$mutantRuns = @('mutant') + $(if ($RepeatMutant) { @('mutant-repeat') } else { @() })
foreach ($m in $mutantRuns) {
	$g, $l = (Join-Path $out "roots-$m\georgia"), (Join-Path $out "roots-$m\lebanon")
	foreach ($pair in @(@($Georgia, $g), @($Lebanon, $l))) {
		robocopy $pair[0] $pair[1] /MIR /NFL /NDL /NJH /NJS /NP | Out-Null
		if ($LASTEXITCODE -ge 8) { throw "copying $($pair[0]) failed" }
		& $python (Join-Path $here 'tools\input-manifest.py') $pair[1] -o (Join-Path $out "inputs-$m-$(Split-Path $pair[1] -Leaf).json") | Out-Null
		& $python (Join-Path $here 'tools\input-manifest.py') $pair[0] --compare (Join-Path $out "inputs-$m-$(Split-Path $pair[1] -Leaf).json") | Out-Null
		if ($LASTEXITCODE -ne 0) { Say "inputs: the copy of $($pair[0]) differs"; exit 1 }
	}
	$roots[$m] = @($g, $l)
}
Say "inputs: each mutant run has a copy of the roots, identical in every input file (tools/input-manifest.py)"

# 4. the accepted regression, on the control and the mutant run(s) side by side
$procs = @{}
foreach ($run in @('control') + $mutantRuns) {
	$binDir = if ($run -eq 'control') { $cb } else { $mb }
	$dir = Join-Path $out "regress-$run"
	New-Item -ItemType Directory -Force $dir | Out-Null
	$procs[$run] = Start-Process -FilePath (Get-Process -Id $PID).Path -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $out "regress-$run.txt") -RedirectStandardError (Join-Path $out "regress-$run.err") -ArgumentList @(
		'-NoProfile', '-File', (Join-Path $here 'tools\regress-windows.ps1'), '-Georgia', $roots[$(if ($run -eq 'control') { 'control' } else { $run })][0],
		'-Lebanon', $roots[$(if ($run -eq 'control') { 'control' } else { $run })][1], '-Bin', $binDir, '-Out', $dir, '-Exact')
	$null = $procs[$run].Handle
}
$procs.Values | ForEach-Object { $_.WaitForExit() }
$result = @{}
foreach ($run in $procs.Keys) {
	$text = Get-Content (Join-Path $out "regress-$run.txt")
	foreach ($s in 'georgia_retail', 'lebanon_retail') {
		$exp = ($text | Where-Object { $_ -like "${s}: * campaign expectations hold" }) -join ''
		$verdict = ($text | Where-Object { $_ -like "${s}: PASS" -or $_ -like "${s}: FAIL*" }) -join ''
		$result["$run $s"] = $verdict
		Say ("{0,-14} {1,-15} {2}; {3}" -f $run, $s, ($exp -replace "^${s}: ", ''), $verdict)
	}
}
foreach ($line in (Get-Content (Join-Path $out 'regress-mutant.txt') | Where-Object { $_ -like '*EXPECTATION FAILED*' })) { Say "mutant: $($line.Trim())" }

# 5. what changed: the mutant's metrics against the control's
& $python (Join-Path $here 'tools\metrics-difference.py') (Join-Path $out 'regress-control') (Join-Path $out 'regress-mutant') > (Join-Path $out 'difference.txt')
Get-Content (Join-Path $out 'difference.txt') | Select-Object -First 40 | ForEach-Object { Say $_ }
if ($RepeatMutant) {
	foreach ($s in 'georgia_retail', 'lebanon_retail') {
		$a = (Get-FileHash (Join-Path $out "regress-mutant\$s.json")).Hash; $b = (Get-FileHash (Join-Path $out "regress-mutant-repeat\$s.json")).Hash
		if ($a -eq $b) { Say "repeat: the mutant's $s metrics are byte-identical in both mutant runs" } else { Say "repeat: the mutant's $s metrics DIFFER between runs"; $status = 1 }
	}
}

# the verdict
$controlOk = $result['control georgia_retail'] -like '*PASS' -and $result['control lebanon_retail'] -like '*PASS'
$detected = @($result.Keys | Where-Object { $_ -like 'mutant *' -and $result[$_] -like '*FAIL*' }).Count -gt 0
if (-not $controlOk) { Say 'MUTATION CHECK: the control does not pass'; exit 1 }
Say $(if ($detected -and $status -eq 0) { "MUTATION CHECK: DETECTED ($Patch)" } else { "MUTATION CHECK: NOT DETECTED ($Patch)" })
exit $(if ($detected -and $status -eq 0) { 0 } else { 1 })
