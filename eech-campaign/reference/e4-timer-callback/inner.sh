#!/bin/sh
# E4 evidence, run inside a pinned container by reproduce.sh.
#
#   inner.sh build <label>       build eech-engine-sys from /red and /green (default target)
#   inner.sh inventory <label>   EECH_C_OPT=2 global-state inventory test on /green
#   inner.sh suite <label>       the whole x86_64 cargo test on /green
#   inner.sh disasm              GCC 12: timer.o as cc-rs built it, i686 and x86_64, red vs green
#
# /red and /green are read-only exports of the two revisions (git archive);
# results are written to /out. Each step records the compiler and Rust
# identity in its log.
set -u
mode=$1
label=${2:-}
out=/out

ident() {
	echo "## $(date -u +%Y-%m-%dT%H:%M:%SZ)"
	echo "## $(gcc --version | head -1)"
	echo "## $(rustc -V) / $(cargo -V)"
	echo "## revision: $(cat /$1/REVISION)"
}

if [ -n "${RUSTUP_INSTALL:-}" ]; then
	# the gcc image has no Rust: install the pinned toolchain
	curl -sSf https://sh.rustup.rs | sh -s -- -y -q --profile minimal --default-toolchain "$RUSTUP_INSTALL" >/dev/null 2>&1
	. "$HOME/.cargo/env"
fi

case "$mode" in
build)
	for t in red green; do
		log=$out/$label-$t-build.log
		{
			ident $t
			echo "## command: cargo build --locked -p eech-engine-sys (CARGO_TARGET_DIR=/tmp/$t, cwd /$t/eech-campaign)"
		} > "$log"
		cd /$t/eech-campaign
		# CC_ENABLE_DEBUG_OUTPUT=1 makes cc-rs print each compiler command (no flag changes)
		CC_ENABLE_DEBUG_OUTPUT=1 CARGO_TARGET_DIR=/tmp/$t cargo build --locked -p eech-engine-sys >> "$log" 2>&1
		status=$?
		echo "## exit status: $status" >> "$log"
		staged=$(ls /tmp/$t/debug/build/eech-engine-sys-*/out/tree/modules/system/timer.c 2>/dev/null | head -1)
		if [ -n "$staged" ]; then
			echo "## staged timer.c: $(grep -c 'EECH headless (E4)' "$staged") E4 site(s)" >> "$log"
			echo "## timer.c compiler command (from the build script's output):" >> "$log"
			grep -hE '^running: .*modules/system/timer\.c"?$' /tmp/$t/debug/build/eech-engine-sys-*/output /tmp/$t/debug/build/eech-engine-sys-*/stderr 2>/dev/null | head -1 >> "$log"
		fi
		echo "$label $t: exit $status"
	done
	;;
inventory)
	log=$out/$label-inventory-O2.log
	{ ident green; echo "## command: EECH_C_OPT=2 cargo test --locked -p eech-sys --test findings"; } > "$log"
	cd /green/eech-campaign
	EECH_C_OPT=2 CARGO_TARGET_DIR=/tmp/inv cargo test --locked -p eech-sys --test findings >> "$log" 2>&1
	status=$?
	echo "## exit status: $status" >> "$log"
	echo "$label inventory: exit $status"
	;;
suite)
	apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends liblua5.1-0-dev >/dev/null 2>&1
	log=$out/$label-suite.log
	{ ident green; echo "## command: cargo test --locked (x86_64-unknown-linux-gnu)"; } > "$log"
	cd /green/eech-campaign
	CARGO_TARGET_DIR=/tmp/suite cargo test --locked >> "$log" 2>&1
	status=$?
	echo "## exit status: $status" >> "$log"
	echo "$label suite: exit $status"
	;;
disasm)
	apt-get update -qq >/dev/null && apt-get install -y -qq --no-install-recommends gcc-multilib >/dev/null 2>&1
	rustup target add i686-unknown-linux-gnu >/dev/null 2>&1
	{ ident green; echo "## red: $(cat /red/REVISION)"; } > $out/disasm-identity.txt
	for t in red green; do
		for target in i686-unknown-linux-gnu x86_64-unknown-linux-gnu; do
			arch=${target%%-*}
			cd /$t/eech-campaign
			# cc-rs prints each compiler command with CC_ENABLE_DEBUG_OUTPUT=1;
			# it changes no flag
			CC_ENABLE_DEBUG_OUTPUT=1 CARGO_TARGET_DIR=/tmp/d-$t cargo build --locked -q -p eech-engine-sys --target $target \
				|| { echo "disasm: $t $target build failed"; exit 1; }
			build=$(ls -d /tmp/d-$t/$target/debug/build/eech-engine-sys-*/ | while read d; do [ -f "$d/output" ] && echo "$d"; done | head -1)
			grep -E '^running: .*modules/system/timer\.c"?$' "$build/output" | head -1 | sed -E "s#/tmp/d-$t#<target>#g; s#/$t/#/<tree>/#g; s#eech-engine-sys-[0-9a-f]+#eech-engine-sys-<hash>#g; s#[0-9a-f]{16}-timer\.o#<hash>-timer.o#g" > $out/timer-$t-$arch.command.txt
			obj=$(ls "$build"/out/*-timer.o | head -1)
			objdump -d --no-show-raw-insn "$obj" | tail -n +4 > $out/timer-$t-$arch.dis
			# address-normalised: drop instruction addresses and symbol offsets,
			# so that only changed instructions remain
			sed -E 's/^ *[0-9a-f]+:\t//; s/<([A-Za-z_][A-Za-z0-9_]*)\+0x[0-9a-f]+>/<\1+off>/g; s/\b[0-9a-f]+ <([A-Za-z_][A-Za-z0-9_]*)(\+off)?>/ADDR <\1\2>/g; s/# [0-9a-f]+ </# ADDR </g; s/^[0-9a-f]{16} </</' \
				$out/timer-$t-$arch.dis > $out/timer-$t-$arch.norm
		done
	done
	for arch in i686 x86_64; do
		{
			echo "# timer.o, $arch: red vs green"
			if cmp -s $out/timer-red-$arch.dis $out/timer-green-$arch.dis; then
				echo "disassembly: IDENTICAL ($(wc -l < $out/timer-red-$arch.dis) lines)"
			else
				echo "disassembly: DIFFERS ($(diff $out/timer-red-$arch.dis $out/timer-green-$arch.dis | grep -c '^[<>]') changed lines)"
			fi
			echo "address-normalised diff:"
			diff $out/timer-red-$arch.norm $out/timer-green-$arch.norm && echo "(none)"
			echo "compiler commands differ only in paths:"
			diff $out/timer-red-$arch.command.txt $out/timer-green-$arch.command.txt && echo "(identical)"
		} > $out/timer-$arch.comparison.txt
		head -2 $out/timer-$arch.comparison.txt | tail -1
	done
	;;
esac
