# Runs one bounded DCS probe stage in an isolated Saved Games profile and
# collects its evidence. It starts its own DCS process (hidden), never touches
# a pre-existing one, and stops only the process tree it started.
#
#   pwsh -File run-dcs-probe.ps1 -Stage load -Candidate <dir with eech_dc.dll> [-Root <EECH root copy>] [-CancelLogin]
#
# DCS is started as bin-mt\DCS.exe -w <profile> --no-launcher --mission <probe miz>;
# --mission is the route observed to load the mission (the GUI hook cannot start it).
# Refuses to start while any DCS.exe is running. Aborts if the default
# profile's dcs.log changes (which would mean -w was not honoured). The verdict
# is the process outcome only; what the probe reached is read from probe.log.
param(
	[Parameter(Mandatory)] [ValidateSet('env', 'load', 'boot', 'frames')] [string] $Stage,
	[Parameter(Mandatory)] [string] $Candidate,
	[string] $Root = 'E:\eech-runs\dcs-probe\lebanon-root',
	[string] $Dcs = 'D:\Program Files\Eagle Dynamics\DCS World OpenBeta',
	[string] $WriteDirName = 'DCS.M5Probe',
	[string] $Work = 'E:\eech-runs\dcs-probe',
	[string] $Template = 'CoreMods\WWII Units\FW-190D-9\Missions\fw190.miz',
	[int] $RunSeconds = 20,
	[int] $TimeoutSeconds = 420,
	# A profile without saved credentials waits at a native 'DCS Login' dialog. This clicks
	# Cancel on it, then OK on 'Authorization cancelled', in the owned process only; DCS then
	# runs unauthorised (paid modules are skipped, free content loads)
	[switch] $CancelLogin,
	# boot/frames run the campaign in DCS; refused unless the candidate's native error path
	# has passed the DCS-Lua controls (README: "Before boot or frames")
	[string] $ErrorControlsEvidence
)
$ErrorActionPreference = 'Stop'
if ($Stage -in 'boot', 'frames' -and -not ($ErrorControlsEvidence -and (Test-Path $ErrorControlsEvidence))) {
	throw "stage $Stage needs -ErrorControlsEvidence <passing DCS-Lua error-control record for this candidate>"
}
$here = $PSScriptRoot
$runId = "{0}-{1}" -f (Get-Date -Format 'yyyyMMdd-HHmmss'), $Stage
$out = Join-Path $Work "runs\$runId"
New-Item -ItemType Directory -Force $out, "$Work\bin", "$Work\missions" | Out-Null
$saved = Join-Path $env:USERPROFILE 'Saved Games'
$writeDir = Join-Path $saved $WriteDirName
$defaultLog = Join-Path $saved 'DCS.openbeta\Logs\dcs.log'
$manifest = [ordered]@{ run = $runId; stage = $Stage; started = (Get-Date).ToString('o') }

function Hash($p) { (Get-FileHash -Algorithm SHA256 $p).Hash.ToLower() }

# top-level dialogs of one process: title and static texts (edit-box contents are never read)
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
	public static List<string> Dialogs (uint pid, bool cancelLogin) {
		var found = new List<string> ();
		EnumWindows ((h, l) => {
			uint owner; GetWindowThreadProcessId (h, out owner);
			if (owner != pid || Class (h) != "#32770") return true;
			var parts = new List<string> { Text (h) };
			EnumChildWindows (h, (c, l2) => { var k = Class (c); if (k == "Static" || k == "Button") { var t = Text (c); if (t != "") parts.Add (t); } return true; }, IntPtr.Zero);
			if (cancelLogin && Text (h) == "DCS Login") { SendMessage (GetDlgItem (h, 2), 0x00F5, IntPtr.Zero, IntPtr.Zero); parts.Add ("[Cancel clicked]"); }
			else if (cancelLogin && Text (h) == "Authorization cancelled") { SendMessage (GetDlgItem (h, 1), 0x00F5, IntPtr.Zero, IntPtr.Zero); parts.Add ("[OK clicked]"); }
			found.Add (string.Join (" | ", parts));
			return true;
		}, IntPtr.Zero);
		return found;
	}
}
'@

if (Get-Process -Name DCS -ErrorAction SilentlyContinue) { throw 'a DCS process is already running: not starting another' }

