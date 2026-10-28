# Sanitizer runtime investigation

The first observer check compiled successfully, then repeatedly emitted
AddressSanitizer:DEADLYSIGNAL. The author stopped its container and retained the
first4KB; the full reported7.6GB flood was discarded before an exact full hash.
Root observed7,121,021,503 bytes while it was growing, not its final size. This
directory does not claim full retention or reconstruct that original output.

A separate bounded empty-main program reproduced the symptom without observer
code. Default PIE:4/8 runs exited0,4/8 flooded and hit the64KB log cap (exit153).
With test-only `-fno-pie -no-pie`:8/8 exited0. ELF types3 and2 were verified from
the retained executables. This supports a sanitizer/runtime configuration issue;
it does not prove that the first observer failure had the identical cause or
replace source review. Source, image/compiler identity, statuses and per-file
hashes are retained here; full bounded logs and executable binaries stay private.

The observer-check wrapper now uses non-PIE for both the sanitizer availability
probe and the real observer check, with bounded output/time and kill grace. It
fails on a failed probe instead of silently claiming sanitized coverage. The new
real-C guard passed24 checks with compile/run/Docker/outer exits0. Shipping flags,
the main Dockerfile, accepted baselines and campaign inputs are unchanged.
