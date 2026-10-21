#!/usr/bin/env python3
# Self-test of the DCS probe tooling without DCS. It checks:
# - build-probe-miz.py on a synthetic mission: what it strips and adds, the
#   parser/serialiser round trip, byte-identical output, and its error exits;
# - with a Lua 5.1 interpreter, eech-dcs-probe.lua and eech-m5probe-hook.lua
#   under fake DCS globals and a fake eech_dc: the error path, a malformed engine
#   object shape, and, with a controlled os.clock, the callback cost and gap
#   statistics the frame-budget measurement reports.
# The runner (run-dcs-probe.ps1) and the PE link check need Windows binaries
# and are not covered here.
#
#   python -I dcs-probe-selftest.py <lua 5.1 interpreter>
#   python -I dcs-probe-selftest.py --skip-lua      (builder only; never reports "all passed")
import importlib.util
import os
import subprocess
import sys
import tempfile
import zipfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.dont_write_bytecode = True  # importing the builder must not leave __pycache__ in the tool directory
spec = importlib.util.spec_from_file_location("builder", os.path.join(HERE, "build-probe-miz.py"))
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)

MISSION = r'''mission =
{
    ["coalition"] =
    {
        ["blue"] =
        {
            ["country"] =
            {
                [1] =
                {
                    ["id"] = 2,
                    ["name"] = "USA",
                    ["plane"] =
                    {
                        ["group"] =
                        {
                        }, -- end of ["group"]
                    }, -- end of ["plane"]
                    ["vehicle"] =
                    {
                    }, -- end of ["vehicle"]
                }, -- end of [1]
            }, -- end of ["country"]
        }, -- end of ["blue"]
    }, -- end of ["coalition"]
    ["needModules"] =
    {
        ["Paid"] = "Paid",
    }, -- end of ["needModules"]
    ["usedModules"] =
    {
        ["Paid"] = true,
    }, -- end of ["usedModules"]
    ["descriptionText"] = "a \"quoted\"\
line\\path",
    ["weather"] =
    {
        ["x"] = -1.5e-3,
        ["y"] = 10,
        ["on"] = false,
    }, -- end of ["weather"]
    ["trig"] =
    {
    }, -- end of ["trig"]
} -- end of mission
'''

failures = []


def check(name, condition, detail=""):
    print(("ok   " if condition else "FAIL ") + name + (f": {detail}" if detail and not condition else ""))
    if not condition:
        failures.append(name)


def build(tmp, out, *args):
    return subprocess.run([sys.executable, "-I", os.path.join(HERE, "build-probe-miz.py"), *args[:1], out, *args[1:]],
                          capture_output=True, text=True, cwd=tmp)


def builder_tests(tmp):
    template = os.path.join(tmp, "template.miz")
    with zipfile.ZipFile(template, "w") as z:
        z.writestr("mission", MISSION)
        z.writestr("options", "options = {}\n")
        z.writestr("warehouses", "warehouses = {}\n")
    original = builder.parse_chunk(MISSION, "mission")
    check("parser reads strings with escaped quotes, newlines and backslashes",
          original["descriptionText"] == 'a "quoted"\nline\\path')
    check("parser reads numbers and booleans", original["weather"] == {"x": -1.5e-3, "y": 10, "on": False})
    check("serialiser round trip", builder.parse_chunk("mission = " + builder.lua_value(original), "mission") == original)

    args = ("load", "lebanon_retail", r"E:\p\probe.lua", r"E:\p\bin", r"E:\p\root", r"E:\p\probe.log")
    first = os.path.join(tmp, "a.miz")
    second = os.path.join(tmp, "b.miz")
    result = build(tmp, first, template, *args)
    check("build exits 0", result.returncode == 0, result.stderr)
    build(tmp, second, template, *args)
    check("same inputs give a byte-identical .miz", open(first, "rb").read() == open(second, "rb").read())
    with zipfile.ZipFile(first) as z:
        names = sorted(z.namelist())
        mission = builder.parse_chunk(z.read("mission").decode("utf-8"), "mission")
    check("other entries are carried over", names == ["mission", "options", "warehouses"])
    country = mission["coalition"]["blue"]["country"][1]
    check("units are stripped, the country is kept", "plane" not in country and "vehicle" not in country and country["name"] == "USA")
    check("module requirements are removed", mission["needModules"] == {} and mission["usedModules"] == {})
    action = mission["trig"]["actions"][1]
    check("one MISSION START DO SCRIPT", mission["trigrules"][1]["predicate"] == "triggerStart" and action.startswith("a_do_script("))
    script = mission["trigrules"][1]["actions"][1]["text"]
    check("the DO SCRIPT sets stage, scenario and paths, then runs the probe",
          'stage = "load"' in script and 'scenario = "lebanon_retail"' in script
          and r"dll_dir = [[E:\p\bin]]" in script and script.endswith(r"dofile ([[E:\p\probe.lua]])"))
    check("unrelated mission data is preserved", mission["weather"] == original["weather"])

    bad_path = build(tmp, os.path.join(tmp, "c.miz"), template, "load", "lebanon_retail", "a]]b", "b", "c", "d")
    check("a path containing ]] exits 2", bad_path.returncode == 2 and "]]" in bad_path.stderr, bad_path.stderr)
    not_zip = os.path.join(tmp, "not.miz")
    open(not_zip, "w").write("not a zip")
    check("a non-zip template exits 2", build(tmp, os.path.join(tmp, "d.miz"), not_zip, *args).returncode == 2)
    check("a missing template exits 2", build(tmp, os.path.join(tmp, "e.miz"), os.path.join(tmp, "none.miz"), *args).returncode == 2)
    check("wrong argument count exits 2", subprocess.run([sys.executable, "-I", os.path.join(HERE, "build-probe-miz.py")],
                                                         capture_output=True).returncode == 2)
    for text in ('mission = { ["a"] = nil }', 'mission = { ["a"] 1 }', 'mission = { ["a" = 1 }'):
        try:
            builder.parse_chunk(text, "mission")
            check(f"parser rejects {text!r}", False)
        except ValueError:
            check(f"parser rejects {text!r}", True)


