#!/usr/bin/env python3
# Tool-input checks for the SUPPLY arrival runners; they need no retail assets or eech-world.
# - lifecycle script: an unknown case= is rejected before eech_dc is required or the
#   installation prepared; a supported case still reaches the module (so the check can fail);
# - native-arguments.ps1: arguments with spaces, quotes, backslashes and empty strings reach a
#   real child process through Start-Process exactly as given (the runners' construction).
#
#   python -I supply-arrivals-tools-selftest.py <lua 5.1 interpreter> <pwsh>
import json
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SCRIPT = os.path.join(os.path.dirname(HERE), "crates", "eech-world", "lua", "supply-arrivals-lifecycle.lua")
HELPER = os.path.join(HERE, "native-arguments.ps1")
lua, pwsh = sys.argv[1:3]


def fail(message):
    sys.exit("FAIL: " + message)


LUA_DRIVER = r"""
local script, case = ...
host = { args = { root = "unused", case = case } }
loaded, prepared = false, false
package.preload.eech_dc = function ()
	loaded = true
	return { prepare_installation = function () prepared = true; error ("stub stops here") end }
end
local ok, message = pcall (dofile, script)
print (string.format ("%s\t%s\t%s\t%s", tostring (ok), tostring (loaded), tostring (prepared), tostring (message)))
"""


def lua_case(case):
    with tempfile.TemporaryDirectory() as d:
        driver = os.path.join(d, "driver.lua")
        open(driver, "w", encoding="utf-8", newline="\n").write(LUA_DRIVER)
        done = subprocess.run([lua, driver, SCRIPT, case], capture_output=True, text=True, timeout=60)
    if done.returncode != 0:
        fail(f"lua driver for case {case!r} exited {done.returncode}: {done.stderr}")
    return done.stdout.rstrip("\n").split("\t", 3)


for case in ("bogus", "", "Missing", "missing "):
    ok, loaded, prepared, message = lua_case(case)
    if (ok, loaded, prepared) != ("false", "false", "false") or "unknown case" not in message:
        fail(f"case {case!r} was not rejected before setup: {ok} {loaded} {prepared} {message}")
    print(f"rejected before setup: case {case!r}")
for case in ("missing", "malformed", "error", "reentry"):
    ok, loaded, prepared, message = lua_case(case)
    if (ok, loaded, prepared) != ("false", "true", "true") or "stub stops here" not in message:
        fail(f"supported case {case!r} did not reach the module: {ok} {loaded} {prepared} {message}")
    print(f"reaches module setup: case {case!r}")

CHILD = "import json, sys\nprint(json.dumps(sys.argv[1:]))\n"
ARGUMENTS = [
    "plain",
    "root=C:/dir with spaces/root",
    'say "hi" there',
    "ends with backslash\\",
    "C:\\dir with spaces\\",
    "backslash\\then\\quote\\\"x",
    'two\\\\"quotes"',
    "",
    "tab\there",
    "-x=1",
]
PWSH_DRIVER = r"""
$ErrorActionPreference = 'Stop'
. $env:HELPER
$items = @(Get-Content -Raw -LiteralPath $env:CASES | ConvertFrom-Json)
$all = @($env:CHILD) + $items
$p = Start-Process -FilePath $env:PY -ArgumentList (ConvertTo-NativeArgumentString $all) -NoNewWindow -PassThru -RedirectStandardOutput $env:OUT
$null = $p.Handle
if (-not $p.WaitForExit(30000)) { $p.Kill(); $p.WaitForExit(); throw 'child timed out' }
exit $p.ExitCode
"""
with tempfile.TemporaryDirectory(prefix="native args ") as d:
    child = os.path.join(d, "child script.py")
    cases, out = os.path.join(d, "cases.json"), os.path.join(d, "argv out.json")
    open(child, "w", encoding="utf-8", newline="\n").write(CHILD)
    open(cases, "w", encoding="utf-8", newline="\n").write(json.dumps(ARGUMENTS))
    env = dict(os.environ, HELPER=HELPER, CASES=cases, CHILD=child, OUT=out, PY=sys.executable)
    done = subprocess.run([pwsh, "-NoProfile", "-NonInteractive", "-Command", PWSH_DRIVER],
                          env=env, capture_output=True, text=True, timeout=120)
    if done.returncode != 0:
        fail(f"argument round trip exited {done.returncode}: {done.stdout}{done.stderr}")
    received = json.load(open(out, encoding="utf-8"))
    if received != ARGUMENTS:
        fail(f"child received {received!r}, expected {ARGUMENTS!r}")
print(f"native arguments: {len(ARGUMENTS)} arguments reached a real child process unchanged")
print("all passed")
