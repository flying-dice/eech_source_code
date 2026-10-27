# Sequential native runs of the clean fixture candidate (one native process at a time). Each step records its own native exit in its run log.
$ErrorActionPreference = 'Continue'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$bin = 'R:\eech_source_code\eech-campaign\target\aim-fixture\f749878ce-lf'
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\queue-1.log" -Append }
Stamp "queue-1 start bin=$bin"
Stamp 'person case target'; & "$base\run-person.ps1" -Bin $bin -Name 'p-t' -Case target
Stamp 'person case control'; & "$base\run-person.ps1" -Bin $bin -Name 'p-c' -Case control
Stamp 'person case target repeat'; & "$base\run-person.ps1" -Bin $bin -Name 'p-t2' -Case target
Stamp '600 s a'; & "$base\run-one.ps1" -Bin $bin -Name 'c-a'
Stamp '600 s b'; & "$base\run-one.ps1" -Bin $bin -Name 'c-b'
Stamp 'queue-1 DONE'
