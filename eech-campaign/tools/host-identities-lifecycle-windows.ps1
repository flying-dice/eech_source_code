# The host-identity lifecycle and failure behaviour on Windows, through the
# public path: eech-world.exe runs lua/host-identities-lifecycle.lua, requires eech_dc and boots
# it. One process per case (EECH boots at most once per process). Each case's
# Original steps are checked against the behaviour characterised at the M1
# baseline; host-identity cases check the new M5 boundary contract. A case
# marked as a known defect is expected to fail that way; if it starts behaving
# differently, the check says so, so a fix or regression is reviewed.
#
# Usage (PowerShell): tools\host-identities-lifecycle-windows.ps1 -Root <prepared retail Lebanon root> [-Bin <dir>] [-Out <dir>]
#   binaries: tools/build-windows.sh [dir] (default target\windows)
#   the root: tools/retail-cvh.sh; boots take a few seconds, no campaign is run
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[string] $Bin,
	# where each case's output goes; default target\lifecycle-windows
	[string] $Out,
	[int] $TimeoutSeconds = 300
)
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $PSScriptRoot
$bin = if ($Bin) { $Bin } else { Join-Path $here 'target\windows' }
$out = if ($Out) { $Out } else { Join-Path $here 'target\host-identities-lifecycle-windows' }
New-Item -ItemType Directory -Force $out | Out-Null
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found: run tools/build-windows.sh" }

# per case: the expected exit (0, or the Windows exception code of a crash) and
# each step as "<step> <ok|error> <text the message contains>"
$load = @('load ok', 'prepare ok')
$notAgain = "not valid in the engine's state"
$cases = [ordered]@{
	boot_twice = @{ exit = 0; steps = $load + @(
		'boot ok', 'frames ok', 'frame_reentry_guards ok', "boot_again error $notAgain", 'roster_not_reset ok',
		'frames_after ok', 'roster_consistent ok', 'clock ok') }
	bad_arguments = @{ exit = 0; steps = $load + @(
		'boot_unknown_gunship error unknown gunship "tiger"',
		'boot_no_install_root error error converting Lua nil to String',
		'boot_no_map error error converting Lua nil to String',
		'boot_no_campaign error error converting Lua nil to String',
		'boot_not_a_table error error converting Lua string to table',
		'boot ok',
		'frame_negative error out of range',
		'frame_not_a_number error error converting Lua string to u32',
		'frame_zero ok', 'frames_after ok') }
	missing_root = @{ exit = 0; steps = $load + @(
		'boot_missing_root error bad argument: rejected by the engine', "boot_after error $notAgain") }
	missing_campaign = @{ exit = 0; steps = $load + @(
		'boot_missing_campaign error EECH fatal error: Error opening file for reading', "boot_after error $notAgain") }
	no_reuse = @{ exit = 0; steps = $load + @(
		'boot ok', 'frames ok', "boot_after_collect error $notAgain", 'require_again ok', "boot_after_require error $notAgain") }
	missing_host = @{ exit = 0; steps = $load + @(
		'boot_without_bind error missing host bind_mobile callback', 'boot_after ok', 'members ok') }
	missing_move = @{ exit = 0; steps = $load + @(
		'boot_without_move error missing host move_vehicle callback', 'boot_after ok', 'members ok') }
	move_unknown_id = @{ exit = 0; steps = $load + @(
		'boot error unknown host id stranger:', "boot_after error $notAgain") }
	host_identity = @{ exit = 0; steps = $load + @(
		'boot ok', 'reentry_guards ok', 'binding_count ok', 'objects_compatible ok') }
	bind_nil = @{ exit = 0; steps = $load + @(
		'boot_bind_nil error bind_mobile must return a string, got nil',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	bind_invalid = @{ exit = 0; steps = $load + @(
		'boot_bind_invalid error bind_mobile identity is 129 bytes; maximum is 128',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	bind_nul = @{ exit = 0; steps = $load + @(
		'boot_bind_nul error bind_mobile identity contains NUL',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	bind_non_utf8 = @{ exit = 0; steps = $load + @(
		'boot_bind_non_utf8 error bind_mobile returned a non-UTF-8 identity',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	bind_duplicate = @{ exit = 0; steps = $load + @(
		'boot_bind_duplicate error host identity duplicate was reused for a new physical incarnation',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	bind_error = @{ exit = 0; steps = $load + @(
		'boot_bind_error error host refused binding',
		'prepare_after error the engine is poisoned', "boot_after error $notAgain") }
	# known defect: a missing map directory crashes the host process during boot
	# (access violation in msvcrt.dll) instead of returning an error to Lua
	missing_map = @{ exit = -1073741819; known_defect = $true; steps = $load }
}

$status = 0
foreach ($name in $cases.Keys) {
	$case = $cases[$name]
	$stdout = Join-Path $out "$name.out"
	$stderr = Join-Path $out "$name.log"
	$p = Start-Process -FilePath $world -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr -ArgumentList @(
		(Join-Path $bin 'lua\host-identities-lifecycle.lua'), "root=$($Root -replace '\\', '/')", "case=$name")
	# read the handle now: Start-Process -PassThru reports no ExitCode otherwise
	$null = $p.Handle
	if (-not $p.WaitForExit($TimeoutSeconds * 1000)) { $p.Kill(); $p.WaitForExit() }
	$lines = @(Get-Content $stdout | Where-Object { $_ -ne '' })
	$problems = @()
	if ($p.ExitCode -ne $case.exit) { $problems += ('exit {0} (0x{0:X8}), expected {1} (0x{1:X8})' -f $p.ExitCode, $case.exit) }
	$expected = @($case.steps)
	if ($case.exit -eq 0) { $expected += 'end' }
	for ($i = 0; $i -lt [Math]::Max($expected.Count, $lines.Count); $i++) {
		$want = if ($i -lt $expected.Count) { $expected[$i] } else { $null }
		$got = if ($i -lt $lines.Count) { $lines[$i] } else { $null }
		if ($null -eq $want) { $problems += "unexpected step: $got"; continue }
		if ($null -eq $got) { $problems += "missing step: $want"; continue }
		$step, $result, $text = $want -split ' ', 3
		$fields = $got -split "`t", 3
		$ok = $fields[0] -eq $step -and ($want -eq 'end' -or $fields[1] -eq $result)
		if ($ok -and $text) { $ok = $fields.Count -ge 3 -and $fields[2].Contains($text) }
		if (-not $ok) { $problems += "step $($i + 1): got [$($got -replace "`t", ' | ')], expected [$want]" }
	}
	if ($problems.Count -eq 0) {
		if ($case.known_defect) { Write-Host "${name}: KNOWN DEFECT (unchanged)" } else { Write-Host "${name}: PASS" }
	} else {
		Write-Host "${name}: FAIL$(if ($case.known_defect) { ' (known defect behaves differently: review its expectation)' })"
		$problems | ForEach-Object { Write-Host "  $_" }
		$status = 1
	}
}
Write-Host "outputs: $out"
exit $status
