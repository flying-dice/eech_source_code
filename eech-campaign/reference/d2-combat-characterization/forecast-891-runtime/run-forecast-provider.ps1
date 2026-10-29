param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out,[Parameter(Mandatory=$true)][ValidateSet('reference','forecast_zero','bad_forecast_negative','bad_forecast_draw','bad_forecast_fraction')][string]$Mode,[ValidateRange(1,600)][int]$Seconds=180)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Refusing to overwrite a provider output'}
foreach($path in @($Bin,$Root,$Out)){if($path -match '[\s&|<>^"]'){throw "Unsafe native argv path: $path"}}
if(-not(Test-Path -LiteralPath $Root)){throw 'Missing verified retail root'}
New-Item -ItemType Directory -Path $Out|Out-Null
$world=Join-Path $Bin 'eech-world.exe'
$lua=(Join-Path $Bin 'lua/aim-provider.lua') -replace '\\','/'
$rootArg=$Root -replace '\\','/'
$summaryArg=(Join-Path $Out 'summary.jsonl') -replace '\\','/'
$argsLine="$lua root=$rootArg scenario=lebanon_retail seconds=$Seconds frame_ms=100 mode=$Mode out=$summaryArg"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;mode=$Mode;kind='Forecast candidate public-path provider; positive authority or expected refusal must be judged from actual summary, not native exit alone';actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "FORECAST_PROVIDER_NATIVE_PID=$($native.Id) mode=$Mode"
$native.WaitForExit()
$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "FORECAST_PROVIDER_NATIVE_ACTUAL_EXIT=$code mode=$Mode"
exit $code
