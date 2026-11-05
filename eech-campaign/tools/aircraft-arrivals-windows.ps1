# Public host-arrival causal checks for non-SUPPLY aircraft through eech-world -> Lua -> eech_dc.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\aircraft-arrivals' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }
# the script under test is the repository's, so a baseline build can be exercised with it
$script = Join-Path $here 'crates\eech-world\lua\aircraft-arrivals.lua'

foreach ($mode in @('reference', 'withhold')) {
	$stdout = Join-Path $out "$mode.out"
	$stderr = Join-Path $out "$mode.log"
	$arguments = @($script, "root=$($Root -replace '\\', '/')", "mode=$mode", 'hours=1', 'delay_seconds=10')
	$process = Start-Process -FilePath $world -ArgumentList (ConvertTo-NativeArgumentString $arguments) -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
	$null = $process.Handle
	if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
		$process.Kill()
		$process.WaitForExit()
		throw "$mode timed out after $TimeoutSeconds seconds"
	}
	if ($process.ExitCode -ne 0) { throw "$mode exited $($process.ExitCode); see $stderr" }
	$lines = @(Get-Content -LiteralPath $stdout)
	if ($lines[-1] -ne "result`tok") { throw "$mode did not report success; see $stdout" }
	Write-Host "${mode}: PASS"
	$lines | ForEach-Object { Write-Host "  $_" }
}
Write-Host "outputs: $out"
