#!/usr/bin/env python3
# Self-test of the DCS probe tooling without DCS. It checks:
# - build-probe-miz.py on a synthetic mission: what it strips and adds, the
#   parser/serialiser round trip, byte-identical output, and its error exits;
# - with a Lua 5.1 interpreter, eech-dcs-probe.lua and eech-m5probe-hook.lua
#   under fake DCS globals and a fake eech_dc: the error path, a malformed engine
#   object shape, and, with a controlled os.clock, the callback cost and gap
#   statistics the frame-budget measurement reports.
# Portable PE fixtures and runner prelaunch guards are also checked without DCS.
#
#   python -I dcs-probe-selftest.py <lua 5.1 interpreter>
#   python -I dcs-probe-selftest.py --skip-lua      (no Lua cases; never reports "all passed")
import hashlib
import importlib.util
import json
import os
import struct
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


def builder_tests(tmp, lua=None):
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
          and 'dll_dir = ' + builder.lua_string(args[3]) in script
          and script.endswith('dofile (' + builder.lua_string(args[2]) + ')'))
    check("unrelated mission data is preserved", mission["weather"] == original["weather"])

    for index, unusual in enumerate((r"C:\a[[b", r"C:\a]", r"C:\a]]b", 'C:\\a"b')):
        paths = list(args[2:])
        paths[index] = unusual
        output = os.path.join(tmp, f"bracket-{index}.miz")
        result = build(tmp, output, template, *args[:2], *paths)
        check(f"unusual path {index} builds", result.returncode == 0, result.stderr)
        if result.returncode == 0:
            with zipfile.ZipFile(output) as z:
                changed = builder.parse_chunk(z.read("mission").decode("utf-8"), "mission")
            generated = changed["trigrules"][1]["actions"][1]["text"]
            check(f"unusual path {index} is represented exactly", builder.lua_string(unusual) in generated, generated)
            if lua:
                compile_result = subprocess.run([lua, "-e", "assert(loadstring(io.read('*a')))"],
                                                input=generated, capture_output=True, text=True)
                check(f"unusual path {index} produces compilable Lua 5.1",
                      compile_result.returncode == 0, compile_result.stderr)
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


def pe_image(path, dll_name, exported, imported=None, machine=0x8664, delayed=False):
    """Write the PE headers and tables that the link checker reads, with fixed RVAs."""
    data = bytearray(0x1000)
    struct.pack_into("<I", data, 0x3C, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, machine, 1)
    struct.pack_into("<H", data, 0x94, 240)
    struct.pack_into("<H", data, 0x98, 0x20B)
    directory = 0x98 + 112
    struct.pack_into("<II", data, directory, 0x1000, 0x80)
    if imported:
        struct.pack_into("<II", data, directory + 8, 0x1100, 0x80)
    if delayed:
        struct.pack_into("<II", data, directory + 13 * 8, 0x1200, 0x10)
    section = 0x98 + 240
    struct.pack_into("<IIII", data, section + 8, 0x800, 0x1000, 0x800, 0x200)

    def at(rva):
        return rva - 0x1000 + 0x200

    struct.pack_into("<I", data, at(0x1000) + 12, 0x1040)
    struct.pack_into("<I", data, at(0x1000) + 24, 1 if exported else 0)
    struct.pack_into("<I", data, at(0x1000) + 32, 0x1050)
    data[at(0x1040):at(0x1040) + len(dll_name) + 1] = dll_name.encode() + b"\0"
    if exported:
        struct.pack_into("<I", data, at(0x1050), 0x1060)
        data[at(0x1060):at(0x1060) + len(exported) + 1] = exported.encode() + b"\0"
    if imported:
        struct.pack_into("<IIIII", data, at(0x1100), 0x1140, 0, 0, 0x1130, 0x1140)
        data[at(0x1130):at(0x1130) + 8] = b"lua.dll\0"
        struct.pack_into("<Q", data, at(0x1140), 0x1160)
        data[at(0x1162):at(0x1162) + len(imported) + 1] = imported.encode() + b"\0"
    with open(path, "wb") as stream:
        stream.write(data)


