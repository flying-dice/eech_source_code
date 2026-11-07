# TEST-ONLY public baseline observation of the sector tallest-structure word (EEDCS-49, D3; docs/d3-sector-height-public-baseline-plan.md): eech-world -> Lua -> eech_dc on a DLL built
# with tools/build-windows-sector-height-observer.sh. ONE run of lua/sector-height-baseline.lua on a retail root the caller prepared (a fresh verified copy, never a consumed root). The
# observer is read-only; the scenario is the ordinary campaign boot and frame loop. Nothing is tuned: hours, frame_ms and seed are the established ordinary defaults unless the caller
# names others, and a run that did not reach a case reports it UNREACHED (a zero counter), it is not extended to make it reach.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Scenario = 'lebanon_retail',
	[double] $Hours = 1,
	[int] $FrameMs = 100,
	[int] $Seed = 1,
	[int] $TimeoutSeconds = 7200
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$null = [IO.Directory]::CreateDirectory($Out)
$world = Join-Path $Bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows-sector-height-observer.sh" }
$script = Join-Path $here 'crates\eech-world\lua\sector-height-baseline.lua'
$observation = Join-Path $Out "sector-height-$Scenario-seed$Seed.txt"
if (Test-Path -LiteralPath $observation) { throw "$observation exists: a baseline observation is never overwritten" }
$stdout = Join-Path $Out "sector-height-$Scenario-seed$Seed.out"
$stderr = Join-Path $Out "sector-height-$Scenario-seed$Seed.log"
$arguments = @($script, "root=$($Root -replace '\\', '/')", "scenario=$Scenario", "hours=$Hours", "frame_ms=$FrameMs", "seed=$Seed", "out=$($observation -replace '\\', '/')")
$process = Start-Process -FilePath $world -ArgumentList (ConvertTo-NativeArgumentString $arguments) -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$null = $process.Handle
if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
	$process.Kill()
	$process.WaitForExit()
	throw "timed out after $TimeoutSeconds seconds; see $stderr"
}
if ($process.ExitCode -ne 0) { throw "eech-world exited $($process.ExitCode); see $stderr" }
$lines = @(Get-Content -LiteralPath $observation)
if ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict whole=1')) { throw "the observation has no whole verdict; see $observation" }
Write-Host "observation: $observation"
Write-Host $lines[-1]
