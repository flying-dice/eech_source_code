# TEST-ONLY (EEDCS-45): runs lua/weapon-case.lua (the controlled physical-launch witness), one native process, with a binary directory built by
# tools/build-windows-weapon-fixture.sh (the shipping binaries have no wpn_trace and the script stops at the first call).
#
#   -Bin      the fixture build output directory (eech-world.exe, eech_dc.dll, lua.dll, lua/)
#   -Root     a private copy of the retail installation (the engine writes into its root: one root per run)
#   -Out      the directory for trace.jsonl, run.log and run.out
#   -Warmup   simulated seconds the war runs before the controlled calls (default 60)
#   -Settle   simulated seconds allowed for the case weapons to reach their terminal fact (the CAP; default 700)
#
# The native exit is the script's exit (nonzero when a check failed or the engine stopped). The paths must hold no spaces (use a subst drive): the native argv is not quoted here.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Out,
	[int] $Warmup = 60,
	[int] $Settle = 700,
	[int] $FrameMs = 100
)
$ErrorActionPreference = 'Stop'
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found" }
New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path $Out).Path
foreach ($p in @($bin, $Root, $out)) { if ($p -match '\s') { throw "path with a space: $p" } }
$script = (Join-Path $bin 'lua\weapon-case.lua') -replace '\\', '/'
$arguments = @($script, "root=$($Root -replace '\\', '/')", "warmup=$Warmup", "settle=$Settle", "frame_ms=$FrameMs", "out=$((Join-Path $out 'trace.jsonl') -replace '\\', '/')") -join ' '
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
	-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
"exit $($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds
exit $proc.ExitCode
