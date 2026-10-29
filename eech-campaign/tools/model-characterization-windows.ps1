# TEST-ONLY (EEDCS-50): runs lua/model-characterization.lua, one native process that boots the campaign and ASKS the original collision-model accessors (it steps no
# frame), with a binary directory built by tools/build-windows-model-fixture.sh. The shipping binaries have no model_* methods; that is reported NOT RUN (exit 3), never a pass.
#
#   -Bin           the fixture build output directory (eech-world.exe, eech_dc.dll, lua.dll, lua/)
#   -Root          a private copy of the retail installation (the engine writes into its root: one root per run)
#   -Out           a directory for model.jsonl, run.log and run.out; it must not already hold any of them (nothing is overwritten)
#   -Scenario      lebanon_retail (default), georgia_retail or georgia
#   -Scenes        comma-separated scene numbers taken from a catalogue (default none: catalogue only)
#   -MaxTriangles  per-scene cap on original calls (default 4096, at most 65536); a model that reaches it is reported `capped`, which is not a full capture
#   -NoCatalogue   skip the catalogue (needs -Scenes)
#
# The paths must hold no spaces (use a subst drive). Exit: 0 ran and every check passed; 3 NOT RUN (prerequisite absent); anything else is the native exit.
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Scenario = 'lebanon_retail',
	[string] $Scenes = '',
	[int] $MaxTriangles = 4096,
	[switch] $NoCatalogue
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
function Not-Run([string] $reason) { Write-Output "NOT RUN: $reason"; exit 3 }

if ($Scenario -notin @('lebanon_retail', 'georgia_retail', 'georgia')) { throw "unknown scenario: $Scenario" }
if ($MaxTriangles -lt 1 -or $MaxTriangles -gt 65536) { throw "MaxTriangles must be within 1..65536" }
if ($Scenes -ne '' -and $Scenes -notmatch '^[0-9]+(,[0-9]+)*$') { throw "Scenes must be comma-separated whole numbers" }
if ($NoCatalogue -and $Scenes -eq '') { throw "-NoCatalogue with no -Scenes asks nothing" }

if (-not (Test-Path -LiteralPath $Bin)) { Not-Run "binary directory $Bin is absent" }
$bin = (Resolve-Path -LiteralPath $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
$dll = Join-Path $bin 'eech_dc.dll'
$script = Join-Path $bin 'lua\model-characterization.lua'
foreach ($p in @($world, $dll, $script)) { if (-not (Test-Path -LiteralPath $p)) { Not-Run "$p is absent" } }
# a shipping DLL has no model_* methods: their names are strings in the fixture DLL only
$latin1 = [System.Text.Encoding]::GetEncoding(28591)
if (-not $latin1.GetString([System.IO.File]::ReadAllBytes($dll)).Contains('model_catalogue')) { Not-Run "$dll is not a model-fixture build" }
if (-not (Test-Path -LiteralPath $Root)) { Not-Run "installation root $Root is absent" }
# without the retail texture palette the engine writes a SYNTHETIC 3D database (no collision geometry anywhere) into the root: that is not this characterisation
foreach ($retail in @('cohokum\3ddata\textures.pal', 'cohokum\3ddata\3dobjs.bin', 'cohokum\3ddata\3dobjdb.bin')) {
	if (-not (Test-Path -LiteralPath (Join-Path $Root $retail))) { Not-Run "$retail is absent under $Root (not a retail installation)" }
}

New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path -LiteralPath $Out).Path
foreach ($p in @($bin, $Root, $out)) { if ($p -match '\s') { throw "path with a space: $p" } }
foreach ($name in @('model.jsonl', 'run.log', 'run.out')) {
	if (Test-Path -LiteralPath (Join-Path $out $name)) { throw "$name already exists in $out and is not overwritten" }
}

$arguments = @(($script -replace '\\', '/'), "root=$($Root -replace '\\', '/')", "scenario=$Scenario", "max_triangles=$MaxTriangles",
	"catalogue=$(if ($NoCatalogue) { 0 } else { 1 })", "out=$((Join-Path $out 'model.jsonl') -replace '\\', '/')")
if ($Scenes -ne '') { $arguments += "scenes=$Scenes" }
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
	-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList (ConvertTo-NativeArgumentString $arguments)
$null = $proc.Handle
$proc.WaitForExit()
"exit $($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds
exit $proc.ExitCode
