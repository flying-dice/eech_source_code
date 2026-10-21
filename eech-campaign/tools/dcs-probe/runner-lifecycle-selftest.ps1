# Exercise the runner's process-tree cleanup with a fake process inventory and a fake
# Stop-Process. Nothing here enumerates, starts or stops a real process, loads Win32 APIs
# or starts DCS. Process identity is a PID together with its creation time, so the cases
# include PID reuse by unrelated processes next to real survivors.
$ErrorActionPreference = 'Stop'
$path = Join-Path $PSScriptRoot 'run-dcs-probe.ps1'
$tokens = $null
$errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($path, [ref]$tokens, [ref]$errors)
if ($errors.Count -ne 0) { throw 'runner has PowerShell parser errors' }
$definition = $ast.Find({ param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and
	$node.Name -eq 'Stop-OwnedTree' }, $true)
if (-not $definition) { throw 'Stop-OwnedTree function not found' }
. ([scriptblock]::Create($definition.Extent.Text))

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
function Cleanup($process, $seen = @()) {
	$script:stopped = @()
	$null = Stop-OwnedTree $process $started $seen
}
function ExitedRoot($exitSeconds) {
	if ($null -eq $exitSeconds) { return [pscustomobject]@{ Id = 100; HasExited = $true } }
	[pscustomobject]@{ Id = 100; HasExited = $true; ExitTime = $started.AddSeconds($exitSeconds) }
}
$runningRoot = [pscustomobject]@{ Id = 100; HasExited = $false }

# the root exited: only its newer descendants are stopped, never the root PID nor an older relation
$script:inventory = @((Row 101 100 1), (Row 102 101 2), (Row 99 100 -1))
Cleanup (ExitedRoot $null)
Expect 'exited root: newer descendants only' 101, 102

# the root is running: it and its descendants are stopped
Cleanup $runningRoot
Expect 'running root: the whole owned tree' 100, 101, 102

# an unrelated process reused the root PID after the root exited (same fixture as the review
# reproduction): the replacement's children are not ours, the original's survivors are
$script:inventory = @((Row 100 999 10), (Row 101 100 1), (Row 201 100 11), (Row 102 101 12), (Row 301 201 13))
Cleanup (ExitedRoot 2)
Expect 'reused root PID: replacement and its children excluded, survivors kept' 101, 102

# the same when the exit time is unavailable: the replacement's creation time bounds the old root
Cleanup (ExitedRoot $null)
Expect 'reused root PID without an exit time: bounded by the replacement' 101, 102

# a running root whose PID cannot have been reused: its late children are still its own
$script:inventory = @((Row 101 100 1), (Row 102 101 700))
Cleanup $runningRoot
Expect 'running root keeps children created late in the run' 100, 101, 102

# ObserveOnly records descendants and stops nothing
$script:inventory = @((Row 101 100 1), (Row 102 101 12), (Row 99 100 -1))
$script:stopped = @()
$seen = Stop-OwnedTree $runningRoot $started @() -ObserveOnly
if ($script:stopped.Count -ne 0) { throw 'ObserveOnly stopped a process' }
if ((@($seen | ForEach-Object { $_.id } | Sort-Object) -join ',') -ne '101,102') { throw 'ObserveOnly did not record 101 and 102' }
Write-Output 'ok   observe only: records 101,102, stops nothing'

# the intermediate 101 exited and an unrelated process reused PID 101 later, with its own child.
# 102 survived the original 101, so it is stopped through the recorded instance. The reusing
# process and its child are not.
$script:inventory = @((Row 102 101 12), (Row 101 555 20), (Row 301 101 21))
Cleanup (ExitedRoot 30) $seen
Expect 'exited intermediate: survivor kept, PID reuse and its child excluded' 102

# a recorded descendant whose PID was reused is never stopped by PID alone
$script:inventory = @((Row 101 555 20))
Cleanup (ExitedRoot 30) $seen
Expect 'recorded PID reused by an unrelated process: nothing stopped' @()

# no process list: a live root alone is stopped, an exited root is not
function Get-CimInstance { param($ClassName) throw 'no process list' }
Cleanup $runningRoot
Expect 'no process list: the live root alone' 100
Cleanup (ExitedRoot 2)
Expect 'no process list: an exited root is not touched' @()

# structure: exactly one final cleanup (the non-observing call), unconditional and in finally
$calls = @($ast.FindAll({ param($node) $node -is [System.Management.Automation.Language.CommandAst] -and
	$node.GetCommandName() -eq 'Stop-OwnedTree' }, $true))
$final = @($calls | Where-Object { -not ($_.CommandElements | Where-Object {
	$_ -is [System.Management.Automation.Language.CommandParameterAst] -and $_.ParameterName -eq 'ObserveOnly' }) })
if ($final.Count -ne 1) { throw "expected one runner cleanup call, found $($final.Count)" }
$parent = $final[0].Parent
while ($parent -and $parent -isnot [System.Management.Automation.Language.TryStatementAst]) {
	if ($parent -is [System.Management.Automation.Language.IfStatementAst]) {
		throw 'cleanup call is conditional'
	}
	$parent = $parent.Parent
}
if (-not $parent -or -not $parent.Finally -or
	$final[0].Extent.StartOffset -lt $parent.Finally.Extent.StartOffset -or
	$final[0].Extent.EndOffset -gt $parent.Finally.Extent.EndOffset) {
	throw 'cleanup call is outside finally'
}
Write-Output 'runner lifecycle cleanup passed with exited and running fake roots and reused PIDs'
