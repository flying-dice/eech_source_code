-- Stand-in for the engine in the tests of lua/collision-characterization.lua (EEDCS-49). The real driver runs against it; only the driver's own flow and its
-- checks of what a probe returns are exercised. The stand-in writes a COMPLETE, well-formed report for the request it is given (every row of the fixture's
-- schema, counts that add up, hex words of the right width) and, in a negative mode, ONE defect in it. It measures nothing about the collision queries:
-- whether the original behaves as characterised is the native run's to show, with a DLL built with the `collision-fixture` feature.
-- Placeholders are filled in by the test: @MODE@, @TRACE@ and @DRIVER@ (the path of a private copy of the real driver script, made by the test).
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

local V3 = '3f800000,40000000,3f800000'

local function row(rows, name, nth)
  local seen = 0
  for _, r in ipairs(rows) do
    if r[1] == name then
      seen = seen + 1
      if seen == (nth or 1) then return r end
    end
  end
  error('stand-in: no row ' .. name)
end

-- the stand-in's world. The class census is what the fixture counts for every candidate in the real sector lists and tree sectors; the structured targets are the
-- first 3 of each class, so min(census, 3). Stationary geometry (5), fixed box (4) and both tree classes (40, 60) exceed the quota of 3; the rest do not.
local PER_CLASS = 3
local WORLD = { moving_geometry = 1, moving_box = 1, stationary_geometry = 5, stationary_box = 1, fixed_geometry = 2, fixed_box = 4, building_geometry = 1,
  building_box = 3, tree_quad = 40, tree_geometry = 60 }
local CLASSES = { 'moving_geometry', 'moving_box', 'stationary_geometry', 'stationary_box', 'fixed_geometry', 'fixed_box', 'building_geometry', 'building_box',
  'tree_quad', 'tree_geometry' }

