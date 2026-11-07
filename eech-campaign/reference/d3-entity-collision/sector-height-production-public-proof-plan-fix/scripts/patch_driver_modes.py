"""The public driver (entity-authority.lua): the sector height AUTHORITY and REFUSAL modes. Real consuming request attribution, eligibility, consumption proof, named outcomes. Byte-exact, count-checked."""
import sys

ROOT = sys.argv[1]
P = ROOT + "/eech-campaign/crates/eech-world/lua/entity-authority.lua"
raw = open(P, "rb").read()
assert b"\r\n" not in raw
d = raw.decode("utf-8")


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


# ---- the modes (header table, before `local MODES = {`)
rep("""local MODES = {
	control = { altering = false },""", """-- the three patterns of a sector height reply's `p`, as an array of text (nil when it is not a text)
local function p_words (text)
	if type (text) ~= "string" then return nil end
	local out = {}
	for token in string.gmatch (text, "[^,]+") do out [#out + 1] = token end
	return out
end
-- the word one unit away, for an altered pattern (Lua 5.1 has no bit operations): the low bit flipped
local function other_pattern (pattern, delta)
	local n = tonumber (string.sub (pattern, 3), 16) + delta
	return string.format ("0x%08x", n)
end
local MODES = {
	control = { altering = false },""")

rep("""MODES.scan_real = { answer = MODES.forced_hit.answer, paired = true }
""", """MODES.scan_real = { answer = MODES.forced_hit.answer, paired = true }
-- the SECTOR HEIGHT modes (Task49, the host-owned sector height; hosted profile, needs weapon-fixture). The campaign's own getter asks the host for a sector's word (scan phase 17) at the first read of a scan FIRST; the wrapper
-- credits only an ELIGIBLE read: ARMED (after the case launch), joined to the real FIRST answer that just named that very sector (the cursor words equal, once), the served entry a NON-MOBILE placement of the host's roster
-- (the entity types the host's candidate question compares the word for), the reply complete (the state, three patterns, no name). The configured secondary input is the existing forced-hit entity_collision answer, held as in scan_real.
--   sector_height_control       the control: nothing is altered. Credited only with at least one eligible read whose word a candidate question of the served entry carried back as fact 12 (consumption), no mismatch
--                               between the word answered and the word consumed, the restore (phase 18) reported but NOT required (a real Core save/load is not reached in the weapon window: UNREACHED, never credited)
--   sector_height_wrong_word    the host's changed input: the first eligible read is answered with the word one unit away; the campaign must CARRY that word in the next candidate question of the entry (consumption)
--                               and the provider's own named refusal [candidate.sector_height] is the only fault
--   sector_height_withheld      the provider raises at the first eligible read:                          [model-provider:entity_scan.callback]
--   sector_height_malformed     two patterns instead of three (refused by the bridge's list check):    [model-provider:entity_scan.answer]
--   sector_height_wrong_state   `restored` for a read:                                                  [model-provider:entity_scan.state]
--   sector_height_wrong_sector  the sector echoed one unit away:                                        [model-provider:entity_scan.sector_height]
-- No actor, ray, duration or seed is searched: a window with no eligible read is UNREACHED. After a fatal stop nothing is read from the engine.
MODES.sector_height_control = { answer = MODES.forced_hit.answer, paired = true, height_control = true }
MODES.sector_height_wrong_word = {
	answer = MODES.forced_hit.answer, paired = true, marker = "entity_candidate.refused", consume = true,
	height = function (_, real) local w = p_words (real.p) return { result = "height", p = w [1] .. "," .. w [2] .. "," .. other_pattern (w [3], tonumber (string.sub (w [3], 3), 16) % 2 == 1 and -1 or 1) } end,
}
MODES.sector_height_withheld = {
	answer = MODES.forced_hit.answer, paired = true, marker = "entity_scan.callback",
	height = function () error ("the host withholds its sector height answer", 0) end,
}
MODES.sector_height_malformed = {
	answer = MODES.forced_hit.answer, paired = true, marker = "entity_scan.answer",
	height = function (_, real) local w = p_words (real.p) return { result = "height", p = w [1] .. "," .. w [2] } end,
}
MODES.sector_height_wrong_state = {
	answer = MODES.forced_hit.answer, paired = true, marker = "entity_scan.state",
	height = function (_, real) return { result = "restored", p = real.p } end,
}
MODES.sector_height_wrong_sector = {
	answer = MODES.forced_hit.answer, paired = true, marker = "entity_scan.sector_height",
	height = function (_, real) local w = p_words (real.p) return { result = "height", p = other_pattern (w [1], 1) .. "," .. w [2] .. "," .. w [3] } end,
}
""")

