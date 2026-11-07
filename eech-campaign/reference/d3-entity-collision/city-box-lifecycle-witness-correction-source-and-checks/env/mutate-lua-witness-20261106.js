const fs = require('fs');
const p = process.argv[2];
let s = fs.readFileSync(p, 'utf8');
function swap(a, b) { const parts = s.split(a); if (parts.length !== 2) throw new Error('match ' + a.slice(0, 80)); s = parts.join(b); }
// M1: the interventions no longer require the SELECTED record's BEFORE consumption
swap(`and life.selected ~= nil and (life.before or 0) >= 1`, `and life.selected ~= nil`);
// M2: the private release acknowledgement is no longer checked against the full contract
swap(`local problem = not ok and "the host's release was not answered" or city_reply_problem (acknowledged, { event = "release", phase = 16, keys = { real.name } })`, `local problem = not ok and "the host's release was not answered" or nil`);
swap(`if field ~= "result" and field ~= "name" then problem`, `if false then problem`);
// M3: the consumption is the window-wide count of any city's lookups (an aggregate join)
swap(`local before_selected = selected and (life.before or 0) or -1`, `local before_selected = scan.fixed.city_joined - joined_start`);
swap(`local after_selected = selected and ((life.joined_by [selected] or 0) - (life.relist_mark or (life.joined_by [selected] or 0))) or -1`, `local after_selected = scan.fixed.city_joined - joined_start`);
// M4: faults are no longer checked before UNREACHED
swap(`local faulted = faults.request + faults.reply ~= 0`, `local faulted = false`);
// M5: the equality declaration is no longer cross-checked against the tuples
swap(`if result.query_equal == "1" and not same then return nil, "result: query_equal=1 contradicts the before and after answers" end`, ``);
fs.writeFileSync(p, s);
