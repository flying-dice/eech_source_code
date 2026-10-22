# Public host-lifecycle causal check for the retained natural helicopter loss through eech-world -> Lua -> eech_dc.
# The control run lets the reference host answer everything and resolves the host id of the Mi-17 (legacy entity
# 87568, Wolfpack) from the campaign's binding, when it starts dying, and which lifecycle phases it passes through.
# The hold run has the host decline to spin that helicopter's rotor down for -HoldSeconds from its first
# rotor_down step, then answer again. The campaign must not see the rotor change while it is held, and must see it
# change once the host answers, the first change coming from the host's rotor_down answer. The fall, contact,
# settle and crash phases are reported as coverage; no terminal state is claimed.
# The default hold is 1 s, a window derived from the control run: the subject starts dying at about 1585.0 s, is
# buried by the settle at about 1587.7 to 1588.3 s, and that burial clears the rotor independently of rotor_down,
# ending the rotor_down calls about 3.3 s after the death. A longer hold (the earlier 20 s default) therefore
# cannot isolate the host's rotor_down answer; it was tried, stopped, and its logs are retained, not erased.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	[string] $Out,
	[double] $Hours = 0.65,
	[double] $HoldSeconds = 1,
	[int] $TimeoutSeconds = 1800
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\helicopter-lifecycle' }
$null = [IO.Directory]::CreateDirectory($out)
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows.sh" }
$script = Join-Path $here 'crates\eech-world\lua\helicopter-lifecycle.lua'

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

Write-Host 'control lifecycle'
$control = Invoke-Run 'control-lifecycle' @('mode=control')
if ($control[-1] -ne "result`tcontrol") { throw "control did not report a result; see $out\control-lifecycle.out" }
$subject = Get-Field $control 'subject'

Write-Host "hold rotor of $subject ($HoldSeconds s)"
$held = Invoke-Run 'hold-lifecycle' @('mode=hold', "select=$subject", "hold_seconds=$HoldSeconds")
if ($held[-1] -ne "result`tok") { throw "hold did not report ok; see $out\hold-lifecycle.out" }
Write-Host 'PASS'
Write-Host "outputs: $out"