def pe_tests(tmp):
    checker = os.path.join(HERE, "pe-lua-link-check.py")
    host = os.path.join(tmp, "lua.dll")
    module = os.path.join(tmp, "eech_dc.dll")
    pe_image(host, "lua.dll", "lua_foo")

    def run(candidate=module, runtime=host):
        return subprocess.run([sys.executable, "-I", checker, candidate, runtime], capture_output=True, text=True)

    pe_image(module, "eech_dc.dll", "luaopen_eech_dc", "lua_foo")
    result = run()
    check("synthetic PE link pair passes", result.returncode == 0 and "PASS (link level" in result.stdout, result.stdout + result.stderr)
    pe_image(module, "eech_dc.dll", "luaopen_eech_dc", "lua_missing")
    result = run()
    check("PE checker rejects absent Lua symbol", result.returncode == 1 and "missing lua_missing" in result.stdout, result.stdout + result.stderr)
    pe_image(module, "eech_dc.dll", "other_export", "lua_foo")
    result = run()
    check("PE checker rejects absent entry point", result.returncode == 1 and "FAIL: module exports luaopen_eech_dc" in result.stdout, result.stdout + result.stderr)
    pe_image(module, "eech_dc.dll", "luaopen_eech_dc", "lua_foo", delayed=True)
    result = run()
    check("PE checker rejects delay imports", result.returncode == 1 and "FAIL: module has no delay imports" in result.stdout, result.stdout + result.stderr)
    pe_image(module, "eech_dc.dll", "luaopen_eech_dc", "lua_foo", machine=0x14C)
    result = run()
    check("PE checker rejects non-x64 machine", result.returncode == 1 and "FAIL: module is x86-64 PE32+" in result.stdout, result.stdout + result.stderr)
    for name, data in (("short", b"MZ"), ("zero", bytes(256))):
        malformed = os.path.join(tmp, name)
        with open(malformed, "wb") as stream:
            stream.write(data)
        result = run(malformed)
        check(f"PE checker rejects {name} input", result.returncode == 2 and "error:" in result.stderr, result.stdout + result.stderr)
    result = run(os.path.join(tmp, "missing"))
    check("PE checker rejects missing input", result.returncode == 2 and "error:" in result.stderr, result.stdout + result.stderr)


def retained_tests():
    verifier = os.path.join(HERE, "..", "..", "reference", "m5-dcs-probes", "verify_retained.py")
    result = subprocess.run([sys.executable, "-I", verifier], capture_output=True, text=True)
    check("retained evidence bytes match the repository manifest",
          result.returncode == 0 and ", 0 problems" in result.stdout,
          result.stdout + result.stderr)


