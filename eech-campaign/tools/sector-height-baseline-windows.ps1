# TEST-ONLY public baseline observation of the sector tallest-structure word (EEDCS-49, D3; docs/d3-sector-height-public-baseline-plan.md): eech-world -> Lua -> eech_dc on a DLL built
# with tools/build-windows-sector-height-observer.sh. ONE run of lua/sector-height-baseline.lua on a retail root the caller prepared (a fresh verified copy, never a consumed root). The
# observer is read-only; the scenario is the ordinary campaign boot over the ESTABLISHED finite ordinary short window (hours=0.001, 3.6 simulated seconds, frame_ms=100, seed=1). A larger
# window needs its own named, bounded plan before launch and is never chosen by what the observation reached; a case absent from the window is reported UNREACHED (a zero counter).
#
# SCOPE (Task49, the host-owned sector height): `-Scope original` (the default) observes the campaign-held word in a build that keeps the ORIGINAL hooks; `-Scope hosted-scan` runs the same driver against a HOSTED
# observer build and writes only the scan reads (the word the hosted getter was answered at each FIRST, and fact 12 of every offer), the clearance guard records and the per-sector scan rows; with `-Accepted <the accepted original
# report>` the driver compares those rows by sector at the same window (see lua/sector-height-baseline.lua, SCOPE hosted-scan) and writes `sho-compare` and a scoped `sho-verdict`. The original link, event, shape, clearance-word and
# session records are NOT comparable in a hosted build and are not written.
#
# This wrapper never terminates a native process. A wait that ends while the process is still running is NOT a failure: the process receipt (pid, birth, command line, argument hash, paths) is
# already on disk, the process is left alone, and the script exits 2 ("still running") without writing a completion. Only the waiting parent can record the actual exit code, so for a long
# window start this script from the persistent monitor with -WaitSeconds 0 (wait without limit, the default). Every output (observation, stdout, stderr, both receipts) is created NEW and
# atomically: the script refuses to start if any of them exists, so no earlier log or partial report is ever overwritten or truncated.
param(
	[Parameter(Mandatory = $true)] [string] $Root,
	[Parameter(Mandatory = $true)] [string] $Bin,
	[Parameter(Mandatory = $true)] [string] $Out,
	[string] $Scenario = 'lebanon_retail',
	[string] $Hours = '0.001',
	[int] $FrameMs = 100,
	[int] $Seed = 1,
	[int] $WaitSeconds = 0,
	[ValidateSet('original', 'hosted-scan')] [string] $Scope = 'original',
	[string] $Accepted = ''
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'native-arguments.ps1')
$here = Split-Path -Parent $PSScriptRoot
$null = [IO.Directory]::CreateDirectory($Out)
$world = Join-Path $Bin 'eech-world.exe'
if (-not (Test-Path -LiteralPath $world)) { throw "$world not found: run tools/build-windows-sector-height-observer.sh" }
$script = Join-Path $here 'crates\eech-world\lua\sector-height-baseline.lua'
$stem = "sector-height-$Scenario-seed$Seed"
$observation = Join-Path $Out "$stem.txt"
$stdout = Join-Path $Out "$stem.out"
$stderr = Join-Path $Out "$stem.log"
$processReceipt = Join-Path $Out "$stem-process.json"
$completionReceipt = Join-Path $Out "$stem-completion.json"

function New-OwnedFile($path, $text) {
	# CREATE_NEW: refuses ANY existing target atomically (an earlier log, partial report or receipt is never overwritten)
	$stream = [IO.File]::Open($path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
	try { if ($text) { $bytes = [Text.Encoding]::UTF8.GetBytes($text); $stream.Write($bytes, 0, $bytes.Length) } } finally { $stream.Dispose() }
}
foreach ($p in @($observation, $stdout, $stderr, $processReceipt, $completionReceipt)) {
	if (Test-Path -LiteralPath $p) { throw "$p exists: a baseline output is never overwritten" }
}
# the redirect targets and the receipts are claimed first, atomically; the driver creates the observation exclusively itself
New-OwnedFile $stdout $null
New-OwnedFile $stderr $null

$arguments = @($script, "root=$($Root -replace '\\', '/')", "scenario=$Scenario", "hours=$Hours", "frame_ms=$FrameMs", "seed=$Seed", "out=$($observation -replace '\\', '/')")
if ($Scope -ne 'original') { $arguments += "scope=$Scope" }
if ($Accepted) {
	if ($Scope -ne 'hosted-scan') { throw '-Accepted is only for -Scope hosted-scan' }
	if (-not (Test-Path -LiteralPath $Accepted)) { throw "$Accepted not found: the accepted original report is read, never created" }
	$arguments += "accepted=$($Accepted -replace '\\', '/')"
}
$argumentString = ConvertTo-NativeArgumentString $arguments
$process = Start-Process -FilePath $world -ArgumentList $argumentString -NoNewWindow -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
$null = $process.Handle
$sha = [Security.Cryptography.SHA256]::Create()
$argumentHash = ([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($argumentString))) -replace '-', '').ToLowerInvariant()
$receipt = [ordered]@{
	state = 'running'; pid = $process.Id; birth_utc = $process.StartTime.ToUniversalTime().ToString('o'); executable = $world
	arguments = $argumentString; arguments_sha256 = $argumentHash; observation = $observation; stdout = $stdout; stderr = $stderr
	hours = $Hours; frame_ms = $FrameMs; seed = $Seed; scenario = $Scenario; started_utc = [DateTime]::UtcNow.ToString('o')
	note = 'created before the wait; the process is never terminated by this script'
}
New-OwnedFile $processReceipt (($receipt | ConvertTo-Json) + "`n")

$finished = if ($WaitSeconds -gt 0) { $process.WaitForExit($WaitSeconds * 1000) } else { $process.WaitForExit(); $true }
if (-not $finished) {
	Write-Host "STILL RUNNING (not a failure, not terminated): pid $($process.Id) born $($receipt.birth_utc); receipt $processReceipt"
	Write-Host 'No completion is recorded: observe the process by pid, birth and argument hash; the actual exit code is only known to a waiting parent.'
	exit 2
}
$exit = $process.ExitCode
$completion = [ordered]@{ state = 'terminal'; pid = $process.Id; birth_utc = $receipt.birth_utc; actual_exit = $exit; completed_utc = [DateTime]::UtcNow.ToString('o'); observation = $observation }
New-OwnedFile $completionReceipt (($completion | ConvertTo-Json) + "`n")
if ($exit -ne 0) { throw "eech-world exited $exit; see $stderr and $completionReceipt" }
$lines = @(Get-Content -LiteralPath $observation)
if ($Scope -eq 'hosted-scan') {
	if ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict scope=hosted-scan scan_whole=1')) { throw "the hosted-scan observation has no whole scan verdict; see $observation" }
	if ($Accepted -and -not $lines[-1].Contains('status=EQUAL_AT_SCOPE')) { throw "the hosted scan did not compare equal at scope; see $observation" }
}
elseif ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict whole=1')) { throw "the observation has no whole verdict; see $observation" }
Write-Host "observation: $observation"
Write-Host $lines[-1]
