//! Windows: a process that dies inside the engine leaves no trace by itself
//! (no Rust panic, no message). This reports, on stderr, a fatal exception
//! (code, address, and offset in the module it happened in) and any exit the
//! engine's C code makes, before the process goes.

#[cfg(windows)]
mod imp {
    use std::ffi::c_void;

    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        parameters: u32,
        information: [usize; 15],
    }

    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut c_void,
    }

    extern "system" {
        fn AddVectoredExceptionHandler(first: u32, handler: unsafe extern "system" fn(*mut ExceptionPointers) -> i32) -> *mut c_void;
        fn GetModuleHandleExA(flags: u32, name: *const c_void, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameA(module: *mut c_void, name: *mut u8, size: u32) -> u32;
    }

    extern "C" {
        fn atexit(f: extern "C" fn()) -> i32;
    }

    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 4;
    const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 2;

    /// the fatal codes: access violation, stack overflow, illegal instruction,
    /// integer divide by zero, privileged instruction, in-page error
    fn fatal(code: u32) -> bool {
        matches!(code, 0xC000_0005 | 0xC000_00FD | 0xC000_001D | 0xC000_0094 | 0xC000_0096 | 0xC000_0006)
    }

    unsafe extern "system" fn handler(pointers: *mut ExceptionPointers) -> i32 {
        // SAFETY: the system passes valid exception pointers
        let record = unsafe { &*(*pointers).record };
        if fatal(record.code) {
            let mut module: *mut c_void = std::ptr::null_mut();
            let mut name = [0u8; 260];
            let (base, file) = unsafe {
                if GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, record.address, &mut module) != 0 {
                    let n = GetModuleFileNameA(module, name.as_mut_ptr(), name.len() as u32) as usize;
                    (module as usize, String::from_utf8_lossy(&name[..n]).into_owned())
                } else {
                    (0, "?".into())
                }
            };
            eprintln!(
                "eech-world: fatal exception 0x{:08x} at {:p} ({} + 0x{:x}); access {:?}",
                record.code,
                record.address,
                file,
                (record.address as usize).wrapping_sub(base),
                &record.information[..(record.parameters as usize).min(2)]
            );
            // the call chain, by frame pointers (the engine's C keeps them): x64 CONTEXT has Rsp at 0x98, Rbp at 0xa0
            let context = unsafe { (*pointers).context as *const u8 };
            let (rsp, mut rbp) = unsafe { (*(context.add(0x98) as *const usize), *(context.add(0xa0) as *const usize)) };
            let mut chain = Vec::new();
            for _ in 0..24 {
                // a frame lies above the stack pointer, within the 16 MB stack, 8-byte aligned
                if rbp < rsp || rbp > rsp + (16 << 20) || rbp % 8 != 0 {
                    break;
                }
                let (next, ret) = unsafe { (*(rbp as *const usize), *((rbp + 8) as *const usize)) };
                chain.push(format!("+0x{:x}", ret.wrapping_sub(base)));
                if next <= rbp {
                    break;
                }
                rbp = next;
            }
            eprintln!("eech-world: call chain (offsets in {}): {}", file, chain.join(" "));
        }
        0 // EXCEPTION_CONTINUE_SEARCH
    }

    extern "C" fn on_exit() {
        eprintln!("eech-world: process exit");
    }

    pub fn install() {
        // SAFETY: registering process-wide handlers with valid function pointers
        unsafe {
            AddVectoredExceptionHandler(1, handler);
            atexit(on_exit);
        }
    }
}

/// installs the crash report (Windows; nothing elsewhere)
pub fn install() {
    #[cfg(windows)]
    imp::install();
}
