#!/bin/sh
# Regenerates eech-campaign/corpus/*.jsonl.gz from eech-core-ts (its scenario
# model, the TSTL port and the canonical 32-bit C reference). Needs Node, the
# eech-core-ts dependencies (npm ci) and a C compiler with 32-bit support.
#   tools/corpus/export.sh                      recorded fixtures and matrices
#   EECH_CORPUS_FRESH=500 tools/corpus/export.sh    500 fresh random scenarios per generator
set -eu
here=$(cd "$(dirname "$0")" && pwd)
core="$here/../../../eech-core-ts"
# resolve vitest and the eech-core-ts toolchain from this directory
ln -sfn "$core/node_modules" "$here/node_modules"
cd "$core"
exec npx vitest run --config "$here/vitest.config.mts"
