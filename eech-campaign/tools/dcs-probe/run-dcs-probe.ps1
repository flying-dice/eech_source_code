# Runs one bounded DCS probe stage in an isolated Saved Games profile and
# collects its evidence. It starts its own DCS process (hidden), never touches
# a pre-existing one, and stops only the process tree it started.
#
#   pwsh -File run-dcs-probe.ps1 -Stage load -Candidate <dir with eech_dc.dll> [-Root <EECH root copy>] [-Scenario lebanon_retail] [-CancelLogin]
#
# DCS is started as bin-mt\DCS.exe -w <profile> --no-launcher --mission <probe miz>;
# --mission is the route observed to load the mission (the GUI hook cannot start it).
# Refuses to start while any DCS.exe is running. Aborts if the default
# profile's dcs.log changes (which would mean -w was not honoured). The verdict
# is the process outcome only; what the probe reached is read from probe.log.
param(
	[Parameter(Mandatory)] [ValidateSet('env', 'load', 'boot', 'frames')] [string] $Stage,
	[Parameter(Mandatory)] [string] $Candidate,
	# the EECH installation root the campaign boots from (a copy: boot chdirs into it, prepare writes to it)
	[string] $Root = 'E:\eech-runs\dcs-probe\lebanon-root',
	[ValidateSet('lebanon_retail', 'georgia_retail')] [string] $Scenario = 'lebanon_retail',
	[string] $Dcs = 'D:\Program Files\Eagle Dynamics\DCS World OpenBeta',
	[string] $WriteDirName = 'DCS.M5Probe',
	# the installation's default profile, which must stay untouched; its dcs.log is watched
	[string] $DefaultWriteDirName = 'DCS.openbeta',
	[string] $SavedGames = (Join-Path $env:USERPROFILE 'Saved Games'),
	[string] $Work = 'E:\eech-runs\dcs-probe',
	[string] $Template = 'CoreMods\WWII Units\FW-190D-9\Missions\fw190.miz',
	[int] $RunSeconds = 20,
	[int] $TimeoutSeconds = 420,
	# A profile without saved credentials waits at a native 'DCS Login' dialog. This clicks
	# Cancel on it, then OK on 'Authorization cancelled', in the owned process only; DCS then
	# runs unauthorised (paid modules are skipped, free content loads)
	[switch] $CancelLogin,
	# boot/frames require a passing DCS-Lua error-control record for this exact DLL and host Lua.
	[string] $ErrorControlsEvidence,
	# Portable guard test: validates inputs without modifying profiles or launching DCS.
	[switch] $ValidatePrelaunchOnly
)
$ErrorActionPreference = 'Stop'
$here = $PSScriptRoot
. (Join-Path $here 'owned-process-tree.ps1')
$comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
$separator = [string][IO.Path]::DirectorySeparatorChar

