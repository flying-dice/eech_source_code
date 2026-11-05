param(
 [Parameter(Mandatory=$true)][string]$Bin,
 [Parameter(Mandatory=$true)][string]$Repo,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ReviewedDriverSha,
 [Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ReviewedPlanSha
)
$ErrorActionPreference='Stop'
$base='R:/d2-evidence/combat-characterization-20261028'
$python='C:/Users/starscream/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe'
$driver=Join-Path $base 'run-timed-provider.ps1'
$planPath=Join-Path $base 'root-timed-provider-corrected-driver-plan.json'
$queue=Join-Path $base 'timed-provider-queue.json'
if((Get-FileHash -LiteralPath $driver -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ReviewedDriverSha){throw 'Reviewed timed driver changed'}
if((Get-FileHash -LiteralPath $planPath -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ReviewedPlanSha){throw 'Reviewed timed plan changed'}
if(Test-Path -LiteralPath $queue){throw 'Timed provider queue already exists; inspect before replay'}
$plan=Get-Content -LiteralPath $planPath -Raw|ConvertFrom-Json
if($plan.seconds -ne 180 -or $plan.cases.Count -ne 11){throw 'Unexpected timed plan configuration'}
if(-not(Test-Path -LiteralPath (Join-Path $Repo 'tools/input-manifest.py'))){throw 'Missing frozen repository input verifier'}
foreach($case in $plan.cases){
 if($case.name -notmatch '^timed-provider-[a-z0-9-]+-01$' -or $case.mode -notmatch '^[a-z_]+$'){throw 'Unsafe provider plan entry'}
 $prep=Get-Content -LiteralPath "$base/root-$($case.name)-preparation.json" -Raw|ConvertFrom-Json
 if($prep.input_manifest_actual_exit -ne 0){throw 'Timed provider input preparation failed'}
 if(Test-Path -LiteralPath "$base/out/$($case.name)"){throw 'Timed provider output already exists'}
}
$completed=@()
foreach($case in $plan.cases){
 if(@(Get-CimInstance Win32_Process | Where-Object Name -eq 'eech-world.exe').Count -ge 7){throw 'Seven native slots already in use; do not restart or exceed capacity'}
 [ordered]@{status='running';updated_utc=[DateTime]::UtcNow.ToString('o');current=$case;seconds_maximum=180;completed=$completed;scope='Execution only; actual host authority/refusal requires reached counters and specific reasons'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
 $prelaunchLog="$base/root-$($case.name)-inputs-prelaunch.log"
 & $python -B -X utf8 (Join-Path $Repo 'tools/input-manifest.py') "$base/roots/$($case.name)" --compare (Join-Path $Repo 'reference/lebanon-3h/inputs.json') *> $prelaunchLog
 $inputCode=$LASTEXITCODE
 if($inputCode -ne 0){throw "Timed provider prelaunch inputs failed: $inputCode"}
 if((Get-FileHash -LiteralPath $driver -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ReviewedDriverSha){throw 'Timed driver changed during queue'}
 & $driver -Bin $Bin -Root "$base/roots/$($case.name)" -Out "$base/out/$($case.name)" -Mode $case.mode -Seconds 180
 $code=$LASTEXITCODE
 $completed+=@{name=$case.name;mode=$case.mode;prelaunch_input_actual_exit=$inputCode;actual_driver_native_exit=$code}
 if($code -ne 0){
  [ordered]@{status='failed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=$code;remaining_not_started=$true}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
  exit $code
 }
}
[ordered]@{status='completed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=0;scope='All11 driver/native exits0; trigger, authority and refusal proof still required'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
exit 0
