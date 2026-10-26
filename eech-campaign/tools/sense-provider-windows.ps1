# EEDCS-43: runs lua/sense-provider.lua (the public-path checks of the host sensing provider), ONE native process at a time, one mode per process
# (the engine is one per process), each on a fresh private copy of an installation root.
#
#   -Bin     a build output directory (eech-world.exe, eech_dc.dll, lua.dll, lua/); the shipping build or a fixture build
#   -Source  the installation root to copy from (never run on: the engine writes into its root)
#   -Work    the directory that receives one root copy and one output directory per mode
#   -Modes   reference, blocked, far, missing, bad_begin, bad_nil, bad_state, bad_reason, bad_accept_range, bad_draw_for_range, bad_error
#   -Seconds simulated seconds per mode (default 300)
#
# The paths must hold no spaces (use a subst drive): the native argv is not quoted here. Prints one line per mode with the native exit code.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Source,
	[Parameter(Mandatory = $true)] [string] $Work,
	[string[]] $Modes = @('reference', 'blocked', 'far'),
	[int] $Seconds = 300,
	[string] $Scenario = 'lebanon_retail'
)
$ErrorActionPreference = 'Stop'
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found" }
New-Item -ItemType Directory -Force $Work | Out-Null
$work = (Resolve-Path $Work).Path
foreach ($p in @($bin, $Source, $work)) { if ($p -match '\s') { throw "path with a space: $p" } }
$script = (Join-Path $bin 'lua\sense-provider.lua') -replace '\\', '/'
foreach ($mode in $Modes) {
	$root = Join-Path $work "root-$mode"
	$out = Join-Path $work "out-$mode"
	if (Test-Path $root) { throw "$root exists: use a fresh work directory" }
	robocopy $Source $root /E /NFL /NDL /NJH /NJS /NP | Out-Null
	New-Item -ItemType Directory -Force $out | Out-Null
	$arguments = @($script, "root=$($root -replace '\\', '/')", "scenario=$Scenario", "seconds=$Seconds", "mode=$mode", "out=$((Join-Path $out 'summary.jsonl') -replace '\\', '/')") -join ' '
	$start = Get-Date
	$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
		-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList $arguments
	$null = $proc.Handle
	$proc.WaitForExit()
	"{0} exit {1} after {2:N0} s" -f $mode, $proc.ExitCode, ((Get-Date) - $start).TotalSeconds
}
