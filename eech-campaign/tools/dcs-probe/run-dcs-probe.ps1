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

if ([IO.Path]::GetFullPath($writeDir).TrimEnd('\', '/') -eq [IO.Path]::GetFullPath($defaultDir).TrimEnd('\', '/')) {
	throw "-WriteDirName must not resolve to the default profile $DefaultWriteDirName"
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

# Stops a started process and its descendants, and returns the stopped PIDs. A child
# counts only if it started after its parent: Windows keeps a dead parent's PID in
# ParentProcessId, so a reused PID must not adopt an unrelated process.
function Stop-OwnedTree($process, $started) {
	$ids = if ($process.HasExited) { @() } else { @($process.Id) }
	try {
		$all = @(Get-CimInstance Win32_Process)
		$tree = @(@{ id = $process.Id; started = $started })
		for ($i = 0; $i -lt $tree.Count; $i++) {
			$parent = $tree[$i]
			$tree += @($all | Where-Object { $_.ParentProcessId -eq $parent.id -and $_.CreationDate -ge $parent.started } |
				ForEach-Object { @{ id = $_.ProcessId; started = $_.CreationDate } })
		}
		$ids = @($tree | Where-Object { $_.id -ne $process.Id -or -not $process.HasExited } | ForEach-Object { $_.id })
	} catch {}   # no process list: the owned root alone is stopped
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
	}
} catch {
	$verdict = "runner error: $($_.Exception.Message)"
} finally {
	# whatever happened above, the owned DCS does not outlive the runner
	$stopped = Stop-OwnedTree $p $processStarted
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
