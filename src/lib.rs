//! # tracefp
//!
//! A stack backtracking library based on frame-pointer.
//!
//! # Requirements
//!
//! When compiling your project, set the following environment variables:
//!
//! - `CFLAGS` += `-fno-omit-frame-pointer -mno-omit-leaf-frame-pointer`
//! - `CXXFLAGS` += `-fno-omit-frame-pointer -mno-omit-leaf-frame-pointer`
//! - `RUSTFLAGS` += `-Cforce-frame-pointers=yes`
//!
//! And add the following parameters to `cargo build`:
//!
//! ```shell
//! rustup component add rust-src
//! cargo build -Z build-std --target x86_64-unknown-linux-gnu
//! ```
//!
//! Where `x86_64-unknown-linux-gnu` can be replaced with other values, and `build-std` currently only supports nightly Rust.
//!
//! > NOTE: When you're using this library on **macOS + aarch64**, you don't need to do anything, as all libraries for this platform turn on frame-pointer by default.
//!
//! # Examples
//!
//! ## Stack backtrace
//!
//! ```rust
//! fn main() {
//!     func1_inlined();
//! }
//!
//! #[inline(always)]
//! fn func1_inlined() {
//!     func2()
//! }
//!
//! fn func2() {
//!     tracefp::trace(|pc| {
//!         println!("{:#x}", pc);
//!         backtrace::resolve(pc as _, |s| {
//!             println!("    {:?}", s.name());
//!         });
//!         true
//!     });
//! }
//! ```
//!
//! Sample output:
//!
//! ```text
//! 0x0
//! 0x100d7348b
//!     Some(hello::func2::h53002ef4ebe4d7d7)
//! 0x100d73337
//!     Some(hello::func1_inlined::h30751d2ee2774466)
//!     Some(hello::main::h994e0b3179971102)
//! 0x100d72ddf
//!     Some(core::ops::function::FnOnce::call_once::h3dec9d79421d8d27)
//! 0x100d71723
//!     Some(std::sys_common::backtrace::__rust_begin_short_backtrace::h9755a7454510e50f)
//! 0x100d716db
//!     Some(std::rt::lang_start::{{closure}}::ha86392d061932837)
//! 0x100e4065f
//!     Some(core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once::h8eb3ac20f80eabfa)
//!     Some(std::panicking::try::do_call::ha6ddf2c638427188)
//!     Some(std::panicking::try::hda8741de507c1ad0)
//!     Some(std::panic::catch_unwind::h82424a01f258bd39)
//!     Some(std::rt::lang_start_internal::{{closure}}::h67e296ed5b030b7b)
//!     Some(std::panicking::try::do_call::hd3dd7e7e10f6424e)
//!     Some(std::panicking::try::ha0a7bd8122e3fb7c)
//!     Some(std::panic::catch_unwind::h809b0e1092e9475d)
//!     Some(std::rt::lang_start_internal::h358b6d58e23c88c7)
//! 0x100d716a3
//!     Some(std::rt::lang_start::h1342399ebba7a37d)
//! 0x100d734df
//!     Some("_main")
//! 0x101059087
//! 0xd82e7fffffffffff
//! ```
//!
//! ## Stack backtrace in signal handler
//!
//! ```rust
//! use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, SIGPROF};
//!
//! fn main() {
//!     // Register perf signal handler.
//!     let h = SigHandler::SigAction(perf_signal_handler);
//!     let a = SigAction::new(h, SaFlags::SA_SIGINFO, SigSet::empty());
//!     unsafe {
//!         sigaction(SIGPROF, &a).unwrap();
//!     }
//!
//!     // Send a SIGPROF signal to the current process.
//!     unsafe {
//!         libc::kill(libc::getpid(), libc::SIGPROF);
//!     }
//!
//!     // Block until the signal handler finishes executing.
//!     loop {}
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn perf_signal_handler(_: libc::c_int, _: *mut libc::siginfo_t, ucontext: *mut libc::c_void) {
//!     tracefp::trace_from_ucontext(ucontext, |pc| {
//!         println!("{:#x}", pc);
//!         backtrace::resolve(pc as _, |s| {
//!             println!("    {:?}", s.name());
//!         });
//!         true
//!     });
//!     std::process::exit(0);
//! }
//! ```
//!
//! Sample output:
//!
//! ```text
//! 0x1c32e4824
//!     Some("_thread_get_state")
//! 0x100409093
//!     Some(core::ops::function::FnOnce::call_once::h775fb44fbbe53d95)
//! 0x1004090eb
//!     Some(std::sys_common::backtrace::__rust_begin_short_backtrace::h3acd0b11747c5033)
//! 0x100408a2b
//!     Some(std::rt::lang_start::{{closure}}::hf7b77a4d60d2f840)
//! 0x1004d7faf
//!     Some(core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once::h8eb3ac20f80eabfa)
//!     Some(std::panicking::try::do_call::ha6ddf2c638427188)
//!     Some(std::panicking::try::hda8741de507c1ad0)
//!     Some(std::panic::catch_unwind::h82424a01f258bd39)
//!     Some(std::rt::lang_start_internal::{{closure}}::h67e296ed5b030b7b)
//!     Some(std::panicking::try::do_call::hd3dd7e7e10f6424e)
//!     Some(std::panicking::try::ha0a7bd8122e3fb7c)
//!     Some(std::panic::catch_unwind::h809b0e1092e9475d)
//!     Some(std::rt::lang_start_internal::h358b6d58e23c88c7)
//! 0x1004089f3
//!     Some(std::rt::lang_start::hd321e36029dcfdd2)
//! 0x1004093d7
//!     Some("_main")
//! 0x1007bd087
//! 0x921a7fffffffffff
//! ```

/// Inspects the current call-stack, passing all active PCs into the closure
/// provided to calculate a stack trace.
///
/// The closure's return value is an indication of whether the backtrace should
/// continue. A return value of `false` will terminate the backtrace and return
/// immediately.
pub fn trace<F>(f: F)
where
    F: FnMut(u64) -> bool,
{
    let mut ucontext: libc::ucontext_t = unsafe { std::mem::zeroed() };
    #[cfg(target_os = "macos")]
    {
        let mut mcontext: libc::__darwin_mcontext64 = unsafe { std::mem::zeroed() };
        ucontext.uc_mcontext = &mut mcontext as *mut libc::__darwin_mcontext64;
    }
    let ucontext = &mut ucontext as *mut libc::ucontext_t as *mut libc::c_void;
    unsafe {
        if getcontext(ucontext) != 0 {
            return;
        }
    }
    trace_from_ucontext(ucontext, f)
}

/// Inspects the call-stack from `ucontext`, passing all active PCs into the closure
/// provided to calculate a stack trace.
///
/// The closure's return value is an indication of whether the backtrace should
/// continue. A return value of `false` will terminate the backtrace and return
/// immediately.
pub fn trace_from_ucontext<F>(ucontext: *mut libc::c_void, mut f: F)
where
    F: FnMut(u64) -> bool,
{
    let Registers { mut pc, mut fp } = match Registers::from_ucontext(ucontext) {
        Some(v) => v,
        None => return,
    };
    if !f(pc) {
        return;
    }
    while fp != 0 {
        pc = match load::<u64>(fp + 8) {
            Some(v) => v,
            None => return,
        };
        pc -= 1;
        if !f(pc) {
            return;
        }
        fp = match load::<u64>(fp) {
            Some(v) => v,
            None => return,
        };
    }
}

extern "C" {
    // getcontext() in libc.
    //
    // We declare here instead of using `libc::getcontext()` directly because
    // `libc::getcontext()` is not found on macOS.
    fn getcontext(_ucontext: *mut libc::c_void) -> libc::c_int;
}

// Register context for stack backtracking.
#[derive(Debug, Copy, Clone)]
struct Registers {
    pc: u64,
    fp: u64,
}

impl Registers {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    fn from_ucontext(ucontext: *mut libc::c_void) -> Option<Self> {
        let ucontext = ucontext as *mut libc::ucontext_t;
        if ucontext.is_null() {
            return None;
        }
        let mcontext = unsafe { (*ucontext).uc_mcontext };
        Some(Self {
            pc: mcontext.gregs[libc::REG_RIP as usize] as u64,
            fp: mcontext.gregs[libc::REG_RBP as usize] as u64,
        })
    }

    #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
    fn from_ucontext(ucontext: *mut libc::c_void) -> Option<Self> {
        let ucontext = ucontext as *mut libc::ucontext_t;
        if ucontext.is_null() {
            return None;
        }
        unsafe {
            let mcontext = (*ucontext).uc_mcontext;
            if mcontext.is_null() {
                return None;
            }
            Some(Self {
                pc: (*mcontext).__ss.__rip,
                fp: (*mcontext).__ss.__rbx,
            })
        }
    }

    #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
    fn from_ucontext(ucontext: *mut libc::c_void) -> Option<Self> {
        let ucontext = ucontext as *mut libc::ucontext_t;
        if ucontext.is_null() {
            return None;
        }
        let mcontext = unsafe { (*ucontext).uc_mcontext };
        Ok(Self {
            pc: mcontext.pc,
            fp: mcontext.regs[29],
        })
    }

    #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
    fn from_ucontext(ucontext: *mut libc::c_void) -> Option<Self> {
        let ucontext = ucontext as *mut libc::ucontext_t;
        if ucontext.is_null() {
            return None;
        }
        unsafe {
            let mcontext = (*ucontext).uc_mcontext;
            if mcontext.is_null() {
                return None;
            }
            Some(Self {
                pc: (*mcontext).__ss.__pc,
                fp: (*mcontext).__ss.__fp,
            })
        }
    }
}

// Load the value at the `address`.
//
// Note that although `load` is not unsafe, it is implemented by unsafe
// internally and simply attempts to read the specified address. So the
// correctness of the address needs to be guaranteed by the caller.
#[inline]
#[cfg(not(all(target_os = "linux", feature = "memory-access-check")))]
fn load<T: Copy>(address: u64) -> Option<T> {
    unsafe { Some(*(address as *const T)) }
}

// Load the value at the `address`.
//
// A memory accessibility check will be performed before accessing the
// target address.
#[inline]
#[cfg(all(target_os = "linux", feature = "memory-access-check"))]
fn load<T: Copy>(address: u64) -> Option<T> {
    if access_check::can_access(address) {
        unsafe { Some(*(address as *const T)) }
    } else {
        None
    }
}

#[cfg(all(target_os = "linux", feature = "memory-access-check"))]
mod access_check {
    use std::mem::MaybeUninit;

    thread_local! {
        static CAN_ACCESS_PIPE: [libc::c_int; 2] = {
            unsafe {
                let mut fds = MaybeUninit::<[libc::c_int; 2]>::uninit();
                let res = libc::pipe2(fds.as_mut_ptr() as *mut libc::c_int, libc::O_CLOEXEC | libc::O_NONBLOCK);
                if res == 0 {
                    [fds.assume_init()[0], fds.assume_init()[1]]
                } else {
                    [-1, -1]
                }
            }
        };
    }

    /// Check whether the target address is valid.
    pub fn can_access(address: u64) -> bool {
        CAN_ACCESS_PIPE.with(|pipes| unsafe {
            // The pipe initialization failed at that time.
            if pipes[0] == -1 || pipes[1] == -1 {
                return false;
            }
            // Clear data that already exists in the pipe.
            let mut buffer = [0u8; 8];
            let can_read = loop {
                let size = libc::read(pipes[0], buffer.as_mut_ptr() as _, buffer.len() as _);
                if size == -1 {
                    match (*libc::__errno_location()) as libc::c_int {
                        libc::EINTR => continue,
                        libc::EAGAIN => break true,
                        _ => break false,
                    }
                } else if size > 0 {
                    break true;
                }
            };
            if !can_read {
                return false;
            }
            // Try to write "data" to the pipe, let the kernel access the address, if
            // the address is invalid, we will fail the write.
            loop {
                let size = libc::write(pipes[1], address as _, 1);
                if size == -1 {
                    match (*libc::__errno_location()) as libc::c_int {
                        libc::EINTR => continue,
                        libc::EAGAIN => break true,
                        _ => break false,
                    }
                } else if size > 0 {
                    break true;
                }
            }
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_can_access() {
            let v1 = 1;
            let v2 = Box::new(1);
            assert!(can_access(&v1 as *const i32 as u64));
            assert!(can_access(v2.as_ref() as *const i32 as u64));
            assert!(!can_access(0));
            assert!(!can_access(u64::MAX));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let val = i8::MIN;
        let loc = &val as *const i8 as u64;
        assert_eq!(load::<i8>(loc), Some(val));
        let val = u64::MAX;
        let loc = &val as *const u64 as u64;
        assert_eq!(load::<u64>(loc), Some(val));
    }
}