def runner_guard_tests(tmp):
    script = os.path.join(HERE, "run-dcs-probe.ps1")
    saved = os.path.join(tmp, "Saved Games")
    default_log = os.path.join(saved, "DCS.openbeta", "Logs", "dcs.log")
    candidate = os.path.join(tmp, "candidate")
    dcs = os.path.join(tmp, "DCS")
    work = os.path.join(tmp, "work")
    os.makedirs(os.path.dirname(default_log))
    os.makedirs(candidate)
    os.makedirs(os.path.join(dcs, "bin-mt"))
    with open(default_log, "wb") as stream:
        stream.write(b"do not change")
    dll = os.path.join(candidate, "eech_dc.dll")
    lua = os.path.join(dcs, "bin-mt", "lua.dll")
    with open(dll, "wb") as stream:
        stream.write(b"synthetic candidate")
    with open(lua, "wb") as stream:
        stream.write(b"synthetic DCS Lua")
    def sha(path):
        with open(path, "rb") as stream:
            return hashlib.sha256(stream.read()).hexdigest()
    evidence = os.path.join(tmp, "controls.json")
    record = {
        "candidate_eech_dc_sha256": sha(dll), "dcs_lua_sha256": sha(lua),
        "controls": {"first_error": {"completed": True, "exit_code": 0},
                     "repeated_errors": {"completed": True, "exit_code": 0}},
    }

    def run(stage="boot", write_dir="DCS.M5Probe", evidence_path=evidence, default_name="DCS.openbeta"):
        return subprocess.run([
            "pwsh", "-NoProfile", "-File", script, "-Stage", stage, "-Candidate", candidate,
            "-Dcs", dcs, "-SavedGames", saved, "-Work", work,
            "-WriteDirName", write_dir, "-DefaultWriteDirName", default_name,
            "-ErrorControlsEvidence", evidence_path, "-ValidatePrelaunchOnly",
        ], capture_output=True, text=True)

    with open(evidence, "w") as stream:
        json.dump(record, stream)
    result = run()
    check("runner accepts matched passing controls without launch", result.returncode == 0 and "prelaunch guards passed" in result.stdout,
          result.stdout + result.stderr)
    for alias in ("DCS.openbeta", r".\DCS.openbeta", "DCS.openbeta\\", r"..\Saved Games\DCS.openbeta"):
        result = run(write_dir=alias)
        check(f"runner rejects default profile alias {alias!r}", result.returncode != 0 and "default profile" in result.stderr,
              result.stdout + result.stderr)
    # Physical identity, not spelling: junctions (Windows) or symlinks (elsewhere) that reach the
    # protected default profile must be refused, and the profile must be left untouched.
    default_dir = os.path.join(saved, "DCS.openbeta")
    harmless = os.path.join(tmp, "elsewhere")
    os.makedirs(harmless)
    links = []

    def link(path, target):
        kind = "Junction" if os.name == "nt" else "SymbolicLink"
        made = subprocess.run(["pwsh", "-NoProfile", "-Command",
                               "New-Item -ItemType $env:LINK_KIND -Path $env:LINK_PATH -Target $env:LINK_TARGET | Out-Null"],
                              capture_output=True, text=True,
                              env={**os.environ, "LINK_KIND": kind, "LINK_PATH": path, "LINK_TARGET": target})
        check(f"fixture {kind} {os.path.basename(path)} is created", made.returncode == 0, made.stderr)
        links.append(path)

    def unlink(path):
        # remove the link itself only, never its target
        try:
            os.rmdir(path) if os.name == "nt" else os.unlink(path)
        except OSError:
            pass

    try:
        link(os.path.join(saved, "DCS.Alias"), default_dir)
        link(os.path.join(saved, "DCS.SavedGamesAlias"), saved)
        os.makedirs(os.path.join(saved, "DCS.LogsAlias"))
        link(os.path.join(saved, "DCS.LogsAlias", "Logs"), os.path.dirname(default_log))
        link(os.path.join(saved, "DCS.InsideAlias"), os.path.dirname(default_log))
        link(os.path.join(saved, "DCS.HarmlessAlias"), harmless)
        before = os.stat(default_log).st_mtime_ns
        for name, why in (("DCS.Alias", "a junction or symlink to the default profile"),
                          (os.path.join("DCS.Alias", "Probe"), "a directory below a link to the default profile"),
                          ("DCS.SavedGamesAlias", "a link to the directory that contains the default profile"),
                          ("DCS.LogsAlias", "a Logs link that reaches the default profile's log"),
                          ("DCS.InsideAlias", "a link into the default profile")):
            result = run(write_dir=name)
            check(f"runner rejects {why}",
                  result.returncode != 0 and "default profile" in result.stderr and "prelaunch guards passed" not in result.stdout,
                  result.stdout + result.stderr)
        result = run(write_dir="DCS.HarmlessAlias")
        check("runner accepts a link that reaches nowhere near the default profile",
              result.returncode == 0 and "prelaunch guards passed" in result.stdout, result.stdout + result.stderr)
        check("the default profile log is untouched by the guard tests",
              os.stat(default_log).st_mtime_ns == before and open(default_log, "rb").read() == b"do not change")
    finally:
        for path in reversed(links):
            unlink(path)
    check("removing the fixture links left the default profile intact", os.path.exists(default_log))
    # names that Windows reads as an NTFS stream, a wildcard, or a dropped trailing dot/space
    # alias the protected directory or are globbed by later commands: refused as names
    for name in ("DCS.openbeta::$INDEX_ALLOCATION", "DCS.openbeta:$I30:$INDEX_ALLOCATION/Probe",
                 "DCS.[o]penbeta", "DCS.openbet?", "DCS.openbet*", "Probe<x", "Probe|x", "DCS.openbeta.", "DCS.openbeta "):
        result = run(write_dir=name)
        check(f"runner rejects the profile name {name!r}",
              result.returncode != 0 and ("not allowed in a profile name" in result.stderr or "ending in a dot or space" in result.stderr)
              and "prelaunch guards passed" not in result.stdout, result.stdout + result.stderr)
    result = run(default_name="DCS.[o]penbeta")
    check("runner rejects a wildcard in -DefaultWriteDirName", result.returncode != 0 and "not allowed in a profile name" in result.stderr,
          result.stdout + result.stderr)
    arbitrary = os.path.join(tmp, "arbitrary.txt")
    with open(arbitrary, "w") as stream:
        stream.write("any existing file")
    result = run(evidence_path=arbitrary)
    check("runner rejects arbitrary existing evidence file", result.returncode != 0, result.stdout + result.stderr)
    result = run(stage="load", evidence_path=arbitrary)
    check("load guard does not require error controls", result.returncode == 0 and "prelaunch guards passed" in result.stdout,
          result.stdout + result.stderr)
    for mismatch, field in (("candidate", "candidate_eech_dc_sha256"), ("host Lua", "dcs_lua_sha256")):
        changed = dict(record)
        changed[field] = "0" * 64
        with open(evidence, "w") as stream:
            json.dump(changed, stream)
        result = run()
        check(f"runner rejects wrong {mismatch} hash", result.returncode != 0 and "does not show passing" in result.stderr,
              result.stdout + result.stderr)
    # the record is untrusted JSON: coerced types, arrays and look-alikes must not pass
    def variant(change):
        copy = json.loads(json.dumps(record))
        change(copy)
        return copy

    def both(control, **fields):
        def change(copy):
            for name in ("first_error", "repeated_errors"):
                copy["controls"][name] = dict(copy["controls"][name], **fields) if fields else control
        return change

    typed = {
        "array hashes and array fields": lambda c: (c.update(candidate_eech_dc_sha256=[], dcs_lua_sha256=[]),
                                                    both(None, completed=[], exit_code=[])(c)),
        "array in place of the controls": lambda c: c.update(controls=[]),
        "array record": None,
        "numeric hash": lambda c: c.update(candidate_eech_dc_sha256=123),
        "short hash": lambda c: c.update(dcs_lua_sha256=c["dcs_lua_sha256"][:63]),
        'completed "true"': both(None, completed="true"),
        "completed 1": both(None, completed=1),
        "completed false": both(None, completed=False),
        "exit_code false": both(None, exit_code=False),
        'exit_code "0"': both(None, exit_code="0"),
        "exit_code 0.0": both(None, exit_code=0.0),
        "exit_code null": both(None, exit_code=None),
        "first_error missing": lambda c: c["controls"].pop("first_error"),
        "repeated_errors not an object": lambda c: c["controls"].update(repeated_errors=[]),
    }
    for label, change in typed.items():
        with open(evidence, "w") as stream:
            json.dump([] if change is None else variant(change), stream)
        result = run()
        check(f"runner rejects evidence with {label}", result.returncode != 0 and "does not show passing" in result.stderr,
              result.stdout + result.stderr)
    with open(evidence, "w") as stream:
        json.dump(variant(lambda c: c.update(candidate_eech_dc_sha256=c["candidate_eech_dc_sha256"].upper())), stream)
    result = run()
    check("runner accepts a matching upper-case hash (hex is case-insensitive)",
          result.returncode == 0 and "prelaunch guards passed" in result.stdout, result.stdout + result.stderr)
    record["controls"]["first_error"]["exit_code"] = 1
    with open(evidence, "w") as stream:
        json.dump(record, stream)
    result = run()
    check("runner rejects failing native error control", result.returncode != 0 and "does not show passing" in result.stderr,
          result.stdout + result.stderr)
    check("prelaunch tests never create a run directory", not os.path.exists(work))
    os.remove(default_log)
    result = run(stage="load")
    check("runner rejects a missing default-profile log before writes",
          result.returncode != 0 and "to watch" in result.stderr and not os.path.exists(work),
          result.stdout + result.stderr)
    lifecycle = subprocess.run(["pwsh", "-NoProfile", "-File", os.path.join(HERE, "runner-lifecycle-selftest.ps1")],
                               capture_output=True, text=True)
    check("runner cleans owned descendants when the root is running or exited",
          lifecycle.returncode == 0 and "cleanup passed" in lifecycle.stdout,
          lifecycle.stdout + lifecycle.stderr)


