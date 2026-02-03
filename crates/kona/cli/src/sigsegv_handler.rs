//! Signal handler to extract a backtrace from reth, which is originally from stack overflow.
//!
//! Implementation modified from [reth](https://github.com/paradigmxyz/reth/blob/main/crates/cli/util/src/sigsegv_handler.rs#L120).
//!
//! Implementation modified from [`rustc`](https://github.com/rust-lang/rust/blob/3dee9775a8c94e701a08f7b2df2c444f353d8699/compiler/rustc_driver_impl/src/signal_handler.rs).

use std::{
    alloc::{Layout, alloc},
    fmt, mem, ptr,
};

unsafe extern "C" {
    fn backtrace_symbols_fd(buffer: *const *mut libc::c_void, size: libc::c_int, fd: libc::c_int);
}

fn backtrace_stderr(buffer: &[*mut libc::c_void]) {
    let size = buffer.len().try_into().unwrap_or_default();
    // SAFETY: backtrace_symbols_fd is a standard C library function that writes
    // symbol information to a file descriptor. The buffer pointer and size are valid.
    unsafe { backtrace_symbols_fd(buffer.as_ptr(), size, libc::STDERR_FILENO) };
}

/// Unbuffered, unsynchronized writer to stderr.
///
/// Only acceptable because everything will end soon anyways.
struct RawStderr(());

impl fmt::Write for RawStderr {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        // SAFETY: libc::write is a standard C library function. We pass valid pointer
        // and length from the string slice.
        let ret = unsafe { libc::write(libc::STDERR_FILENO, s.as_ptr().cast(), s.len()) };
        if ret == -1 { Err(fmt::Error) } else { Ok(()) }
    }
}

/// We don't really care how many bytes we actually get out. SIGSEGV comes for our head.
/// Splash stderr with letters of our own blood to warn our friends about the monster.
macro_rules! raw_errln {
    ($tokens:tt) => {
        let _ = ::core::fmt::Write::write_fmt(&mut RawStderr(()), format_args!($tokens));
        let _ = ::core::fmt::Write::write_char(&mut RawStderr(()), '\n');
    };
}

/// Signal handler installed for SIGSEGV
extern "C" fn print_stack_trace(_: libc::c_int) {
    const MAX_FRAMES: usize = 256;
    let mut stack_trace: [*mut libc::c_void; MAX_FRAMES] = [ptr::null_mut(); MAX_FRAMES];
    // SAFETY: libc::backtrace fills the stack_trace array with return addresses.
    // The array is properly sized and initialized.
    let stack = unsafe {
        let depth = libc::backtrace(stack_trace.as_mut_ptr(), MAX_FRAMES as i32);
        if depth == 0 {
            return;
        }
        &stack_trace[0..depth as usize]
    };

    raw_errln!("error: base interrupted by SIGSEGV, printing backtrace\n");
    let mut written = 1;
    let mut consumed = 0;
    let cycled = |(runner, walker)| runner == walker;
    let mut cyclic = false;
    if let Some(period) = stack.iter().skip(1).step_by(2).zip(stack).position(cycled) {
        let period = period.saturating_add(1);
        let Some(offset) = stack.iter().skip(period).zip(stack).position(cycled) else {
            return;
        };

        let next_cycle = stack[offset..].chunks_exact(period).skip(1);
        let cycles = 1 + next_cycle
            .zip(stack[offset..].chunks_exact(period))
            .filter(|(next, prev)| next == prev)
            .count();
        backtrace_stderr(&stack[..offset]);
        written += offset;
        consumed += offset;
        if cycles > 1 {
            raw_errln!("\n### cycle encountered after {offset} frames with period {period}");
            backtrace_stderr(&stack[consumed..consumed + period]);
            raw_errln!("### recursed {cycles} times\n");
            written += period + 4;
            consumed += period * cycles;
            cyclic = true;
        };
    }
    let rem = &stack[consumed..];
    backtrace_stderr(rem);
    raw_errln!("");
    written += rem.len() + 1;

    let random_depth = || 8 * 16;
    if cyclic || stack.len() > random_depth() {
        raw_errln!("note: base unexpectedly overflowed its stack! this is a bug");
        written += 1;
    }
    if stack.len() == MAX_FRAMES {
        raw_errln!("note: maximum backtrace depth reached, frames may have been lost");
        written += 1;
    }
    raw_errln!("note: we would appreciate a report at https://github.com/base/base");
    written += 1;
    if written > 24 {
        raw_errln!("note: backtrace dumped due to SIGSEGV! resuming signal");
    }
}

/// Installs a SIGSEGV handler.
///
/// When SIGSEGV is delivered to the process, print a stack trace and then exit.
pub fn install() {
    // SAFETY: This sets up signal handling infrastructure using libc functions.
    // The alternate stack and signal action are properly initialized with zeroed memory.
    // The function pointer cast is required for the sigaction struct.
    unsafe {
        let alt_stack_size: usize = min_sigstack_size() + 64 * 1024;
        let mut alt_stack: libc::stack_t = mem::zeroed();
        alt_stack.ss_sp = alloc(Layout::from_size_align(alt_stack_size, 1).unwrap()).cast();
        alt_stack.ss_size = alt_stack_size;
        libc::sigaltstack(&alt_stack, ptr::null_mut());

        let mut sa: libc::sigaction = mem::zeroed();
        sa.sa_sigaction = print_stack_trace as *const () as libc::sighandler_t;
        sa.sa_flags = libc::SA_NODEFER | libc::SA_RESETHAND | libc::SA_ONSTACK;
        libc::sigemptyset(&mut sa.sa_mask);
        libc::sigaction(libc::SIGSEGV, &sa, ptr::null_mut());
    }
}

/// Modern kernels on modern hardware can have dynamic signal stack sizes.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn min_sigstack_size() -> usize {
    const AT_MINSIGSTKSZ: core::ffi::c_ulong = 51;
    // SAFETY: getauxval is a standard C library function that returns auxiliary vector values.
    let dynamic_sigstksz = unsafe { libc::getauxval(AT_MINSIGSTKSZ) };
    libc::MINSIGSTKSZ.max(dynamic_sigstksz as _)
}

/// Not all OS support hardware where this is needed.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
const fn min_sigstack_size() -> usize {
    libc::MINSIGSTKSZ
}