local function build(seed, entity_cases, tree_cases)
  local trees = mode ~= 'no_trees'
  local census, targets = {}, {}
  for _, name in ipairs(CLASSES) do
    census[name] = WORLD[name]
    if not trees and (name == 'tree_quad' or name == 'tree_geometry') then census[name] = 0 end
    targets[name] = math.min(census[name], PER_CLASS)
  end
  local T = 0 -- entity targets
  for _, name in ipairs(CLASSES) do if not string.find(name, 'tree') then T = T + targets[name] end end
  local tq, tgm = targets.tree_quad, targets.tree_geometry
  local ttargets = tq + tgm
  local function kind(group, k, cases, hit) return { 'col-entity-kind', { { 'group', group }, { 'kind', k }, { 'cases', cases }, { 'hit', hit } } } end
  local function tkind(group, k, cases, hit) return { 'col-tree-kind', { { 'group', group }, { 'kind', k }, { 'cases', cases }, { 'hit', hit } } } end
  local function common(event, group, cases, hit, touched, checked, exists, skipped, derived)
    return { event, { { 'group', group }, { 'cases', cases }, { 'hit', hit }, { 'miss', cases - hit }, { 'record_touched', touched }, { 'warm_same', cases },
      { 'warm_different', 0 }, { 'miss_point_written', 0 }, { 'miss_normal_written', 0 }, { 'input_mutated', 0 }, { 'nearer_checked', checked },
      { 'nearer_exists', exists }, { 'skipped_axis', skipped }, { 'derived_axis', derived or 0 } } }
  end
  -- entity: T targets, one primary ray of each of four kinds, 9 follow-up exclusion queries (2 hits), 2 edge rays, and the random cases
  local e_random_hit = math.floor(entity_cases / 4)
  local e_struct_cases, e_struct_hit = 4 * T + 9 + 2, 5 + 4 + 0 + 1 + 2
  -- tree: 4 flag combinations of the through ray, 8 other kinds, 2 edge rays; random cases less the axis-aligned ones that are never probed
  local t_skipped = trees and math.floor(tree_cases / 10) or 0
  local t_random = trees and (tree_cases - t_skipped) or 0
  local t_random_hit = math.floor(t_random / 5)
  local through_hits = trees and 6 or 0
  local t_struct_cases = 4 * ttargets + 8 * ttargets + 2
  local t_struct_hit = trees and (6 + 1 + 1) or 0
  local candidates = 8 + 6 + 4 -- mobile (2 moving + 6 stationary), fixed objects, buildings
  local rows = {
    { 'col-probe', { { 'seed', seed }, { 'entity_random', entity_cases }, { 'tree_random', tree_cases }, { 'per_class', 3 }, { 'samples_hit', 16 }, { 'samples_miss', 8 } } },
    { 'col-map', { { 'world_sectors_x', 3 }, { 'world_sectors_z', 2 }, { 'world_sector_side', 4096 }, { 'terrain_sectors_x', 6 }, { 'terrain_sectors_z', 4 },
      { 'terrain_sector_side', 2048 }, { 'delta_time', '3dcccccd' } } },
    { 'col-guard', { { 'record_restored', 1 }, { 'record_words_before', '0123456789abcdef' }, { 'record_words_after', '0123456789abcdef' }, { 'rng_unchanged', 1 },
      { 'world_unchanged', 1 }, { 'world_before', '00112233445566ff' }, { 'world_after', '00112233445566ff' } } },
    { 'col-fp', { { 'arch', 'x86_64' }, { 'x87_masks', 63 }, { 'sse_masks', 63 } } },
    { 'col-census-entity', { { 'list_entities', 50 }, { 'mobile_candidates', 8 }, { 'mobile_moving', 2 }, { 'mobile_stationary', 6 }, { 'fixed_plain', 6 }, { 'cities', 2 },
      { 'cities_with_buildings', 2 }, { 'buildings', 4 }, { 'listed_buildings', 0 }, { 'geometry_path', 9 }, { 'box_path', 9 }, { 'align_terrain', 3 },
      { 'pool', candidates } } },
    { 'col-census-tree', { { 'tree_data', trees and 1 or 0 }, { 'trees_total', trees and 100 or 0 }, { 'sectors_with_trees', trees and 10 or 0 },
      { 'trees_type0', trees and 40 or 0 }, { 'trees_type1', trees and 60 or 0 }, { 'forest_geometry', trees and 1 or 0 }, { 'pool', trees and 100 or 0 } } },
    { 'col-census-class', {} },
    { 'col-targets', {} },
    kind('struct', 'through', T, 5), kind('struct', 'reverse', T, 4), kind('struct', 'lateral', T, 0), kind('struct', 'above', T, 1), kind('struct', 'exclusion', 9, 2),
    kind('struct', 'edge', 2, 0), kind('random', 'random', entity_cases, e_random_hit),
    { 'col-entity-exclusion', { { 'through_cases', T }, { 'through_hit_target', 3 }, { 'through_hit_other', 2 }, { 'base_cases', 3 }, { 'src_removed', 3 },
      { 'tgt_removed', 3 }, { 'both_removed', 3 }, { 'decoy_cases', 3 }, { 'decoy_unchanged', 3 }, { 'building_base', 1 }, { 'building_src_removed', 0 },
      { 'building_city_src_removed', 1 }, { 'building_city_tgt_removed', 1 }, { 'chain_cases', 5 }, { 'chain_depth1', 5 }, { 'chain_depth2', 2 }, { 'chain_depth3', 1 },
      { 'chain_stalled', 1 }, { 'chain_order_violations', 0 } } },
    common('col-entity-common', 'struct', e_struct_cases, e_struct_hit, 10, 10, 1, 0),
    common('col-entity-common', 'random', entity_cases, e_random_hit, entity_cases, e_random_hit, 0, 0),
  }
  for _, name in ipairs(CLASSES) do
    table.insert(row(rows, 'col-census-class')[2], { name, census[name] })
    table.insert(row(rows, 'col-targets')[2], { name, targets[name] })
  end
  -- the fixture emits EVERY positive bucket of hits by entity kind: the struct buckets add up to the struct hits (12), the random buckets to the random hits
  local function type_row(group, k, name, hits) return { 'col-entity-type', { { 'group', group }, { 'type', k }, { 'name', name }, { 'hits', hits } } } end
  rows[#rows + 1] = type_row('struct', 5, 'ENTITY_TYPE_ROUTED_VEHICLE', 4)
  rows[#rows + 1] = type_row('struct', 7, 'ENTITY_TYPE_ANTI_AIRCRAFT', 5)
  rows[#rows + 1] = type_row('struct', 9, 'ENTITY_TYPE_CITY_BUILDING', 3)
  local first, second = e_random_hit - math.floor(e_random_hit / 2), math.floor(e_random_hit / 2)
  if first > 0 then rows[#rows + 1] = type_row('random', 5, 'ENTITY_TYPE_ROUTED_VEHICLE', first) end
  if second > 0 then rows[#rows + 1] = type_row('random', 9, 'ENTITY_TYPE_CITY_BUILDING', second) end
  local more = {
    tkind('struct', 'through', 4 * ttargets, through_hits), tkind('struct', 'reverse', ttargets, trees and 1 or 0), tkind('struct', 'over', ttargets, 0),
    tkind('struct', 'under', ttargets, 0), tkind('struct', 'lateral', ttargets, 0), tkind('struct', 'slant', ttargets, trees and 1 or 0), tkind('struct', 'high', ttargets, 0),
    tkind('struct', 'vertical', ttargets, 0), tkind('struct', 'short', ttargets, 0), tkind('struct', 'edge', 2, 0), tkind('random', 'random', t_random, t_random_hit),
  }
  for _, r in ipairs(more) do rows[#rows + 1] = r end
  local flag_hits = { quad = { [0] = { [0] = 1, [1] = 1 }, [1] = { [0] = 0, [1] = 1 } }, geometry = { [0] = { [0] = 1, [1] = 1 }, [1] = { [0] = 0, [1] = 1 } } }
  for _, class in ipairs { 'quad', 'geometry' } do
    for ignore = 0, 1 do
      for skip = 0, 1 do
        rows[#rows + 1] = { 'col-tree-flags', { { 'class', class }, { 'ignore_edge', ignore }, { 'skip_top', skip },
          { 'cases', class == 'quad' and tq or tgm }, { 'hit', trees and flag_hits[class][ignore][skip] or 0 } } }
      end
    end
  end
  -- derived axis rays (a shortened nearer-check ray with a zero x or z extent) are counted among the nearer-checks: 1 structured, 2 random
  rows[#rows + 1] = common('col-tree-common', 'struct', t_struct_cases, t_struct_hit, t_struct_cases, trees and 4 or 0, 0, 0, trees and 1 or 0)
  rows[#rows + 1] = common('col-tree-common', 'random', t_random, t_random_hit, t_random, t_random_hit, 0, t_skipped, t_random_hit >= 2 and 2 or 0)
  rows[#rows + 1] = { 'col-physical', { { 'entity_struct', 'fedcba9876543210' }, { 'entity_random', '1111111111111111' }, { 'tree_struct', '2222222222222222' },
    { 'tree_random', '3333333333333333' } } }
  -- a legacy word of decimal digits only must stay text
  rows[#rows + 1] = { 'col-legacy', { { 'entity_struct', '1234567890123456' }, { 'entity_random', '0000000000000001' }, { 'tree_struct', '0000000000000002' },
    { 'tree_random', '0000000000000003' } } }
  -- the samples: exactly the quota (16 hits, 8 misses), or every case if there were fewer, entity first
  local function samples(name, hits, misses)
    for i = 1, math.min(16, hits) do
      rows[#rows + 1] = { 'col-sample', { { 'kind', name }, { 'role', 0 }, { 'hit', 1 }, { 'type', name == 'entity' and 5 or -1 }, { 'hosted', 0 }, { 'a', V3 }, { 'b', V3 },
        { 'point', V3 }, { 'normal', V3 } } }
    end
    for i = 1, math.min(8, misses) do
      rows[#rows + 1] = { 'col-sample', { { 'kind', name }, { 'role', 0 }, { 'hit', 0 }, { 'type', -1 }, { 'hosted', 0 }, { 'a', V3 }, { 'b', V3 }, { 'point', '00000000,00000000,00000000' },
        { 'normal', '5a5a5a5a,5a5a5a5a,5a5a5a5a' } } }
    end
  end
  samples('entity', e_struct_hit + e_random_hit, (e_struct_cases - e_struct_hit) + (entity_cases - e_random_hit))
  samples('tree', t_struct_hit + t_random_hit, (t_struct_cases - t_struct_hit) + (t_random - t_random_hit))
  return rows
end

local function field(rows, name, key, nth)
  for _, f in ipairs(row(rows, name, nth)[2]) do if f[1] == key then return f end end
  error('stand-in: no key ' .. key .. ' in ' .. name)
end

local function set(rows, name, key, value, nth) field(rows, name, key, nth)[2] = value end

local function drop(rows, name, key, nth)
  local fields = row(rows, name, nth)[2]
  for i, f in ipairs(fields) do if f[1] == key then table.remove(fields, i) return end end
  error('stand-in: no key ' .. key .. ' in ' .. name)
end

local function drop_row(rows, name, nth)
  local target = row(rows, name, nth)
  for i, r in ipairs(rows) do if r == target then table.remove(rows, i) return end end
end

-- the nth row of an event whose given field has the given value
local function find(rows, name, key, value)
  for _, r in ipairs(rows) do
    if r[1] == name then
      for _, f in ipairs(r[2]) do if f[1] == key and f[2] == value then return r end end
    end
  end
  error('stand-in: no ' .. name .. ' with ' .. key .. '=' .. tostring(value))
end

local function drop_kind_row(rows, name, group, kind)
  local target
  for _, r in ipairs(rows) do
    if r[1] == name and r[2][1][2] == group and r[2][2][2] == kind then target = r end
  end
  for i, r in ipairs(rows) do if r == target then table.remove(rows, i) return end end
  error('stand-in: no row ' .. name .. ' ' .. group .. ' ' .. kind)
end

local function drop_sample(rows, kind, hit)
  for i, r in ipairs(rows) do
    if r[1] == 'col-sample' and r[2][1][2] == kind and r[2][3][2] == hit then table.remove(rows, i) return end
  end
  error('stand-in: no sample')
end

-- one defect per mode; the test names the exact message the driver must refuse it with
local modes = {
  ok = function() end,
  no_trees = function() end,
  refuse = function() end,
  differs_between_repeats = function() end,
  unrestored = function(r) set(r, 'col-guard', 'record_restored', 0) end,
  words_differ = function(r) set(r, 'col-guard', 'record_words_after', 'ffffffffffffffff') end,
  rng_changed = function(r) set(r, 'col-guard', 'rng_unchanged', 0) end,
  world_flag = function(r) set(r, 'col-guard', 'world_unchanged', 0) end,
  world_fold_differs = function(r) set(r, 'col-guard', 'world_after', 'ffffffffffffffff') end,
  fp_x87_unmasked = function(r) set(r, 'col-fp', 'x87_masks', 59) end,
  fp_sse_unmasked = function(r) set(r, 'col-fp', 'sse_masks', 0) end,
  fp_missing_row = function(r) drop_row(r, 'col-fp') end,
  fp_arch_bad = function(r) set(r, 'col-fp', 'arch', 'x86-64') end,
  wrongheader = function(r, entity_cases) set(r, 'col-probe', 'entity_random', entity_cases + 1) end,
  quota_changed = function(r) set(r, 'col-probe', 'per_class', 4) end,
  sample_quota_changed = function(r) set(r, 'col-probe', 'samples_hit', 32) end,
  physical_missing_row = function(r) drop_row(r, 'col-physical') end,
  physical_missing_field = function(r) drop(r, 'col-physical', 'tree_struct') end,
  physical_short = function(r) set(r, 'col-physical', 'tree_struct', '1') end,
  physical_upper = function(r) set(r, 'col-physical', 'entity_struct', 'FEDCBA9876543210') end,
  legacy_missing_row = function(r) drop_row(r, 'col-legacy') end,
  count_hex = function(r) set(r, 'col-targets', 'moving_box', '0x1') end,
  count_negative = function(r) set(r, 'col-targets', 'fixed_box', '-1') end,
  count_float = function(r) set(r, 'col-entity-exclusion', 'src_removed', '1.5') end,
  flag_two = function(r) set(r, 'col-census-tree', 'forest_geometry', 2) end,
  hex_width = function(r) set(r, 'col-map', 'delta_time', '3dcccc') end,
  common_cases_wrong = function(r) set(r, 'col-entity-common', 'cases', 44, 1) end,
  warm_sum = function(r) set(r, 'col-entity-common', 'warm_same', 40, 1) end,
  touched_more_than_cases = function(r) set(r, 'col-tree-common', 'record_touched', 99, 1) end,
  nearer_more_than_checked = function(r) set(r, 'col-entity-common', 'nearer_exists', 11, 1) end,
  exclusion_more_than_base = function(r) set(r, 'col-entity-exclusion', 'src_removed', 4) end,
  chain_not_nested = function(r) set(r, 'col-entity-exclusion', 'chain_depth3', 3) end,
  hit_target_other_sum = function(r) set(r, 'col-entity-exclusion', 'through_hit_other', 1) end,
  targets_over_quota = function(r) set(r, 'col-targets', 'fixed_box', 4) end,
  -- a class the census found must be selected exactly min(census, 3): none at all, one short of its census, one short of the quota (the quota binds), and the trees
  entity_unselected = function(r) set(r, 'col-targets', 'stationary_geometry', 0) end,
  entity_under_selected_small_class = function(r) set(r, 'col-targets', 'moving_box', 0) end,
  entity_under_selected_at_quota = function(r) set(r, 'col-targets', 'fixed_box', 2) end,
  tree_under_selected = function(r) set(r, 'col-targets', 'tree_quad', 2) end,
  tree_unselected = function(r) set(r, 'col-targets', 'tree_geometry', 0) end,
  -- the class census must agree with the census it refines
  class_census_missing = function(r) drop_row(r, 'col-census-class') end,
  class_census_sum = function(r) set(r, 'col-census-class', 'fixed_box', 3) end,
  class_census_geometry_sum = function(r) set(r, 'col-census-class', 'moving_geometry', 2) end,
  class_census_tree_sum = function(r) set(r, 'col-census-class', 'tree_quad', 39) end,
  class_census_tree_split = function(r) set(r, 'col-census-class', 'tree_quad', 41) set(r, 'col-census-class', 'tree_geometry', 59) end,
  derived_over_checked = function(r) set(r, 'col-tree-common', 'derived_axis', 5, 1) end,
  derived_missing = function(r) drop(r, 'col-tree-common', 'derived_axis', 1) end,
  entity_derived_axis = function(r) set(r, 'col-entity-common', 'derived_axis', 1, 1) end,
  census_paths_wrong = function(r) set(r, 'col-census-entity', 'box_path', 7) end,
  tree_total_wrong = function(r) set(r, 'col-census-tree', 'trees_type1', 59) end,
  tree_flag_cases = function(r) set(r, 'col-tree-flags', 'cases', 4, 1) end,
  tree_flag_hits = function(r) set(r, 'col-tree-flags', 'hit', 2, 4) end,
  kind_missing = function(r) drop_kind_row(r, 'col-entity-kind', 'struct', 'edge') end,
  kind_duplicate = function(r) r[#r + 1] = find(r, 'col-entity-kind', 'kind', 'edge') end,
  kind_unexpected = function(r) r[#r + 1] = { 'col-entity-kind', { { 'group', 'struct' }, { 'kind', 'bogus' }, { 'cases', 1 }, { 'hit', 0 } } } end,
  tree_kind_missing = function(r) drop_kind_row(r, 'col-tree-kind', 'struct', 'short') end,
  type_group_bad = function(r) set(r, 'col-entity-type', 'group', 'elsewhere', 1) end,
  type_name_bad = function(r) set(r, 'col-entity-type', 'name', 'not a token', 1) end,
  type_hits_over = function(r) set(r, 'col-entity-type', 'hits', 5, 1) end,
  -- the fixture emits every positive bucket: one deleted (struct, then random), or all of them, must be refused
  type_bucket_missing = function(r) drop_row(r, 'col-entity-type', 2) end,
  type_random_bucket_missing = function(r) drop_row(r, 'col-entity-type', 4) end,
  type_buckets_omitted = function(r) for _ = 1, 5 do drop_row(r, 'col-entity-type', 1) end end,
  sample_missing = function(r) drop_sample(r, 'entity', 1) end,
  sample_unknown_kind = function(r) set(r, 'col-sample', 'kind', 'rock', 1) end,
  sample_triple_short = function(r) set(r, 'col-sample', 'a', '3f800000,40000000', 1) end,
  unknown_row = function(r) r[#r + 1] = { 'col-extra', { { 'n', 1 } } } end,
  duplicate_row = function(r) r[#r + 1] = row(r, 'col-map') end,
  unexpected_key = function(r) local fields = row(r, 'col-guard')[2] fields[#fields + 1] = { 'bonus', 1 } end,
  bare_word = function(r) local fields = row(r, 'col-map')[2] fields[#fields + 1] = { nil, 'oops' } end,
  duplicate_key = function(r) local fields = row(r, 'col-map')[2] fields[#fields + 1] = { 'world_sectors_x', 3 } end,
}

local function report(seed, entity_cases, tree_cases)
  local rows = build(seed, entity_cases, tree_cases)
  local mutate = modes[mode]
  if mutate == nil then error('stand-in: no mode ' .. mode) end
  mutate(rows, entity_cases, tree_cases)
  local lines = {}
  for _, r in ipairs(rows) do
    local words = { r[1] }
    for _, f in ipairs(r[2]) do words[#words + 1] = f[1] and (f[1] .. '=' .. tostring(f[2])) or f[2] end
    lines[#lines + 1] = table.concat(words, ' ')
  end
  return table.concat(lines, '\n') .. '\n'
end

function engine:collision_probe(seed, entity_cases, tree_cases)
  probes = probes + 1
  note('probe ' .. seed .. ' ' .. entity_cases .. ' ' .. tree_cases)
  if mode == 'refuse' then error('col-refused reason=no_terrain_loaded', 0) end
  if mode == 'refuse_controls' then
    -- a refusal whose text is what a real one can be: a traceback's tabs and newlines, a quote, a backslash, every control byte 0 to 31 in order, and UTF-8
    local controls = {}
    for b = 0, 31 do controls[#controls + 1] = string.char(b) end
    error('col-refused reason=bad "quoted" back\\slash\n\tstack traceback:\n\t[C]: in function \'error\'\n\tcontrols=' .. table.concat(controls) .. ' end \195\169', 0)
  end
  if mode == 'differs_between_repeats' then
    -- a valid report whose legacy hash changes from one probe to the next
    local text = report(seed, entity_cases, tree_cases)
    return (string.gsub(text, '1234567890123456', string.format('%016d', probes), 1))
  end
  return report(seed, entity_cases, tree_cases)
end

package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@DRIVER@')
print('COLLISION_CALLER_DONE')