FAKE_DCS = r'''
-- fake DCS mission-state globals and a fake eech_dc; arg[1] probe, arg[2] scenario, arg[3] stage
local lines = {}
env = { info = function (s) lines[#lines + 1] = s end }
local now = 0
timer = { getTime = function () return now end, getAbsTime = function () return 43200 end,
          scheduleFunction = function (f) scheduled = f end }
world = { event = { S_EVENT_MISSION_END = 12 }, addEventHandler = function (h) event_handler = h end,
          getPersistenceData = function () return nil end,
          setPersistenceHandler = function (_, f) save_persistence = f end }
local scenario = arg[2]
if scenario == "log_write_failure" then
  io.open = function () return { write = function () return nil, "synthetic disk full" end,
                                 close = function () end } end
end
package.preload.eech_dc = function ()
  if scenario == "module_boolean" then return true end
  if scenario == "require_error" then error ("module unavailable") end
  return { name = "fake 0", prepare_installation = function ()
      if scenario == "prepare_error" then error ("prepare failed") end
    end,
    boot = function (config)
      if type (config) ~= "table" then error ("bad argument #1: error converting Lua string to table") end
      if scenario == "boot_error" then error ("boot failed") end
      return setmetatable ({}, { __index = {
        frame = function () if scenario == "frame_error" then error ("frame failed") end end,
        objects = function ()
          if scenario == "bad" then return { { alive = true, kind = nil, position = { 1 } } } end
          return { { alive = true, kind = "keysite", position = { 1, 2, 3 } } }
        end,
        clock = function () return { elapsed_seconds = 1, day = 1 } end } })
    end }
end
EECH_DCS_PROBE = { stage = arg[3], root = "x", summary_every = 0.1,
                   dll_dir = scenario == "dll_dir" and "C:/synthetic" or nil,
                   log_file = scenario == "log_open_failure" and "missing-dir/probe.log"
                       or ((scenario == "log_file" or scenario == "log_write_failure") and "probe.log" or nil) }
dofile (arg[1])
if scenario == "dll_dir" then print ("CPATH_PREFIX=" .. package.cpath) end
if save_persistence then save_persistence () end
if scheduled then
  if scenario == "mission_end" then
    event_handler:onEvent ({ id = world.event.S_EVENT_MISSION_END })
    print ("SCHEDULE_STOP=" .. tostring (scheduled ()))
  else
    now = 0.25
    print ("SCHEDULE_RETURN=" .. tostring (scheduled ()))
    if scenario == "good" then now = 0.5; print ("SCHEDULE_RETURN=" .. tostring (scheduled ())) end
  end
end
for _, l in ipairs (lines) do print (l) end
'''

