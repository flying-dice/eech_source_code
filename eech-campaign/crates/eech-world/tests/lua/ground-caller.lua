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

-- the samples the fixture takes, for a run of `points` and `impacts` cases whose tag and type are the case number modulo 6 and 8 (the stand-in's own
-- choice: the real tags and types come from the fixture's generator, which this stand-in does not reproduce): in case order, a point case is sampled
-- while fewer than 2 earlier cases of its tag were, an impact case while fewer than 4 of its type were
local V3 = '3f800000,40000000,3f800000'

local function point_sample(n, tag)
  return { 'ground-sample', { { 'kind', 'point' }, { 'n', n }, { 'tag', tag }, { 'x', '40000000' }, { 'y', '3f800000' }, { 'z', '00000000' },
    { 'inside', 1 }, { 'above', 1 }, { 'above_touched', 0 }, { 'below', 0 }, { 'below_touched', 0 }, { 'above_elev', '3f800000' }, { 'above_type', 2 },
    { 'below_elev', '3f800000' }, { 'below_type', 2 } } }
end

local function impact_sample(n, type_number)
  return { 'ground-sample', { { 'kind', 'impact' }, { 'n', n }, { 'type', type_number }, { 'a0', V3 }, { 'b0', V3 }, { 'class_alias', 3 }, { 'pa', V3 },
    { 'qa', V3 }, { 'class_separate', 3 }, { 'pb', V3 }, { 'qb', V3 }, { 'rb', V3 }, { 'alias_equal', 1 }, { 'warm_same', 1 }, { 'touched', 1 },
    { 'elev', '3f800000' }, { 'type_word', 2 } } }
end

local function impact_types(impacts)
  local tally = { [0] = 0, 0, 0, 0, 0, 0, 0, 0 }
  for i = 0, impacts - 1 do tally[i % 8] = tally[i % 8] + 1 end
  return tally
end

-- a report as ordered rows of ordered key/value fields, so that a mode can change exactly one thing
local function build(seed, points, impacts)
  local function fold(cases, value) if cases == 0 then return EMPTY end return value end
  local types = impact_types(impacts)
  -- every point is above the ground and not below it, so the sampled answers (above=1, below=0) agree with the counters
  local rows = {
    { 'ground-probe', { { 'seed', seed }, { 'points', points }, { 'impacts', impacts }, { 'samples_per_point_tag', 2 }, { 'samples_per_impact_type', 4 } } },
    -- two grids over the same ground: the campaign's sectors are twice the terrain's (3 x 2 of 4096 m against 6 x 4 of 2048 m), as a campaign may be
    { 'ground-map', { { 'world_sectors_x', 3 }, { 'world_sectors_z', 2 }, { 'world_sector_side', 4096 }, { 'terrain_sectors_x', 6 }, { 'terrain_sectors_z', 4 },
      { 'terrain_sector_side', 2048 } } },
    { 'ground-record', { { 'anchored', 1 }, { 'restored', 1 }, { 'words_before', '0123456789abcdef' }, { 'words_after', '0123456789abcdef' } } },
    { 'ground-points', { { 'cases', points }, { 'in_map', points }, { 'off_map', 0 }, { 'input_mutated', 0 } } },
    { 'ground-above', { { 'cases', points }, { 'true', points }, { 'false', 0 }, { 'record_touched', points }, { 'true_record_untouched', 0 },
      { 'warm_same', points }, { 'warm_start_intact', points } } },
    { 'ground-below', { { 'cases', points }, { 'true', 0 }, { 'false', points }, { 'record_touched', points }, { 'true_record_untouched', 0 },
      { 'warm_same', points }, { 'warm_start_intact', points }, { 'off_map_true', 0 }, { 'off_map_record_touched', 0 } } },
    { 'ground-complement', { { 'in_map_cases', points }, { 'complement', points }, { 'not_complement', 0 }, { 'both_true', 0 }, { 'both_false', 0 } } },
    { 'ground-impact', { { 'cases', impacts }, { 'a_corrected', 0 }, { 'b_corrected', 0 }, { 'xz_changed', 0 }, { 'alias_equal', impacts },
      { 'alias_unequal', 0 }, { 'warm_same', impacts }, { 'warm_different', 0 }, { 'record_touched', impacts }, { 'record_untouched', 0 },
      { 'impact_equals_corrected_below', 0 }, { 'warm_start_intact', impacts } } },
    { 'ground-impact-classes', { { 'invalid', 0 }, { 'water', 0 }, { 'forest', 0 }, { 'land', impacts }, { 'other', 0 } } },
    { 'ground-impact-types', { { 't0', types[0] }, { 't1', types[1] }, { 't2', types[2] }, { 't3', types[3] }, { 't4', types[4] }, { 't5', types[5] },
      { 't6', types[6] }, { 't7', types[7] } } },
    { 'ground-asserts', { { 'original_below_endpoint_still_above', 0 }, { 'extra_above_endpoint_not_above', 0 } } },
    -- a hash word of decimal digits only must stay text
    { 'ground-hash', { { 'combined', '1234567890123456' }, { 'above', fold(points, '0000000000000001') }, { 'below', fold(points, '0000000000000002') },
      { 'impact', fold(impacts, '3000000000000000') } } },
    { 'ground-physical', { { 'combined', 'fedcba9876543210' }, { 'above', fold(points, '1111111111111111') }, { 'below', fold(points, '2222222222222222') },
      { 'impact', fold(impacts, '3333333333333333') } } },
  }
  -- the complete, self-consistent sample set: point samples first (in case order), then impact samples
  local taken = {}
  for i = 0, points - 1 do
    local tag = i % 6
    taken[tag] = (taken[tag] or 0) + 1
    if taken[tag] <= 2 then rows[#rows + 1] = point_sample(i, tag) end
  end
  taken = {}
  for i = 0, impacts - 1 do
    local kind = i % 8
    taken[kind] = (taken[kind] or 0) + 1
    if taken[kind] <= 4 then rows[#rows + 1] = impact_sample(i, kind) end
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

-- the k-th sample row of a kind (1-based), its field by key, and the samples of a kind dropped one or all
local function nth(rows, kind, k)
  local seen = 0
  for _, r in ipairs(rows) do
    if r[1] == 'ground-sample' and r[2][1][2] == kind then
      seen = seen + 1
      if seen == k then return r end
    end
  end
  error('stand-in: no ' .. kind .. ' sample ' .. k)
end

local function field_of(r, key)
  for _, f in ipairs(r[2]) do if f[1] == key then return f end end
  error('stand-in: no key ' .. key)
end

local function drop_nth(rows, kind, k)
  local target = nth(rows, kind, k)
  for i, r in ipairs(rows) do if r == target then table.remove(rows, i) return end end
end

local function drop_all(rows, kind)
  local i = 1
  while i <= #rows do
    if rows[i][1] == 'ground-sample' and rows[i][2][1][2] == kind then table.remove(rows, i) else i = i + 1 end
  end
end

-- every impact case of type 0: the tally says so, and `taken` samples of it are written (cases 0 .. taken - 1)
local function concentrate(rows, impacts, taken)
  local types = { 't0', 't1', 't2', 't3', 't4', 't5', 't6', 't7' }
  for i, name in ipairs(types) do set(rows, 'ground-impact-types', name, i == 1 and impacts or 0) end
  drop_all(rows, 'impact')
  for n = 0, taken - 1 do rows[#rows + 1] = impact_sample(n, 0) end
end

-- the point samples replaced by exactly these (n, tag) pairs, placed before the impact samples as the fixture writes them
local function replace_points(rows, pairs_list)
  drop_all(rows, 'point')
  local at = #rows + 1
  for i, r in ipairs(rows) do
    if r[1] == 'ground-sample' and r[2][1][2] == 'impact' then at = i break end
  end
  for k, p in ipairs(pairs_list) do table.insert(rows, at + k - 1, point_sample(p[1], p[2])) end
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
  types_sum = function(r) local f = field(r, 'ground-impact-types', 't0') f[2] = f[2] - 1 end,
  family_sum = function(r, points) set(r, 'ground-below', 'false', points + 2) end,
  warm_intact_short = function(r, points, impacts) set(r, 'ground-impact', 'warm_start_intact', impacts - 1) end,
  warm_intact_point_short = function(r, points) set(r, 'ground-above', 'warm_start_intact', points - 1) end,
  duplicate_row = function(r) table.insert(r, 2, row(r, 'ground-map')) end,
  unknown_row = function(r) r[#r + 1] = { 'ground-extra', { { 'n', 1 } } } end,
  unexpected_key = function(r) add_field(r, 'ground-asserts', 'bonus', 1) end,
  bare_word = function(r) add_word(r, 'ground-map', 'oops') end,
  duplicate_key = function(r) add_field(r, 'ground-map', 'world_sectors_x', 3) end,
  -- the grids: the same grid for both is accepted; the extents they cover must agree; every count is present and at least 1
  map_grids_same = function(r)
    set(r, 'ground-map', 'world_sectors_x', 6) set(r, 'ground-map', 'world_sectors_z', 4) set(r, 'ground-map', 'world_sector_side', 2048)
  end,
  map_extent_x_differs = function(r) set(r, 'ground-map', 'terrain_sectors_x', 7) end,
  map_extent_z_differs = function(r) set(r, 'ground-map', 'terrain_sectors_z', 5) end,
  map_terrain_key_missing = function(r) drop(r, 'ground-map', 'terrain_sectors_z') end,
  map_zero_grid = function(r) set(r, 'ground-map', 'terrain_sector_side', 0) end,
  map_old_schema = function(r) row(r, 'ground-map')[2] = { { 'sectors_x', 5 }, { 'sectors_z', 4 }, { 'sector_side', 2048 } } end,
  empty_fold = function(r) set(r, 'ground-hash', 'above', EMPTY) end,
  full_fold = function(r) set(r, 'ground-physical', 'impact', '3333333333333333') end,
  sample_bad_hex = function(r) set(r, 'ground-sample', 'x', '4000', 'point') end,
  sample_missing = function(r) drop_all(r, 'point') end,
  sample_unknown_kind = function(r) set(r, 'ground-sample', 'kind', 'rock', 'point') end,
  sample_case_out_of_range = function(r, points) set(r, 'ground-sample', 'n', points, 'point') end,
  sample_triple_short = function(r) set(r, 'ground-sample', 'pa', '3f800000,40000000', 'impact') end,
  sample_flag = function(r) set(r, 'ground-sample', 'inside', 2, 'point') end,
  -- case numbers: distinct, increasing, in range, and case 0 always sampled
  sample_n_duplicate_point = function(r) nth(r, 'point', 2)[2][2][2] = 0 end,
  sample_n_order_point = function(r) nth(r, 'point', 2)[2][2][2] = 4 nth(r, 'point', 3)[2][2][2] = 2 end,
  sample_first_not_zero_point = function(r) drop_nth(r, 'point', 1) end,
  sample_n_duplicate_impact = function(r) nth(r, 'impact', 2)[2][2][2] = 0 end,
  sample_first_not_zero_impact = function(r) drop_nth(r, 'impact', 1) end,
  sample_out_of_range_impact = function(r, points, impacts) set(r, 'ground-sample', 'n', impacts, 'impact') end,
  sample_point_after_impact = function(r, points) local last = nth(r, 'point', points) drop_nth(r, 'point', points) r[#r + 1] = last end,
  -- tag and type domains, and the per-tag and per-type caps
  sample_tag_domain = function(r) set(r, 'ground-sample', 'tag', 6, 'point') end,
  sample_type_domain = function(r) set(r, 'ground-sample', 'type', 8, 'impact') end,
  sample_tag_cap = function(r) for k = 1, 3 do field_of(nth(r, 'point', k), 'tag')[2] = 0 end end,
  sample_type_cap = function(r, points, impacts) for k = 1, impacts do field_of(nth(r, 'impact', k), 'type')[2] = 0 end end,
  -- exact impact sample count per type: min(tally, 4)
  sample_type_omitted = function(r) drop_nth(r, 'impact', 3) end,
  sample_type_extra = function(r) field_of(nth(r, 'impact', 2), 'type')[2] = 0 end,
  impact_types_concentrated_short = function(r, points, impacts) concentrate(r, impacts, 1) end,
  impact_types_concentrated_complete = function(r, points, impacts) concentrate(r, impacts, 4) end,
  header_sample_cap_zero = function(r) set(r, 'ground-probe', 'samples_per_point_tag', 0) end,
  header_point_quota_larger = function(r) set(r, 'ground-probe', 'samples_per_point_tag', 3) end,
  header_impact_quota_larger = function(r) set(r, 'ground-probe', 'samples_per_impact_type', 8) end,
  -- NO CASE MAY BE SKIPPED BEFORE A CLASS IS FULL (5 point cases, no tag reaches 2 samples): an interior gap, another interior gap, an unfinished tail,
  -- and a gap that a LATER saturation does not excuse
  sample_point_gap_n1 = function(r) drop_nth(r, 'point', 2) end,
  sample_point_gap_n2 = function(r) drop_nth(r, 'point', 3) end,
  sample_point_tail_n4 = function(r) drop_nth(r, 'point', 5) end,
  points_gap_before_saturation = function(r) replace_points(r, { { 0, 0 }, { 2, 0 } }) end,
  -- the same for impacts, in the form the exact per-type count alone cannot see: all five cases of type 0, four samples, but case 1 is missing
  impact_concentrated_gap = function(r, points, impacts)
    concentrate(r, impacts, 0)
    for _, n in ipairs({ 0, 2, 3, 4 }) do r[#r + 1] = impact_sample(n, 0) end
  end,
  -- legitimate omissions, accepted: once a tag holds its 2 samples, later cases may belong to it
  points_same_tag_saturated = function(r) replace_points(r, { { 0, 0 }, { 1, 0 } }) end,
  points_saturated_then_other_tags = function(r) replace_points(r, { { 0, 0 }, { 1, 0 }, { 3, 1 }, { 4, 2 } }) end,
  -- sample values against the counters they are counted in
  sample_off_map_with_above = function(r) set(r, 'ground-sample', 'inside', 0, 'point') end,
  sample_off_map_counter = function(r) set(r, 'ground-sample', 'inside', 0, 'point') set(r, 'ground-sample', 'above', -1, 'point') end,
  sample_inside_not_asked = function(r) set(r, 'ground-sample', 'above', -1, 'point') end,
  sample_above_counter = function(r) set(r, 'ground-sample', 'above', 0, 'point') end,
  sample_below_counter = function(r) set(r, 'ground-sample', 'below', 1, 'point') end,
  sample_alias_counter = function(r) set(r, 'ground-sample', 'alias_equal', 0, 'impact') end,
  sample_warm_counter = function(r) set(r, 'ground-sample', 'warm_same', 0, 'impact') end,
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
  if mode == 'refuse_controls' then
    -- a refusal whose text is what a real one can be: a traceback's tabs and newlines, a quote, a backslash, every control byte 0 to 31 in order, and UTF-8
    local controls = {}
    for b = 0, 31 do controls[#controls + 1] = string.char(b) end
    error('ground-refused reason=bad "quoted" back\\slash\n\tstack traceback:\n\t[C]: in function \'error\'\n\tcontrols=' .. table.concat(controls) .. ' end \195\169', 0)
  end
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
