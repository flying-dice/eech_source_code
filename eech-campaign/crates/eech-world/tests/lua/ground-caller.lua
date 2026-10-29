-- Stand-in for the engine in the tests of lua/ground-characterization.lua (EEDCS-48). The real driver runs against it; only the driver's own flow and its
-- checks of what a probe returns are exercised. The stand-in writes a COMPLETE, well-formed report for the request it is given (every row of the fixture's
-- schema, counts that add up, hex words of the right width) and, in a negative mode, ONE defect in it. It measures nothing about the ground queries.
-- Placeholders are filled in by the test: @MODE@, @TRACE@ and @LUA_DIR@.
local mode = '@MODE@'

local function note(text)
  local f = io.open('@TRACE@', 'a')
  f:write(text, '\n')
  f:close()
end

local time = 0
local probes = 0
local engine = {}
function engine:clock() return { elapsed_seconds = time } end
function engine:frame(ms) time = time + ms / 1000 note('frame') end
function engine:objects()
  return {
    { id = 2, kind = 'helicopter', side = 'blue', alive = true },
    { id = 1, kind = 'ground_vehicle', side = 'red', alive = false },
  }
end

local EMPTY = 'cbf29ce484222325'

-- a report as ordered rows of ordered key/value fields, so that a mode can change exactly one thing
local function build(seed, points, impacts)
  local function fold(cases, value) if cases == 0 then return EMPTY end return value end
  local up = math.floor(points / 2)
  local rows = {
    { 'ground-probe', { { 'seed', seed }, { 'points', points }, { 'impacts', impacts }, { 'samples_per_point_tag', 2 }, { 'samples_per_impact_type', 4 } } },
    { 'ground-map', { { 'sectors_x', 5 }, { 'sectors_z', 4 }, { 'sector_side', 2048 } } },
    { 'ground-record', { { 'anchored', 1 }, { 'restored', 1 }, { 'words_before', '0123456789abcdef' }, { 'words_after', '0123456789abcdef' } } },
    { 'ground-points', { { 'cases', points }, { 'in_map', points }, { 'off_map', 0 }, { 'input_mutated', 0 } } },
    { 'ground-above', { { 'cases', points }, { 'true', up }, { 'false', points - up }, { 'record_touched', points }, { 'true_record_untouched', 0 },
      { 'warm_same', points }, { 'warm_start_intact', points } } },
    { 'ground-below', { { 'cases', points }, { 'true', up }, { 'false', points - up }, { 'record_touched', points }, { 'true_record_untouched', 0 },
      { 'warm_same', points }, { 'warm_start_intact', points }, { 'off_map_true', 0 }, { 'off_map_record_touched', 0 } } },
    { 'ground-complement', { { 'in_map_cases', points }, { 'complement', points }, { 'not_complement', 0 }, { 'both_true', 0 }, { 'both_false', 0 } } },
    { 'ground-impact', { { 'cases', impacts }, { 'a_corrected', 0 }, { 'b_corrected', 0 }, { 'xz_changed', 0 }, { 'alias_equal', impacts },
      { 'alias_unequal', 0 }, { 'warm_same', impacts }, { 'warm_different', 0 }, { 'record_touched', impacts }, { 'record_untouched', 0 },
      { 'impact_equals_corrected_below', 0 }, { 'warm_start_intact', impacts } } },
    { 'ground-impact-classes', { { 'invalid', 0 }, { 'water', 0 }, { 'forest', 0 }, { 'land', impacts }, { 'other', 0 } } },
    { 'ground-impact-types', { { 't0', impacts }, { 't1', 0 }, { 't2', 0 }, { 't3', 0 }, { 't4', 0 }, { 't5', 0 }, { 't6', 0 }, { 't7', 0 } } },
    { 'ground-asserts', { { 'original_below_endpoint_still_above', 0 }, { 'extra_above_endpoint_not_above', 0 } } },
    -- a hash word of decimal digits only must stay text
    { 'ground-hash', { { 'combined', '1234567890123456' }, { 'above', fold(points, '0000000000000001') }, { 'below', fold(points, '0000000000000002') },
      { 'impact', fold(impacts, '3000000000000000') } } },
    { 'ground-physical', { { 'combined', 'fedcba9876543210' }, { 'above', fold(points, '1111111111111111') }, { 'below', fold(points, '2222222222222222') },
      { 'impact', fold(impacts, '3333333333333333') } } },
  }
  if points > 0 then
    rows[#rows + 1] = { 'ground-sample', { { 'kind', 'point' }, { 'n', 0 }, { 'tag', 0 }, { 'x', '40000000' }, { 'y', '3f800000' }, { 'z', '00000000' },
      { 'inside', 1 }, { 'above', 1 }, { 'above_touched', 0 }, { 'below', 0 }, { 'below_touched', 0 }, { 'above_elev', '3f800000' }, { 'above_type', 2 },
      { 'below_elev', '3f800000' }, { 'below_type', 2 } } }
  end
  if impacts > 0 then
    local v = '3f800000,40000000,3f800000'
    rows[#rows + 1] = { 'ground-sample', { { 'kind', 'impact' }, { 'n', 0 }, { 'type', 0 }, { 'a0', v }, { 'b0', v }, { 'class_alias', 3 }, { 'pa', v },
      { 'qa', v }, { 'class_separate', 3 }, { 'pb', v }, { 'qb', v }, { 'rb', v }, { 'alias_equal', 1 }, { 'warm_same', 1 }, { 'touched', 1 },
      { 'elev', '3f800000' }, { 'type_word', 2 } } }
  end
  return rows