# inputs: the DLL only (never the candidate's own lua.dll), the probe, the hook, the mission
Copy-Item (Join-Path $Candidate 'eech_dc.dll') "$Work\bin\eech_dc.dll" -Force
Copy-Item "$here\eech-dcs-probe.lua" "$Work\eech-dcs-probe.lua" -Force
$miz = "$Work\missions\probe-$Stage.miz"
$template = Join-Path $Dcs $Template
& python -I "$here\build-probe-miz.py" $template $miz $Stage "$Work\eech-dcs-probe.lua" "$Work\bin" $Root "$out\probe.log"
if ($LASTEXITCODE -ne 0) { throw 'mission build failed' }
New-Item -ItemType Directory -Force "$writeDir\Scripts\Hooks" | Out-Null
Copy-Item "$here\eech-m5probe-hook.lua" "$writeDir\Scripts\Hooks\eech-m5probe-hook.lua" -Force
$manifest.inputs = [ordered]@{
	candidate_eech_dc = @{ path = (Join-Path $Candidate 'eech_dc.dll'); sha256 = Hash "$Work\bin\eech_dc.dll" }
	probe = @{ sha256 = Hash "$Work\eech-dcs-probe.lua" }
	hook = @{ sha256 = Hash "$writeDir\Scripts\Hooks\eech-m5probe-hook.lua" }
	template = @{ path = $Template; sha256 = Hash $template }
	mission = @{ path = $miz; sha256 = Hash $miz }
	dcs_exe = @{ path = "$Dcs\bin-mt\DCS.exe"; sha256 = Hash "$Dcs\bin-mt\DCS.exe" }
	dcs_lua = @{ sha256 = Hash "$Dcs\bin-mt\lua.dll" }
	mission_scripting = @{ sha256 = Hash "$Dcs\Scripts\MissionScripting.lua" }
	autoupdate_version = (Get-Content "$Dcs\autoupdate.cfg" -Raw | ConvertFrom-Json).version
}
$defaultBefore = if (Test-Path $defaultLog) { (Get-Item $defaultLog).LastWriteTimeUtc.Ticks } else { 0 }
if (Test-Path "$writeDir\Logs\dcs.log") { Remove-Item "$writeDir\Logs\dcs.log" }

$env:EECH_M5PROBE_RUN_SECONDS = "$RunSeconds"
$arguments = @('-w', $WriteDirName, '--no-launcher', '--mission', "`"$miz`"")
$manifest.cancel_login = [bool]$CancelLogin
if ($ErrorControlsEvidence) { $manifest.error_controls_evidence = @{ path = $ErrorControlsEvidence; sha256 = Hash $ErrorControlsEvidence } }
$manifest.command = "Start-Process `"$Dcs\bin-mt\DCS.exe`" -ArgumentList $($arguments -join ' ') -WindowStyle Hidden -PassThru"
$p = Start-Process -FilePath "$Dcs\bin-mt\DCS.exe" -ArgumentList $arguments -WorkingDirectory "$Dcs\bin-mt" -WindowStyle Hidden -PassThru
$manifest.pid = $p.Id
$t0 = Get-Date
$verdict = 'timeout'
$dialogs = [ordered]@{}
while ($true) {
	Start-Sleep -Seconds 2
	$elapsed = ((Get-Date) - $t0).TotalSeconds
	if (-not $p.HasExited) {
		foreach ($d in [ProbeWindows]::Dialogs([uint32]$p.Id, [bool]$CancelLogin)) { if (-not $dialogs.Contains($d)) { $dialogs[$d] = [Math]::Round($elapsed) } }
	}
	$defaultNow = if (Test-Path $defaultLog) { (Get-Item $defaultLog).LastWriteTimeUtc.Ticks } else { 0 }
	if ($defaultNow -ne $defaultBefore) { $verdict = 'aborted: the default profile log changed'; break }
	if ($p.HasExited) { $verdict = 'exited with code {0} (0x{0:X8})' -f $p.ExitCode; break }
	if ($elapsed -gt $TimeoutSeconds) { break }
}
$manifest.elapsed_seconds = [Math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
$stopped = @()
if (-not $p.HasExited) {
	# the owned process tree only: DCS.exe started above and its descendants
	$tree = @($p.Id)
	$all = Get-CimInstance Win32_Process
	for ($i = 0; $i -lt $tree.Count; $i++) { $tree += @($all | Where-Object { $_.ParentProcessId -eq $tree[$i] } | ForEach-Object { $_.ProcessId }) }
	foreach ($id in ($tree | Select-Object -Unique)) { try { Stop-Process -Id $id -Force -ErrorAction Stop; $stopped += $id } catch {} }
}
$manifest.verdict = $verdict
$manifest.dialogs = @($dialogs.Keys | ForEach-Object { @{ seconds = $dialogs[$_]; text = $_ } })
$manifest.stopped_pids = $stopped
Start-Sleep -Seconds 3
if (Test-Path "$writeDir\Logs") {
	Get-ChildItem "$writeDir\Logs" -File | Where-Object { $_.LastWriteTime -ge $t0.AddSeconds(-5) } | Copy-Item -Destination $out
}
$manifest.outputs = @(Get-ChildItem $out -File | ForEach-Object { @{ name = $_.Name; bytes = $_.Length; sha256 = Hash $_.FullName } })
$manifest.default_profile_log_unchanged = ($defaultBefore -eq $(if (Test-Path $defaultLog) { (Get-Item $defaultLog).LastWriteTimeUtc.Ticks } else { 0 }))
$manifest.finished = (Get-Date).ToString('o')
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding utf8 "$out\manifest.json"
Write-Output "run $runId -> $out ($verdict)"