FAKE_DCS = r'''
-- fake DCS mission-state globals and a fake eech_dc; arg[1] probe, arg[2] "good"|"bad" shape, arg[3] stage
local lines = {}
env = { info = function (s) lines[#lines + 1] = s end }
local now = 0
timer = { getTime = function () return now end, getAbsTime = function () return 43200 end,
          scheduleFunction = function (f) scheduled = f end }
world = { event = { S_EVENT_MISSION_END = 12 }, addEventHandler = function () end,
          getPersistenceData = function () return nil end, setPersistenceHandler = function () end }
local shape = arg[2]
package.preload.eech_dc = function ()
  return { name = "fake 0", prepare_installation = function () end,
    boot = function (config)
      if type (config) ~= "table" then error ("bad argument #1: error converting Lua string to table") end
      return setmetatable ({}, { __index = {
        frame = function () end,
        objects = function ()
          if shape == "bad" then return { { alive = true, kind = nil, position = { 1 } } } end
          return { { alive = true, kind = "keysite", position = { 1, 2, 3 } } }
        end,
        clock = function () return { elapsed_seconds = 1, day = 1 } end } })
    end }
end
EECH_DCS_PROBE = { stage = arg[3], root = "x", summary_every = 0.1 }
dofile (arg[1])
if scheduled then now = 0.25; scheduled () end
for _, l in ipairs (lines) do print (l) end
'''

FAKE_GUI = r'''
-- fake DCS GUI-state globals for the hook; arg[1] hook
log = { INFO = 1, write = function (_, _, s) print (s) end }
DCS = { setUserCallbacks = function () print ("callbacks set") end }
lfs = { writedir = function () return "w" end }
dofile (arg[1])
'''


TIMING = r'''
-- a controlled os.clock (seconds): the driver sets it per callback, and the fake engine adds its
-- costs; arg[1] probe, arg[2] mode
local probe, mode = arg[1], arg[2]
local now, wall, scheduled = 0, 0, nil
os.clock = function () return wall end
env = { info = function (line) print (line) end }
timer = { getTime = function () return now end, getAbsTime = function () return 0 end,
          scheduleFunction = function (f) scheduled = f end }
world = { event = {}, addEventHandler = function () end }
package.preload.eech_dc = function () return {
  name = "timing fake", prepare_installation = function () end,
  boot = function (config)
    if type (config) ~= "table" then error ("bad argument") end
    return {
      frame = function () if mode == "cost" then wall = wall + 0.001 end end,       -- 1 ms per frame
      objects = function () if mode == "cost" then wall = wall + 0.050 end return {} end,  -- 50 ms
      clock = function () return { elapsed_seconds = now, day = 1 } end }
  end } end
-- the expected values below depend on these settings, so none is left to a probe default
EECH_DCS_PROBE = { stage = "frames", root = "synthetic", frame_ms = 100, interval = 0.05, max_substeps = 10,
                   summary_every = mode == "cost" and 0.1 or 0.5 }
dofile (probe)
if mode == "cost" then
  -- callback 1 (0.25 s): two frames (2 ms), then a summary and a 50 ms checkpoint: 52 ms in all
  now, wall = 0.25, 1; scheduled ()
  -- callbacks 2 and 3: one frame each; callback 3 writes the next summary
  now = 0.30; scheduled ()
  now = 0.40; scheduled ()
else
  -- four callbacks exactly 1 s of wall time apart, two per summary window
  for i = 1, 4 do now, wall = i * 0.25, i; scheduled () end
end
'''


