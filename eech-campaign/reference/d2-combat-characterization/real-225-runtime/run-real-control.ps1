param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out)
$ErrorActionPreference='Stop'
if (Test-Path -LiteralPath $Out) { throw 'Refusing to overwrite a control output' }
foreach ($path in @($Bin,$Root,$Out)) { if ($path -match '[\s&|<>^"]') { throw "Unsafe native argv path: $path" } }
if (-not (Test-Path -LiteralPath $Root)) { throw 'Missing retail root' }
New-Item -ItemType Directory -Path $Out | Out-Null
$world=Join-Path $Bin 'eech-world.exe'
$lua=(Join-Path $Bin 'lua/campaign.lua') -replace '\\','/'
$rootArg=$Root -replace '\\','/'
$outArg=$Out -replace '\\','/'
$argsLine="$lua root=$rootArg scenario=lebanon_retail hours=0.25 record_every=10 checkpoint_every=3600 observe_every=60 acmi=$outArg/recording.acmi observe=$outArg/observations.jsonl metrics=$outArg/metrics.json observe_supply=0"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;kind='Real-damage candidate physical-output control; comparison with retained original required, not combat census or fullpack acceptance';reference_provenance='Retained ab-base root provenance inferred; no independent original input verification invented';capacity='Lead permits seven native processes on eight physical cores, separated roots; one implementation lane';actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_DAMAGE_CONTROL_NATIVE_PID=$($native.Id)"
$native.WaitForExit()
$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content (Join-Path $Out 'run.json')
Write-Output "REAL_DAMAGE_CONTROL_NATIVE_ACTUAL_EXIT=$code"
exit $code


