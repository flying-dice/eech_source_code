param(
 [Parameter(Mandatory=$true)][string]$Bin,
 [Parameter(Mandatory=$true)][string]$Root,
 [Parameter(Mandatory=$true)][string]$Out,
 [Parameter(Mandatory=$true)][ValidateSet('timed_never','timed_rate150','bad_health_second_draw','bad_health_fatal')][string]$Mode
)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Focused probe output already exists'}
$wrapper='R:/d2-evidence/combat-characterization-20261028/timed-aa-probe.lua'
foreach($path in @($Bin,$Root,$Out,$wrapper)){if($path -match '[\s&|<>^"]'){throw 'Unsafe native argument path'}}
if(-not(Test-Path -LiteralPath $Root)){throw 'Missing verified retail root'}
$world=Join-Path $Bin 'eech-world.exe'
$caseScript=(Join-Path $Bin 'lua/combat-case.lua') -replace '\\','/'
if(-not(Test-Path -LiteralPath $caseScript)){throw 'Missing original controlled script'}
New-Item -ItemType Directory -Path $Out|Out-Null
$rootArg=$Root -replace '\\','/'
$traceArg=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$summaryArg=(Join-Path $Out 'probe-summary.jsonl') -replace '\\','/'
$argsLine="$wrapper root=$rootArg out=$traceArg warmup=60 frame_ms=100 timed_seconds=20 timed_probe_mode=$Mode case_script=$caseScript probe_summary=$summaryArg"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$world;arguments=$argsLine;root=$Root;out=$Out;mode=$Mode;kind='Focused changed-host AA authority/refusal around unchanged controlled scenario; expected error must match actual reason';wrapper_sha256=(Get-FileHash -LiteralPath $wrapper -Algorithm SHA256).Hash.ToLowerInvariant();case_script_sha256=(Get-FileHash -LiteralPath $caseScript -Algorithm SHA256).Hash.ToLowerInvariant();actual_native_exit=$null}
$native=Start-Process -FilePath $world -ArgumentList $argsLine -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record['status']='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Out 'run.json') -Encoding utf8
Write-Output "TIMED_AA_PROBE_NATIVE_PID=$($native.Id) mode=$Mode"
$native.WaitForExit();$code=$native.ExitCode
$record['status']='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record['actual_native_exit']=$code
$record|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Out 'run.json') -Encoding utf8
Write-Output "TIMED_AA_PROBE_NATIVE_ACTUAL_EXIT=$code mode=$Mode"
exit $code
