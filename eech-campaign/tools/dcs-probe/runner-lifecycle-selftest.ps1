# Exercise the runner's process-tree cleanup (owned-process-tree.ps1, shared with the runner)
# with a fake process inventory and a fake Stop-Process. Nothing here enumerates, starts or
# stops a real process, loads Win32 APIs or starts DCS. Process identity is a PID together
# with its creation time, so the cases include PID reuse by unrelated processes next to real
# survivors.
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'owned-process-tree.ps1')

$started = [datetime]'2026-10-21T08:00:00Z'
$script:inventory = @()
$script:stopped = @()
function Get-CimInstance { param($ClassName) return $script:inventory }
function Stop-Process { param($Id, [switch]$Force, $ErrorAction) $script:stopped += $Id }
function Row($id, $parent, $seconds) {
	[pscustomobject]@{ ProcessId = $id; ParentProcessId = $parent; CreationDate = $started.AddSeconds($seconds) }
}
function Expect($name, [int[]] $expected) {
	$actual = @($script:stopped | Sort-Object)
	if (($actual -join ',') -ne (($expected | Sort-Object) -join ',')) {
		throw "$name`: stopped $($actual -join ',') but expected $(($expected | Sort-Object) -join ',')"
	}
	Write-Output "ok   $name -> $($actual -join ',')"
}
function Cleanup($process, $recorded = @()) {
	$script:stopped = @()
	$null = Stop-OwnedTree $process $started $recorded
}
function ExitedRoot($exitSeconds) {
	if ($null -eq $exitSeconds) { return [pscustomobject]@{ Id = 100; HasExited = $true } }
	[pscustomobject]@{ Id = 100; HasExited = $true; ExitTime = $started.AddSeconds($exitSeconds) }
}
$runningRoot = [pscustomobject]@{ Id = 100; HasExited = $false }

# the root exited: its children created before the exit are stopped, never the root PID nor an older relation
$script:inventory = @((Row 101 100 1), (Row 102 101 2), (Row 99 100 -1))
Cleanup (ExitedRoot 5)
Expect 'exited root: descendants created in its lifetime only' 101, 102

# the root is running: it and its descendants are stopped
Cleanup $runningRoot
Expect 'running root: the whole owned tree' 100, 101, 102

# an unrelated process reused the root PID after the root exited (the review reproduction): the
# replacement's children are not ours, the original's survivors are
$script:inventory = @((Row 100 999 10), (Row 101 100 1), (Row 201 100 11), (Row 102 101 12), (Row 301 201 13))
Cleanup (ExitedRoot 2)
Expect 'reused root PID: replacement and its children excluded, survivors kept' 101, 102

# an exited root with no known exit time proves no lifetime, so nothing is discovered through its PID
Cleanup (ExitedRoot $null)
Expect 'exited root without an exit time: no discovery through its PID' @()

# a running root whose PID cannot have been reused: its late children are still its own
$script:inventory = @((Row 101 100 1), (Row 102 101 700))
Cleanup $runningRoot
Expect 'running root keeps children created late in the run' 100, 101, 102

# recording while the root runs keeps the tree reachable after the root exits with no exit time
$script:inventory = @((Row 101 100 1), (Row 102 101 12), (Row 99 100 -1))
$recorded = @(Get-OwnedInstances $runningRoot $started)
if ((@($recorded | ForEach-Object { $_.id } | Sort-Object) -join ',') -ne '101,102') { throw 'recording did not find 101 and 102' }
Write-Output 'ok   recording: finds 101,102 and stops nothing'
Cleanup (ExitedRoot $null) $recorded
Expect 'recorded descendants outlive a root with no exit time' 101, 102

# the intermediate 101 exited and an unrelated process reused PID 101 later, with its own child.
# 102 survived the original 101 and was recorded, so it is stopped. The reuser and its child are not.
$script:inventory = @((Row 102 101 12), (Row 101 555 20), (Row 301 101 21))
Cleanup (ExitedRoot 30) $recorded
Expect 'exited intermediate: recorded survivor kept, PID reuse and its child excluded' 102

# a recorded descendant whose PID was reused is never stopped by PID alone
$script:inventory = @((Row 101 555 20))
Cleanup (ExitedRoot 30) $recorded
Expect 'recorded PID reused by an unrelated process: nothing stopped' @()

# the gap that remained: a recorded child exits, an unrelated process reuses its PID, starts
# child 301 and exits. 301 keeps ParentProcessId 101 but its parent instance is not the recorded
# one, so it is not adopted. The running root is still stopped.
$script:inventory = @((Row 301 101 21), (Row 102 101 12))
Cleanup $runningRoot @(@{ id = 101; started = $started.AddSeconds(1) })
Expect 'recorded child gone, PID reused and its child orphaned: not adopted' 100

# a recorded child that is still the same instance does pass its own children on
$script:inventory = @((Row 101 100 1), (Row 301 101 21))
Cleanup $runningRoot @(@{ id = 101; started = $started.AddSeconds(1) })
Expect 'recorded child still alive: its new child is adopted' 100, 101, 301

# no process list: a live root alone is stopped, an exited root is not
function Get-CimInstance { param($ClassName) throw 'no process list' }
Cleanup $runningRoot
Expect 'no process list: the live root alone' 100
Cleanup (ExitedRoot 2)
Expect 'no process list: an exited root is not touched' @()

Write-Output 'runner lifecycle cleanup passed with exited and running fake roots and reused PIDs'
