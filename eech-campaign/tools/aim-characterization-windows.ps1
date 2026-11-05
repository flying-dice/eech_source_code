# TEST-ONLY (EEDCS-44): runs lua/aim-characterization.lua, one native process, with a binary directory built by
# tools/build-windows-aim-fixture.sh (the shipping binaries have no aim_trace and the script stops at the first call).
#
#   -Bin          the fixture build output directory (eech-world.exe, eech_dc.dll, lua.dll, lua/)
#   -Root         a private copy of the retail installation (the engine writes into its root: one root per run)
#   -Out          the directory for trace.jsonl, run.log and run.out
#   -Scenario     lebanon_retail (default), georgia_retail or georgia
#   -Seconds      simulated seconds (default 600)
#   -Checkpoint   simulated seconds between checkpoint reports (default 60; 0 disables)
#
# The paths must hold no spaces (use a subst drive): the native argv is not quoted here.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Scenario = 'lebanon_retail',
	[int] $Seconds = 600,
	[int] $FrameMs = 100,
	[int] $Checkpoint = 60
)
$ErrorActionPreference = 'Stop'
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found" }
New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path $Out).Path
foreach ($p in @($bin, $Root, $out)) { if ($p -match '\s') { throw "path with a space: $p" } }
$script = (Join-Path $bin 'lua\aim-characterization.lua') -replace '\\', '/'
$arguments = @($script, "root=$($Root -replace '\\', '/')", "scenario=$Scenario", "seconds=$Seconds", "frame_ms=$FrameMs", "checkpoint_every=$Checkpoint",
	"out=$((Join-Path $out 'trace.jsonl') -replace '\\', '/')") -join ' '
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
	-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
"exit $($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds
exit $proc.ExitCode
