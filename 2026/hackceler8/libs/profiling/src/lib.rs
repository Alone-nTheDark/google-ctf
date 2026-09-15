#![no_std]

#[allow(dead_code)]
const FUNCTION_TRACER_PORT: *mut u16 = 0x400000 as *mut u16;
#[allow(dead_code)]
const TRACE_PORT: *mut u16 = 0x400002 as *mut u16;

pub const fn fvn1a(bytes: &[u8]) -> u32 {
    let mut hash = 2166136261u32;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(16777619);
        i += 1;
    }

    hash
}

pub const fn fvn1a_trace_id(bytes: &[u8]) -> u16 {
    let hash = fvn1a(bytes);
    let hash = (hash >> 16) ^ (hash & 0xFFFF);
    hash as u16
}

#[macro_export]
macro_rules! trace {
    ($name:expr) => {
        #[cfg(all(target_arch = "m68k", feature = "profiling"))]
        {
            // Enforce compile time calculation
            const MSG_ID: u16 = profiling::fvn1a_trace_id($name.as_bytes());
            profiling::trace(MSG_ID);
        }
    };
}

#[inline(always)]
pub fn start_of_frame() {
    #[cfg(all(target_arch = "m68k", feature = "profiling"))]
    unsafe {
        core::ptr::write_volatile(FUNCTION_TRACER_PORT, 0);
    }
}

#[inline(always)]
pub fn trace(id: u16) {
    #[cfg(all(target_arch = "m68k", feature = "profiling"))]
    unsafe {
        core::ptr::write_volatile(TRACE_PORT, id);
    }
    let _ = id;
}

#[inline(always)]
fn function_trace(id: u16) {
    #[cfg(all(target_arch = "m68k", feature = "profiling"))]
    unsafe {
        core::ptr::write_volatile(FUNCTION_TRACER_PORT, id);
    }
    let _ = id;
}

/// Used by the profiling macro to report enter/exit signals.
pub struct ProfilingGuard(u16);

impl ProfilingGuard {
    pub fn for_function_id(id: u16) -> Self {
        // assert!((id & 0x8000) == 0);
        function_trace(id);
        Self(id | 0x8000)
    }
}

impl core::ops::Drop for ProfilingGuard {
    fn drop(&mut self) {
        function_trace(self.0);
    }
}