# ---- the counters and the entity types the host compares the word for
rep("""scan.fixed.height_total, scan.fixed.height_answered, scan.fixed.restore_total, scan.fixed.restore_answered = 0, 0, 0, 0
""", """scan.fixed.height_total, scan.fixed.height_answered, scan.fixed.restore_total, scan.fixed.restore_answered = 0, 0, 0, 0
-- the sector height reads ATTRIBUTED to the real consumer (modes sector_height_*): the roster entity types the host stated at each INSERT (the host's candidate question compares the word for the fixed family and a city), the FIRST
-- answer awaiting its read, every read, the reads joined to a FIRST, the ELIGIBLE ones, the reads of any other caller (the clearance, the pack), the one altered answer, and the candidate questions that carried the word back (fact 12)
scan.height = { types = {}, pending = nil, reads = 0, scan_joined = 0, eligible = 0, other = 0, altered = 0, consumed = 0, consumed_altered = 0, mismatch = 0 }
local FIXED_TYPE_WORDS = { ["0x40a00000"] = true, ["0x40c00000"] = true, ["0x40e00000"] = true, ["0x41880000"] = true, ["0x41a80000"] = true, ["0x41c00000"] = true, ["0x41d00000"] = true, ["0x41e80000"] = true, ["0x41f00000"] = true }
""")

rep("""	if parsed.event == "insert" then
		roster_live [answer.name] = true
""", """	if parsed.event == "insert" then
		roster_live [answer.name] = true
		scan.height.types [answer.name] = parsed.words [4]
""")

# ---- the FIRST answer awaiting its read
rep("""		scan [real.result] = scan [real.result] + 1
		if real.result == "first" or real.result == "next" then""", """		scan [real.result] = scan [real.result] + 1
		if real.result == "first" and type (real.p) == "string" then
			local cursor = words_of_text (real.p, 2)
			if cursor then scan.height.pending = { x = cursor [1], z = cursor [2], key = real.name or "", read = false } end
		end
		if real.result == "first" or real.result == "next" then""")

# ---- attribution, eligibility and the one intervention
rep("""			faults.reply = faults.reply + 1
			first_fault = first_fault or ("scan reply: " .. height_problem)
		end
		return real
	end""", """			faults.reply = faults.reply + 1
			first_fault = first_fault or ("scan reply: " .. height_problem)
		end
		if read and height_problem == nil then
			local h = scan.height
			local pend = h.pending
			local eligible = false
			h.reads = h.reads + 1
			if pend ~= nil and not pend.read and parsed.words [13] == pend.x and parsed.words [14] == pend.z then
				pend.read = true
				h.scan_joined = h.scan_joined + 1
				if armed and pend.key ~= "" and FIXED_TYPE_WORDS [h.types [pend.key] or ""] then
					eligible = true
					pend.eligible = true
					h.eligible = h.eligible + 1
				end
			else
				h.other = h.other + 1
			end
			local answer = real
			if eligible and plan.height and h.altered == 0 then
				h.altered = 1
				pend.altered = true
				answer = plan.height (parsed, real)
			end
			if eligible then
				local given = p_words (answer.p)
				pend.word = given and given [3] or nil
			end
			return answer
		end
		return real
	end""")

