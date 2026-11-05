# Public host-movement causal check for ships and infantry through eech-world -> Lua -> eech_dc.
# The census counts what the retained scenario moves, per kind. For each kind the census shows the host moving,
# the control run finds a member the campaign sees move (a man: arrive at a waypoint) and when; the hold run has
# the host decline to move that member for longer than that, then move it again. The campaign must not see it
# move while it is held and must see it move (a man: advance, or the host observe his loss) once it is moved.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[string[]] $Kinds = @('person'),
	[double] $Hours = 1,
	[int] $MarginSeconds = 60,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\surface-movement' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }
$script = Join-Path $here 'crates\eech-world\lua\surface-movement.lua'

function Invoke-Run($name, $extra) {
	$stdout = Join-Path $out "$name.out"
	$stderr = Join-Path $out "$name.log"
	$arguments = @($script, "root=$($Root -replace '\\', '/')", "hours=$Hours") + $extra
	$process = Start-Process -FilePath $world -ArgumentList (ConvertTo-NativeArgumentString $arguments) -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
	$null = $process.Handle
	if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
		$process.Kill()
		$process.WaitForExit()
		throw "$name timed out after $TimeoutSeconds seconds"
	}
	if ($process.ExitCode -ne 0) { throw "$name exited $($process.ExitCode); see $stderr" }
	$lines = @(Get-Content -LiteralPath $stdout)
	$lines | ForEach-Object { Write-Host "  $_" }
	return , $lines
}
function Get-Field($lines, $name) { ($lines | Where-Object { $_ -like "$name`t*" } | Select-Object -First 1) -split "`t" | Select-Object -Last 1 }

Write-Host 'missing: the campaign refuses to start without a host move_vehicle callback'
$missing = Invoke-Run 'missing' @('mode=missing')
if ($missing[-1] -ne "result`tok") { throw "missing did not report ok; see $out" }

Write-Host 'census'
$census = Invoke-Run 'census' @('mode=census')
if ($census[-1] -ne "result`tcensus") { throw "census did not report a result; see $out\census.out" }

foreach ($kind in $Kinds) {
	Write-Host "control $kind"
	$control = Invoke-Run "control-$kind" @('mode=control', "kind=$kind")
	if ($control[-1] -ne "result`tcontrol") { throw "control-$kind did not report a result; see $out\control-$kind.out" }
	$subject = Get-Field $control 'subject'
	$hold = [math]::Ceiling([double](Get-Field $control 'selected_at')) + $MarginSeconds

	Write-Host "hold $kind $subject ($hold s)"
	$held = Invoke-Run "hold-$kind" @('mode=hold', "kind=$kind", "select=$subject", "hold_seconds=$hold")
	if ($held[-1] -ne "result`tok") { throw "hold-$kind did not report ok; see $out\hold-$kind.out" }
}
Write-Host 'PASS'
Write-Host "outputs: $out"