end

local function row(rows, name, kind)
  for _, r in ipairs(rows) do
    if r[1] == name then
      if name ~= 'ground-sample' or r[2][1][2] == kind then return r end
    end
  end
  error('stand-in: no row ' .. name)
end

local function field(rows, name, key, kind)
  for _, f in ipairs(row(rows, name, kind)[2]) do if f[1] == key then return f end end
  error('stand-in: no key ' .. key .. ' in ' .. name)
end

local function set(rows, name, key, value, kind) field(rows, name, key, kind)[2] = value end

local function drop(rows, name, key, kind)
  local fields = row(rows, name, kind)[2]
  for i, f in ipairs(fields) do if f[1] == key then table.remove(fields, i) return end end
  error('stand-in: no key ' .. key .. ' in ' .. name)
end

local function drop_row(rows, name, kind)
  for i, r in ipairs(rows) do
    if r[1] == name and (name ~= 'ground-sample' or r[2][1][2] == kind) then table.remove(rows, i) return end
  end
  error('stand-in: no row ' .. name)
end

-- a word that is not key=value, appended to a row
local function add_word(rows, name, word) local fields = row(rows, name)[2] fields[#fields + 1] = { nil, word } end
local function add_field(rows, name, key, value) local fields = row(rows, name)[2] fields[#fields + 1] = { key, value } end

-- one defect per mode; the test names the exact message the driver must refuse it with
local modes = {
  ok = function() end,
  refuse = function() end,
  differs_between_repeats = function() end,
  unrestored = function(r, p, i) set(r, 'ground-record', 'restored', 0) end,
  words_differ = function(r) set(r, 'ground-record', 'words_after', 'ffffffffffffffff') end,
  not_anchored = function(r) set(r, 'ground-record', 'anchored', 0) end,
  wrongheader = function(r, points) set(r, 'ground-probe', 'points', points + 1) end,
  record_words_absent = function(r) drop(r, 'ground-record', 'words_before') drop(r, 'ground-record', 'words_after') end,
  record_before_absent = function(r) drop(r, 'ground-record', 'words_before') end,
  record_word_short = function(r) set(r, 'ground-record', 'words_after', 'abcd') end,
  record_word_upper = function(r) set(r, 'ground-record', 'words_before', '0123456789ABCDEF') set(r, 'ground-record', 'words_after', '0123456789ABCDEF') end,
  hash_bare = function(r) row(r, 'ground-hash')[2] = {} end,
  hash_missing_field = function(r) drop(r, 'ground-hash', 'impact') end,
  hash_short = function(r) set(r, 'ground-hash', 'below', '1') end,
  physical_bare = function(r) row(r, 'ground-physical')[2] = {} end,
  physical_missing_field = function(r) drop(r, 'ground-physical', 'combined') end,
  physical_missing_row = function(r) drop_row(r, 'ground-physical') end,
  hash_row_missing = function(r) drop_row(r, 'ground-hash') end,
  count_missing = function(r) drop(r, 'ground-above', 'warm_same') end,
  count_wrong = function(r, points) set(r, 'ground-above', 'cases', points + 1) end,
  count_hex = function(r) set(r, 'ground-below', 'cases', '0x5') end,
  count_negative = function(r) set(r, 'ground-impact', 'cases', '-1') end,
  count_float = function(r) set(r, 'ground-impact', 'a_corrected', '1.5') end,
  impact_cases_wrong = function(r, points, impacts) set(r, 'ground-impact', 'cases', impacts - 1) end,
  classes_sum = function(r, points, impacts) set(r, 'ground-impact-classes', 'land', impacts - 1) end,
  types_sum = function(r, points, impacts) set(r, 'ground-impact-types', 't0', impacts - 1) end,
  family_sum = function(r, points) set(r, 'ground-below', 'false', points) end,
  warm_intact_short = function(r, points, impacts) set(r, 'ground-impact', 'warm_start_intact', impacts - 1) end,
  warm_intact_point_short = function(r, points) set(r, 'ground-above', 'warm_start_intact', points - 1) end,
  duplicate_row = function(r) table.insert(r, 2, row(r, 'ground-map')) end,
  unknown_row = function(r) r[#r + 1] = { 'ground-extra', { { 'n', 1 } } } end,
  unexpected_key = function(r) add_field(r, 'ground-asserts', 'bonus', 1) end,
  bare_word = function(r) add_word(r, 'ground-map', 'oops') end,
  duplicate_key = function(r) add_field(r, 'ground-map', 'sectors_x', 5) end,
  empty_fold = function(r) set(r, 'ground-hash', 'above', EMPTY) end,
  full_fold = function(r) set(r, 'ground-physical', 'impact', '3333333333333333') end,
  sample_bad_hex = function(r) set(r, 'ground-sample', 'x', '4000', 'point') end,
  sample_missing = function(r) drop_row(r, 'ground-sample', 'point') end,
  sample_unknown_kind = function(r) set(r, 'ground-sample', 'kind', 'rock', 'point') end,
  sample_case_out_of_range = function(r, points) set(r, 'ground-sample', 'n', points, 'point') end,
  sample_triple_short = function(r) set(r, 'ground-sample', 'pa', '3f800000,40000000', 'impact') end,
  sample_flag = function(r) set(r, 'ground-sample', 'inside', 2, 'point') end,
}

local function report(seed, points, impacts)
  local rows = build(seed, points, impacts)
  local mutate = modes[mode]
  if mutate == nil then error('stand-in: no mode ' .. mode) end
  mutate(rows, points, impacts)
  local lines = {}
  for _, r in ipairs(rows) do
    local words = { r[1] }
    for _, f in ipairs(r[2]) do words[#words + 1] = f[1] and (f[1] .. '=' .. tostring(f[2])) or f[2] end
    lines[#lines + 1] = table.concat(words, ' ')
  end
  return table.concat(lines, '\n') .. '\n'
end

function engine:ground_probe(seed, points, impacts)
  probes = probes + 1
  note('probe ' .. seed .. ' ' .. points .. ' ' .. impacts)
  if mode == 'refuse' then error('ground-refused reason=no_terrain_loaded', 0) end
  if mode == 'differs_between_repeats' then
    -- a valid report whose legacy combined hash changes from one probe to the next
    local text = report(seed, points, impacts)
    return (string.gsub(text, '1234567890123456', string.format('%016d', probes), 1))
  end
  return report(seed, points, impacts)
end

package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@LUA_DIR@/ground-characterization.lua')
print('GROUND_CALLER_DONE')
