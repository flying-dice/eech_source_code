# Native-error controls (docs/m5-dcs-lua-abi.md): runs the build's eech-world.exe and eech_dc.dll
# with a chosen lua.dll on each script in controls/, -Reps times. Each run is a hidden process,
# bounded by -TimeoutSec, in its own directory under -Out, which keeps the run's stdout.txt and
# stderr.txt.
#
# Each control is a self-contained script for a host that has only this file: they repeat a few lines on purpose.
#
#   empty        the host and lua.dll alone
#   plain-error  100 errors raised by lua.dll's own C: no native callback on the error path
#   host-log     100 failing host.log({}) calls: a host native callback, no campaign loaded
#   load-only    require ('eech_dc') and a collection
#   first-error  one failing DLL callback, pcall(dc.boot, 'not a table'), then a collection
#   lua-compat   100 failing DLL callbacks, then a collection
#
# A control passes when it exits 0 and prints its end marker. results.jsonl has one record per
# run, with the exit code, the last marker and the SHA-256 of every input; summary.txt has one
# row per control. Exit 0 only if every run of every control passes.
#
# Usage: tools\native-error-controls\run-controls.ps1 -Bin <dir with eech-world.exe and eech_dc.dll> -LuaDll <lua.dll> -Out <new dir> [-Reps 3] [-TimeoutSec 60]
param(
	[Parameter(Mandatory)] [string]$Bin,
	[Parameter(Mandatory)] [string]$LuaDll,
	[Parameter(Mandatory)] [string]$Out,
	[ValidateRange(1, 100)] [int]$Reps = 3,
	[ValidateRange(1, 3600)] [int]$TimeoutSec = 60
)
$ErrorActionPreference = 'Stop'
$controls = Join-Path $PSScriptRoot 'controls'
$endMarker = [ordered]@{
	'empty' = 'COMPAT_EMPTY_LUA='; 'plain-error' = 'PLAIN_100_ERRORS=PASS'; 'host-log' = 'AFTER_GC'
	'load-only' = 'AFTER_GC'; 'first-error' = 'AFTER_GC'; 'lua-compat' = 'COMPAT_100_ERROR_CALLBACKS=PASS'
}
if (Test-Path $Out) { throw "$Out exists: give a new directory" }
function Get-Sha256($path) { (Get-FileHash -Algorithm SHA256 $path).Hash.ToLower() }

# the runs share one copy of the binaries with the chosen lua.dll
$binCopy = Join-Path $Out 'bin'
New-Item -ItemType Directory $binCopy | Out-Null
Copy-Item (Join-Path $Bin 'eech-world.exe'), (Join-Path $Bin 'eech_dc.dll') $binCopy
Copy-Item $LuaDll (Join-Path $binCopy 'lua.dll')
$inputs = [ordered]@{
	host = Get-Sha256 (Join-Path $binCopy 'eech-world.exe'); module = Get-Sha256 (Join-Path $binCopy 'eech_dc.dll'); lua = Get-Sha256 (Join-Path $binCopy 'lua.dll')
}

$results = Join-Path $Out 'results.jsonl'
foreach ($control in $endMarker.Keys) {
	$script = Join-Path $controls "$control.lua"
	for ($rep = 1; $rep -le $Reps; $rep++) {
		$dir = Join-Path $Out "$control-$rep"
		New-Item -ItemType Directory $dir | Out-Null
		$run = Start-Process -FilePath (Join-Path $binCopy 'eech-world.exe') -ArgumentList "`"$script`"" -WorkingDirectory $dir -WindowStyle Hidden -PassThru `
			-RedirectStandardOutput (Join-Path $dir 'stdout.txt') -RedirectStandardError (Join-Path $dir 'stderr.txt')
		$null = $run.Handle # keep the handle: ExitCode is lost if a fast process exits before it is opened
		$timedOut = -not $run.WaitForExit($TimeoutSec * 1000)
		# the process may exit between the wait and the kill
		if ($timedOut) { try { $run.Kill() } catch [System.InvalidOperationException] {}; $run.WaitForExit() }
		$stdout = @(Get-Content (Join-Path $dir 'stdout.txt'))
		$reachedEnd = [bool]($stdout | Where-Object { $_.StartsWith($endMarker[$control]) })
		[ordered]@{
			control = $control; rep = $rep; exit = ('0x{0:x8}' -f $run.ExitCode); timedOut = $timedOut; reachedEnd = $reachedEnd
			pass = (-not $timedOut -and $run.ExitCode -eq 0 -and $reachedEnd); lastMarker = ($stdout | Select-Object -Last 1)
			script = Get-Sha256 $script; host = $inputs.host; module = $inputs.module; lua = $inputs.lua
		} | ConvertTo-Json -Compress | Add-Content $results
	}
}

$runs = Get-Content $results | ConvertFrom-Json
$runs | Group-Object control | ForEach-Object {
	[pscustomobject]@{ control = $_.Name; pass = "$(@($_.Group | Where-Object pass).Count)/$($_.Count)"; exits = ($_.Group.exit -join ','); last = $_.Group[0].lastMarker }
} | Format-Table -AutoSize | Out-String -Width 200 | Tee-Object (Join-Path $Out 'summary.txt')
"host $($inputs.host)`nmodule $($inputs.module)`nlua $($inputs.lua)" | Tee-Object -Append (Join-Path $Out 'summary.txt')
if (@($runs | Where-Object { -not $_.pass }).Count -gt 0) { exit 1 }
