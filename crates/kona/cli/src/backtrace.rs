//! Helper to set the backtrace env var.

/// Sets the `RUST_BACKTRACE` environment variable to 1 if it is not already set.
pub fn enable() {
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        // SAFETY: This is called early in main before any other threads are spawned.
        // The race condition with another process setting RUST_BACKTRACE is acceptable.
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }
}
