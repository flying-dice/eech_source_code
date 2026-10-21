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
function RowMs($id, $parent, $milliseconds) {
	[pscustomobject]@{ ProcessId = $id; ParentProcessId = $parent; CreationDate = $started.AddMilliseconds($milliseconds) }
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

# an orphaned grandchild: the intermediate 101 is gone. Unrecorded, nothing proves 102 descends from
# the root, so it is not adopted; recorded while 101 was alive, it is.
$script:inventory = @((Row 102 101 2))
Cleanup (ExitedRoot 5)
Expect 'orphaned grandchild, never recorded: not adopted' @()
Cleanup (ExitedRoot 5) @(@{ id = 101; started = $started.AddSeconds(1) })
Expect 'orphaned grandchild of a recorded, now gone, parent: not adopted either' @()
Cleanup (ExitedRoot 5) @(@{ id = 102; started = $started.AddSeconds(2) })
Expect 'orphaned grandchild recorded itself: adopted' 102

# a child created at the root's start counts, one created 1 ms before it does not
$script:inventory = @((RowMs 101 100 0), (RowMs 102 100 -1))
Cleanup (ExitedRoot 5)
Expect 'child created at the root start / 1 ms before it' 101

# a child created after the root exited cannot be its child
$script:inventory = @((Row 101 100 1), (Row 102 100 5.5))
Cleanup (ExitedRoot 5)
Expect 'child created after the root exited: not adopted' 101

# duplicate inventory rows yield one stop per instance
$script:inventory = @((Row 101 100 1), (Row 101 100 1), (Row 102 101 2), (Row 102 101 2))
Cleanup $runningRoot
Expect 'duplicate inventory rows: each instance stopped once' 100, 101, 102

# a parent cycle among unrelated processes terminates and adopts none of them
$script:inventory = @((Row 101 100 1), (Row 201 202 3), (Row 202 201 4))
Cleanup $runningRoot
Expect 'unrelated parent cycle: terminates, not adopted' 100, 101

# a row without a creation time proves nothing, and does not hide the real children
$script:inventory = @(([pscustomobject]@{ ProcessId = 301; ParentProcessId = 100; CreationDate = $null }), (Row 101 100 1))
Cleanup $runningRoot
Expect 'null CreationDate row ignored, real child kept' 100, 101

# the root exits between two reads: the state is read once, so the root and its late child are
# judged together (a mixed read would stop the root yet drop its late child)
$script:inventory = @((Row 101 100 1), (Row 103 100 700))
$global:flipReads = 0
$flipping = [pscustomobject]@{ Id = 100; ExitTime = $started.AddSeconds(5) }
$flipping | Add-Member -MemberType ScriptProperty -Name HasExited -Value { $global:flipReads++; $global:flipReads -gt 1 }
Cleanup $flipping
Expect 'root exiting between two reads: one consistent state' 100, 101, 103

# nothing was started (Start-Process failed): nothing to stop, no error
$script:stopped = @()
$null = Stop-OwnedTree $null $null
Expect 'no process was started: nothing stopped' @()

# no process list: a live root alone is stopped, an exited root is not
function Get-CimInstance { param($ClassName) throw 'no process list' }
Cleanup $runningRoot
Expect 'no process list: the live root alone' 100
Cleanup (ExitedRoot 2)
Expect 'no process list: an exited root is not touched' @()

# Safety property, not layout: the runner's only Stop-OwnedTree call is in a finally block that
# always runs, and the try it belongs to also covers Start-Process and the run-seconds bound, so
# a failure after DCS starts cannot skip the cleanup.
$ast = [System.Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'run-dcs-probe.ps1'), [ref]$null, [ref]$null)
$calls = @($ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Stop-OwnedTree' }, $true))
if ($calls.Count -ne 1) { throw "expected one runner Stop-OwnedTree call, found $($calls.Count)" }
$node = $calls[0].Parent
$try = $null
while ($node) {
	if ($node -is [System.Management.Automation.Language.TryStatementAst] -and $node.Finally -and
		$calls[0].Extent.StartOffset -ge $node.Finally.Extent.StartOffset -and $calls[0].Extent.EndOffset -le $node.Finally.Extent.EndOffset) { $try = $node; break }
	$node = $node.Parent
}
if (-not $try) { throw 'the runner cleanup call is not inside a finally block' }
$start = @($ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'Start-Process' }, $true))
$bound = @($ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.AssignmentStatementAst] -and $n.Left.Extent.Text -eq '$env:EECH_M5PROBE_RUN_SECONDS' }, $true))
foreach ($setup in $start + $bound) {
	if ($setup.Extent.StartOffset -lt $try.Body.Extent.StartOffset -or $setup.Extent.EndOffset -gt $try.Body.Extent.EndOffset) {
		throw "setup outside the try that owns the cleanup: $($setup.Extent.Text)"
	}
}
if ($start.Count -ne 1 -or $bound.Count -ne 1) { throw 'expected one Start-Process and one run-seconds assignment in the runner' }
Write-Output 'ok   structure: Stop-OwnedTree sits in the finally of the try that holds Start-Process and the run bound'

Write-Output 'runner lifecycle cleanup passed with exited and running fake roots and reused PIDs'
