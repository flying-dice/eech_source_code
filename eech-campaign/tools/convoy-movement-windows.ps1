# Public host-movement causal check for ground convoys through eech-world -> Lua -> eech_dc.
# The control run finds a convoy the campaign sees arrive and how long that takes; the hold run has
# the host decline to move that convoy for longer than that, then move it again. The campaign must not
# advance while it is held and must advance (or the host observe its loss) once it moves.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[int] $MarginSeconds = 60,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\convoy-movement' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }
$script = Join-Path $here 'crates\eech-world\lua\convoy-movement.lua'

function Invoke-Run($name, $extra) {
	$stdout = Join-Path $out "$name.out"
	$stderr = Join-Path $out "$name.log"
	$arguments = @($script, "root=$($Root -replace '\\', '/')", 'hours=1') + $extra
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

Write-Host 'missing: the campaign refuses to start without a host move_vehicle callback'
$missing = Invoke-Run 'missing' @('mode=missing')
if ($missing[-1] -ne "result`tok") { throw "missing did not report ok; see $out" }

Write-Host 'control'
$control = Invoke-Run 'control' @('mode=control')
if ($control[-1] -ne "result`tcontrol") { throw "control did not report a result; see $out\control.out" }
function Get-Field($lines, $name) { ($lines | Where-Object { $_ -like "$name`t*" } | Select-Object -First 1) -split "`t" | Select-Object -Last 1 }
$subject = Get-Field $control 'subject'
$span = [double](Get-Field $control 'arrived_at') - [double](Get-Field $control 'first_move_at')
$hold = [math]::Ceiling($span) + $MarginSeconds

Write-Host "hold $subject ($hold s)"
$held = Invoke-Run 'hold' @('mode=hold', "select=$subject", "hold_seconds=$hold")
if ($held[-1] -ne "result`tok") { throw "hold did not report ok; see $out\hold.out" }
Write-Host 'PASS'
Write-Host "outputs: $out"
