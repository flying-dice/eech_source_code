# Georgia 600 s witness with the corrected aim fixture build f9f15f52f, started only after the Lebanon sequence (run-final2.ps1) has finished:
# one new native scenario process at a time in this lane.
$ErrorActionPreference = 'Stop'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-characterization-20261026'
$Bin = 'R:\eech_source_code\eech-campaign\target\aim-fixture\f9f15f52f'
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\run-georgia.log" -Append }
while (-not (Select-String -Path "$base\run-final.log" -Pattern 'ALL DONE' -Quiet)) { Start-Sleep -Seconds 20 }
Stamp 'start 600 s Georgia (georgia_retail) witness f-g'
$o = & 'R:\eech_source_code\eech-campaign\tools\aim-characterization-windows.ps1' -Bin $Bin -Root "$base\roots\f-g" -Out "$base\out\f-g" -Scenario georgia_retail -Seconds 600
Stamp "end f-g: script exit=$LASTEXITCODE; $o"
