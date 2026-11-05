# Public host-identity and durable-roster audit through eech-world and eech_dc.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[ValidateSet('georgia_retail', 'lebanon_retail')] [string] $Scenario = 'lebanon_retail',
	[double] $Hours = 3,
	[int] $TargetEntity = -1,
	[switch] $RequireReuse,
	[switch] $RequireTransfer,
	[switch] $RequireTargetReleased,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\host-identities' }
$null = [IO.Directory]::CreateDirectory($out)
$stdout = Join-Path $out "$Scenario.out"
$stderr = Join-Path $out "$Scenario.log"
$arguments = @(
	(Join-Path $bin 'lua\host-identities.lua'),
	"root=$($Root -replace '\\', '/')", "scenario=$Scenario", "hours=$Hours", "target_entity=$TargetEntity",
	"require_reuse=$(if ($RequireReuse) { 1 } else { 0 })",
	"require_transfer=$(if ($RequireTransfer) { 1 } else { 0 })",
	"require_target_released=$(if ($RequireTargetReleased) { 1 } else { 0 })"
)
$process = Start-Process -FilePath (Join-Path $bin 'eech-world.exe') -ArgumentList $arguments -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$null = $process.Handle
if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
	$process.Kill()
	$process.WaitForExit()
	throw "host identity audit timed out after $TimeoutSeconds seconds"
}
if ($process.ExitCode -ne 0) { throw "host identity audit exited $($process.ExitCode); see $stderr" }
$lines = @(Get-Content -LiteralPath $stdout)
if ($lines[-1] -ne "result`tok") { throw "host identity audit did not report success; see $stdout" }
$lines | ForEach-Object { Write-Host $_ }
Write-Host "outputs: $out"
