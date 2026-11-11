#!/bin/sh
set -eu
base=/target/x86_64-pc-windows-gnu/release
for profile in ordinary hosted original; do
 case "$profile" in
 ordinary) suffix=3ee86ff288c8e5b9; root=/src/eech-campaign/target/windows-city-box-4c0b89263 ;;
 hosted) suffix=f1a9d979734127b0; root=/src/eech-campaign/target/weapon-fixture/city-box-4c0b89263 ;;
 original) suffix=0c38ad68eea565f8; root=/src/eech-campaign/target/weapon-fixture/city-box-original-4c0b89263 ;;
 esac
 mkdir "/out/$profile"
 cp "$base/.fingerprint/eech-engine-sys-$suffix/lib-eech_engine_sys.json" "/out/$profile/cargo-feature-manifest.json"
 lib="$base/deps/libeech_engine_sys-$suffix.rlib"
 x86_64-w64-mingw32-ar t "$lib" >/tmp/members.txt
 sed -n '/eech_engine\.o$/p;/mb_hostplace\.o$/p;/eech_wpn_fixture\.o$/p;/eech_mq_report_check\.o$/p' /tmp/members.txt >/tmp/selected.txt
 while IFS= read -r member; do
  x86_64-w64-mingw32-ar p "$lib" "$member" >"/out/$profile/$member"
  x86_64-w64-mingw32-nm -g --defined-only "/out/$profile/$member" >"/out/$profile/$member.symbols.txt"
 done </tmp/selected.txt
 x86_64-w64-mingw32-objdump -p "$root/eech_dc.dll" >"/out/$profile/dll-exports.txt"
done
