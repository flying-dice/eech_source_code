param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out,[int]$Seconds=600)
$ErrorActionPreference='Stop'
if (Test-Path -LiteralPath $Out) { throw 'Refusing to overwrite an existing witness output' }
foreach ($path in @($Bin,$Root,$Out)) { if ($path -match '[\s&|<>^"]') { throw "Unsafe native argv path: $path" } }
if (-not (Test-Path -LiteralPath $Root)) { throw 'Missing verified retail root' }
New-Item -ItemType Directory -Path $Out | Out-Null
$world=Join-Path $Bin 'eech-world.exe'
$lua=(Join-Path $Bin 'lua/combat-characterization.lua') -replace '\\','/'
$rootArg=$Root -replace '\\','/'
$traceArg=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$argsLine="$lua root=$rootArg scenario=lebanon_retail seconds=$Seconds frame_ms=100 checkpoint_every=60 out=$traceArg"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;kind='Real-damage candidate combat witness; comparison with retained original required, not fullpack acceptance';actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_DAMAGE_NATIVE_PID=$($native.Id)"
$native.WaitForExit()
$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_DAMAGE_NATIVE_ACTUAL_EXIT=$code"
exit $code


