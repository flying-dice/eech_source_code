# Exercise the runner's process-tree cleanup with fake process inventory and
# Stop-Process, without loading Win32 APIs or starting DCS.
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
$script:inventory = @(
	[pscustomobject]@{ ProcessId = 101; ParentProcessId = 100; CreationDate = $started.AddSeconds(1) },
	[pscustomobject]@{ ProcessId = 102; ParentProcessId = 101; CreationDate = $started.AddSeconds(2) },
	[pscustomobject]@{ ProcessId = 99; ParentProcessId = 100; CreationDate = $started.AddSeconds(-1) }
)
$script:stopped = @()
function Get-CimInstance { param($ClassName) return $script:inventory }
function Stop-Process { param($Id, [switch]$Force, $ErrorAction) $script:stopped += $Id }

$exited = [pscustomobject]@{ Id = 100; HasExited = $true }
$null = Stop-OwnedTree $exited $started
if ($script:stopped.Count -ne 2 -or 101 -notin $script:stopped -or 102 -notin $script:stopped -or
	100 -in $script:stopped -or 99 -in $script:stopped) {
	throw "exited root cleanup did not stop exactly its newer descendants: $($script:stopped -join ',')"
}

$script:stopped = @()
$running = [pscustomobject]@{ Id = 100; HasExited = $false }
$null = Stop-OwnedTree $running $started
if ($script:stopped.Count -ne 3 -or 100 -notin $script:stopped -or 101 -notin $script:stopped -or
	102 -notin $script:stopped -or 99 -in $script:stopped) {
	throw "running root cleanup did not stop exactly the owned tree: $($script:stopped -join ',')"
}

$calls = @($ast.FindAll({ param($node) $node -is [System.Management.Automation.Language.CommandAst] -and
	$node.GetCommandName() -eq 'Stop-OwnedTree' }, $true))
if ($calls.Count -ne 1) { throw 'expected one runner cleanup call' }
$parent = $calls[0].Parent
while ($parent -and $parent -isnot [System.Management.Automation.Language.TryStatementAst]) {
	if ($parent -is [System.Management.Automation.Language.IfStatementAst]) {
		throw 'cleanup call is conditional'
	}
	$parent = $parent.Parent
}
if (-not $parent -or -not $parent.Finally -or
	$calls[0].Extent.StartOffset -lt $parent.Finally.Extent.StartOffset -or
	$calls[0].Extent.EndOffset -gt $parent.Finally.Extent.EndOffset) {
	throw 'cleanup call is outside finally'
}
Write-Output 'runner lifecycle cleanup passed with exited and running fake roots'