def summaries(out):
    rows = []
    for line in out.splitlines():
        if "event=frames_summary" in line:
            rows.append(dict(part.split("=", 1) for part in line.split()[2:] if "=" in part))
    return rows


def timing_tests(tmp, lua):
    script = os.path.join(tmp, "timing.lua")
    open(script, "w").write(TIMING)
    probe = os.path.join(HERE, "eech-dcs-probe.lua")

    def run(mode):
        result = subprocess.run([lua, script, probe, mode], capture_output=True, text=True)
        check(f"{mode}: the timing run exits 0", result.returncode == 0, result.stderr)
        return result.stdout + result.stderr

    out = run("cost")
    rows = summaries(out)
    check("cost: two summaries", len(rows) == 2, out)
    if len(rows) == 2:
        first, second = rows
        check("cost: the stepping part of the summary-writing callback is 2 ms",
              first.get("step_max_ms") == "2.0", str(first))
        check("cost: no callback has completed before the first summary is written",
              first.get("callbacks_completed") == "0" and first.get("callback_max_ms") == "-", str(first))
        check("cost: the complete 52 ms callback (frames + summary + checkpoint) is reported, in the next window",
              second.get("callback_max_ms") == "52.0" and second.get("callbacks_completed") == "2", str(second))

    out = run("gap")
    rows = summaries(out)
    check("gap: two summaries", len(rows) == 2, out)
    if len(rows) == 2:
        first, second = rows
        check("gap: constant 1000 ms spacing gives a 1000 ms mean in every window",
              first.get("gap_mean_ms") == "1000.0" and second.get("gap_mean_ms") == "1000.0", f"{first} {second}")
        check("gap: the mean is over the gaps actually sampled (1, then 2 reaching back across the reset)",
              first.get("gaps") == "1" and second.get("gaps") == "2", f"{first} {second}")


def lua_tests(tmp, lua):
    probe = os.path.join(HERE, "eech-dcs-probe.lua")
    fake = os.path.join(tmp, "fake-dcs.lua")
    open(fake, "w").write(FAKE_DCS)

    def run(shape, stage):
        result = subprocess.run([lua, fake, probe, shape, stage], capture_output=True, text=True, cwd=tmp)
        return result.returncode, result.stdout + result.stderr

    code, out = run("good", "frames")
    check("probe frames stage runs to the end", code == 0, out)
    for event in ("env", "persistence_handler", "load", "error_path_begin", "error_path", "boot", "frames_start", "checkpoint", "frames_summary"):
        check(f"probe records event={event}", f"event={event} " in out or out.rstrip().endswith(f"event={event}"), out)
    check("the module error is recorded as raised, not a crash", "event=error_path message=" in out and "raised=true" in out, out)
    check("the error-path begin marker precedes the result", out.find("event=error_path_begin") < out.find("event=error_path message="))
    check("frames is cumulative, frames_window per summary", "frames=2 " in out and "frames_window=2" in out, out)
    check("values with spaces are quoted", 'name="fake 0"' in out, out)

    code, out = run("bad", "frames")
    check("a malformed engine object is recorded as checkpoint_error", code == 0 and "event=checkpoint_error" in out, out)
    check("frames still run after a checkpoint error", "event=frames_summary" in out, out)

    code, out = run("good", "nonsense")
    check("an unknown stage is a config_error", code == 0 and "event=config_error" in out, out)

    hook = os.path.join(HERE, "eech-m5probe-hook.lua")
    gui = os.path.join(tmp, "fake-gui.lua")
    open(gui, "w").write(FAKE_GUI)
    environment = {k: v for k, v in os.environ.items() if k != "EECH_M5PROBE_RUN_SECONDS"}
    out = subprocess.run([lua, gui, hook], capture_output=True, text=True, env=environment).stdout
    check("the hook is inert without EECH_M5PROBE_RUN_SECONDS", "inactive" in out and "callbacks set" not in out, out)
    out = subprocess.run([lua, gui, hook], capture_output=True, text=True, env={**environment, "EECH_M5PROBE_RUN_SECONDS": "20"}).stdout
    check("the hook sets its callbacks when the runner set the bound", "callbacks set" in out and "run_seconds=20" in out, out)


def main(argv):
    if len(argv) != 2 or (argv[1].startswith("-") and argv[1] != "--skip-lua"):
        print("usage: dcs-probe-selftest.py <lua 5.1 interpreter> | --skip-lua", file=sys.stderr)
        return 2
    skip_lua = argv[1] == "--skip-lua"
    with tempfile.TemporaryDirectory() as tmp:
        builder_tests(tmp)
        if not skip_lua:
            lua_tests(tmp, argv[1])
            timing_tests(tmp, argv[1])
    if failures:
        print(f"{len(failures)} failed")
        return 1
    print("builder tests passed; Lua tests NOT RUN (--skip-lua)" if skip_lua else "all passed")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
