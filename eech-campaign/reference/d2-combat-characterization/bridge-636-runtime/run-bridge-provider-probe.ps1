param([Parameter(Mandatory=$true)][string]$Bin,[Parameter(Mandatory=$true)][string]$Root,[Parameter(Mandatory=$true)][string]$Out,[ValidateSet('lebanon_retail','georgia_retail')][string]$Scenario='lebanon_retail',[Parameter(Mandatory=$true)][ValidateSet('own','own_fall','missing','bad_shape')][string]$Mode)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Host bridge output already exists; preserve and inspect'}
foreach($bridgePath in @($Bin,$Root,$Out)){if($bridgePath -match '[\s&|<>^"]'){throw "Unsafe native argv path: $bridgePath"}}
$bridgeWorld=Join-Path $Bin 'eech-world.exe'
$bridgeLua=(Join-Path $Bin 'lua/bridge-case.lua') -replace '\\','/'
if(-not(Test-Path -LiteralPath $Root) -or -not(Test-Path -LiteralPath $bridgeWorld) -or -not(Test-Path -LiteralPath $bridgeLua)){throw 'Missing verified input or immutable fixture build'}
New-Item -ItemType Directory -Path $Out|Out-Null
$bridgeRoot=$Root -replace '\\','/'
$bridgeTrace=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$bridgeProbe="R:/d2-evidence/combat-characterization-20261028/bridge-provider-probe.lua"
$bridgeOwn=((Join-Path $Bin "lua/own-weapon.lua") -replace "\\","/")
$bridgeProviderOut=((Join-Path $Out "provider.jsonl") -replace "\\","/")
$bridgeArguments="$bridgeProbe root=$bridgeRoot scenario=$Scenario out=$bridgeTrace warmup=5 frame_ms=100 window_seconds=12 criterion=0 bridge_mode=$Mode bridge_script=$bridgeLua own_script=$bridgeOwn provider_out=$bridgeProviderOut"
$bridgeRun=[ordered]@{status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$bridgeWorld;arguments=$bridgeArguments;root=$Root;out=$Out;scenario=$Scenario;kind='R5 altered-host bridge probe: own/own-fall authority or specific expected refusal. Default raw equality does not apply; a nonzero native exit alone is not a genuine refusal.';provider_mode=$Mode;probe_script_sha256=(Get-FileHash -LiteralPath $bridgeProbe -Algorithm SHA256).Hash.ToLowerInvariant();own_script_sha256=(Get-FileHash -LiteralPath $bridgeOwn -Algorithm SHA256).Hash.ToLowerInvariant();bridge_script_sha256=(Get-FileHash -LiteralPath $bridgeLua -Algorithm SHA256).Hash.ToLowerInvariant();build_info_sha256=(Get-FileHash -LiteralPath (Join-Path $Bin 'BUILD-INFO.txt') -Algorithm SHA256).Hash.ToLowerInvariant();actual_native_exit=$null}
$bridgeNative=Start-Process -FilePath $bridgeWorld -ArgumentList $bridgeArguments -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$bridgeNative.Handle
$bridgeRun.status='running';$bridgeRun['native_pid']=$bridgeNative.Id;$bridgeRun['created_utc']=$bridgeNative.StartTime.ToUniversalTime().ToString('o')
$bridgeRun|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "BRIDGE_PROVIDER_NATIVE_PID=$($bridgeNative.Id) scenario=$Scenario"
$bridgeNative.WaitForExit()
$bridgeExit=$bridgeNative.ExitCode
$bridgeRun.status='completed';$bridgeRun['completed_utc']=[DateTime]::UtcNow.ToString('o');$bridgeRun.actual_native_exit=$bridgeExit
$bridgeRun|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "BRIDGE_PROVIDER_NATIVE_ACTUAL_EXIT=$bridgeExit scenario=$Scenario"
exit $bridgeExit
