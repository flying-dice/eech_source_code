# TEST-ONLY (EEDCS-50, D3): runs lua/model-host-compare.lua, one native process of the HOST alone (no campaign DLL is loaded, no engine is booted), which loads the
# collision-model store from the retail disk assets under -Root and compares it with a retained ORIGINAL model-driver report. The ordinary build's eech-world.exe and lua/
# directory are used (tools/build-windows.sh); no fixture build is involved.
#
#   -Bin           the build output directory (eech-world.exe, lua.dll, lua/)
#   -Root          a private copy of the retail installation (it is only READ)
#   -Out           a directory for host-model.jsonl, run.log and run.out; it must not already hold any of them (nothing is overwritten)
#   -Original      a retained original model-driver report (its JSON Lines), to compare the store with (optional)
#   -Scenes        comma-separated scene numbers whose host triangles are listed (default none)
#   -MaxTriangles  per-scene cap on listed triangles (default 4096, at most 65536); a model that reaches it is reported `host-model-capped`
#   -NoCatalogue   do not list every scene (needs -Scenes or -Original)
#
# The paths must hold no spaces (use a subst drive). Exit: 0 ran and everything compared matched; 3 NOT RUN (a prerequisite is absent); anything else is the native exit
# (a mismatch, a refused store or a failed check is non-zero: it is never a pass).
param(
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Original = '',
	[string] $Scenes = '',
	[int] $MaxTriangles = 4096,
	[switch] $NoCatalogue
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
function Not-Run([string] $reason) { Write-Output "NOT RUN: $reason"; exit 3 }

if ($MaxTriangles -lt 1 -or $MaxTriangles -gt 65536) { throw "MaxTriangles must be within 1..65536" }
if ($Scenes -ne '' -and $Scenes -notmatch '^[0-9]+(,[0-9]+)*$') { throw "Scenes must be comma-separated whole numbers" }
if ($NoCatalogue -and $Scenes -eq '' -and $Original -eq '') { throw "-NoCatalogue with no -Scenes and no -Original asks nothing" }

if (-not (Test-Path -LiteralPath $Bin)) { Not-Run "binary directory $Bin is absent" }
$bin = (Resolve-Path -LiteralPath $Bin).Path
$world = Join-Path $bin 'eech-world.exe'
$script = Join-Path $bin 'lua\model-host-compare.lua'
foreach ($p in @($world, $script)) { if (-not (Test-Path -LiteralPath $p)) { Not-Run "$p is absent" } }
if (-not (Test-Path -LiteralPath $Root)) { Not-Run "installation root $Root is absent" }
foreach ($retail in @('cohokum\3ddata\bininfo.bin', 'cohokum\3ddata\3dobjs.bin', 'cohokum\3ddata\3dobjs.pts', 'cohokum\3ddata\3dobjs.sp', 'cohokum\3ddata\3dobjdb.bin')) {
	if (-not (Test-Path -LiteralPath (Join-Path $Root $retail))) { Not-Run "$retail is absent under $Root (not a retail installation)" }
}
if ($Original -ne '' -and -not (Test-Path -LiteralPath $Original)) { Not-Run "the original report $Original is absent" }

New-Item -ItemType Directory -Force $Out | Out-Null
$out = (Resolve-Path -LiteralPath $Out).Path
$paths = @($bin, $Root, $out)
if ($Original -ne '') { $paths += (Resolve-Path -LiteralPath $Original).Path }
foreach ($p in $paths) { if ($p -match '\s') { throw "path with a space: $p" } }
foreach ($name in @('host-model.jsonl', 'run.log', 'run.out')) {
	if (Test-Path -LiteralPath (Join-Path $out $name)) { throw "$name already exists in $out and is not overwritten" }
}

$arguments = @(($script -replace '\\', '/'), "root=$($Root -replace '\\', '/')", "max_triangles=$MaxTriangles",
	"catalogue=$(if ($NoCatalogue) { 0 } else { 1 })", "out=$((Join-Path $out 'host-model.jsonl') -replace '\\', '/')")
if ($Scenes -ne '') { $arguments += "scenes=$Scenes" }
if ($Original -ne '') { $arguments += "original=$(((Resolve-Path -LiteralPath $Original).Path) -replace '\\', '/')" }
$start = Get-Date
$proc = Start-Process -FilePath $world -WorkingDirectory $out -NoNewWindow -PassThru -RedirectStandardError (Join-Path $out 'run.log') `
	-RedirectStandardOutput (Join-Path $out 'run.out') -ArgumentList (ConvertTo-NativeArgumentString $arguments)
$null = $proc.Handle
$proc.WaitForExit()
"exit $($proc.ExitCode) after {0:N0} s" -f ((Get-Date) - $start).TotalSeconds
exit $proc.ExitCode
