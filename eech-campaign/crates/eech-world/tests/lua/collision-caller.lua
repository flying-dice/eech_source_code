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

local trace_plan = nil -- what the last built report says about the tree queries, for the transcript the stand-in writes

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
    { 'col-flags', { { 'restored', 1 }, { 'words', 16384 }, { 'before', '1122334455667788' }, { 'after_queries', 'aabbccddeeff0011' }, { 'after_restore', '1122334455667788' },
      { 'updated_changed_words', 3 }, { 'updated_cleared_bits', 7 }, { 'moved_changed_words', 0 }, { 'moved_cleared_bits', 0 }, { 'rotated_changed_words', 0 },
      { 'rotated_cleared_bits', 0 }, { 'added_bits', 0 } } },
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
  trace_plan = { trees = trees, s_cases = t_struct_cases, s_hits = t_struct_hit, s_near = trees and 4 or 0, s_derived = trees and 1 or 0, r_cases = t_random, r_hits = t_random_hit,
    r_near = t_random_hit, r_derived = (t_random_hit >= 2) and 2 or 0 }
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
  -- the update-flag arrays: the copy written back, the restored arrays equal to the copy, the original queries' own effect reported coherently
  flags_row_missing = function(r) drop_row(r, 'col-flags') end,
  flags_unrestored = function(r) set(r, 'col-flags', 'restored', 0) end,
  flags_hash_differs = function(r) set(r, 'col-flags', 'after_restore', 'ffffffffffffffff') end,
  flags_changed_over_words = function(r) set(r, 'col-flags', 'updated_changed_words', 99999) end,
  flags_changed_without_bits = function(r) set(r, 'col-flags', 'updated_cleared_bits', 0) end,
  flags_hash_inconsistent = function(r) set(r, 'col-flags', 'after_queries', '1122334455667788') end,
  flags_words_zero = function(r) set(r, 'col-flags', 'words', 0) end,
  -- feasibility of the counts for 32-bit words, one defect each (the valid row is updated 3 changed words / 7 cleared bits, the rest 0, added_bits 0):
  -- bits with no changed word at all, hashes equal; one array's clears over 32 per changed word; cleared plus added over 32 per changed word in all; and a changed array
  -- with no cleared bit while another array's clears keep the old totals satisfied
  flags_bits_without_changed_words = function(r)
    set(r, 'col-flags', 'updated_changed_words', 0)
    set(r, 'col-flags', 'updated_cleared_bits', 1)
    set(r, 'col-flags', 'after_queries', '1122334455667788')
  end,
  flags_cleared_over_capacity = function(r) set(r, 'col-flags', 'updated_cleared_bits', 97) end,
  flags_aggregate_over_capacity = function(r) set(r, 'col-flags', 'added_bits', 90) end,
  flags_array_lacks_bits = function(r) set(r, 'col-flags', 'moved_changed_words', 2) end,
  -- a legitimate aggregated added-bit diagnostic is ACCEPTED: moved 2 changed words with no cleared bit need 2 added bits
  flags_added_bits_legitimate = function(r)
    set(r, 'col-flags', 'moved_changed_words', 2)
    set(r, 'col-flags', 'added_bits', 2)
  end,
  -- a rejected report is kept whole: an unknown row whose value holds quote, backslash, a control byte and UTF-8
  raw_with_controls = function(r) r[#r + 1] = { 'col-extra', { { 'x', '\1"\\\195\169' } } } end,
  -- the tree-query transcript, one defect each (the report is the valid one)
  trace_overflow = function() end,
  trace_extra_lookups = function() end,
  trace_short = function() end,
  trace_stage_order = function() end,
  trace_warm_differs = function() end,
  trace_init_normal = function() end,
  trace_not_repeatable = function() end,
  trace_facts_tree_total = function() end,
  trace_triangle_count = function() end,
  trace_forest_capped = function() end,
  -- the controlled geometry witness: single defects in the witness row, the geometry facts or the per-query geometry fields (all no-ops for the ordinary report)
  witness_row_missing = function() end,
  witness_row_unarmed = function() end,
  witness_field_unarmed = function() end,
  witness_facts_unarmed = function() end,
  witness_not_restored = function() end,
  witness_data_not_restored = function() end,
  witness_scene_not_restored = function() end,
  witness_objects_differ = function() end,
  witness_entry_differs = function() end,
  witness_scene_index_differs = function() end,
  witness_copy_differs = function() end,
  witness_wrong_mesh = function() end,
  witness_label_wrong = function() end,
  witness_count_mismatch = function() end,
  witness_depth_mismatch = function() end,
  witness_last_without_hit = function() end,
  witness_depth_range = function() end,
  witness_true_exceeds_calls = function() end,
  witness_triangle_count = function() end,
  witness_facts_mismatch = function() end,
  witness_arm_fails = function() end,
  -- impossible decoder call histories with coherent report buckets (the conservation rule); zero_call_depth is also refused by the older neutral-depth rule
  witness_history_true_count = function() end,
  witness_history_final_depth = function() end,
  witness_history_incomplete_walk = function() end,
  witness_history_exhausted_depth = function() end,
  witness_history_calls_without_true = function() end,
  witness_history_zero_call_depth = function() end,
  -- warm pairing, lookup coherence, and the valid outside/boundary transcripts
  trace_warm_endpoint = function() end,
  trace_warm_flags = function() end,
  trace_warm_context = function() end,
  trace_near_warm_context = function() end,
  trace_lookup_x = function() end,
  trace_lookup_z = function() end,
  trace_lookup_order = function() end,
  trace_lookup_count = function() end,
  trace_outside_wrong = function() end,
  trace_outside_ok = function() end,
  trace_boundary_inclusive = function() end,
  trace_boundary_outside_ok = function() end,
  trace_unknown_row = function() end,
  -- failures of the NEW trace file's I/O and of the disarm (the defect is injected below, not in the report or the transcript)
  trace_io_write = function() end,
  trace_io_flush = function() end,
  trace_io_close = function() end,
  trace_disarm_fails = function() end,
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

-- A complete, self-consistent transcript for the last report (the stand-in measures nothing: it lays out records whose counts, hits, warm agreements, derived rays and tree
-- facts add up to the report), and in a trace_* mode ONE defect in it.
local ZERO3, SENT3 = '00000000,00000000,00000000', '5a5a5a5a,5a5a5a5a,5a5a5a5a'
local trace_armed = false
local trace_reads = 0

-- The controlled geometry witness. The stand-in lays out, for the last report, the per-query geometry facts a real witness would record (they add up to the witness row); it measures
-- nothing about the original's geometry branch.
local witness_armed, witness_in_trace = false, false
local WITNESS_TRIANGLES = 6

-- A POSSIBLE decoder call history for the fixed 6-triangle model: each complete walk asks 6 times (yes) and once more (no); a walk that hits at depth d asks d + 1 times (yes) and the
-- query returns. F complete walks before the last: g_true = 6 * F + (hit at depth d ? d + 1 : 0), g_calls = g_true + F, and an exhausted history's last yes is at depth 5.
local function walk_history(complete, hit_depth)
  local true_calls = WITNESS_TRIANGLES * complete + (hit_depth and (hit_depth + 1) or 0)
  return { calls = true_calls + complete, true_calls = true_calls, last = hit_depth and 1 or 0, depth = hit_depth or ((complete > 0) and (WITNESS_TRIANGLES - 1) or 0) }
end

local function geometry_fields(r, i, state)
  local g
  if r.hit == 1 and i % 7 == 0 then
    -- a hit through the quad path although geometry was present: one or two complete walks, no geometry hit
    g = walk_history(1 + (i % 2), nil)
  elseif r.hit == 1 then
    -- a geometry hit at depth 0 to 5, after none, one or two complete walks of other trees
    g = walk_history(math.floor((i - 1) / WITNESS_TRIANGLES) % 3, (i - 1) % WITNESS_TRIANGLES)
  elseif i % 2 == 0 then
    -- a true miss after one, two or three complete walks
    g = walk_history(1 + (i % 3), nil)
  else
    g = walk_history(0, nil)
  end
  -- impossible histories whose report buckets (recomputed from the same fields) stay coherent: only the conservation rule can reject them
  if mode == 'witness_history_true_count' and r.hit == 1 and not state.history_done then g = { calls = 2, true_calls = 2, last = 1, depth = 0 } state.history_done = true end
  if mode == 'witness_history_final_depth' and r.hit == 1 and not state.history_done then g = { calls = 2, true_calls = 2, last = 1, depth = 3 } state.history_done = true end
  if mode == 'witness_history_incomplete_walk' and r.hit == 0 and i % 2 == 0 and not state.history_done then g = { calls = 3, true_calls = 2, last = 0, depth = 1 } state.history_done = true end
  if mode == 'witness_history_exhausted_depth' and r.hit == 0 and i % 2 == 0 and not state.history_done then g = { calls = 7, true_calls = 6, last = 0, depth = 4 } state.history_done = true end
  if mode == 'witness_history_calls_without_true' and r.hit == 0 and i % 2 == 1 and not state.history_done then g = { calls = 3, true_calls = 0, last = 0, depth = 0 } state.history_done = true end
  if mode == 'witness_history_zero_call_depth' and r.hit == 0 and i % 2 == 1 and not state.history_done then g = { calls = 0, true_calls = 0, last = 0, depth = 2 } state.history_done = true end
  if mode == 'witness_last_without_hit' and r.hit == 0 and not state.last_done then g = { calls = 3, true_calls = 2, last = 1, depth = 1 } state.last_done = true end
  if mode == 'witness_depth_range' and r.hit == 1 and not state.depth_done then g.depth = WITNESS_TRIANGLES state.depth_done = true end
  if mode == 'witness_true_exceeds_calls' and i == 1 then g = { calls = 1, true_calls = 2, last = 0, depth = 0 } end
  return g
end

local function trace_lines()
  local plan = trace_plan
  -- records as field tables first, so a mode can change one field; the terrain lookups are DERIVED from the final endpoints (old then new, each only if inside the map: the stand-in
  -- map is x and z from 0 to 32767.0, inclusive), so a valid transcript's lookups are always coherent with its endpoints and a control breaks exactly one thing
  local recs = {}
  local function add(group, kind, stage, hit, a, b, point, normal)
    recs[#recs + 1] = { group = group, kind = kind, role = kind, stage = stage, ignore_edge = 0, skip_top = 1, a = a, b = b, hit = hit, point = point, normal = normal }
  end
  local A = '3f800000,40000000,3f800000'
  local function group(g, cases, hits, near, derived)
    for i = 1, cases do
      local hit = (i <= hits) and 1 or 0
      local kind = (g == 3) and 10 or ((i - 1) % 10)
      local b = '40000000,41000000,40400000'
      local point, normal = ZERO3, SENT3
      if hit == 1 then point, normal = '3f800000,40000000,40800000', V3 end
      add(g, kind, 0, hit, A, b, point, normal)
      add(g, kind, 1, hit, A, b, point, normal)
      if hit == 1 and i <= near then
        -- the shortened nearer-check ray; the first 'derived' of them have a zero x extent
        local nb = (i <= derived) and '3f800000,41000000,40400000' or '40000000,41000000,40400000'
        add(g, kind, 2, 0, A, nb, ZERO3, SENT3)
        add(g, kind, 3, 0, A, nb, ZERO3, SENT3)
      end
    end
  end
  group(2, plan.s_cases, plan.s_hits, plan.s_near, plan.s_derived)
  group(3, plan.r_cases, plan.r_hits, plan.r_near, plan.r_derived)

  -- single-defect modes on the field tables (record 1 is ordinal 0)
  if mode == 'trace_stage_order' then recs[2].stage = 0 end
  if mode == 'trace_warm_differs' then recs[2].point = '3f800000,40000000,40800001' end
  if mode == 'trace_init_normal' then recs[1].init_normal = ZERO3 end
  if mode == 'trace_not_repeatable' and trace_reads >= 2 then recs[3].elev0 = '41200001' end
  -- a warm repeat or a nearer-check warm repeat that is not the same query as its cold one
  if mode == 'trace_warm_endpoint' then recs[2].b = '40000001,41000000,40400000' end
  if mode == 'trace_warm_flags' then recs[2].ignore_edge = 1 end
  if mode == 'trace_warm_context' then recs[2].kind = 5 end
  if mode == 'trace_near_warm_context' then recs[4].role = 7 end
  -- lookups that are not the original's: a wrong x, a wrong z, the two swapped, one too few, and one for an endpoint outside the map
  if mode == 'trace_lookup_x' then recs[1].l0_text = '3f800001,3f800000,41200000' end
  if mode == 'trace_lookup_z' then recs[1].l1_text = '40000000,40400001,41200000' end
  if mode == 'trace_lookup_order' then
    recs[1].l0_text = '40000000,40400000,41200000'
    recs[1].l1_text = '3f800000,3f800000,41200000'
  end
  if mode == 'trace_lookup_count' then recs[1].lookups_override = 1 end
  if mode == 'trace_outside_wrong' then
    recs[1].b = 'c0000000,41000000,40400000'
    recs[1].lookups_override = 2
  end
  -- valid transcripts the checker must ACCEPT: the new endpoint outside the map (one lookup), exactly on the inclusive maximum (two) and one ulp past it (one)
  local function new_endpoint(b) recs[1].b = b recs[2].b = b end
  if mode == 'trace_outside_ok' then new_endpoint('c0000000,41000000,40400000') end
  if mode == 'trace_boundary_inclusive' then new_endpoint('46fffe00,41000000,40400000') end
  if mode == 'trace_boundary_outside_ok' then new_endpoint('46fffe01,41000000,40400000') end

  local function inside(triple)
    local x, z = triple:match('^(%x+),%x+,(%x+)$')
    return x, z, triple:match('^(%x+),') ~= 'c0000000' and triple:match('^(%x+),') ~= '46fffe01'
  end
  local lines = {}
  local gstate = {}
  local gstats = { q = { 0, 0, 0, 0 }, b = { 0, 0, 0, 0 }, g = { 0, 0, 0, 0 }, o = { 0, 0, 0, 0 }, e = { 0, 0, 0, 0 }, d = { 0, 0, 0, 0, 0, 0, 0, 0 }, beyond = 0 }
  for i, r in ipairs(recs) do
    local lookups = {}
    local ax, az, a_in = inside(r.a)
    local bx, bz, b_in = inside(r.b)
    if a_in then lookups[#lookups + 1] = { ax, az } end
    if b_in then lookups[#lookups + 1] = { bx, bz } end
    if r.lookups_override == 2 and #lookups == 1 then lookups[2] = { bx, bz } end
    if r.lookups_override == 1 then lookups = { lookups[1] } end
    local elev0 = r.elev0 or '41200000'
    local l0 = r.l0_text or (lookups[1] and (lookups[1][1] .. ',' .. lookups[1][2] .. ',' .. elev0) or ZERO3)
    local l1 = r.l1_text or (lookups[2] and (lookups[2][1] .. ',' .. lookups[2][2] .. ',41200000') or ZERO3)
    lines[#lines + 1] = string.format('tq ordinal=%d group=%d kind=%d role=%d stage=%d ignore_edge=%d skip_top=%d a=%s b=%s init_point=%s init_normal=%s lookups=%d extra_lookups=0 ' ..
      'l0=%s l1=%s hit=%d point=%s normal=%s', i - 1, r.group, r.kind, r.role, r.stage, r.ignore_edge, r.skip_top, r.a, r.b, r.init_point or ZERO3, r.init_normal or SENT3,
      #lookups, l0, l1, r.hit, r.point, r.normal)
    if witness_in_trace or mode == 'witness_field_unarmed' then
      local g = geometry_fields(r, i, gstate)
      lines[#lines] = lines[#lines] .. string.format(' g_calls=%d g_true=%d g_last=%d g_depth=%d', g.calls, g.true_calls, g.last, g.depth)
      local st = r.stage + 1
      gstats.q[st] = gstats.q[st] + 1
      if g.true_calls > 0 then
        gstats.b[st] = gstats.b[st] + 1
        if g.last == 0 then gstats.e[st] = gstats.e[st] + 1 end
      end
      if r.hit == 1 then
        if g.last == 1 then
          gstats.g[st] = gstats.g[st] + 1
          if r.stage == 0 then gstats.d[g.depth + 1] = (gstats.d[g.depth + 1] or 0) + 1 end
        else
          gstats.o[st] = gstats.o[st] + 1
        end
      end
    end
  end
  recs = lines
  if mode == 'trace_short' then table.remove(recs) end
  if mode == 'trace_unknown_row' then recs[#recs + 1] = 'tq-extra n=1' end

  local trees = plan.trees
  local base_triangles = witness_in_trace and ((mode == 'witness_triangle_count') and 5 or WITNESS_TRIANGLES) or 2
  local facts = {
    'tf-layout record_size=8 x_offset=0 z_offset=2 y_offset=4 colour_offset=6 probe=feff341200cdabef height_only=0000000000000000 type_only=0000000000000000',
    string.format('tf-map terrain_sectors_x=6 terrain_sectors_z=4 terrain_sector_side=2048 xz_scale=3a000000 height_scale=3c000000 height_minimum=c0000000 max_tree_height=35 ' ..
      'map_min_x=00000000 map_min_z=00000000 map_max_x=46fffe00 map_max_z=46fffe00 tree_total=%d sectors_with_trees=%d', trees and ((mode == 'trace_facts_tree_total') and 99 or 100) or 0, trees and 10 or 0),
    -- a COHERENT truncation in trace_forest_capped: the emitter's cap is 512, so a forest with more triangles gives exactly 512 indexed rows, triangles=512 and capped=1
    string.format('tf-forest geometry=%d triangles=%d capped=%d fold=0123456789abcdef', trees and 1 or 0,
      trees and ((mode == 'trace_forest_capped') and 512 or ((mode == 'trace_triangle_count') and 3 or base_triangles)) or 0, (trees and mode == 'trace_forest_capped') and 1 or 0),
  }
  if trees then
    local triangles = (mode == 'trace_forest_capped') and 512 or base_triangles
    for i = 0, triangles - 1 do facts[#facts + 1] = string.format('tf-triangle i=%d p0=%s p1=%s p2=%s', i, V3, V3, V3) end
    if witness_in_trace or mode == 'witness_facts_unarmed' then
      facts[#facts + 1] = string.format('tf-witness label=controlled_forest_geometry source=fixture_owned_mesh mesh_points=%d mesh_faces=3 mesh_surfaces=2 mesh_triangles=6 mesh_fold=%s',
        (mode == 'witness_facts_mismatch') and 13 or 12, 'fedcba9876543210')
    end
    for i = 0, 7 do facts[#facts + 1] = string.format('tf-sample sx=1 sz=2 count=12 index=%d bytes=00112233445566%02x', i, i) end
  end
  return recs, facts, gstats
end

local function witness_row()
  local _, _, g = trace_lines()
  local objects_after = (mode == 'witness_objects_differ') and '0123456789abcdee' or '0123456789abcdef'
  local entry_after = (mode == 'witness_entry_differs') and '00112233445566fe' or '00112233445566ff'
  local parts = { string.format('col-geometry-witness label=%s source=fixture_owned_mesh installed=1 restored=%d data_restored=%d scene_restored=%d copy_identical=%d objects_count=4097 ' ..
    'scene_index_before=-1 scene_index_after=%d objects_before=0123456789abcdef objects_after=%s entry_before=00112233445566ff entry_after=%s mesh_points=%d mesh_faces=3 mesh_surfaces=2 ' ..
    'mesh_triangles=6 mesh_fold=fedcba9876543210', (mode == 'witness_label_wrong') and 'some_other_witness' or 'controlled_forest_geometry', (mode == 'witness_not_restored') and 0 or 1,
    (mode == 'witness_data_not_restored') and 0 or 1, (mode == 'witness_scene_not_restored') and 0 or 1, (mode == 'witness_copy_differs') and 0 or 1,
    (mode == 'witness_scene_index_differs') and 4096 or -1, objects_after, entry_after, (mode == 'witness_wrong_mesh') and 11 or 12) }
  for stage = 1, 4 do
    local b = g.b[stage]
    if mode == 'witness_count_mismatch' and stage == 1 then b = b + 1 end
    parts[#parts + 1] = string.format('q%d=%d b%d=%d g%d=%d o%d=%d e%d=%d', stage - 1, g.q[stage], stage - 1, b, stage - 1, g.g[stage], stage - 1, g.o[stage], stage - 1, g.e[stage])
  end
  for depth = 1, 8 do
    local d = g.d[depth] or 0
    if mode == 'witness_depth_mismatch' and depth == 1 then d = d + 1 end
    parts[#parts + 1] = string.format('d%d=%d', depth - 1, d)
  end
  parts[#parts + 1] = 'dbeyond=0'
  return table.concat(parts, ' ') .. '\n'
end

function engine:collision_trace_arm(on)
  trace_armed = on
  note('trace_arm ' .. tostring(on))
  if mode == 'trace_disarm_fails' and on == false then error('stand-in disarm failure', 0) end
end

function engine:collision_trace_read(kind, offset, count)
  local recs, facts = trace_lines()
  local total = #recs
  if kind == 0 then
    trace_reads = trace_reads + 1
    local state = string.format('tr-state armed=1 records=%d capacity=32768 overflow=%d extra_lookups=%d facts_overflow=0 facts_bytes=%d', total,
      mode == 'trace_overflow' and 1 or 0, mode == 'trace_extra_lookups' and 3 or 0, 100)
    return state .. '\n' .. table.concat(facts, '\n') .. '\n', total
  end
  local lines = {}
  for i = offset + 1, math.min(offset + count, total) do lines[#lines + 1] = recs[i] end
  if #lines == 0 then return '', total end
  return table.concat(lines, '\n') .. '\n', total
end

function engine:collision_geometry_arm(on)
  note('geometry_arm ' .. tostring(on))
  if mode == 'witness_arm_fails' then error('stand-in arm failure', 0) end
  witness_armed = on
end

function engine:collision_probe(seed, entity_cases, tree_cases)
  probes = probes + 1
  -- one-shot, as in the fixture: consumed by the next probe whatever its outcome
  local armed = witness_armed
  witness_armed = false
  witness_in_trace = armed
  note('probe ' .. seed .. ' ' .. entity_cases .. ' ' .. tree_cases .. (armed and ' witness' or ''))
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
  local text = report(seed, entity_cases, tree_cases)
  if (armed and mode ~= 'witness_row_missing') or mode == 'witness_row_unarmed' then text = text .. witness_row() end
  return text
end

-- I/O failure injection for the driver's NEW trace file only: a thin proxy over the real file whose write, flush or close fails in the trace_io_* modes. Every other file the
-- driver opens (its report output, its existence checks) is the real one.
local io_fault = ({ trace_io_write = 'write', trace_io_flush = 'flush', trace_io_close = 'close' })[mode]
if io_fault then
  local real_open = io.open
  io.open = function(path, how)
    local file, err = real_open(path, how)
    if file and how == 'w' and path == host.args.trace_out then
      local proxy = {}
      function proxy:write(...)
        if io_fault == 'write' then return nil, 'stand-in write failure' end
        local ok, e = file:write(...)
        if not ok then return nil, e end
        return self
      end
      function proxy:flush()
        if io_fault == 'flush' then return nil, 'stand-in flush failure' end
        return file:flush()
      end
      function proxy:close()
        local ok, e = file:close()
        if io_fault == 'close' then return nil, 'stand-in close failure' end
        return ok, e
      end
      return proxy
    end
    return file, err
  end
end

package.loaded['eech_dc'] = {
  prepare_installation = function() note('prepare') end,
  boot = function() note('boot') return engine end,
}
dofile('@DRIVER@')
print('COLLISION_CALLER_DONE')
