param([Parameter(Mandatory=$true)][string]$Bin)
$ErrorActionPreference='Stop'
$base='R:/d2-evidence/combat-characterization-20261028'
$driver=Join-Path $base 'run-forecast-provider.ps1'
if((Get-FileHash -LiteralPath $driver -Algorithm SHA256).Hash.ToLowerInvariant() -ne '7ddf7608ad72bf5fe342ba5da2a5342641c5fc3bd17bc0bf42163c3b8c0dd505'){throw 'Reviewed provider driver changed'}
$queue=Join-Path $base 'forecast-provider-queue.json'
if(Test-Path -LiteralPath $queue){throw 'Queue record exists; inspect before any replay'}
$plan=@(
  @{name='forecast-provider-reference-01';mode='reference'},
  @{name='forecast-provider-zero-01';mode='forecast_zero'},
  @{name='forecast-provider-negative-01';mode='bad_forecast_negative'},
  @{name='forecast-provider-draw-01';mode='bad_forecast_draw'},
  @{name='forecast-provider-fraction-01';mode='bad_forecast_fraction'}
)
$completed=@()
foreach($case in $plan){
  [ordered]@{status='running';updated_utc=[DateTime]::UtcNow.ToString('o');current=$case;seconds_maximum=180;completed=$completed;scope='Sequential native execution only; authority and forecast-specific refusal evidence assessed separately'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
  & $driver -Bin $Bin -Root "$base/roots/$($case.name)" -Out "$base/out/$($case.name)" -Mode $case.mode -Seconds 180
  $code=$LASTEXITCODE
  $completed+=@{name=$case.name;mode=$case.mode;actual_driver_native_exit=$code}
  if($code -ne 0){
    [ordered]@{status='failed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=$code;remaining_not_started=$true}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
    exit $code
  }
}
[ordered]@{status='completed';updated_utc=[DateTime]::UtcNow.ToString('o');completed=$completed;actual_exit=0;scope='All five driver/native exits0; not an authority/refusal acceptance claim'}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath $queue -Encoding utf8
exit 0
