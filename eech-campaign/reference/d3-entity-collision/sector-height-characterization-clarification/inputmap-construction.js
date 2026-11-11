// Input map of everything the sector-height checks read: aphavoc, modules and eech-campaign (minus docs, reference, target, tools),
// tracked files at their working-tree bytes plus untracked non-ignored ones. node inputmap.js <label>
const fs = require('fs');
const cp = require('child_process');
const crypto = require('crypto');
const repo = 'R:/d3-evidence/collision-characterization-20261029';
const label = process.argv[2];
const git = (args) => cp.execFileSync('git', args, { cwd: repo, maxBuffer: 1 << 28 });
const names = (buf) => buf.toString('utf8').split('\0').filter(Boolean);
const all = [...new Set([...names(git(['ls-files', '-z'])), ...names(git(['ls-files', '-z', '-o', '--exclude-standard']))])].sort();
const keep = (p) => /^(aphavoc|modules)\//.test(p) || (/^eech-campaign\//.test(p) && !/^eech-campaign\/(docs|reference|target|tools)\//.test(p));
const rows = [];
let missing = 0;
for (const p of all.filter(keep)) {
  let b;
  try { b = fs.readFileSync(repo + '/' + p); } catch (e) { missing++; continue; }
  rows.push([p, crypto.createHash('sha256').update(b).digest('hex'), b.length]);
}
const agg = crypto.createHash('sha256').update(rows.map((r) => r[0] + '\t' + r[1] + '\n').join('')).digest('hex');
const status = git(['status', '--porcelain']).toString('utf8').split('\n').filter(Boolean);
const out = {
  label,
  head: git(['rev-parse', 'HEAD']).toString().trim(),
  branch: git(['branch', '--show-current']).toString().trim(),
  files: rows.length,
  missing,
  aggregate_sha256: agg,
  status_porcelain: status,
  scope: 'tracked and untracked-non-ignored files under aphavoc/, modules/ and eech-campaign/ except docs, reference, target, tools: the inputs of the cargo checks; raw working-tree bytes',
  taken_utc: new Date().toISOString(),
  changed_vs_head: Object.fromEntries(rows.filter((r) => status.some((s) => s.endsWith(r[0]))).map((r) => [r[0], { sha256: r[1], bytes: r[2] }])),
};
fs.writeFileSync('R:/d3-evidence/' + label + '.rows.tsv', rows.map((r) => r.join(String.fromCharCode(9))).join(String.fromCharCode(10)) + String.fromCharCode(10));
fs.writeFileSync('R:/d3-evidence/' + label + '.json', JSON.stringify(out, null, 2) + '\n');
console.log(label, rows.length, agg, status.join(' | '));
