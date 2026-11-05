"""Make an observational copy of shipping campaign.lua for the final D2 R4 probe, exporting all raw per-frame samples.

This is not the regression runner or a replacement acceptance script. It only
times the existing engine:frame call. Final ordinary regression uses unmodified
shipping scripts. DCS callback/scheduling costs cannot be inferred from this.
"""
import argparse
import hashlib
import json
import shutil
from pathlib import Path

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("--campaign", type=Path, required=True)
p.add_argument("--out", type=Path, required=True)
a = p.parse_args()
original = a.campaign.read_bytes()
text = original.decode("utf-8").replace("\r\n", "\n")
anchor = "\tengine:frame (frame_ms)"
if text.count(anchor) != 1:
    raise SystemExit("Expected exactly one original engine:frame call")
if a.out.exists() or a.out.with_suffix(".source.json").exists():
    raise SystemExit("Refusing to replace an existing measurement script/provenance")
siblings = {}
for name in ("metrics.lua", "observations.lua"):
    source = a.campaign.parent / name
    target = a.out.parent / name
    raw = source.read_bytes()
    if target.exists() and target.read_bytes() != raw:
        raise SystemExit(f"Different existing companion: {target}")
    siblings[name] = hashlib.sha256(raw).hexdigest()
a.out.parent.mkdir(parents=True, exist_ok=True)
for name in siblings:
    target = a.out.parent / name
    if not target.exists():
        shutil.copyfile(a.campaign.parent / name, target)

prefix = r'''-- ROOT observational native-frame probe, D2 R4. Not shipping acceptance.
-- Measure only engine:frame, including its nested host callbacks. File writes,
-- objects/recording/metrics and outer-loop work are outside the measured span.
-- os.clock's Windows resolution limits individual values (M5 DCS risk F1).
local probe_path = assert(host.args.frame_cost, "frame_cost=<new absolute output>")
local probe_existing = io.open(probe_path, "rb")
if probe_existing then probe_existing:close(); error("frame_cost output exists") end
local probe_file = assert(io.open(probe_path, "wb"))
local probe_values, probe_sum, probe_frames = {}, 0, 0
local probe_window, probe_window_sum, probe_window_max = 0, 0, 0
probe_file:write('{"kind":"header","scope":"native engine:frame only; no DCS scheduling or budget","timer":"os.clock","unit":"ms","window_frames":600,"raw_frame_samples":true}\n')
probe_file:flush()
local function probe_add(ms)
  assert(ms >= 0 and ms < math.huge, "invalid elapsed frame cost")
  probe_frames = probe_frames + 1
  assert(probe_frames <= 200000, "frame measurement bound exceeded")
  probe_values[probe_frames] = ms
  -- Raw samples are exported after the timed call, so quantiles can be recomputed.
  probe_file:write(string.format('{"kind":"frame","frame":%d,"elapsed_ms":%.17g}\n', probe_frames, ms))
  probe_sum, probe_window_sum = probe_sum + ms, probe_window_sum + ms
  probe_window, probe_window_max = probe_window + 1, math.max(probe_window_max, ms)
  if probe_window == 600 then
    probe_file:write(string.format('{"kind":"window","through_frame":%d,"frames":%d,"mean_ms":%.9g,"max_ms":%.9g}\n', probe_frames, probe_window, probe_window_sum/probe_window, probe_window_max))
    probe_file:flush()
    probe_window, probe_window_sum, probe_window_max = 0, 0, 0
  end
end
'''
replacement = '''\tlocal probe_started = os.clock()
\tengine:frame (frame_ms)
\tlocal probe_elapsed = (os.clock() - probe_started) * 1000
\tprobe_add(probe_elapsed)'''
suffix = r'''
-- A final record proves this instrumented script reached normal completion.
if probe_window > 0 then
  probe_file:write(string.format('{"kind":"window","through_frame":%d,"frames":%d,"mean_ms":%.9g,"max_ms":%.9g}\n', probe_frames, probe_window, probe_window_sum/probe_window, probe_window_max))
end
assert(probe_frames > 0, "no frames were measured")
table.sort(probe_values)
local function probe_quantile(q)
  return probe_values[math.max(1, math.ceil(q * probe_frames))]
end
probe_file:write(string.format('{"kind":"complete","frames":%d,"mean_ms":%.9g,"max_ms":%.9g,"p50_ms":%.9g,"p95_ms":%.9g,"p99_ms":%.9g,"quantiles":"nearest rank; timer quantized"}\n', probe_frames, probe_sum/probe_frames, probe_values[probe_frames], probe_quantile(.5), probe_quantile(.95), probe_quantile(.99)))
assert(probe_file:close())
'''
generated = (prefix + text.replace(anchor, replacement) + suffix).encode("utf-8")
a.out.write_bytes(generated)
record = {
    "shipping_campaign": str(a.campaign.resolve()),
    "shipping_campaign_sha256": hashlib.sha256(original).hexdigest(),
    "instrumented_copy": str(a.out.resolve()),
    "instrumented_sha256": hashlib.sha256(generated).hexdigest(),
    "exact_companion_copies": siblings,
    "changes": "preamble, clock span around sole existing engine:frame, per-frame samples outside timed span, final statistics; original remaining script retained",
    "limits": "Native-only frame cost. Not DCS callback/scheduling/objects cost, not a budget or final regression acceptance. Compare complete resulting recordings separately.",
}
a.out.with_suffix(".source.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(json.dumps(record))
