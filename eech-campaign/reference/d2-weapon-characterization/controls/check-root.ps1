# check-root.ps1 -Name <root name>: the repository's input manifest of a root compared with its reference (after a run); prints the actual exit.
param([Parameter(Mandatory=$true)][string]$Name, [string]$Map = 'lebanon')
$python = 'C:\Users\starscream\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe'
$repo = 'R:\eech_source_code\eech-campaign'
$ref = if ($Map -eq 'georgia') { "$repo\reference\m4-s3-mutation\inputs-georgia.json" } else { "$repo\reference\lebanon-3h\inputs.json" }
$out = & $python -B -X utf8 "$repo\tools\input-manifest.py" "R:\d2-evidence\weapon-characterization-20261027\roots\$Name" --compare $ref 2>&1
"${Name} manifest-exit=$LASTEXITCODE : $out"
exit $LASTEXITCODE
