//
// The SIGSEGV handler may only reach write () and _exit (). This is checked on
// the compiled harness object, not by reading the source: the handler's static
// call graph is walked through objdump's disassembly and relocations, so a
// libc call added directly, through a helper, or by the compiler (e.g.
// __stack_chk_fail, an implicit memcpy) fails the test.
//

import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { expect, it } from "vitest";
// @ts-expect-error plain ESM build script without type declarations
import { HARNESS_BUILD_DIR, HARNESS_OBJECT_NAME } from "../../c-reference/build.mjs";

const HANDLER = "segmentation_fault";
const ALLOWED_EXTERNAL = new Set(["write", "_exit"]);

function callGraph(objectFile: string): Map<string, Set<string>> {
	const dump = spawnSync("objdump", ["-dr", "--no-show-raw-insn", objectFile], { encoding: "utf8" });
	if (dump.status !== 0) {
		throw new Error(`objdump failed: ${dump.error ?? ""}${dump.stderr}`);
	}

	const graph = new Map<string, Set<string>>();
	let current: Set<string> | undefined;

	for (const line of dump.stdout.split("\n")) {
		const header = /^[0-9a-f]+ <([^>]+)>:$/.exec(line);
		if (header) {
			current = new Set<string>();
			graph.set(header[1], current);
			continue;
		}
		if (current === undefined) {
			continue;
		}
		// direct calls and tail jumps to a symbol start (x86-64 call/jmp, arm64 bl/b)
		const direct = /\s(?:call|callq|jmp|jmpq|bl|b)\s+[0-9a-f]+ <([^>+]+)>/.exec(line);
		if (direct) {
			current.add(direct[1]);
		}
		// relocated calls (x86-64 PLT32, arm64 CALL26 / JUMP26)
		const reloc = /R_(?:X86_64_PLT32|AARCH64_CALL26|AARCH64_JUMP26)\s+([^\s+-]+)/.exec(line);
		if (reloc) {
			current.add(reloc[1]);
		}
	}

	return graph;
}

it("the SIGSEGV handler reaches no library call but write () and _exit ()", () => {
	const graph = callGraph(join(HARNESS_BUILD_DIR as string, HARNESS_OBJECT_NAME as string));

	expect(graph.has(HANDLER)).toBe(true);

	const external = new Set<string>();
	const seen = new Set<string>();
	const pending = [HANDLER];

	while (pending.length > 0) {
		const fn = pending.pop() as string;
		if (seen.has(fn)) {
			continue;
		}
		seen.add(fn);
		for (const target of graph.get(fn) ?? []) {
			if (graph.has(target)) {
				pending.push(target);
			} else {
				external.add(target);
			}
		}
	}

	expect([...external].sort()).toEqual([...ALLOWED_EXTERNAL].sort());
	// sanity: the walk really followed the helpers
	expect(seen.has("emit_final_state")).toBe(true);
	expect(seen.has("write_all")).toBe(true);
});
