# Public missing/error/malformed/re-entry checks for the SUPPLY arrival callback.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\supply-arrivals-lifecycle' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }

foreach ($case in @('missing', 'malformed', 'error', 'reentry')) {
	$stdout = Join-Path $out "$case.out"
	$stderr = Join-Path $out "$case.log"
	$arguments = @((Join-Path $bin 'lua\supply-arrivals-lifecycle.lua'), "root=$($Root -replace '\\', '/')", "case=$case")
	$process = Start-Process -FilePath $world -ArgumentList $arguments -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
	$null = $process.Handle
	if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
		$process.Kill()
		$process.WaitForExit()
		throw "$case timed out after $TimeoutSeconds seconds"
	}
	if ($process.ExitCode -ne 0) { throw "$case exited $($process.ExitCode); see $stderr" }
	$lines = @(Get-Content -LiteralPath $stdout)
	if ($lines[-1] -ne "result`tok") { throw "$case did not report success; see $stdout" }
	Write-Host "${case}: PASS"
}
Write-Host "outputs: $out"
