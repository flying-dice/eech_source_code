# Side-effect-free runner guards shared by run-dcs-probe.ps1 and its self-test.
# Callers supply a process snapshot, so testing never needs to enumerate or stop a real process.
function Assert-NoRunningDcs($ProcessSnapshot) {
	if (@($ProcessSnapshot).Count -ne 0) {
		throw 'a DCS process is already running: not starting another'
	}
}

# Reads only the supplied path. The runner establishes that it is the protected default log
# before calling this helper; the self-test uses a scratch file.
function Get-LogTimestampTicks([string] $Path) {
	(Get-Item -LiteralPath $Path).LastWriteTimeUtc.Ticks
}

function Test-LogTimestampUnchanged([string] $Path, [long] $ExpectedTicks) {
	(Get-LogTimestampTicks $Path) -eq $ExpectedTicks
}

function Get-DefaultLogGuardVerdict([string] $Path, [long] $ExpectedTicks) {
	if (-not (Test-LogTimestampUnchanged $Path $ExpectedTicks)) {
		return 'aborted: the default profile log changed'
	}
	return $null
}
