param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out,[Parameter(Mandatory=$true)][string]$Script)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Refusing existing frame-cost output'}
foreach($path in @($Bin,$Root,$Out,$Script)){if($path -match '[\s&|<>^"]'){throw "Unsafe native argv path: $path"}}
foreach($path in @($Root,$Script,(Join-Path $Bin 'eech-world.exe'),(Join-Path $Bin 'BUILD-INFO.txt'))){if(-not(Test-Path -LiteralPath $path)){throw "Missing required path $path"}}
New-Item -ItemType Directory -Path $Out | Out-Null
$nativePath=Join-Path $Bin 'eech-world.exe'
$outArg=$Out -replace '\\','/'
$rootArg=$Root -replace '\\','/'
$scriptArg=$Script -replace '\\','/'
$arguments="$scriptArg root=$rootArg scenario=lebanon_retail hours=0.25 frame_ms=100 record_every=10 checkpoint_every=3600 observe_every=60 acmi=$outArg/recording.acmi observe=$outArg/observations.jsonl metrics=$outArg/metrics.json observe_supply=0 frame_cost=$outArg/frame-cost.jsonl"
$record=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$nativePath;arguments=$arguments;root=$Root;out=$Out;script=$Script;script_sha256=(Get-FileHash -LiteralPath $Script -Algorithm SHA256).Hash.ToLowerInvariant();build_info_sha256=(Get-FileHash -LiteralPath (Join-Path $Bin 'BUILD-INFO.txt') -Algorithm SHA256).Hash.ToLowerInvariant();kind='R4 observational shipping-script copy: timing ONLY engine:frame; no DCS scheduling/budget or final pack acceptance';resource_context='Background process list captured separately; raw distributions do not prove causal benchmark attribution';actual_native_exit=$null}
Get-CimInstance Win32_Process | Where-Object {$_.Name -match 'eech-world|claude|wsl|vmmem'} | Select-Object ProcessId,CreationDate,Name,CommandLine | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $Out 'background-processes-start.json')
$native=Start-Process -FilePath $nativePath -ArgumentList $arguments -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$native.Handle
$record.status='running';$record['native_pid']=$native.Id;$record['created_utc']=$native.StartTime.ToUniversalTime().ToString('o')
$record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "FRAME_COST_NATIVE_PID=$($native.Id)"
$native.WaitForExit()
$record.status='completed';$record['completed_utc']=[DateTime]::UtcNow.ToString('o');$record.actual_native_exit=$native.ExitCode
$record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "FRAME_COST_NATIVE_ACTUAL_EXIT=$($native.ExitCode)"
exit $native.ExitCode
