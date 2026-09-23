//
// Slice 4 behaviour matrix (test/scenarios/keysite-cargo.cases.ts) under
// JavaScript semantics. The same cases run against the original C in
// test/c-reference/keysite-cargo.cref.test.ts and under Lua 5.1.
//

import { describe, expect, it } from "vitest";
import { getGameStatus, initialiseCampaignCore, setGameStatus } from "../../src";
import { GameStatusType } from "../../src/generated/c-enums";
import { InMemoryMobilePhysicalState } from "../adapters/in-memory-mobile-physical-state";
import { InMemoryObject3DMetadata } from "../adapters/in-memory-object-3d-metadata";
import { RecordingEntityReplication } from "../adapters/recording-entity-replication";
import { ScriptedClock } from "../adapters/scripted-clock";
import { C_REFERENCE_RANDOM_KEYSITE_CARGO } from "../scenarios/generated/c-reference-random-keysite-cargo.cases";
import { KEYSITE_CARGO_CASES } from "../scenarios/keysite-cargo.cases";
import { firstUnmatchedLine, runLifecycle } from "../scenarios/lifecycle-scenario";

describe("keysite.c :: update_keysite_cargo", () => {
	for (const c of KEYSITE_CARGO_CASES) {
		it(`${c.id} [${c.c}]`, () => {
			const output = runLifecycle(c.spec);

			expect(firstUnmatchedLine(output, c.expected), output.join("\n")).toBe("");

			for (const prefix of c.absent) {
				expect(output.filter((line) => line.startsWith(prefix)), `absent: ${prefix}`).toEqual([]);
			}
		});
	}
});

describe("the recorded keysite cargo scenarios (outputs of the original C)", () => {
	// run back to back, as a host would reinitialise the core between sessions
	it("all 150 reproduce the C's output", () => {
		for (const c of C_REFERENCE_RANDOM_KEYSITE_CARGO) {
			expect(runLifecycle(c.spec), c.id).toEqual(c.expected);
		}
	});
});

describe("global.c :: game_status", () => {
	it("is zero-initialised (GAME_STATUS_UNINITIALISED) by every new campaign core", () => {
		const ports = () => ({
			mobilePhysicalState: new InMemoryMobilePhysicalState(),
			entityReplication: new RecordingEntityReplication(),
			clock: new ScriptedClock(),
			object3DMetadata: new InMemoryObject3DMetadata(),
		});
		initialiseCampaignCore(ports());
		expect(getGameStatus()).toBe(GameStatusType.GAME_STATUS_UNINITIALISED);
		setGameStatus(GameStatusType.GAME_STATUS_INITIALISING);
		expect(getGameStatus()).toBe(GameStatusType.GAME_STATUS_INITIALISING);
		initialiseCampaignCore(ports());
		expect(getGameStatus()).toBe(GameStatusType.GAME_STATUS_UNINITIALISED);
	});
});
