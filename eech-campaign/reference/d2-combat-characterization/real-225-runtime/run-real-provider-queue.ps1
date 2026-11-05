param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][ValidatePattern('^[0-9a-f]{64}$')][string]$ReviewedDriverSha)
$ErrorActionPreference='Stop'
$base='R:/d2-evidence/combat-characterization-20261028'
$driver=Join-Path $base 'run-real-provider.ps1'
if((Get-FileHash -LiteralPath $driver -Algorithm SHA256).Hash.ToLowerInvariant() -ne $ReviewedDriverSha){throw 'Reviewed real provider driver changed'}
$queue=Join-Path $base 'real-provider-queue.json'
if(Test-Path -LiteralPath $queue){throw 'Real provider queue exists; inspect before replay'}
$plan=@(
 @{name='real-provider-reference-01';mode='reference'},
 @{name='real-provider-none-01';mode='damage_none'},
 @{name='real-provider-bad-level-01';mode='bad_damage_level'},
 @{name='real-provider-bad-draw-twice-01';mode='bad_damage_draw_twice'},
 @{name='real-provider-bad-exit-01';mode='bad_damage_exit'},
 @{name='real-provider-bad-class-01';mode='bad_damage_class'},
 @{name='real-provider-bad-unwritten-01';mode='bad_damage_unwritten'},
 @{name='real-provider-bad-missing-01';mode='bad_damage_missing'}
)
foreach($case in $plan){
 $prep=Get-Content -LiteralPath "$base/root-$($case.name)-preparation.json" -Raw|ConvertFrom-Json
 if($prep.input_manifest_actual_exit-ne0){throw 'Provider root preparation was not successful'}
 if(Test-Path -LiteralPath "$base/out/$($case.name)"){throw 'Provider output already exists'}
}
$completed=@()
foreach($case in $plan){
 [ordered]@{status='running';updated_utc=[DateTime]::UtcNow.ToString('o');current=$case;seconds_maximum=180;completed=$completed;scope='Execution only; actual model authority and specific refusals require summary review'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
 & $driver -Bin $Bin -Root "$base/roots/$($case.name)" -Out "$base/out/$($case.name)" -Mode $case.mode -Seconds 180
 $code=$LASTEXITCODE;$completed+=@{name=$case.name;mode=$case.mode;actual_driver_native_exit=$code}
 if($code-ne0){[ordered]@{status='failed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=$code;remaining_not_started=$true}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8;exit $code}
}
[ordered]@{status='completed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=0;scope='All8 driver/native exits0; not evidence of authority/refusal alone'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
exit 0
