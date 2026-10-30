param(
    [Parameter(Mandatory=$true)][string]$Bin,
    [Parameter(Mandatory=$true)][string]$Root,
    [Parameter(Mandatory=$true)][string]$Out,
    [ValidateSet('site','segment')][string]$Kind='site',
    [ValidateSet('none','damage','destroyed')][string]$Report='destroyed',
    [ValidateSet('none','unknown','duplicate','raise')][string]$Control='none',
    [ValidateSet('reference','own')][string]$Identity='reference'
)
$ErrorActionPreference='Stop'
if(Test-Path -LiteralPath $Out){throw 'Refusing existing object-report output; preserve and inspect it'}
foreach($objectPath in @($Bin,$Root,$Out)){
    if($objectPath -match '[\s&|<>^\"]'){throw "Unsafe native argv path: $objectPath"}
}
$objectWorld=Join-Path $Bin 'eech-world.exe'
$objectLua=(Join-Path $Bin 'lua/object-report-case.lua') -replace '\\','/'
$objectBuild=Join-Path $Bin 'BUILD-INFO.txt'
foreach($objectPath in @($Root,$objectWorld,$objectLua,$objectBuild)){
    if(-not(Test-Path -LiteralPath $objectPath)){throw "Missing verified input/build path: $objectPath"}
}
New-Item -ItemType Directory -Path $Out | Out-Null
$objectRoot=$Root -replace '\\','/'
$objectTrace=(Join-Path $Out 'trace.jsonl') -replace '\\','/'
$objectArguments="$objectLua root=$objectRoot scenario=lebanon_retail out=$objectTrace warmup=5 frame_ms=100 window_seconds=12 kind=$Kind report=$Report control=$Control identity=$Identity"
$objectRun=[ordered]@{
    status='starting';when_utc=[DateTime]::UtcNow.ToString('o');binary=$objectWorld;arguments=$objectArguments
    root=$Root;out=$Out;kind=$Kind;report=$Report;control=$Control;identity=$Identity
    purpose='R3 shipping API witness: physical report/withhold state and end-of-run refusal. Native exit alone is insufficient; inspect actual completed census/before/after/summary and primary guard.'
    limits='Controlled host-reported fact, not a weapon hit, fullpack, performance result, arbitrary-delay or all-family coverage.'
    script_sha256=(Get-FileHash -LiteralPath $objectLua -Algorithm SHA256).Hash.ToLowerInvariant()
    build_info_sha256=(Get-FileHash -LiteralPath $objectBuild -Algorithm SHA256).Hash.ToLowerInvariant()
    actual_native_exit=$null
}
$objectRun | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'run.json')
$objectNative=Start-Process -FilePath $objectWorld -ArgumentList $objectArguments -WorkingDirectory $Out -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $Out 'run.out') -RedirectStandardError (Join-Path $Out 'run.log')
$null=$objectNative.Handle
$objectRun.status='running'
$objectRun['native_pid']=$objectNative.Id
$objectRun['created_utc']=$objectNative.StartTime.ToUniversalTime().ToString('o')
$objectRun | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "OBJECT_REPORT_NATIVE_PID=$($objectNative.Id) kind=$Kind report=$Report control=$Control identity=$Identity"
$objectNative.WaitForExit()
$objectRun.status='completed'
$objectRun['completed_utc']=[DateTime]::UtcNow.ToString('o')
$objectRun.actual_native_exit=$objectNative.ExitCode
$objectRun | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $Out 'run.json')
Write-Output "OBJECT_REPORT_NATIVE_ACTUAL_EXIT=$($objectNative.ExitCode)"
exit $objectNative.ExitCode