# A profile name is a relative path of plain names. Characters that Windows reads as a stream
# (':'), a wildcard or a device or reserved character are refused: they can alias another
# directory ("DCS.openbeta::$INDEX_ALLOCATION") or be globbed by a later command. A trailing dot
# or space is dropped by Windows, so it is an alias too.
function Assert-ProfileName([string] $Label, [string] $Name) {
	if (-not $Name.Trim()) { throw "$Label is empty" }
	foreach ($component in $Name.Split([char[]]@('\', '/'), [StringSplitOptions]::RemoveEmptyEntries)) {
		if ($component -match '[\x00-\x1f:\[\]*?<>|"]') { throw "$Label '$Name' has a character that is not allowed in a profile name (: [ ] * ? < > | `" or a control character)" }
		if ($component -notin '.', '..' -and $component -match '[. ]$') { throw "$Label '$Name' has a component ending in a dot or space" }
		# a DOS device name, with or without an extension, is not a directory
		if ($component -match '^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\..*)?$') { throw "$Label '$Name' uses the reserved device name '$component'" }
	}
}
Assert-ProfileName '-WriteDirName' $WriteDirName
Assert-ProfileName '-DefaultWriteDirName' $DefaultWriteDirName

# Every [IO.*] call resolves a relative path against the process directory, not against $PWD, so
# a relative input is made absolute against $PWD first.
function Resolve-AgainstPwd([string] $Path) {
	if ([IO.Path]::IsPathRooted($Path)) { return [IO.Path]::GetFullPath($Path) }
	[IO.Path]::GetFullPath((Join-Path $PWD.ProviderPath $Path))
}
$SavedGames = Resolve-AgainstPwd $SavedGames
$Work = Resolve-AgainstPwd $Work
$Candidate = Resolve-AgainstPwd $Candidate
$Dcs = Resolve-AgainstPwd $Dcs
$Root = Resolve-AgainstPwd $Root
if ($ErrorControlsEvidence) { $ErrorControlsEvidence = Resolve-AgainstPwd $ErrorControlsEvidence }
$runId = "{0}-{1}" -f (Get-Date -Format 'yyyyMMdd-HHmmss'), $Stage
$out = Join-Path $Work "runs\$runId"
$writeDir = Join-Path $SavedGames ($WriteDirName.Replace('\', '/'))
$defaultDir = Join-Path $SavedGames ($DefaultWriteDirName.Replace('\', '/'))
$defaultLog = Join-Path $defaultDir 'Logs/dcs.log'
$dcsBin = Join-Path $Dcs 'bin-mt'
$dcsExe = Join-Path $dcsBin 'DCS.exe'
$manifest = [ordered]@{ run = $runId; stage = $Stage; started = (Get-Date).ToString('o') }

function Hash($p) { (Get-FileHash -Algorithm SHA256 -LiteralPath $p).Hash.ToLower() }
function DefaultLogTicks { (Get-Item -LiteralPath $defaultLog).LastWriteTimeUtc.Ticks }

# Physical identity of a path, for the protected-profile guard. Spelling is not identity:
# a junction or symlink anywhere in a path lets another name reach the same directory.
# Each existing component is resolved through links to its final target and takes its
# on-disk spelling; components that do not exist yet are kept as written. A link that
# cannot be resolved, or whose target is not a plain drive-letter path (a volume GUID, a
# verbatim or UNC spelling of the same directory), throws: those spellings cannot be compared
# with a plain path, so the guard fails closed instead of guessing.
function Assert-PlainPath([string] $Path, [string] $What) {
	$plain = if ($IsWindows) { $Path -match '^[A-Za-z]:[\\/]' } else { $Path.StartsWith('/') }
	if (-not $plain) { throw "$What is not a plain absolute path ($Path)" }
}
function Resolve-PhysicalPath([string] $Path) {
	$current = [IO.Path]::GetFullPath($Path)
	Assert-PlainPath $current "path $Path"
	for ($hop = 0; $hop -le 32; $hop++) {
		$root = [IO.Path]::GetPathRoot($current)
		$parts = @($current.Substring($root.Length).Split([char[]]@('\', '/'), [StringSplitOptions]::RemoveEmptyEntries))
		$resolved = $root
		$redirected = $false
		for ($i = 0; $i -lt $parts.Count -and -not $redirected; $i++) {
			$name = $parts[$i]
			if ($name.IndexOfAny([char[]]@('*', '?')) -ge 0) { throw "wildcard in path: $Path" }
			$entry = $null
			if ([IO.Directory]::Exists($resolved)) {
				$entry = [IO.DirectoryInfo]::new($resolved).GetFileSystemInfos($name) |
					Where-Object { [string]::Equals($_.Name, $name, $comparison) } | Select-Object -First 1
			}
			if (-not $entry) {
				return [IO.Path]::GetFullPath([IO.Path]::Combine($resolved, ($parts[$i..($parts.Count - 1)] -join $separator)))
			}
			if ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint) {
				$target = $entry.ResolveLinkTarget($true)
				if (-not $target) { throw "cannot resolve the reparse point $($entry.FullName)" }
				Assert-PlainPath $target.FullName "the target of the link $($entry.FullName)"
				$rest = if ($i + 1 -lt $parts.Count) { $parts[($i + 1)..($parts.Count - 1)] } else { @() }
				$current = [IO.Path]::GetFullPath([IO.Path]::Combine($target.FullName, ($rest -join $separator)))
				$redirected = $true
			} else {
				$resolved = $entry.FullName
			}
		}
		if (-not $redirected) { return $resolved }
	}
	throw "too many links while resolving $Path"
}

# The probe profile is written to (hook installed, dcs.log replaced). None of those paths may
# be, lie inside, or contain the protected default profile or its dcs.log, however they are
# spelled or linked.
try {
	$protected = @((Resolve-PhysicalPath $defaultDir), (Resolve-PhysicalPath $defaultLog))
	$written = @($writeDir) + @('Logs', 'Logs/dcs.log', 'Scripts', 'Scripts/Hooks', 'Scripts/Hooks/eech-m5probe-hook.lua' |
		ForEach-Object { Join-Path $writeDir $_ })
	foreach ($path in $written) {
		$physical = (Resolve-PhysicalPath $path).TrimEnd('\', '/')
		foreach ($guarded in $protected) {
			$guarded = $guarded.TrimEnd('\', '/')
			if ([string]::Equals($physical, $guarded, $comparison) -or
				$physical.StartsWith($guarded + $separator, $comparison) -or
				$guarded.StartsWith($physical + $separator, $comparison)) {
				throw "$path is, lies inside or contains the default profile $DefaultWriteDirName ($physical vs $guarded)"
			}
		}
	}
} catch {
	throw "-WriteDirName $WriteDirName is not a safe probe profile: $($_.Exception.Message)"
}
if (-not (Test-Path -LiteralPath $defaultLog -PathType Leaf)) {
	throw "no $defaultLog to watch: pass the installation's -DefaultWriteDirName"
}
# The record is untrusted JSON. It is read with System.Text.Json so that nothing is coerced or
# merged: the top level must be an object (an array is refused, even a one-element one that
# ConvertFrom-Json would unroll), no object repeats a key (case-variant keys count as repeats),
# key names match exactly, a hash is a string of exactly 64 hex digits with no surrounding
# whitespace (hex digits match case-insensitively), completed is the JSON boolean true and
# exit_code is the integer 0.
function Test-UniqueKeys($Element) {
	if ($Element.ValueKind -eq 'Object') {
		$names = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
		foreach ($property in $Element.EnumerateObject()) {
			if (-not $names.Add($property.Name) -or -not (Test-UniqueKeys $property.Value)) { return $false }
		}
	} elseif ($Element.ValueKind -eq 'Array') {
		foreach ($item in $Element.EnumerateArray()) { if (-not (Test-UniqueKeys $item)) { return $false } }
	}
	return $true
}
function Test-ErrorControls([string] $Raw, [string] $CandidateHash, [string] $HostLuaHash) {
	function Get-Member($Element, [string] $Name) {
		$value = [System.Text.Json.JsonElement]::new()   # Undefined unless found
		if ($Element.ValueKind -eq 'Object') { $null = $Element.TryGetProperty($Name, [ref] $value) }
		return $value
	}
	function Test-Hash($Element, [string] $Expected) {
		if ($Element.ValueKind -ne 'String') { return $false }
		$text = $Element.GetString()
		return ($text -cmatch '\A[0-9A-Fa-f]{64}\z') -and ($text.ToLowerInvariant() -ceq $Expected)
	}
	function Test-Control($Element) {
		$completed = Get-Member $Element 'completed'
		$exit = Get-Member $Element 'exit_code'
		return $Element.ValueKind -eq 'Object' -and $completed.ValueKind -eq 'True' -and
			$exit.ValueKind -eq 'Number' -and $exit.GetRawText() -ceq '0'
	}
	$document = [System.Text.Json.JsonDocument]::Parse($Raw)
	try {
		$record = $document.RootElement
		$controls = Get-Member $record 'controls'
		return $record.ValueKind -eq 'Object' -and (Test-UniqueKeys $record) -and
			(Test-Hash (Get-Member $record 'candidate_eech_dc_sha256') $CandidateHash) -and
			(Test-Hash (Get-Member $record 'dcs_lua_sha256') $HostLuaHash) -and
			(Test-Control (Get-Member $controls 'first_error')) -and (Test-Control (Get-Member $controls 'repeated_errors'))
	} finally { $document.Dispose() }
}
if ($Stage -in 'boot', 'frames') {
	if (-not $ErrorControlsEvidence -or -not (Test-Path -LiteralPath $ErrorControlsEvidence -PathType Leaf)) {
		throw "stage $Stage needs -ErrorControlsEvidence <passing DCS-Lua error-control JSON record>"
	}
	$candidateHash = Hash (Join-Path $Candidate 'eech_dc.dll')
	$hostLuaHash = Hash (Join-Path $dcsBin 'lua.dll')
	try { $passing = Test-ErrorControls (Get-Content -LiteralPath $ErrorControlsEvidence -Raw) $candidateHash $hostLuaHash }
	catch { throw "invalid error-control JSON: $($_.Exception.Message)" }
	if (-not $passing) {
		throw 'error-control record does not show passing first and repeated errors for this candidate and DCS Lua'
	}
}
if ($ValidatePrelaunchOnly) { Write-Output 'prelaunch guards passed'; return }

# ScanDialogs lists one process's top-level dialogs (title, static and button texts; edit-box
# contents are never read). With dismissLogin it also clicks Cancel on 'DCS Login' and OK on
# 'Authorization cancelled', and marks the clicks in the returned text.
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class ProbeWindows {
	delegate bool Visit (IntPtr h, IntPtr l);
	[DllImport ("user32.dll")] static extern bool EnumWindows (Visit f, IntPtr l);
	[DllImport ("user32.dll")] static extern bool EnumChildWindows (IntPtr p, Visit f, IntPtr l);
	[DllImport ("user32.dll")] static extern uint GetWindowThreadProcessId (IntPtr h, out uint pid);
	[DllImport ("user32.dll")] static extern int GetWindowText (IntPtr h, StringBuilder s, int n);
	[DllImport ("user32.dll")] static extern int GetClassName (IntPtr h, StringBuilder s, int n);
	[DllImport ("user32.dll")] static extern IntPtr GetDlgItem (IntPtr d, int id);
	[DllImport ("user32.dll")] static extern IntPtr SendMessage (IntPtr h, uint m, IntPtr w, IntPtr l);
	static string Text (IntPtr h) { var s = new StringBuilder (512); GetWindowText (h, s, 512); return s.ToString (); }
	static string Class (IntPtr h) { var s = new StringBuilder (256); GetClassName (h, s, 256); return s.ToString (); }
	public static List<string> ScanDialogs (uint pid, bool dismissLogin) {
		var found = new List<string> ();
		EnumWindows ((h, l) => {
			uint owner; GetWindowThreadProcessId (h, out owner);
			if (owner != pid || Class (h) != "#32770") return true;
			var parts = new List<string> { Text (h) };
			EnumChildWindows (h, (c, l2) => { var k = Class (c); if (k == "Static" || k == "Button") { var t = Text (c); if (t != "") parts.Add (t); } return true; }, IntPtr.Zero);
			if (dismissLogin && Text (h) == "DCS Login") { SendMessage (GetDlgItem (h, 2), 0x00F5, IntPtr.Zero, IntPtr.Zero); parts.Add ("[Cancel clicked]"); }
			else if (dismissLogin && Text (h) == "Authorization cancelled") { SendMessage (GetDlgItem (h, 1), 0x00F5, IntPtr.Zero, IntPtr.Zero); parts.Add ("[OK clicked]"); }
			found.Add (string.Join (" | ", parts));
			return true;
		}, IntPtr.Zero);
		return found;
	}
}
'@

if (Get-Process -Name DCS -ErrorAction SilentlyContinue) { throw 'a DCS process is already running: not starting another' }
foreach ($directory in $out, "$Work\bin", "$Work\missions") { $null = [IO.Directory]::CreateDirectory($directory) }

# inputs: the DLL only (never the candidate's own lua.dll), the probe, the hook, the mission
Copy-Item -LiteralPath (Join-Path $Candidate 'eech_dc.dll') -Destination "$Work\bin\eech_dc.dll" -Force
Copy-Item -LiteralPath "$here\eech-dcs-probe.lua" -Destination "$Work\eech-dcs-probe.lua" -Force
$miz = "$Work\missions\probe-$Stage.miz"
$templatePath = Join-Path $Dcs $Template
& python -I "$here\build-probe-miz.py" $templatePath $miz $Stage $Scenario "$Work\eech-dcs-probe.lua" "$Work\bin" $Root "$out\probe.log"
if ($LASTEXITCODE -ne 0) { throw 'mission build failed' }
$hook = "$writeDir\Scripts\Hooks\eech-m5probe-hook.lua"
$null = [IO.Directory]::CreateDirectory("$writeDir\Scripts\Hooks")
# unlink first: a hard link at the destination would otherwise be written through to another file
[IO.File]::Delete($hook)
[IO.File]::Copy("$here\eech-m5probe-hook.lua", $hook)
$manifest.inputs = [ordered]@{
	candidate_eech_dc = @{ path = (Join-Path $Candidate 'eech_dc.dll'); sha256 = Hash "$Work\bin\eech_dc.dll" }
	probe = @{ sha256 = Hash "$Work\eech-dcs-probe.lua" }
	hook = @{ sha256 = Hash $hook }
	template = @{ path = $Template; sha256 = Hash $templatePath }
	mission = @{ path = $miz; sha256 = Hash $miz }
	dcs_exe = @{ path = $dcsExe; sha256 = Hash $dcsExe }
	dcs_lua = @{ sha256 = Hash (Join-Path $dcsBin 'lua.dll') }
	mission_scripting = @{ sha256 = Hash "$Dcs\Scripts\MissionScripting.lua" }
	autoupdate_version = (Get-Content -LiteralPath "$Dcs\autoupdate.cfg" -Raw | ConvertFrom-Json).version
}
$manifest.scenario = $Scenario
$defaultLogTicksBefore = DefaultLogTicks
[IO.File]::Delete("$writeDir\Logs\dcs.log")   # the file or link itself, if present

$arguments = @('-w', $WriteDirName, '--no-launcher', '--mission', "`"$miz`"")
$manifest.cancel_login = [bool]$CancelLogin
if ($ErrorControlsEvidence) { $manifest.error_controls_evidence = @{ path = $ErrorControlsEvidence; sha256 = Hash $ErrorControlsEvidence } }
$manifest.command = "Start-Process `"$dcsExe`" -ArgumentList $($arguments -join ' ') -WorkingDirectory `"$dcsBin`" -WindowStyle Hidden -PassThru"
$t0 = Get-Date
$verdict = 'timeout'
$dialogs = [ordered]@{}
$stopped = @()
$seen = @()   # owned descendants recorded while the run is live (PID plus creation time)
$p = $null
$processStarted = $null
# Everything from the environment bound onward is inside the try, so that whatever fails after
# DCS has started (reading its start time included) the finally still stops the owned tree and
# drops the bound.
try {
	$env:EECH_M5PROBE_RUN_SECONDS = "$RunSeconds"
	$p = Start-Process -FilePath $dcsExe -ArgumentList $arguments -WorkingDirectory $dcsBin -WindowStyle Hidden -PassThru
	$manifest.pid = $p.Id
	$processStarted = $p.StartTime
	while ($true) {
		Start-Sleep -Seconds 2
		$elapsed = ((Get-Date) - $t0).TotalSeconds
		if (-not $p.HasExited) {
			foreach ($d in [ProbeWindows]::ScanDialogs([uint32]$p.Id, [bool]$CancelLogin)) { if (-not $dialogs.Contains($d)) { $dialogs[$d] = [Math]::Round($elapsed) } }
		}
		if ((DefaultLogTicks) -ne $defaultLogTicksBefore) { $verdict = 'aborted: the default profile log changed'; break }
		if ($p.HasExited) { $verdict = 'exited with code {0} (0x{0:X8})' -f $p.ExitCode; break }
		if ($elapsed -gt $TimeoutSeconds) { break }
		try { $seen = @(Get-OwnedInstances $p $processStarted $seen) } catch {}
	}
} catch {
	$verdict = "runner error: $($_.Exception.Message)"
} finally {
	# whatever happened above, the owned DCS does not outlive the runner
	$stopped = Stop-OwnedTree $p $processStarted $seen
	# the hook's run bound must not leak to a later manual launch from this shell
	Remove-Item Env:EECH_M5PROBE_RUN_SECONDS -ErrorAction SilentlyContinue
}
$manifest.elapsed_seconds = [Math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
$manifest.verdict = $verdict
$manifest.dialogs = @($dialogs.Keys | ForEach-Object { @{ seconds = $dialogs[$_]; text = $_ } })
$manifest.stopped_pids = $stopped
Start-Sleep -Seconds 3
if ([IO.Directory]::Exists("$writeDir\Logs")) {
	foreach ($log in Get-ChildItem -LiteralPath "$writeDir\Logs" -File | Where-Object { $_.LastWriteTime -ge $t0.AddSeconds(-5) }) {
		[IO.File]::Copy($log.FullName, (Join-Path $out $log.Name), $true)
	}
}
$manifest.outputs = @(Get-ChildItem -LiteralPath $out -File | ForEach-Object { @{ name = $_.Name; bytes = $_.Length; sha256 = Hash $_.FullName } })
$manifest.default_profile_log_unchanged = (Test-Path -LiteralPath $defaultLog) -and ($defaultLogTicksBefore -eq (DefaultLogTicks))
$manifest.finished = (Get-Date).ToString('o')
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 -LiteralPath "$out\manifest.json"
Write-Output "run $runId -> $out ($verdict)"
