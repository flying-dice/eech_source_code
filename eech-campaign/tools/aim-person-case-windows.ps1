# TEST-ONLY (EEDCS-44): runs lua/aim-person-case.lua, one native process, with a binary directory built by tools/build-windows-aim-fixture.sh
# (the shipping binaries have no aim_person_case and the script stops at the first call).
#
#   -Bin       the fixture build output directory (eech-world.exe, eech_dc.dll, lua.dll, lua/)
#   -Root      a private copy of the retail installation (the engine writes into its root: one root per run)
#   -Out       the directory for trace.jsonl, run.log and run.out
#   -Scenario  lebanon_retail (default) or georgia_retail
#   -Case      target (the person is given a target through the campaign's own link) or control (the same selection, no link)
#
# The paths must hold no spaces (use a subst drive): the native argv is not quoted here.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Scenario = 'lebanon_retail',
	[string] $Case = 'target',
	[int] $Warmup = 20,
	[int] $Frames = 20,
	[int] $FrameMs = 100
)
$ErrorActionPreference = 'Stop'
$bin = (Resolve-Path $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
if (-not (Test-Path $world)) { throw "$world not found" }
New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path $Out).Path
foreach ($p in @($bin, $Root, $out)) { if ($p -match '\s') { throw "path with a space: $p" } }
$script = (Join-Path $bin 'lua\aim-person-case.lua') -replace '\\', '/'
$arguments = @($script, "root=$($Root -replace '\\', '/')", "scenario=$Scenario", "case=$Case", "warmup=$Warmup", "frames=$Frames", "frame_ms=$FrameMs",
	"out=$((Join-Path $out 'trace.jsonl') -replace '\\', '/')") -join ' '
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
	-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList $arguments
$null = $proc.Handle
$proc.WaitForExit()
"exit $($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds
exit $proc.ExitCode