# ---- consumption by the candidate question of the served entry
rep("""				if parsed.host_id ~= served then
					faults.request = faults.request + 1
					first_fault = first_fault or ("candidate request: the candidate names the placement " .. shown (parsed.host_id) .. ", which is not the entry the host last served (" .. tostring (served) .. ")")
				else
					joined_now = true
				end""", """				if parsed.host_id ~= served then
					faults.request = faults.request + 1
					first_fault = first_fault or ("candidate request: the candidate names the placement " .. shown (parsed.host_id) .. ", which is not the entry the host last served (" .. tostring (served) .. ")")
				else
					joined_now = true
					-- the word the campaign was ANSWERED for the sector of this entry is the one it must carry back as fact 12 of every question about the entry (consumption: a changed answer must show up here)
					local h, pend = scan.height, scan.height.pending
					if parsed.words [11] == ZERO and pend ~= nil and pend.eligible and pend.word ~= nil and parsed.host_id == pend.key then
						if parsed.words [13] == pend.word then
							h.consumed = h.consumed + 1
							if pend.altered then h.consumed_altered = h.consumed_altered + 1 end
						else
							h.mismatch = h.mismatch + 1
							faults.request = faults.request + 1
							first_fault = first_fault or ("candidate request: fact 12 " .. shown (parsed.words [13]) .. " is not the word the host answered for the sector (" .. shown (pend.word) .. ")")
						end
					end
				end""")

# ---- the row fields (both emit sites)
rep("""sector_height_total = f.height_total, sector_height_answered = f.height_answered, sector_restore_total = f.restore_total, sector_restore_answered = f.restore_answered,
""", """sector_height_total = f.height_total, sector_height_answered = f.height_answered, sector_restore_total = f.restore_total, sector_restore_answered = f.restore_answered,
		height_reads = scan.height.reads, height_scan_joined = scan.height.scan_joined, height_eligible = scan.height.eligible, height_other_callers = scan.height.other, height_altered = scan.height.altered,
		height_consumed = scan.height.consumed, height_consumed_altered = scan.height.consumed_altered, height_mismatch = scan.height.mismatch,
""")
rep("""sector_height_total = scan.fixed.height_total, sector_height_answered = scan.fixed.height_answered, sector_restore_total = scan.fixed.restore_total, sector_restore_answered = scan.fixed.restore_answered,
""", """sector_height_total = scan.fixed.height_total, sector_height_answered = scan.fixed.height_answered, sector_restore_total = scan.fixed.restore_total, sector_restore_answered = scan.fixed.restore_answered,
			height_reads = scan.height.reads, height_scan_joined = scan.height.scan_joined, height_eligible = scan.height.eligible, height_other_callers = scan.height.other, height_altered = scan.height.altered,
			height_consumed = scan.height.consumed, height_consumed_altered = scan.height.consumed_altered, height_mismatch = scan.height.mismatch,
""")

