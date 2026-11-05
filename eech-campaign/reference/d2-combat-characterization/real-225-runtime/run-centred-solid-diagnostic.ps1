param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Refusing to overwrite a centred-solid diagnostic output'}
$lua='R:/d2-evidence/combat-characterization-20261028/centred-solid-diagnostic/centred-solid-case.lua'
foreach($path in @($Bin,$Root,$Out,$lua)){if($path -match '[\s&|<>^"]'){throw "Unsafe native argv path: $path"}}
if(-not(Test-Path -LiteralPath $Root)){throw 'Missing verified fresh retail root'}
New-Item -ItemType Directory -Path $Out|Out-Null
$world=Join-Path $Bin 'eech-world.exe'
$rootArg=$Root -replace '\\','/'
$traceArg=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$argsLine="$lua root=$rootArg out=$traceArg warmup=60 frame_ms=100 timed_seconds=0 kinds=ROUTED_VEHICLE control=OBJECT"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;kind='NEW centred-solid diagnostic preconditions, not baseline campaign; direct HIT_TARGET at zero separation should preserve original no-damage result and exact diagnostic bits';actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "CENTRED_SOLID_NATIVE_PID=$($native.Id)"
$native.WaitForExit()
$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "CENTRED_SOLID_NATIVE_ACTUAL_EXIT=$code"
exit $code
