# Public host-arrival causal checks for the remaining physical arrival and proximity queries
# through eech-world -> Lua -> eech_dc. A selection the scenario never produces is reported as
# UNAVAILABLE and fails the run unless it is listed in -Allow; it is never counted as a pass.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[string[]] $Selections = @('ground', 'altitude', 'weapon', 'action'),
	[string[]] $Allow = @(),
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\remaining-arrivals' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }
$script = Join-Path $here 'crates\eech-world\lua\remaining-arrivals.lua'

$runs = @(@{ name = 'census'; arguments = @('mode=census') })
foreach ($selection in $Selections) { $runs += @{ name = "withhold-$selection"; arguments = @('mode=withhold', "select=$selection", 'delay_seconds=10') } }

$failed = @()
foreach ($run in $runs) {
	$stdout = Join-Path $out "$($run.name).out"
	$stderr = Join-Path $out "$($run.name).log"
	$arguments = @($script, "root=$($Root -replace '\\', '/')", 'hours=1') + $run.arguments
	$process = Start-Process -FilePath $world -ArgumentList (ConvertTo-NativeArgumentString $arguments) -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
	$null = $process.Handle
	if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
		$process.Kill()
		$process.WaitForExit()
		throw "$($run.name) timed out after $TimeoutSeconds seconds"
	}
	if ($process.ExitCode -ne 0) { throw "$($run.name) exited $($process.ExitCode); see $stderr" }
	$lines = @(Get-Content -LiteralPath $stdout)
	$verdict = $lines[-1]
	if ($verdict -eq "result`tok") {
		Write-Host "$($run.name): PASS"
	}
	elseif ($verdict -eq "result`tunavailable") {
		$selection = $run.name -replace '^withhold-', ''
		if ($Allow -contains $selection) { Write-Host "$($run.name): UNAVAILABLE (allowed)" }
		else { Write-Host "$($run.name): UNAVAILABLE"; $failed += $run.name }
	}
	else { throw "$($run.name) did not report a result; see $stdout" }
	$lines | ForEach-Object { Write-Host "  $_" }
}
Write-Host "outputs: $out"
if ($failed.Count -gt 0) { throw "selections not exercised: $($failed -join ', ')" }
