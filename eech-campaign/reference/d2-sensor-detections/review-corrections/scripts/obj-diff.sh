#!/bin/sh
docker run --rm -v eech-win-target-d2-sensor-shipping-4:/a -v eech-win-target-d2-sensor-shipping-5:/b eech-build sh -c '
for v in a b; do f=$(ls /$v/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/*-mb_hostsense.o | head -1); x86_64-w64-mingw32-objdump -d -r $f | sed -e "s/^ *[0-9a-f]*:/:/" | grep -v "file format" > /tmp/$v.txt; done
diff /tmp/a.txt /tmp/b.txt | head -60; echo "differing lines: $(diff /tmp/a.txt /tmp/b.txt | grep -c "^[<>]")"'