# ---- the verdicts: refusal modes
rep("""		if failure == nil then return "mode " .. mode .. " completed: the engine did not stop at " .. plan.marker end
		if failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [model-provider:" .. failure.marker .. "], not at [model-provider:" .. plan.marker .. "]: " .. failure.line end
		if altered + cand_altered + scan_altered + pose_altered + withdrawn + displaced + math.max (scan.withdrawn, 0)""", """		if plan.height and scan.height.altered == 0 and (faults.request > 0 or faults.reply > 0) then return "mode " .. mode .. ": no sector height answer was altered because the run failed first (request_faults=" .. faults.request .. " reply_faults=" .. faults.reply .. "): " .. tostring (first_fault) end
		if plan.height and scan.height.altered == 0 and failure ~= nil and failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [model-provider:" .. failure.marker .. "], not at [model-provider:" .. plan.marker .. "], before an eligible sector height read was altered: " .. failure.line end
		if plan.height and scan.height.altered == 0 and failure == nil then return "UNREACHED: " .. mode .. ": no ELIGIBLE sector height read (" .. scan.height.reads .. " reads, " .. scan.height.scan_joined .. " joined to a scan FIRST, " .. scan.height.eligible .. " armed with a served non-mobile entry, " .. scan.height.other .. " by another caller): nothing was altered, and no actor, time or geometry is searched for" end
		if plan.height and plan.consume and scan.height.consumed_altered < 1 then return "mode " .. mode .. " stopped at its marker, but no candidate question of the served entry carried the altered word as fact 12 (consumption not shown: height_consumed_altered=" .. scan.height.consumed_altered .. ")" end
		if plan.height and (faults.request ~= 0 or faults.reply ~= 0 or scan.height.mismatch ~= 0) then return "mode " .. mode .. " had a fault before the stop (request_faults=" .. faults.request .. " reply_faults=" .. faults.reply .. " mismatch=" .. scan.height.mismatch .. "): " .. tostring (first_fault) end
		if failure == nil then return "mode " .. mode .. " completed: the engine did not stop at " .. plan.marker end
		if failure.marker ~= plan.marker then return "mode " .. mode .. " stopped at [model-provider:" .. failure.marker .. "], not at [model-provider:" .. plan.marker .. "]: " .. failure.line end
		if plan.height and plan.consume and not string.find (failure.line, "[candidate.sector_height]", 1, true) then return "mode " .. mode .. " stopped at its marker, but not for a candidate word that is not the host's: " .. failure.line end
		if altered + cand_altered + scan_altered + pose_altered + withdrawn + displaced + scan.height.altered + math.max (scan.withdrawn, 0)""")

# ---- the verdict: the control
rep("""	if mode == "scan_real" then
		-- the paired control: the real enumeration must have offered entries""", """	if mode == "sector_height_control" then
		-- the paired control of scan_real, plus the sector height: an ELIGIBLE read (armed, joined to the FIRST that named the sector, a non-mobile served entry) whose word a candidate question of that entry carried back
		-- as fact 12; nothing altered, no mismatch. The restore (phase 18) is reported in the row and is NOT required: a real Core save/load is not reached in the weapon window
		if scan_altered ~= 0 or cand_altered ~= 0 or scan.height.altered ~= 0 then return "sector_height_control altered an answer" end
		if scan.offer < 1 or cand.test < 1 or after.entity < 1 then return "UNREACHED: the real enumeration never led to a narrow phase (" .. scan.offer .. " offers, " .. cand.test .. " test decisions, " .. after.entity .. " entity questions)" end
		if scan.height.eligible < 1 then return "UNREACHED: sector_height_control: no ELIGIBLE sector height read (" .. scan.height.reads .. " reads, " .. scan.height.scan_joined .. " joined to a scan FIRST, " .. scan.height.eligible .. " armed with a served non-mobile entry, " .. scan.height.other .. " by another caller): nothing was credited, and no actor, time or geometry is searched for" end
		if scan.height.consumed < 1 then return "UNREACHED: sector_height_control: " .. scan.height.eligible .. " eligible reads, but no candidate question of a served entry carried the word as fact 12 (height_consumed=0)" end
		if scan.height.mismatch ~= 0 then return "sector_height_control: " .. scan.height.mismatch .. " candidate questions carried a word that is not the one the host answered" end
		if altered ~= after.entity or seen.answered_hit ~= after.entity or seen.answered_miss ~= 0 then return string.format ("sector_height_control answered %d hits and %d misses of %d entity questions", seen.answered_hit, seen.answered_miss, after.entity) end
		return nil
	end
	if mode == "scan_real" then
		-- the paired control: the real enumeration must have offered entries""")
open(P, "wb").write(d.encode("utf-8"))
print("ok")
