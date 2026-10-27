# Sequential native runs of the FINAL candidate head (one native process at a time). Each step records its own native exit in its run log.
param([string] $Head = 'd3c9a5c5e')
$ErrorActionPreference = 'Continue'
Set-Location C:\Windows
$base = 'R:\d2-evidence\aim-extraction-20261026'
$fix = "R:\eech_source_code\eech-campaign\target\aim-fixture\$Head"
$ord = "R:\eech_source_code\eech-campaign\target\windows-$Head"
function Stamp($m) { "$(Get-Date -Format 'HH:mm:ss') $m" | Tee-Object -FilePath "$base\queue-3.log" -Append }
Stamp "queue-3 start head=$Head fixture=$fix ordinary=$ord"
Stamp '900 s ordinary'; & "$base\run-ordinary-900.ps1" -Bin $ord -Name 'o3-ab'
# the original oracle build with the provider script added to a PRIVATE copy (the retained original binaries stay untouched)
$orig = "$base\original-with-provider"
if (-not (Test-Path $orig)) {
	Copy-Item -Recurse 'R:\eech_source_code\eech-campaign\target\aim-fixture\0ee1fa46b-clean' $orig
}
Copy-Item "$fix\lua\aim-provider.lua" "$orig\lua\aim-provider.lua" -Force
foreach ($m in @('reference', 'ready_slow', 'step_slow', 'invalid', 'own_model')) {
	Stamp "provider $m (candidate fixture)"; & "$base\run-provider.ps1" -Bin $fix -Name "pv-$m" -Mode $m -Seconds 300
}
foreach ($m in @('ready_slow', 'invalid')) {
	Stamp "provider $m (original oracle build, negative control)"; & "$base\run-provider.ps1" -Bin $orig -Name "orig-pv-$m" -Mode $m -Seconds 300
}
foreach ($m in @('missing', 'bad_nil', 'bad_state', 'bad_ready_state', 'bad_phase', 'bad_nan_vector', 'bad_draw_for_ballistic', 'bad_second_draw', 'bad_negative_range', 'bad_error')) {
	Stamp "provider $m (shipping)"; & "$base\run-provider.ps1" -Bin $ord -Name "ship-pv-$m" -Mode $m -Seconds 300
}
Stamp 'shipping reference provider 120 s'; & "$base\run-provider.ps1" -Bin $ord -Name 'ship-pv-reference' -Mode reference -Seconds 120
Stamp 'queue-3 DONE'
