"""The public driver (entity-authority.lua): phases 17 and 18 (a sector's height read and restore) are parsed, forwarded and their replies checked, never altered. Byte-exact, count-checked."""
import sys

ROOT = sys.argv[1]
P = ROOT + "/eech-campaign/crates/eech-world/lua/entity-authority.lua"
d = open(P, "rb").read().decode("utf-8")
assert b"\r\n" not in open(P, "rb").read()


def rep(old, new):
    global d
    assert d.count(old) == 1, (old[:90], d.count(old))
    d = d.replace(old, new)


rep('["0x41700000"] = 15, ["0x41800000"] = 16 }', '["0x41700000"] = 15, ["0x41800000"] = 16, ["0x41880000"] = 17, ["0x41900000"] = 18 }')
rep('is not 0 to 16" end', 'is not 0 to 18" end')
rep("""	city_words = { [1] = true, [2] = true, [3] = true, [4] = true, [5] = true, [6] = true, [7] = true, [8] = true, [9] = true, [11] = true, [12] = true } }""",
    """	city_words = { [1] = true, [2] = true, [3] = true, [4] = true, [5] = true, [6] = true, [7] = true, [8] = true, [9] = true, [10] = true, [11] = true, [12] = true } }""")
rep("""a city state report states nothing but the position, the box minimum, the box maximum, the sub type and its phase" end""",
    """a city state report states nothing but the position, the box minimum, the box maximum, the shape, the sub type and its phase" end""")

rep("""	if phase >= 7 then
		-- a CITY event: nothing is stated but the phase""", """	if phase == 17 or phase == 18 then
		-- a sector height READ or RESTORE (Task49): the sector's x and z (facts 12 and 13: words 13 and 14), a restore's saved word RAW in fact 0 (word 1) and the phase; nothing else, and no identity place
		local stated = { [12] = true, [13] = true, [14] = true }
		if phase == 18 then stated [1] = true end
		if not others_zero (words, stated) then return nil, "a sector height " .. (phase == 18 and "restore" or "read") .. " states nothing but the sector, its phase and, for a restore, the word" end
		if identity ~= "" then return nil, "a sector height names no placement" end
		return { kind = "sector_height", words = words, phase = phase, event = phase == 18 and "sector_restore" or "sector_height" }
	end
	if phase >= 7 then
		-- a CITY event: nothing is stated but the phase""")

rep("""scan.fixed.expand_total, scan.fixed.expand_answered, scan.fixed.box_withdrawn, scan.fixed.box_withdrawn_key, scan.fixed.box_eligible = 0, 0, 0, "", 0
""", """scan.fixed.expand_total, scan.fixed.expand_answered, scan.fixed.box_withdrawn, scan.fixed.box_withdrawn_key, scan.fixed.box_eligible = 0, 0, 0, "", 0
-- the SECTOR HEIGHT reads and restores (phases 17 and 18, Task49), kept in the same table: the questions seen in either window and those answered well-formed. The word is the host's; this only counts and checks it
scan.fixed.height_total, scan.fixed.height_answered, scan.fixed.restore_total, scan.fixed.restore_answered = 0, 0, 0, 0
""")

rep("""-- the verdict of withdrawn_box_state (nil when credited)""", """-- the first problem of a REAL sector height reply against the contract, or nil: `height` for a read and `restored` for a restore, no placement named, exactly three patterns (the sector's x and z as asked, then the word),
-- and for a restore the word as stated. The word itself is never classified: a negative, infinite or NaN word is valid
function scan.fixed.height_reply_problem (answer, parsed)
	if type (answer) ~= "table" then return "the reply is not a table" end
	if answer.result == "refused" then return "result refused" end
	local wanted = parsed.phase == 18 and "restored" or "height"
	if answer.result ~= wanted then return "a sector height " .. (parsed.phase == 18 and "restore" or "read") .. " answered " .. tostring (answer.result) .. ", not " .. wanted end
	if answer.name ~= nil and answer.name ~= "" then return "a sector height named the placement \\"" .. shown (answer.name) .. "\\"" end
	local words = type (answer.p) == "string" and words_of_text (answer.p, 3) or nil
	if words == nil then return "a sector height answered words that are not three 0x%08x patterns" end
	if words [1] ~= parsed.words [13] or words [2] ~= parsed.words [14] then return "a sector height answered for another sector than the one asked about" end
	if parsed.phase == 18 and words [3] ~= parsed.words [1] then return "the host stored a word for the restore of a sector's height that is not the word stated" end
	return nil
end
-- the verdict of withdrawn_box_state (nil when credited)""")

rep("""	-- the boot-phase mode asks everything in the window before the launch: a malformed question is a request fault there too""", """	-- a SECTOR HEIGHT read or restore (phases 17 and 18, Task49): counted in either window, its reply checked, never altered (the word is the host's: the campaign uses what the host answered and has no fallback)
	if parsed ~= nil and parsed.kind == "sector_height" then
		local f = scan.fixed
		local read = parsed.phase == 17
		if read then f.height_total = f.height_total + 1 else f.restore_total = f.restore_total + 1 end
		if armed and joined then
			faults.request = faults.request + 1
			first_fault = first_fault or ("scan request: " .. joined)
		end
		local height_problem = f.height_reply_problem (real, parsed)
		if height_problem == nil then
			if read then f.height_answered = f.height_answered + 1 else f.restore_answered = f.restore_answered + 1 end
		else
			faults.reply = faults.reply + 1
			first_fault = first_fault or ("scan reply: " .. height_problem)
		end
		return real
	end
	-- the boot-phase mode asks everything in the window before the launch: a malformed question is a request fault there too""")

rep("""city_expand_total = f.expand_total, city_expand_answered = f.expand_answered,
""", """city_expand_total = f.expand_total, city_expand_answered = f.expand_answered,
		sector_height_total = f.height_total, sector_height_answered = f.height_answered, sector_restore_total = f.restore_total, sector_restore_answered = f.restore_answered,
""")
rep("""city_expand_total = scan.fixed.expand_total, city_expand_answered = scan.fixed.expand_answered,
""", """city_expand_total = scan.fixed.expand_total, city_expand_answered = scan.fixed.expand_answered,
			sector_height_total = scan.fixed.height_total, sector_height_answered = scan.fixed.height_answered, sector_restore_total = scan.fixed.restore_total, sector_restore_answered = scan.fixed.restore_answered,
""")
open(P, "wb").write(d.encode("utf-8"))
print("ok")
