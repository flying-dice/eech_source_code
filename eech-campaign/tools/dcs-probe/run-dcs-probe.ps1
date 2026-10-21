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
$runId = "{0}-{1}" -f (Get-Date -Format 'yyyyMMdd-HHmmss'), $Stage
$out = Join-Path $Work "runs\$runId"
$writeDir = Join-Path $SavedGames ($WriteDirName.Replace('\', '/'))
$defaultDir = Join-Path $SavedGames ($DefaultWriteDirName.Replace('\', '/'))
$defaultLog = Join-Path $defaultDir 'Logs/dcs.log'
$dcsBin = Join-Path $Dcs 'bin-mt'
$dcsExe = Join-Path $dcsBin 'DCS.exe'
$manifest = [ordered]@{ run = $runId; stage = $Stage; started = (Get-Date).ToString('o') }

function Hash($p) { (Get-FileHash -Algorithm SHA256 $p).Hash.ToLower() }
function DefaultLogTicks { (Get-Item $defaultLog).LastWriteTimeUtc.Ticks }

# Physical identity of a path, for the protected-profile guard. Spelling is not identity:
# a junction or symlink anywhere in a path lets another name reach the same directory.
# Each existing component is resolved through links to its final target and takes its
# on-disk spelling; components that do not exist yet are kept as written. A link that
# cannot be resolved throws, so the guard fails closed.
function Resolve-PhysicalPath([string] $Path) {
	$comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
	$separator = [string][IO.Path]::DirectorySeparatorChar
	$current = [IO.Path]::GetFullPath($Path)
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
	$comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
	$separator = [string][IO.Path]::DirectorySeparatorChar
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
if ($Stage -in 'boot', 'frames') {
	if (-not $ErrorControlsEvidence -or -not (Test-Path -LiteralPath $ErrorControlsEvidence -PathType Leaf)) {
		throw "stage $Stage needs -ErrorControlsEvidence <passing DCS-Lua error-control JSON record>"
	}
	try { $controls = Get-Content -LiteralPath $ErrorControlsEvidence -Raw | ConvertFrom-Json -AsHashtable }
	catch { throw "invalid error-control JSON: $($_.Exception.Message)" }
	$candidateHash = Hash (Join-Path $Candidate 'eech_dc.dll')
	$hostLuaHash = Hash (Join-Path $dcsBin 'lua.dll')
	if ($controls.candidate_eech_dc_sha256 -ne $candidateHash -or $controls.dcs_lua_sha256 -ne $hostLuaHash -or
		$controls.controls.first_error.completed -ne $true -or $controls.controls.first_error.exit_code -ne 0 -or
		$controls.controls.repeated_errors.completed -ne $true -or $controls.controls.repeated_errors.exit_code -ne 0) {
		throw 'error-control record does not show passing first and repeated errors for this candidate and DCS Lua'
	}
}
if ($ValidatePrelaunchOnly) { Write-Output 'prelaunch guards passed'; return }

# Stops a started process and its descendants, and returns the stopped PIDs. A process
# instance is its PID together with its creation time, because Windows reuses PIDs and keeps
# a dead parent's PID in ParentProcessId. A child belongs to a parent instance only if it was
# created within that instance's lifetime: not before it started, and not after it ended. The
# end is the root's exit time, or the creation of a different process that now holds the
# same PID. So an unrelated process that reuses a PID, and that process's children, are
# never selected, while real survivors of an exited root or intermediate parent still are.
# $seen holds descendant instances recorded earlier (-ObserveOnly returns them): a
# descendant that survives its exited parent stays reachable. A process is stopped only
# if the live process list still shows that PID with that creation time.
function Stop-OwnedTree($process, $started, $seen = @(), [switch] $ObserveOnly) {
	$tolerance = [TimeSpan]::FromMilliseconds(500)   # Process.StartTime and WMI CreationDate round differently
	$rootEnded = $null
	if ($process.HasExited) { try { $rootEnded = $process.ExitTime } catch {} }
	$ids = @(if (-not $process.HasExited) { [int]$process.Id })   # no process list: the live root alone
	$descendants = @($seen)
	try {
		$all = @(Get-CimInstance Win32_Process)
		$tree = [System.Collections.Generic.List[object]]::new()
		$tree.Add(@{ id = [int]$process.Id; started = $started; ended = $rootEnded })
		foreach ($old in @($seen)) {
			if (-not ($old.id -eq $process.Id -and [Math]::Abs(($old.started - $started).TotalMilliseconds) -le $tolerance.TotalMilliseconds)) {
				$tree.Add(@{ id = [int]$old.id; started = $old.started; ended = $null })
			}
		}
		for ($i = 0; $i -lt $tree.Count; $i++) {
			$parent = $tree[$i]
			$end = $parent.ended
			foreach ($row in $all) {
				if ($row.ProcessId -eq $parent.id -and $row.CreationDate -gt ($parent.started + $tolerance) -and
					($null -eq $end -or $row.CreationDate -lt $end)) { $end = $row.CreationDate }
			}
			foreach ($row in $all) {
				if ($row.ParentProcessId -ne $parent.id -or $row.CreationDate -lt $parent.started -or
					($null -ne $end -and $row.CreationDate -gt $end)) { continue }
				$known = $false
				foreach ($instance in $tree) { if ($instance.id -eq $row.ProcessId -and $instance.started -eq $row.CreationDate) { $known = $true } }
				if (-not $known) { $tree.Add(@{ id = [int]$row.ProcessId; started = $row.CreationDate; ended = $null }) }
			}
		}
		$descendants = @($tree | Select-Object -Skip 1)
		$ids = @(if (-not $process.HasExited) { [int]$process.Id })
		foreach ($instance in $descendants) {
			if (@($all | Where-Object { $_.ProcessId -eq $instance.id -and $_.CreationDate -eq $instance.started }).Count -gt 0) { $ids += $instance.id }
		}
	} catch {}
	if ($ObserveOnly) { return , $descendants }
	$stopped = @()
	foreach ($id in $ids) { try { Stop-Process -Id $id -Force -ErrorAction Stop; $stopped += $id } catch {} }
	return , $stopped
}

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
New-Item -ItemType Directory -Force $out, "$Work\bin", "$Work\missions" | Out-Null

# inputs: the DLL only (never the candidate's own lua.dll), the probe, the hook, the mission
Copy-Item (Join-Path $Candidate 'eech_dc.dll') "$Work\bin\eech_dc.dll" -Force
Copy-Item "$here\eech-dcs-probe.lua" "$Work\eech-dcs-probe.lua" -Force
$miz = "$Work\missions\probe-$Stage.miz"
$templatePath = Join-Path $Dcs $Template
& python -I "$here\build-probe-miz.py" $templatePath $miz $Stage $Scenario "$Work\eech-dcs-probe.lua" "$Work\bin" $Root "$out\probe.log"
if ($LASTEXITCODE -ne 0) { throw 'mission build failed' }
New-Item -ItemType Directory -Force "$writeDir\Scripts\Hooks" | Out-Null
Copy-Item "$here\eech-m5probe-hook.lua" "$writeDir\Scripts\Hooks\eech-m5probe-hook.lua" -Force
$manifest.inputs = [ordered]@{
	candidate_eech_dc = @{ path = (Join-Path $Candidate 'eech_dc.dll'); sha256 = Hash "$Work\bin\eech_dc.dll" }
	probe = @{ sha256 = Hash "$Work\eech-dcs-probe.lua" }
	hook = @{ sha256 = Hash "$writeDir\Scripts\Hooks\eech-m5probe-hook.lua" }
	template = @{ path = $Template; sha256 = Hash $templatePath }
	mission = @{ path = $miz; sha256 = Hash $miz }
	dcs_exe = @{ path = $dcsExe; sha256 = Hash $dcsExe }
	dcs_lua = @{ sha256 = Hash (Join-Path $dcsBin 'lua.dll') }
	mission_scripting = @{ sha256 = Hash "$Dcs\Scripts\MissionScripting.lua" }
	autoupdate_version = (Get-Content "$Dcs\autoupdate.cfg" -Raw | ConvertFrom-Json).version
}
$manifest.scenario = $Scenario
$defaultLogTicksBefore = DefaultLogTicks
if (Test-Path "$writeDir\Logs\dcs.log") { Remove-Item "$writeDir\Logs\dcs.log" }

$env:EECH_M5PROBE_RUN_SECONDS = "$RunSeconds"
$arguments = @('-w', $WriteDirName, '--no-launcher', '--mission', "`"$miz`"")
$manifest.cancel_login = [bool]$CancelLogin
if ($ErrorControlsEvidence) { $manifest.error_controls_evidence = @{ path = $ErrorControlsEvidence; sha256 = Hash $ErrorControlsEvidence } }
$manifest.command = "Start-Process `"$dcsExe`" -ArgumentList $($arguments -join ' ') -WorkingDirectory `"$dcsBin`" -WindowStyle Hidden -PassThru"
$p = Start-Process -FilePath $dcsExe -ArgumentList $arguments -WorkingDirectory $dcsBin -WindowStyle Hidden -PassThru
$processStarted = $p.StartTime
$manifest.pid = $p.Id
$t0 = Get-Date
$verdict = 'timeout'
$dialogs = [ordered]@{}
$stopped = @()
$seen = @()   # descendants observed while the run is live (PID plus creation time)
try {
	while ($true) {
		Start-Sleep -Seconds 2
		$elapsed = ((Get-Date) - $t0).TotalSeconds
		if (-not $p.HasExited) {
			foreach ($d in [ProbeWindows]::ScanDialogs([uint32]$p.Id, [bool]$CancelLogin)) { if (-not $dialogs.Contains($d)) { $dialogs[$d] = [Math]::Round($elapsed) } }
		}
		if ((DefaultLogTicks) -ne $defaultLogTicksBefore) { $verdict = 'aborted: the default profile log changed'; break }
		if ($p.HasExited) { $verdict = 'exited with code {0} (0x{0:X8})' -f $p.ExitCode; break }
		if ($elapsed -gt $TimeoutSeconds) { break }
		try { $seen = Stop-OwnedTree $p $processStarted $seen -ObserveOnly } catch {}
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
if (Test-Path "$writeDir\Logs") {
	Get-ChildItem "$writeDir\Logs" -File | Where-Object { $_.LastWriteTime -ge $t0.AddSeconds(-5) } | Copy-Item -Destination $out
}
$manifest.outputs = @(Get-ChildItem $out -File | ForEach-Object { @{ name = $_.Name; bytes = $_.Length; sha256 = Hash $_.FullName } })
$manifest.default_profile_log_unchanged = (Test-Path $defaultLog) -and ($defaultLogTicksBefore -eq (DefaultLogTicks))
$manifest.finished = (Get-Date).ToString('o')
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 "$out\manifest.json"
Write-Output "run $runId -> $out ($verdict)"
