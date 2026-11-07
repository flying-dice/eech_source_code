import sys

P = sys.argv[1] + "/eech-campaign/tools/sector-height-baseline-windows.ps1"
d = open(P, "rb").read().decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep("""# This wrapper never terminates a native process.""", """# SCOPE (Task49, the host-owned sector height): `-Scope original` (the default) observes the campaign-held word in a build that keeps the ORIGINAL hooks; `-Scope hosted-scan` runs the same driver against a HOSTED
# observer build and writes only the scan reads (the word the hosted getter was answered at each FIRST, and fact 12 of every offer), the clearance guard records and the per-sector scan rows; with `-Accepted <the accepted original
# report>` the driver compares those rows by sector at the same window (see lua/sector-height-baseline.lua, SCOPE hosted-scan) and writes `sho-compare` and a scoped `sho-verdict`. The original link, event, shape, clearance-word and
# session records are NOT comparable in a hosted build and are not written.
#
# This wrapper never terminates a native process.""")
rep("""	[int] $WaitSeconds = 0
)""", """	[int] $WaitSeconds = 0,
	[ValidateSet('original', 'hosted-scan')] [string] $Scope = 'original',
	[string] $Accepted = ''
)""")
rep("""$arguments = @($script, "root=$($Root -replace '\\\\', '/')", "scenario=$Scenario", "hours=$Hours", "frame_ms=$FrameMs", "seed=$Seed", "out=$($observation -replace '\\\\', '/')")
""", """$arguments = @($script, "root=$($Root -replace '\\\\', '/')", "scenario=$Scenario", "hours=$Hours", "frame_ms=$FrameMs", "seed=$Seed", "out=$($observation -replace '\\\\', '/')")
if ($Scope -ne 'original') { $arguments += "scope=$Scope" }
if ($Accepted) {
	if ($Scope -ne 'hosted-scan') { throw '-Accepted is only for -Scope hosted-scan' }
	if (-not (Test-Path -LiteralPath $Accepted)) { throw "$Accepted not found: the accepted original report is read, never created" }
	$arguments += "accepted=$($Accepted -replace '\\\\', '/')"
}
""")
rep("""$lines = @(Get-Content -LiteralPath $observation)
if ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict whole=1')) { throw "the observation has no whole verdict; see $observation" }""", """$lines = @(Get-Content -LiteralPath $observation)
if ($Scope -eq 'hosted-scan') {
	if ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict scope=hosted-scan scan_whole=1')) { throw "the hosted-scan observation has no whole scan verdict; see $observation" }
	if ($Accepted -and -not $lines[-1].Contains('status=EQUAL_AT_SCOPE')) { throw "the hosted scan did not compare equal at scope; see $observation" }
}
elseif ($lines.Count -lt 2 -or -not $lines[-1].StartsWith('sho-verdict whole=1')) { throw "the observation has no whole verdict; see $observation" }""")
open(P, "wb").write(d.encode("utf-8"))
print("ok")
