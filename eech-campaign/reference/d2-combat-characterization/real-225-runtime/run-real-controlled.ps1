param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out,[string]$Kinds="",[string]$Control="",[ValidateRange(0,60)][int]$TimedSeconds=20)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Controlled witness output already exists; refusing overwrite'}
foreach($path in @($Bin,$Root,$Out)){if($path -match '[\s&|<>^"]'){throw "Unsafe native argv path: $path"}}
if(-not(Test-Path -LiteralPath $Root)){throw 'Missing verified retail root'}
New-Item -ItemType Directory -Path $Out|Out-Null
$world=Join-Path $Bin 'eech-world.exe'
$lua=(Join-Path $Bin 'lua/combat-case.lua') -replace '\\','/'
$rootArg=$Root -replace '\\','/'
$traceArg=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$argsLine="$lua root=$rootArg out=$traceArg warmup=60 frame_ms=100 timed_seconds=$TimedSeconds"
foreach($selector in @($Kinds,$Control)){if($selector -and $selector -notmatch "^[A-Z_,]+$"){throw "Unsafe fixture selector"}}
if($Kinds){$argsLine+=" kinds=$Kinds"}
if($Control){$argsLine+=" control=$Control"}
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;kind='Real-damage candidate controlled combat witness; precondition done is not physical-path coverage; not extraction/fullpack acceptance';actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_CONTROLLED_NATIVE_PID=$($native.Id)"
$native.WaitForExit()
$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_CONTROLLED_NATIVE_ACTUAL_EXIT=$code"
exit $code

