# Structural (AST) check of tools/sector-height-baseline-windows.ps1: it parses, it can never terminate a process, a wait that ends while the process runs leaves it alone and writes no completion,
# and every output is created through CreateNew. It parses only; no process is started and nothing is launched. Usage: powershell -NoProfile -File check.ps1 <path-to-wrapper>
param([Parameter(Mandatory = $true)] [string] $Path)
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$errors)
$problems = New-Object System.Collections.Generic.List[string]
foreach ($e in $errors) { $problems.Add("parse error: $($e.Message) at line $($e.Extent.StartLineNumber)") }

$kills = $ast.FindAll({ param($n)
	($n -is [System.Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Member.Value -in @('Kill', 'Close', 'Terminate')) -or
	($n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -in @('Stop-Process', 'taskkill', 'taskkill.exe', 'kill', 'Remove-Item', 'Clear-Content', 'Set-Content', 'Out-File'))
}, $true)
foreach ($k in $kills) { $problems.Add("forbidden call at line $($k.Extent.StartLineNumber): $($k.Extent.Text)") }

# the not-finished branch: an if whose condition is -not $finished must reach `exit 2` and must not create the completion receipt
$branch = $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.IfStatementAst] -and $n.Clauses[0].Item1.Extent.Text -match '-not\s+\$finished' }, $true)
if ($branch.Count -ne 1) { $problems.Add("expected exactly one not-finished branch, found $($branch.Count)") }
else {
	$body = $branch[0].Clauses[0].Item2
	$exits = $body.FindAll({ param($n) $n -is [System.Management.Automation.Language.ExitStatementAst] }, $true)
	if ($exits.Count -ne 1 -or $exits[0].Pipeline.Extent.Text -ne '2') { $problems.Add('the not-finished branch must exit 2 (still running, not a failure)') }
	$writes = $body.FindAll({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'New-OwnedFile' }, $true)
	if ($writes.Count -ne 0) { $problems.Add('the not-finished branch must write no receipt or completion') }
	$throws = $body.FindAll({ param($n) $n -is [System.Management.Automation.Language.ThrowStatementAst] }, $true)
	if ($throws.Count -ne 0) { $problems.Add('the not-finished branch must not throw: a wait that ends is not a failure') }
}

# every file the script creates goes through New-OwnedFile, whose only open is CreateNew
$opens = $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Member.Value -eq 'Open' }, $true)
if ($opens.Count -ne 1 -or $opens[0].Extent.Text -notmatch 'CreateNew') { $problems.Add('the only file open must be CreateNew') }

# the process receipt is written BEFORE the wait
$receiptWrite = $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.CommandAst] -and $n.GetCommandName() -eq 'New-OwnedFile' -and $n.Extent.Text -match 'processReceipt' }, $true)
$wait = $ast.FindAll({ param($n) $n -is [System.Management.Automation.Language.InvokeMemberExpressionAst] -and $n.Member.Value -eq 'WaitForExit' }, $true)
if ($receiptWrite.Count -ne 1 -or $wait.Count -lt 1 -or $receiptWrite[0].Extent.StartOffset -gt $wait[0].Extent.StartOffset) { $problems.Add('the process receipt must be written before the wait') }

if ($problems.Count -gt 0) { $problems | ForEach-Object { Write-Host "FAIL: $_" }; exit 1 }
Write-Host 'AST check passed: parses; no process termination or truncating call; the still-running branch exits 2 without a receipt or a throw; CreateNew only; the process receipt precedes the wait'
