# Dot-source to get ConvertTo-NativeArgumentString. Start-Process -ArgumentList joins an
# array with single spaces and no quoting, so any element containing a space is split into
# several arguments; pass the single string this returns instead.
function ConvertTo-NativeArgumentString {
	param([Parameter(Mandatory = $true)] [AllowEmptyString()] [string[]] $Argument)
	# Windows argv rules (CommandLineToArgvW, MSVCRT): an argument with white space, a quote or
	# no characters is quoted; backslashes are literal unless they precede a quote, where they
	# are doubled, and a trailing run is doubled before the closing quote.
	$quoted = foreach ($item in $Argument) {
		if ($item.Length -gt 0 -and $item -notmatch '[\s"]') { $item; continue }
		$escaped = [regex]::Replace($item, '(\\*)"', '$1$1\"')
		$escaped = [regex]::Replace($escaped, '(\\+)$', '$1$1')
		'"' + $escaped + '"'
	}
	$quoted -join ' '
}
