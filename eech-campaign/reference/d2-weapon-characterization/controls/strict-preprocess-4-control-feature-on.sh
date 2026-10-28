#!/bin/sh
# usage: strict-preprocess-4.sh <cargo target volume of a build> : feature OFF preprocesses identically to the baseline blobs (git e9cea88a2) for the four weapon sources.
# Numeric constants are kept verbatim; only a source-line number immediately after a quoted .c path is normalised; compiler failures propagate (set -e, subshell status).
set -eu
cd '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/eech_source_code'
vol=$1
docker run --rm -v "$PWD:/src:ro" -v "$vol:/target:ro" -v '/mnt/c/Users/starscream/Documents/ChatGPT/Enemy Engaged DCS/d2-evidence/weapon-characterization-20261027/base-src:/basesrc:ro' eech-build sh -eu -c '
tree=$(find /target/x86_64-pc-windows-gnu/release/build -path "*/eech-engine-sys-*/out/tree" -type d | head -1)
test -n "$tree"
files="aphavoc/source/entity/mobile/weapon/wn_creat.c aphavoc/source/entity/mobile/weapon/wn_move.c aphavoc/source/entity/mobile/weapon/wn_updt.c aphavoc/source/entity/mobile/weapon/wn_cllsn.c"
mkdir -p /tmp/base /tmp/work
flags="-E -P -w -std=gnu99 -fcommon -fwrapv -DWIN32 -DEECH_CAMPAIGN_HOST_IDENTITIES -DEECH_CAMPAIGN_HOST_ARRIVALS -DEECH_CAMPAIGN_HOST_MOVEMENT -DEECH_CAMPAIGN_HOST_LOS -include /src/eech-campaign/crates/eech-engine-sys/csrc/eech_win32_names.h -I /src/eech-campaign/crates/eech-engine-sys/compat -I /src/eech-campaign/crates/eech-engine-sys/csrc -I $tree/modules -I $tree/aphavoc/source"
failed=0
for f in $files; do
    mkdir -p "/tmp/base/$(dirname "$f")" "/tmp/work/$(dirname "$f")"
    cp "/basesrc/$f" "/tmp/base/$f"
    cp "/src/$f" "/tmp/work/$f"
    (cd /tmp/base; x86_64-w64-mingw32-gcc $flags -iquote "$tree/$(dirname "$f")" "$f" > /tmp/base.raw)
    (cd /tmp/work; x86_64-w64-mingw32-gcc $flags -DEECH_WEAPON_FIXTURE -iquote "$tree/$(dirname "$f")" "$f" > /tmp/off.raw)
    # Only a numeric argument immediately following a quoted .c path is a source-line diagnostic; preserve every other numeric literal verbatim.
    sed -E "/^[[:space:]]*$/d; s/(\"[^\"]+\.c\",[[:space:]]*)[0-9]+/\1SOURCE_LINE/g" /tmp/base.raw > /tmp/base.pp
    sed -E "/^[[:space:]]*$/d; s/(\"[^\"]+\.c\",[[:space:]]*)[0-9]+/\1SOURCE_LINE/g" /tmp/off.raw > /tmp/off.pp
    test -s /tmp/base.pp
    test -s /tmp/off.pp
    if cmp -s /tmp/base.pp /tmp/off.pp; then
        echo "$f: strict feature-off equivalence PASS ($(wc -c < /tmp/off.pp) bytes preprocessed)"
    else
        echo "$f: strict feature-off equivalence DIFFERS"
        diff -u /tmp/base.pp /tmp/off.pp | head -35 || true
        failed=1
    fi
done
exit "$failed"
'
echo "STRICT_CONTROL_FEATURE_ON_EXIT=$?"