FAKE_GUI = r'''
-- fake DCS GUI-state globals for the hook; arg[1] hook
log = { INFO = 1, write = function (_, _, s) print (s) end }
local callbacks, real_time, paused = nil, 0, true
DCS = { setUserCallbacks = function (c) callbacks = c; print ("callbacks set") end,
        getRealTime = function () return real_time end, getModelTime = function () return 0 end,
        getPause = function () return paused end, getMissionName = function () return "probe" end,
        setPause = function (v) paused = v; print ("setPause=" .. tostring (v)) end,
        exitProcess = function () print ("exitProcess") end }
lfs = { writedir = function () return "w" end }
dofile (arg[1])
if callbacks then
  callbacks.onSimulationStart ()
  real_time = 1; callbacks.onSimulationFrame ()
  real_time = 21; callbacks.onSimulationFrame ()
  real_time = 22; callbacks.onSimulationFrame ()
end
'''


TIMING = r'''
-- a controlled os.clock (seconds): the driver sets it per callback, and the fake engine adds its
-- costs; arg[1] probe, arg[2] mode
local probe, mode = arg[1], arg[2]
local now, wall, scheduled = 0, 0, nil
os.clock = function () return wall end
env = { info = function (line) if mode == "error" then wall = wall + 0.002 end; print (line) end }
timer = { getTime = function () return now end, getAbsTime = function () return 0 end,
          scheduleFunction = function (f) scheduled = f end }
world = { event = {}, addEventHandler = function () end }
package.preload.eech_dc = function () return {
  name = "timing fake", prepare_installation = function () end,
  boot = function (config)
    if type (config) ~= "table" then error ("bad argument") end
    return {
      frame = function ()
        if mode == "cost" then wall = wall + 0.001 end
        if mode == "error" then wall = wall + 0.007; error ("slow frame error") end
      end,
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
elseif mode == "error" then
  now, wall = 0.25, 1
  print ("SCHEDULE_STOP=" .. tostring (scheduled ()))
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

    out = run("error")
    check("error: failing scheduled callback stops", "SCHEDULE_STOP=nil" in out, out)
    check("error: the final summary includes the failing callback's measured cost",
          "event=frames_final" in out and "callbacks_completed=1" in out and "callback_max_ms=9.0" in out, out)
    check("error: terminal cost includes the final summary write",
          "event=callback_final" in out and "cost_ms=11.0" in out, out)


def lua_tests(tmp, lua):
    probe = os.path.join(HERE, "eech-dcs-probe.lua")
    fake = os.path.join(tmp, "fake-dcs.lua")
    open(fake, "w").write(FAKE_DCS)

    def run(scenario, stage):
        result = subprocess.run([lua, fake, probe, scenario, stage], capture_output=True, text=True, cwd=tmp)
        return result.returncode, result.stdout + result.stderr

    code, out = run("good", "frames")
    check("probe frames stage runs to the end", code == 0, out)
    for event in ("env", "persistence_handler", "load", "error_path_begin", "error_path", "boot", "frames_start", "checkpoint", "frames_summary"):
        check(f"probe records event={event}", f"event={event} " in out or out.rstrip().endswith(f"event={event}"), out)
    check("the module error is recorded as raised, not a crash", "event=error_path message=" in out and "raised=true" in out, out)
    check("the error-path begin marker precedes the result", out.find("event=error_path_begin") < out.find("event=error_path message="))
    rows = summaries(out)
    check("second window separates cumulative and window frames",
          len(rows) == 2 and rows[0].get("frames") == "2" and rows[0].get("frames_window") == "2"
          and rows[1].get("frames") == "5" and rows[1].get("frames_window") == "3", str(rows))
    check("successful scheduled callbacks return the next model times",
          [l for l in out.splitlines() if l.startswith("SCHEDULE_RETURN=")] == ["SCHEDULE_RETURN=0.3", "SCHEDULE_RETURN=0.55"], out)
    check("values with spaces are quoted", 'name="fake 0"' in out, out)

    code, out = run("bad", "frames")
    check("a malformed engine object is recorded as checkpoint_error", code == 0 and "event=checkpoint_error" in out, out)
    check("frames still run after a checkpoint error", "event=frames_summary" in out, out)

    code, out = run("good", "nonsense")
    check("an unknown stage is a config_error", code == 0 and "event=config_error" in out, out)

    code, out = run("module_boolean", "load")
    check("non-table module is recorded without indexing it",
          code == 0 and "event=load" in out and "ok=false" in out and "expected_table_got_boolean" in out, out)
    code, out = run("require_error", "load")
    check("require failure is recorded", code == 0 and "event=load" in out and "module unavailable" in out, out)
    code, out = run("dll_dir", "load")
    check("configured DLL directory is prepended to cpath", code == 0 and "CPATH_PREFIX=C:/synthetic\\?.dll;" in out, out)
    code, out = run("prepare_error", "boot")
    check("failed preparation is recorded and blocks boot", code == 0 and "event=prepare" in out
          and "prepare failed" in out and "event=boot" not in out, out)
    code, out = run("boot_error", "boot")
    check("failed boot is recorded", code == 0 and "event=boot" in out and "boot failed" in out, out)
    code, out = run("log_open_failure", "env")
    check("failed log open is visible in dcs.log output", code == 0 and "event=log_status" in out and "opened=false" in out, out)
    code, out = run("log_file", "env")
    log_path = os.path.join(tmp, "probe.log")
    log_contents = ""
    if os.path.exists(log_path):
        with open(log_path) as stream:
            log_contents = stream.read()
    check("log_file receives and flushes probe records", code == 0 and "event=env" in log_contents, out)
    code, out = run("log_write_failure", "env")
    check("failed log write is visible in dcs.log output",
          code == 0 and "event=log_error" in out and "synthetic disk full" in out, out)
    code, out = run("frame_error", "frames")
    check("frame error writes a final summary and stops scheduling",
          code == 0 and "event=frame_error" in out and "event=frames_final" in out and "SCHEDULE_RETURN=nil" in out, out)
    code, out = run("mission_end", "frames")
    check("mission end writes final summary and stops scheduling",
          code == 0 and "event=frames_final" in out and "event=event_handler_cost" in out
          and "SCHEDULE_STOP=nil" in out, out)
    check("no-sample maxima are represented by dash",
          "frame_max_ms=-" in out and "gap_max_ms=-" in out and "lag_max_ms=-" in out, out)
    check("persistence handler writes a save record when called", "event=persistence_save" in out, out)

    hook = os.path.join(HERE, "eech-m5probe-hook.lua")
    gui = os.path.join(tmp, "fake-gui.lua")
    open(gui, "w").write(FAKE_GUI)
    environment = {k: v for k, v in os.environ.items() if k != "EECH_M5PROBE_RUN_SECONDS"}
    out = subprocess.run([lua, gui, hook], capture_output=True, text=True, env=environment).stdout
    check("the hook is inert without EECH_M5PROBE_RUN_SECONDS", "inactive" in out and "callbacks set" not in out, out)
    out = subprocess.run([lua, gui, hook], capture_output=True, text=True, env={**environment, "EECH_M5PROBE_RUN_SECONDS": "20"}).stdout
    check("the hook sets its callbacks when the runner set the bound", "callbacks set" in out and "run_seconds=20" in out, out)
    check("the hook releases briefing pause once and exits at its bound once",
          out.splitlines().count("setPause=false") == 1 and out.splitlines().count("exitProcess") == 1, out)


def main(argv):
    if len(argv) != 2 or (argv[1].startswith("-") and argv[1] != "--skip-lua"):
        print("usage: dcs-probe-selftest.py <lua 5.1 interpreter> | --skip-lua", file=sys.stderr)
        return 2
    skip_lua = argv[1] == "--skip-lua"
    with tempfile.TemporaryDirectory() as tmp:
        builder_tests(tmp, None if skip_lua else argv[1])
        pe_tests(tmp)
        retained_tests()
        runner_guard_tests(tmp)
        if not skip_lua:
            lua_tests(tmp, argv[1])
            timing_tests(tmp, argv[1])
    if failures:
        print(f"{len(failures)} failed")
        return 1
    print("builder, PE, retained-evidence and runner tests passed; Lua tests NOT RUN (--skip-lua)" if skip_lua else "all passed")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
