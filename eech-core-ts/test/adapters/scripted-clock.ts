//
// Deterministic Clock: the frame driver states each frame's delta and whether
// the frame rate is locked. The adapter never measures or invents time.
//

import type { Clock } from "../../src/ports";

export class ScriptedClock implements Clock {
	private deltaTime = 0.1;

	private locked = false;

	public setFrame(deltaTime: number, locked: boolean): void {
		this.deltaTime = deltaTime;
		this.locked = locked;
	}

	public getDeltaTime(): number {
		return this.deltaTime;
	}

	public isFrameRateLocked(): boolean {
		return this.locked;
	}
}
