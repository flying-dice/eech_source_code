//
// C provenance: modules/system/assert.h (ASSERT, debug_assert)
//
// EECH aborts on a failed ASSERT / debug_assert (debug builds). The port
// keeps the check and raises instead, so a violated EECH invariant can never
// be silently skipped. Messages quote the original C expression.
//

export class EechAssertionError extends Error {
	public constructor(public readonly expression: string) {
		super(`ASSERT (${expression})`);
		this.name = "EechAssertionError";
	}
}

// Raised where EECH calls debug_fatal (the game stops). `format` is the C
// format string, which identifies the call site; the message is formatted.
export class EechFatalError extends Error {
	public constructor(
		public readonly format: string,
		message: string = format,
	) {
		super(message);
		this.name = "EechFatalError";
	}
}

// Raised where the original C dereferences a NULL pointer without an ASSERT.
// EECH would crash; the port refuses to continue.
export class EechNullDereferenceError extends Error {
	public constructor(what: string) {
		super(`NULL dereference: ${what}`);
		this.name = "EechNullDereferenceError";
	}
}

// Raised where the original C has undefined behaviour that crashes EECH (e.g.
// integer division by zero, SIGFPE on x86). The port refuses to continue.
export class EechUndefinedBehaviourError extends Error {
	public constructor(what: string) {
		super(`undefined behaviour in the original C: ${what}`);
		this.name = "EechUndefinedBehaviourError";
	}
}

// Raised when campaign code reaches C behaviour that has not been ported yet.
// This is not an EECH behaviour; it keeps partially ported function tables
// from silently falling back to EECH default handlers.
export class UnportedBehaviourError extends Error {
	public constructor(what: string) {
		super(`unported EECH behaviour: ${what}`);
		this.name = "UnportedBehaviourError";
	}
}

export function ASSERT(condition: boolean, expression: string): asserts condition {
	if (!condition) {
		throw new EechAssertionError(expression);
	}
}

export function assertNotNullDereference<T>(value: T | undefined, what: string): asserts value is T {
	if (value === undefined) {
		throw new EechNullDereferenceError(what);
	}
}

//
// Raised where campaign code reaches the boundary of the ported slice: a C
// function the adopted slices select arguments for but do not port. It is
// unported behaviour (production fails loudly there) that carries the
// selection, so conformance runners can report what the ported decision chose
// and the C reference harness can end at the same call with the same payload.
//
export class UnportedBoundaryError extends UnportedBehaviourError {
	public constructor(
		public readonly boundary: string,
		public readonly args: readonly unknown[],
	) {
		super(boundary);
		this.name = "UnportedBoundaryError";
	}
}
