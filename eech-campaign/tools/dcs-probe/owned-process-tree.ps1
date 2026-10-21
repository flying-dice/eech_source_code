# Owned-process-tree helpers, dot-sourced by run-dcs-probe.ps1 and runner-lifecycle-selftest.ps1.
#
# A process instance is its PID together with its creation time: Windows reuses PIDs and
# keeps a dead parent's PID in ParentProcessId, so a PID alone never identifies a process.
# An instance is adopted as owned only on evidence that it descends from the started root:
#   - it is a child of an owned instance that is alive now (same PID and creation time), and
#     was created no earlier than that parent; or
#   - it is a child of the root created within the root's lifetime, when the root has exited
#     and its exit time is known (the upper bound is the exit time, or the creation of a
#     different process now holding the root PID, whichever is first); or
#   - it was recorded earlier, while its parent was alive, and is still the same instance.
# A PID whose recorded instance is gone is never a parent: an unrelated process that reused
# it, and that process's children, are not adopted. The limit is a child of a non-root
# instance that was created and orphaned between two recordings.

# Pure: no process is read, started or stopped. $inventory rows carry ProcessId,
# ParentProcessId and CreationDate. Returns the owned descendants alive in the inventory as
# @{ id; started } instances (the root itself is not included).
function Find-OwnedInstances($rootId, $rootStarted, $rootExited, $rootEnded, $inventory, $recorded = @()) {
	$tolerance = [TimeSpan]::FromMilliseconds(500)   # Process.StartTime and WMI CreationDate round differently
	$rows = @($inventory | Where-Object { $null -ne $_ -and $null -ne $_.CreationDate })   # a row without a creation time proves nothing
	$owned = [System.Collections.Generic.List[object]]::new()
	$parents = [System.Collections.Generic.List[object]]::new()
	if (-not $rootExited) {
		$parents.Add(@{ id = [int]$rootId; started = $rootStarted; ended = $null })
	} elseif ($null -ne $rootEnded) {
		$ended = $rootEnded
		foreach ($row in $rows) {
			if ($row.ProcessId -eq $rootId -and $row.CreationDate -gt ($rootStarted + $tolerance) -and $row.CreationDate -lt $ended) {
				$ended = $row.CreationDate
			}
		}
		$parents.Add(@{ id = [int]$rootId; started = $rootStarted; ended = $ended })
	}
	function Own($row) {
		foreach ($instance in $owned) { if ($instance.id -eq $row.ProcessId -and $instance.started -eq $row.CreationDate) { return $false } }
		$instance = @{ id = [int]$row.ProcessId; started = $row.CreationDate }
		$owned.Add($instance)
		$parents.Add(@{ id = $instance.id; started = $instance.started; ended = $null })
		return $true
	}
	foreach ($old in @($recorded)) {
		foreach ($row in $rows) {
			if ($row.ProcessId -eq $old.id -and $row.CreationDate -eq $old.started) { $null = Own $row }
		}
	}
	for ($i = 0; $i -lt $parents.Count; $i++) {
		$parent = $parents[$i]
		foreach ($row in $rows) {
			if ($row.ParentProcessId -eq $parent.id -and $row.CreationDate -ge $parent.started -and
				($null -eq $parent.ended -or $row.CreationDate -le $parent.ended)) { $null = Own $row }
		}
	}
	return $owned.ToArray()
}

# Reads the process list and returns the owned descendants now alive (see above). The caller
# keeps the result as $recorded for the next call.
function Get-OwnedInstances($process, $started, $recorded = @(), $exited = $null) {
	if ($null -eq $exited) { $exited = [bool]$process.HasExited }
	$ended = $null
	if ($exited) { try { $ended = $process.ExitTime } catch {} }
	return @(Find-OwnedInstances $process.Id $started $exited $ended @(Get-CimInstance Win32_Process) $recorded)
}

# Stops the started process (if still running) and its owned descendants, and returns the
# stopped PIDs. Without a process list the live root alone is stopped.
function Stop-OwnedTree($process, $started, $recorded = @()) {
	if ($null -eq $process) { return , @() }   # nothing was started
	$exited = [bool]$process.HasExited   # read once: the root and its descendants are judged on the same state
	$ids = @(if (-not $exited) { [int]$process.Id })
	try { $ids += @(Get-OwnedInstances $process $started $recorded $exited | ForEach-Object { $_.id }) } catch {}
	$stopped = @()
	foreach ($id in $ids) { try { Stop-Process -Id $id -Force -ErrorAction Stop; $stopped += $id } catch {} }
	return , $stopped
}
