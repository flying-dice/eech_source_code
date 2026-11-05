//! The reference host's own numerical profile (EEDCS-50, D3).
//!
//! EECH_WORLD, the reference host, reproduces the arithmetic context the campaign has always run in: the SSE unit rounds toward zero (the x87 unit is left as it is). That
//! context used to reach the host's own arithmetic by accident, because the first public call of the campaign module (`prepare_installation`) left its toward-zero SSE
//! word behind in the host's thread. The module now hands back the caller's complete floating-point state on every entry, as an independent host (DCS World) requires, so the
//! reference host selects its profile itself, here, explicitly:
//!
//! * the script asks for it with `host.use_reference_arithmetic ()`, immediately after the campaign module's installation has been prepared and before it boots, observes, records
//!   or does any time-budget or metrics arithmetic. Everything the script did before that (argument and scenario parsing, constants, hours) ran in the process's own state;
//! * only the two SSE rounding-control bits (13 and 14 of MXCSR) change. The x87 environment, the exception masks, the sticky flags and every other MXCSR bit are kept;
//! * the thread keeps the profile until the process ends; the module's entries save and restore the caller's complete state, so they preserve it. Nothing here is an
//!   option of the campaign module, and the module has no knowledge of it;
//! * a processor family without an SSE control word is refused with an error, never skipped.
//!
//! This is a statement about the reference host's own execution, not a claim that the campaign's results are independent of the rounding mode.

/// MXCSR bits 13 and 14: the SSE rounding control.
pub const MXCSR_ROUNDING_CONTROL: u32 = 0x6000;
/// The rounding-control value for round toward zero.
pub const MXCSR_TOWARD_ZERO: u32 = 0x6000;

/// The MXCSR word with the rounding control set to toward-zero and every other bit kept.
pub fn with_toward_zero(mxcsr: u32) -> u32 {
    (mxcsr & !MXCSR_ROUNDING_CONTROL) | MXCSR_TOWARD_ZERO
}

/// The MXCSR words around a selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub before: u32,
    pub after: u32,
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse {
    use std::arch::asm;

    pub fn read() -> u32 {
        let mut value: u32 = 0;
        let pointer = &mut value as *mut u32;
        // SAFETY: stmxcsr writes four bytes to the valid local.
        unsafe { asm!("stmxcsr [{0}]", in(reg) pointer, options(nostack)) };
        value
    }

    pub fn write(value: u32) {
        let pointer = &value as *const u32;
        // SAFETY: ldmxcsr reads four bytes from the valid local; the value is the thread's own word with only the rounding-control bits changed, so no reserved bit is set.
        unsafe { asm!("ldmxcsr [{0}]", in(reg) pointer, options(nostack)) };
    }
}

/// Selects SSE round-toward-zero on the calling thread, keeping every other MXCSR bit and leaving the x87 unit alone. Idempotent. The word is read back and any difference is an
/// error, so a selection that did not take effect is never reported as made.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn select_sse_toward_zero() -> Result<Selection, String> {
    let before = sse::read();
    let wanted = with_toward_zero(before);
    sse::write(wanted);
    let after = sse::read();
    if after != wanted {
        return Err(format!(
            "the SSE control word did not take the reference profile: wanted 0x{wanted:08x}, read back 0x{after:08x}"
        ));
    }
    Ok(Selection { before, after })
}

/// Selects SSE round-toward-zero on the calling thread. This processor family has no SSE control word to select, so the profile is refused.
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
pub fn select_sse_toward_zero() -> Result<Selection, String> {
    Err("the reference arithmetic profile (SSE round-toward-zero) is defined for x86 and x86_64 only".to_string())
}

/// The calling thread's MXCSR, for tests and diagnostics.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn current_mxcsr() -> u32 {
    sse::read()
}

#[cfg(all(test, any(target_arch = "x86_64", target_arch = "x86")))]
mod tests {
    use super::*;

    // The thread's own state is put back by every test, so a failure cannot leak a mode into another test on the thread.
    struct Restore(u32);
    impl Drop for Restore {
        fn drop(&mut self) {
            sse::write(self.0);
        }
    }

    #[test]
    fn only_the_rounding_control_bits_change() {
        assert_eq!(with_toward_zero(0x1f80), 0x7f80);
        // a caller at nearest, down and up, with sticky flags and a changed denormal/flush bit, keeps every other bit
        for rounding in 0u32..4 {
            let word = 0x1f80 | (rounding << 13) | 0x0021 | 0x8040;
            let changed = with_toward_zero(word);
            assert_eq!(changed & !MXCSR_ROUNDING_CONTROL, word & !MXCSR_ROUNDING_CONTROL, "rounding {rounding}");
            assert_eq!(changed & MXCSR_ROUNDING_CONTROL, MXCSR_TOWARD_ZERO);
        }
    }

    #[test]
    fn the_selection_takes_effect_keeps_the_flags_and_is_idempotent() {
        let _restore = Restore(sse::read());
        sse::write(0x1f80 | 0x0021); // nearest, all masked, invalid and precision flags raised
        let first = select_sse_toward_zero().unwrap();
        assert_eq!(first.before, 0x1fa1);
        assert_eq!(first.after, 0x7fa1);
        assert_eq!(sse::read(), 0x7fa1);
        let again = select_sse_toward_zero().unwrap();
        assert_eq!((again.before, again.after), (0x7fa1, 0x7fa1));
    }
}
