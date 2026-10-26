#!/bin/sh
# compares the shipping scan objects of two cargo volumes (5c00ab4fa vs a2a041fec) by disassembly with addresses removed
docker run --rm -v eech-win-target-d2-sensor-shipping-4:/a -v eech-win-target-d2-sensor-shipping-5:/b eech-build sh -c '
for v in a b; do
  echo "== volume $v"
  for n in mb_hostsense ac_tgt vh_tgt; do
    f=$(ls /$v/x86_64-pc-windows-gnu/release/build/eech-engine-sys-*/out/*-$n.o | head -1)
    d=$(x86_64-w64-mingw32-objdump -d -r $f | sed -e "s/^ *[0-9a-f]*:/:/" | grep -v "file format" | sha256sum | cut -c1-16)
    echo "$n raw-object $(sha256sum $f | cut -c1-16) disassembly $d"
  done
done'
